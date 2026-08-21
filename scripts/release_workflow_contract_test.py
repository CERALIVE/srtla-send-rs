#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["PyYAML==6.0.2"]
# ///
# ─── How to run ───
# uv run scripts/release_workflow_contract_test.py

from __future__ import annotations

import unittest
from pathlib import Path
from typing import Final

import yaml

from workflow_authority_contract_test import WorkflowAuthorityContractTests
from workflow_contract import JobStatus, load_workflow, simulate, transitive_needs


ROOT: Final = Path(__file__).resolve().parents[1]
CI: Final = ROOT / ".github/workflows/ci.yml"
RELEASE: Final = ROOT / ".github/workflows/release.yml"
BINDINGS: Final = ROOT / ".github/workflows/publish-bindings.yml"


class ReleaseWorkflowContractTests(unittest.TestCase):
    def test_release_debs_build_against_bookworm_glibc(self) -> None:
        with RELEASE.open(encoding="utf-8") as workflow_file:
            workflow = yaml.safe_load(workflow_file)

        build_deb = workflow["jobs"]["build-deb"]
        self.assertEqual(build_deb["container"]["image"], "debian:bookworm-slim")
        matrix = build_deb["strategy"]["matrix"]["include"]
        self.assertEqual(
            {entry["arch"]: entry["objdump"] for entry in matrix},
            {"arm64": "aarch64-linux-gnu-objdump", "amd64": "objdump"},
        )
        steps = {step["name"]: step for step in build_deb["steps"]}
        self.assertIn(
            "build-essential", steps["Install Bookworm build dependencies"]["run"]
        )
        compatibility_check = steps["Verify device GLIBC compatibility"]["run"]
        self.assertIn('"${{ matrix.objdump }}" -T', compatibility_check)
        self.assertIn('dpkg --compare-versions "${required_glibc#GLIBC_}" gt "2.36"', compatibility_check)
        cache = next(
            step
            for step in build_deb["steps"]
            if step.get("uses") == "Swatinem/rust-cache@v2"
        )
        self.assertIn("bookworm", cache["with"]["shared-key"])

    def test_release_publication_needs_every_rust_gate(self) -> None:
        workflow = load_workflow(RELEASE)
        gate = workflow.job("test")

        self.assertTrue(gate.has_command("cargo", "build", "--release"))
        self.assertTrue(gate.has_command("cargo", "fmt", "--check"))
        self.assertTrue(gate.has_command("cargo", "clippy", "-D", "warnings"))
        self.assertTrue(gate.has_command("cargo", "test", "--lib"))
        self.assertTrue(gate.has_command("cargo", "test", "--all-features"))
        self.assertTrue(gate.has_command("cargo", "test", "test-internals"))
        self.assertTrue(gate.has_command("cargo", "audit"))
        self.assertTrue(gate.has_command("cargo", "deny", "advisories", "sources"))
        bounded_tests = tuple(
            command
            for command in gate.commands("cargo")
            if command.has_arguments("test", "--all-features")
            or command.has_arguments("test", "test-internals")
        )
        self.assertEqual(len(bounded_tests), 2)
        self.assertTrue(all(command.is_bounded for command in bounded_tests))

        build_dependencies = workflow.job("build-deb").needs
        self.assertEqual(
            build_dependencies,
            frozenset(("test", "loom", "miri")),
        )
        self.assertEqual(workflow.job("release").needs, frozenset(("build-deb",)))
        self.assertEqual(
            transitive_needs(workflow, "release"),
            frozenset(("test", "loom", "miri", "build-deb")),
        )

    def test_ci_and_release_keep_exact_loom_command_contract(self) -> None:
        for path in (CI, RELEASE):
            with self.subTest(workflow=path.name):
                loom = load_workflow(path).job("loom")
                self.assertEqual(loom.name, "Loom model (subscription manager)")
                self.assertFalse(loom.needs)
                command_steps = tuple(
                    (step, command)
                    for step in loom.steps
                    for command in step.commands
                    if command.executable == "cargo"
                )
                self.assertEqual(len(command_steps), 1)
                step, command = command_steps[0]
                self.assertEqual(step.name, "Run loom model test")
                self.assertEqual(step.environment_value("RUSTFLAGS"), "--cfg loom")
                self.assertEqual(command.prefix, ())
                self.assertEqual(
                    command.arguments, ("test", "--test", "subscription_loom")
                )

    def test_ci_and_release_keep_parallel_miri_semantics(self) -> None:
        # Both the recvmmsg and sendmmsg pure-pointer filters are required in
        # BOTH workflows: the unsafe FFI inventory is recvmmsg + sendmmsg.
        required_filters: Final = (
            "init_rebuilds_self_pointers_after_move",
            "iter_clamps_oversized_msg_len_to_mtu",
            "sockaddr_storage_roundtrip",
            "sendmmsg_pointers_rebuilt_after_move",
            "sendmmsg_prefix_extraction_bounded",
        )
        for path in (CI, RELEASE):
            with self.subTest(workflow=path.name):
                miri = load_workflow(path).job("miri")
                self.assertFalse(miri.needs)
                for test_filter in required_filters:
                    self.assertTrue(
                        miri.has_command("cargo", "miri", "test", test_filter),
                        f"{path.name} miri lane must run {test_filter}",
                    )

    def test_failed_rust_gate_skips_every_release_publication(self) -> None:
        workflow = load_workflow(RELEASE)
        for failed_gate in ("test", "loom", "miri"):
            outcome = simulate(workflow, ((failed_gate, JobStatus.FAILURE),))
            for publication_job in workflow.publication_jobs:
                self.assertEqual(
                    outcome.status(publication_job.job_id), JobStatus.SKIPPED
                )

    def test_bindings_publish_requires_tests_and_verified_tag_provenance(self) -> None:
        workflow = load_workflow(BINDINGS)
        gate = workflow.job("test-bindings")
        verifier = workflow.job("verify-release-ref")
        publish = workflow.job("publish")

        for command in (
            ("install", "--frozen-lockfile"),
            ("run", "lint"),
            ("run", "typecheck"),
            ("run", "test"),
            ("run", "build"),
        ):
            self.assertTrue(gate.has_command("bun", *command))
        self.assertTrue(
            verifier.has_command("bash", "ci/verify-bindings-release-ref.sh")
        )
        self.assertEqual(
            publish.needs, frozenset(("test-bindings", "verify-release-ref"))
        )
        self.assertIn(("id-token", "write"), publish.permissions)

    def test_manual_dispatch_can_only_reach_non_oidc_dry_run(self) -> None:
        workflow = load_workflow(BINDINGS)
        dry_run = workflow.job("dry-run")
        publish = workflow.job("publish")

        self.assertIn("workflow_dispatch", workflow.triggers)
        self.assertIn("workflow_dispatch", dry_run.condition)
        self.assertNotIn(("id-token", "write"), dry_run.permissions)
        self.assertTrue(
            any(
                command.is_dry_run_package_publish
                for command in dry_run.commands("npm")
            )
        )
        self.assertFalse(dry_run.has_external_mutation_authority)

        outcome = simulate(
            workflow,
            (
                ("test-bindings", JobStatus.SUCCESS),
                ("verify-release-ref", JobStatus.SKIPPED),
            ),
        )
        self.assertEqual(outcome.status(publish.job_id), JobStatus.SKIPPED)

    def test_publication_jobs_never_override_failed_dependencies(self) -> None:
        for path in (RELEASE, BINDINGS):
            workflow = load_workflow(path)
            for job in workflow.publication_jobs:
                self.assertFalse(job.allows_failure)
                self.assertNotIn("always()", job.condition)

    def test_ci_executes_release_contracts_and_bounds_netns_capable_tests(self) -> None:
        workflow = load_workflow(CI)
        gate = workflow.job("test")

        self.assertTrue(
            gate.has_command("uv", "run", "scripts/release_workflow_contract_test.py")
        )
        self.assertTrue(
            gate.has_command("bash", "scripts/release_version_contract_test.sh")
        )
        all_features = tuple(
            command
            for command in gate.commands("cargo")
            if command.has_arguments("test", "--all-features")
        )
        self.assertEqual(len(all_features), 1)
        self.assertTrue(all_features[0].is_bounded)

    def test_binding_contracts_run_under_bun_never_on_an_ambient_node(self) -> None:
        # Both scripts evaluate JavaScript. The Rust `test` job declares no JS
        # runtime, so running them there worked only through the GitHub image's
        # ambient Node — an accident of the image, not a declared dependency.
        binding_contracts: Final = (
            "scripts/bindings_release_ref_contract_test.sh",
            "scripts/bindings_package_manager_contract_test.sh",
        )
        workflow = load_workflow(CI)
        rust_gate = workflow.job("test")
        bindings = workflow.job("bindings")

        self.assertFalse(
            any(
                (step.action or "").startswith("actions/setup-node")
                for step in (*rust_gate.steps, *bindings.steps)
            )
        )
        setup_bun_index = next(
            index
            for index, step in enumerate(bindings.steps)
            if step.action == "oven-sh/setup-bun@v2"
        )
        for script in binding_contracts:
            with self.subTest(script=script):
                self.assertFalse(rust_gate.has_command("bash", script))
                self.assertTrue(bindings.has_command("bash", script))
                contract_index = next(
                    index
                    for index, step in enumerate(bindings.steps)
                    if any(
                        command.has_arguments(script) for command in step.commands
                    )
                )
                self.assertGreater(contract_index, setup_bun_index)
                # Repo-root scripts under a job whose default is bindings/typescript.
                self.assertEqual(
                    bindings.steps[contract_index].working_directory, "."
                )


__all__ = ("ReleaseWorkflowContractTests", "WorkflowAuthorityContractTests")


if __name__ == "__main__":
    _ = unittest.main(verbosity=2)

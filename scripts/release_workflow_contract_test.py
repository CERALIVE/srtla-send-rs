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

from workflow_authority_contract_test import WorkflowAuthorityContractTests
from workflow_contract import JobStatus, load_workflow, simulate, transitive_needs


ROOT: Final = Path(__file__).resolve().parents[1]
CI: Final = ROOT / ".github/workflows/ci.yml"
RELEASE: Final = ROOT / ".github/workflows/release.yml"
BINDINGS: Final = ROOT / ".github/workflows/publish-bindings.yml"


class ReleaseWorkflowContractTests(unittest.TestCase):
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

    def test_release_keeps_parallel_miri_semantics(self) -> None:
        miri = load_workflow(RELEASE).job("miri")

        self.assertFalse(miri.needs)
        for test_filter in (
            "init_rebuilds_self_pointers_after_move",
            "iter_clamps_oversized_msg_len_to_mtu",
            "sockaddr_storage_roundtrip",
        ):
            self.assertTrue(miri.has_command("cargo", "miri", "test", test_filter))

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
            ("lint",),
            ("typecheck",),
            ("test",),
            ("build",),
        ):
            self.assertTrue(gate.has_command("pnpm", *command))
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
        self.assertTrue(
            gate.has_command("bash", "scripts/bindings_release_ref_contract_test.sh")
        )
        self.assertTrue(
            gate.has_command(
                "bash", "scripts/bindings_package_manager_contract_test.sh"
            )
        )
        all_features = tuple(
            command
            for command in gate.commands("cargo")
            if command.has_arguments("test", "--all-features")
        )
        self.assertEqual(len(all_features), 1)
        self.assertTrue(all_features[0].is_bounded)


__all__ = ("ReleaseWorkflowContractTests", "WorkflowAuthorityContractTests")


if __name__ == "__main__":
    _ = unittest.main(verbosity=2)

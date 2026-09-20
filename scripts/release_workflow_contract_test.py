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
WORKFLOW_DIRECTORY: Final = ROOT / ".github/workflows"
MIRI_FILTERS: Final = (
    "test_recv_buffer_creation",
    "test_buffer_size",
    "iter_clamps_oversized_msg_len_to_mtu",
    "sockaddr_storage_roundtrip",
    "recv_retry_action_classifies_errors",
)


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
                # Parallel with the main Rust gate, never serialized behind it.
                # `changes` is the seconds-long docs-only detector every CI job
                # hangs off; depending on it is not a serialization.
                self.assertFalse(loom.needs - {"changes"})
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
        # Exactly the syscall-free tests in src/net/batch_recv.rs, and every one
        # of them, in BOTH workflows. miri cannot execute recvmmsg/sendmmsg, so
        # a filter naming a test that binds a socket would not be a stricter
        # lane — it would be a broken one.
        required_filters: Final = MIRI_FILTERS
        for path in (CI, RELEASE):
            with self.subTest(workflow=path.name):
                miri = load_workflow(path).job("miri")
                # Same parallelism rule as the loom lane above.
                self.assertFalse(miri.needs - {"changes"})
                invocations = tuple(
                    command
                    for command in miri.commands("cargo")
                    if command.has_arguments("miri", "test")
                )
                self.assertEqual(len(invocations), len(required_filters))
                for test_filter in required_filters:
                    self.assertTrue(
                        miri.has_command("cargo", "miri", "test", test_filter),
                        f"{path.name} miri lane must run {test_filter}",
                    )
                for command in invocations:
                    self.assertTrue(
                        command.has_arguments(
                            "--lib", "--no-default-features", "test-internals"
                        )
                    )

    def test_failed_rust_gate_skips_every_release_publication(self) -> None:
        workflow = load_workflow(RELEASE)
        for failed_gate in ("test", "loom", "miri"):
            outcome = simulate(workflow, ((failed_gate, JobStatus.FAILURE),))
            for publication_job in workflow.publication_jobs:
                self.assertEqual(
                    outcome.status(publication_job.job_id), JobStatus.SKIPPED
                )

    def test_the_only_release_track_is_the_debian_package(self) -> None:
        # A second publish workflow would be an independently-triggered path to
        # an external registry, so its absence is asserted, not left to review.
        self.assertEqual(
            frozenset(path.name for path in WORKFLOW_DIRECTORY.glob("*.yml")),
            frozenset(("ci.yml", "release.yml")),
        )
        release = load_workflow(RELEASE)
        self.assertEqual(release.triggers, frozenset(("push",)))
        for job in release.publication_jobs:
            self.assertEqual(job.job_id, "release")

    def test_release_publishes_the_srtla_package_artifacts(self) -> None:
        with RELEASE.open(encoding="utf-8") as workflow_file:
            workflow = yaml.safe_load(workflow_file)

        steps = {step["name"]: step for step in workflow["jobs"]["release"]["steps"]}
        attach = steps["Create release and attach .debs"]["with"]["files"]
        for architecture in ("arm64", "amd64"):
            self.assertIn(f"release-assets/srtla_*_{architecture}.deb", attach)
        self.assertNotIn("srtla-send-rs_", attach)

        dispatch = steps["Trigger APT reindex"]["with"]["client-payload"]
        self.assertIn('"component":"srtla"', dispatch)

    def test_publication_jobs_never_override_failed_dependencies(self) -> None:
        workflow = load_workflow(RELEASE)
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
            gate.has_command("bash", "scripts/deb_version_ordering_test.sh")
        )
        all_features = tuple(
            command
            for command in gate.commands("cargo")
            if command.has_arguments("test", "--all-features")
        )
        self.assertEqual(len(all_features), 1)
        self.assertTrue(all_features[0].is_bounded)

    def test_ci_declares_no_javascript_runtime(self) -> None:
        # Every gate here is Rust, Python-via-uv, or bash. A JS runtime in this
        # workflow would only ever be there to serve a publish track that this
        # repository does not have.
        workflow = load_workflow(CI)
        for job in workflow.jobs:
            with self.subTest(job=job.job_id):
                for step in job.steps:
                    action = step.action or ""
                    self.assertFalse(action.startswith("actions/setup-node"))
                    self.assertFalse(action.startswith("oven-sh/setup-bun"))
                self.assertEqual(job.commands("npm"), ())
                self.assertEqual(job.commands("bun"), ())


__all__ = ("ReleaseWorkflowContractTests", "WorkflowAuthorityContractTests")


if __name__ == "__main__":
    _ = unittest.main(verbosity=2)

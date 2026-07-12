#!/usr/bin/env python3

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[1]
RELEASE = ROOT / ".github/workflows/release.yml"
BINDINGS = ROOT / ".github/workflows/publish-bindings.yml"


def load_workflow(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        workflow = yaml.safe_load(handle)
    assert isinstance(workflow, dict)
    assert isinstance(workflow.get("jobs"), dict)
    return workflow


def needs(job: dict[str, Any]) -> list[str]:
    value = job.get("needs", [])
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [str(item) for item in value]
    if isinstance(value, dict):
        return [str(item) for item in value]
    return []


def job_text(job: dict[str, Any]) -> str:
    return json.dumps(job, sort_keys=True)


def publication_jobs(workflow: dict[str, Any]) -> set[str]:
    jobs = workflow["jobs"]
    return {
        job_id
        for job_id, job in jobs.items()
        if any(
            marker in job_text(job)
            for marker in (
                "softprops/action-gh-release",
                "repository-dispatch",
                "npm publish",
            )
        )
    }


def transitive_needs(workflow: dict[str, Any], job_id: str) -> set[str]:
    jobs = workflow["jobs"]
    found: set[str] = set()
    pending = list(needs(jobs[job_id]))
    while pending:
        dependency = pending.pop()
        if dependency in found:
            continue
        found.add(dependency)
        pending.extend(needs(jobs[dependency]))
    return found


def simulate_gate_failure(workflow: dict[str, Any], failed_job: str) -> dict[str, str]:
    jobs = workflow["jobs"]
    statuses = {failed_job: "failure"}
    remaining = set(jobs) - {failed_job}
    while remaining:
        progressed = False
        for job_id in sorted(remaining):
            dependencies = needs(jobs[job_id])
            if any(dependency not in statuses for dependency in dependencies):
                continue
            statuses[job_id] = (
                "skipped"
                if any(statuses[dependency] != "success" for dependency in dependencies)
                else "success"
            )
            remaining.remove(job_id)
            progressed = True
            break
        if not progressed:
            raise AssertionError(f"workflow graph is cyclic or has an unknown dependency: {remaining}")
    return statuses


class ReleaseWorkflowContractTests(unittest.TestCase):
    def test_release_has_full_rust_gate_and_parallel_safety_lanes(self) -> None:
        workflow = load_workflow(RELEASE)
        jobs = workflow["jobs"]
        for job_id in ("test", "loom", "miri", "build-deb", "release"):
            self.assertIn(job_id, jobs)

        rust_gate = job_text(jobs["test"])
        for command in (
            "cargo build --release",
            "cargo fmt --all -- --check",
            "cargo clippy -- -D warnings",
            "cargo test --lib",
            "cargo test --all-features",
            "cargo test --features test-internals",
            "cargo audit",
            "cargo deny check advisories sources",
        ):
            self.assertIn(command, rust_gate)

        self.assertIn("RUSTFLAGS", jobs["loom"].get("steps", [{}])[2].get("env", {}))
        self.assertIn("--cfg loom", job_text(jobs["loom"]))
        for command in (
            "init_rebuilds_self_pointers_after_move",
            "iter_clamps_oversized_msg_len_to_mtu",
            "sockaddr_storage_roundtrip",
        ):
            self.assertIn(command, job_text(jobs["miri"]))

        self.assertGreaterEqual(set(needs(jobs["build-deb"])), {"test", "loom", "miri"})
        self.assertEqual(needs(jobs["release"]), ["build-deb"])
        self.assertTrue({"test", "loom", "miri"} <= transitive_needs(workflow, "release"))

    def test_bindings_publish_has_separate_lint_typecheck_test_gate(self) -> None:
        workflow = load_workflow(BINDINGS)
        jobs = workflow["jobs"]
        self.assertIn("test-bindings", jobs)
        self.assertIn("publish", jobs)
        gate_text = job_text(jobs["test-bindings"])
        for command in (
            "bun install --frozen-lockfile",
            "bun run lint",
            "bun run typecheck",
            "bun test",
            "bun run build",
        ):
            self.assertIn(command, gate_text)
        self.assertNotIn("npm publish", gate_text)
        self.assertEqual(needs(jobs["publish"]), ["test-bindings"])
        self.assertEqual(transitive_needs(workflow, "publish"), {"test-bindings"})

    def test_failed_gate_skips_every_publication_job(self) -> None:
        release = load_workflow(RELEASE)
        for failed_gate in ("test", "loom", "miri"):
            statuses = simulate_gate_failure(release, failed_gate)
            for job_id in publication_jobs(release):
                self.assertEqual(
                    statuses[job_id],
                    "skipped",
                    f"{failed_gate} failure must block release job {job_id}",
                )

        bindings = load_workflow(BINDINGS)
        statuses = simulate_gate_failure(bindings, "test-bindings")
        for job_id in publication_jobs(bindings):
            self.assertEqual(
                statuses[job_id],
                "skipped",
                f"binding gate failure must block publish job {job_id}",
            )

    def test_publication_jobs_do_not_override_failed_needs(self) -> None:
        for path in (RELEASE, BINDINGS):
            workflow = load_workflow(path)
            for job_id in publication_jobs(workflow):
                condition = str(workflow["jobs"][job_id].get("if", ""))
                self.assertNotIn("always()", condition, f"{path}: {job_id}")
                self.assertNotIn("continue-on-error", job_text(workflow["jobs"][job_id]))


if __name__ == "__main__":
    unittest.main(verbosity=2)

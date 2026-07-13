#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["PyYAML==6.0.2"]
# ///
# pyright: reportImplicitRelativeImport=false

# ─── How to run ───
# 1. Install uv (if not installed):
#      curl -LsSf https://astral.sh/uv/install.sh | sh
# 2. Run directly (no venv, no pip install needed):
#      uv run rust_cache_contract_test.py
# 3. Or make executable and run:
#      chmod +x rust_cache_contract_test.py && ./rust_cache_contract_test.py
# ──────────────────

from __future__ import annotations

import unittest
from collections.abc import Mapping
from pathlib import Path
from typing import Final

import yaml
from yaml.nodes import Node
from yaml.loader import SafeLoader

from workflow_yaml import NodeMap, mapping, optional_scalar, scalar, sequence


ROOT: Final = Path(__file__).resolve().parents[1]
WORKFLOWS: Final = (
    ROOT / ".github/workflows/ci.yml",
    ROOT / ".github/workflows/release.yml",
)
JOBS_BY_WORKFLOW: Final[Mapping[str, tuple[str, ...]]] = {
    "ci.yml": (
        "test",
        "loom",
        "miri",
        "build-deb",
        "test-stable",
        "test-beta",
        "test-windows",
        "test-macos",
    ),
    "release.yml": ("test", "loom", "miri", "build-deb"),
}
TOOLCHAIN_BY_JOB: Final[Mapping[str, str]] = {
    "test": "nightly",
    "loom": "nightly-loom",
    "miri": "nightly-miri",
    "build-deb": "nightly",
    "test-stable": "stable",
    "test-beta": "beta",
    "test-windows": "stable",
    "test-macos": "stable",
}
SOURCE_PATTERNS: Final = ("build.rs", "src/**/*.rs", "crates/**/*.rs", "tests/**/*.rs")


class ContractShapeError(RuntimeError):
    pass


def load_jobs(path: Path) -> NodeMap:
    """Parse the workflow's job mapping without evaluating expressions."""
    with path.open(encoding="utf-8") as handle:
        document = yaml.compose(handle, Loader=SafeLoader)
    if document is None:
        raise ContractShapeError(f"empty workflow: {path}")
    return mapping(mapping(document).require("jobs"))


def job_node(jobs: NodeMap, job_id: str) -> Node:
    """Return one named job or fail closed when the workflow shape changes."""
    job = jobs.get(job_id)
    if job is None:
        raise ContractShapeError(f"missing job {job_id}")
    return job


def steps(job: Node) -> tuple[Node, ...]:
    """Return the steps declared by a workflow job."""
    return sequence(mapping(job).require("steps"))


def action_step(job: Node, action: str) -> Node:
    """Return the single step using an action."""
    matches = tuple(
        step
        for step in steps(job)
        if optional_scalar(mapping(step), "uses") == action
    )
    if len(matches) != 1:
        raise ContractShapeError(f"expected one {action} step, found {len(matches)}")
    return matches[0]


def action_steps(job: Node, action: str) -> tuple[Node, ...]:
    """Return all steps using an action so missing actions remain test failures."""
    return tuple(
        step
        for step in steps(job)
        if optional_scalar(mapping(step), "uses") == action
    )


class RustCacheContractTests(unittest.TestCase):
    """Lock Rust cache coverage and keys for both Rust workflows."""

    def test_every_rust_lane_uses_bounded_latest_major_cache(self) -> None:
        for workflow_path in WORKFLOWS:
            with self.subTest(workflow=workflow_path.name):
                jobs = load_jobs(workflow_path)
                for job_id in JOBS_BY_WORKFLOW[workflow_path.name]:
                    with self.subTest(job=job_id):
                        job = job_node(jobs, job_id)
                        cache_steps = action_steps(job, "Swatinem/rust-cache@v2")
                        self.assertEqual(len(cache_steps), 1)
                        if len(cache_steps) != 1:
                            continue
                        cache = mapping(cache_steps[0]).require("with")
                        cache_inputs = mapping(cache)
                        setup = action_step(
                            job, "actions-rust-lang/setup-rust-toolchain@v1"
                        )
                        setup_inputs = mapping(setup).get("with")
                        self.assertIsNotNone(setup_inputs)
                        if setup_inputs is None:
                            continue
                        self.assertEqual(
                            scalar(mapping(setup_inputs).require("cache")),
                            "false",
                        )

                        shared_key = scalar(cache_inputs.require("shared-key"))
                        source_key = scalar(cache_inputs.require("key"))
                        for fragment in (
                            "${{ runner.os }}",
                            "${{ runner.arch }}",
                            TOOLCHAIN_BY_JOB[job_id],
                        ):
                            self.assertIn(fragment, shared_key)
                        self.assertIn("Cargo.lock", source_key)
                        for pattern in SOURCE_PATTERNS:
                            self.assertIn(pattern, source_key)
                        self.assertEqual(
                            scalar(cache_inputs.require("workspaces")),
                            ". -> target",
                        )
                        self.assertEqual(
                            scalar(cache_inputs.require("cache-targets")),
                            "false" if job_id == "miri" else "true",
                        )
                        for input_name in (
                            "add-job-id-key",
                            "cache-bin",
                            "cache-on-failure",
                        ):
                            self.assertEqual(
                                scalar(cache_inputs.require(input_name)), "false"
                            )
                        if job_id == "build-deb":
                            for fragment in ("${{ matrix.arch }}", "${{ matrix.target }}"):
                                self.assertIn(fragment, shared_key)

    def test_rust_lanes_do_not_override_cache_contract_with_failure_tolerance(
        self,
    ) -> None:
        for workflow_path in WORKFLOWS:
            with self.subTest(workflow=workflow_path.name):
                jobs = load_jobs(workflow_path)
                for job_id in JOBS_BY_WORKFLOW[workflow_path.name]:
                    with self.subTest(job=job_id):
                        job_map = mapping(job_node(jobs, job_id))
                        self.assertIsNone(job_map.get("continue-on-error"))
                        for step in steps(job_node(jobs, job_id)):
                            self.assertIsNone(mapping(step).get("continue-on-error"))


if __name__ == "__main__":
    _ = unittest.main(verbosity=2)

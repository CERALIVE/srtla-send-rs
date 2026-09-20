#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["PyYAML==6.0.2"]
# ///
# ─── How to run ───
# uv run scripts/workflow_authority_contract_test.py

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from textwrap import dedent
from typing import Final

from workflow_contract import Workflow, load_workflow


ROOT: Final = Path(__file__).resolve().parents[1]
CI: Final = ROOT / ".github/workflows/ci.yml"
RELEASE: Final = ROOT / ".github/workflows/release.yml"
SETUP_UV_ACTION: Final = "astral-sh/setup-uv@v8.3.2"


def load_fixture(source: str) -> Workflow:
    with TemporaryDirectory() as directory:
        path = Path(directory) / "workflow.yml"
        path.write_text(dedent(source), encoding="utf-8")
        return load_workflow(path)


class WorkflowAuthorityContractTests(unittest.TestCase):
    def test_any_write_permission_marks_unknown_execution_as_publication(self) -> None:
        workflow = load_fixture(
            """
            name: adversarial publication shapes
            on: push
            permissions:
              contents: read
            jobs:
              gh-release:
                permissions:
                  contents: write
                steps:
                  - run: gh release create v1.0.0
              api-write:
                permissions:
                  contents: write
                steps:
                  - run: curl -X POST https://example.invalid/releases
              npx-publisher:
                permissions:
                  id-token: write
                steps:
                  - run: npx package-publisher
              bun-publisher:
                permissions:
                  id-token: write
                steps:
                  - run: bunx package-publisher
              helper-script:
                permissions:
                  packages: write
                steps:
                  - run: bash scripts/publish.sh
            """
        )

        self.assertEqual(
            frozenset(job.job_id for job in workflow.publication_jobs),
            frozenset(
                (
                    "gh-release",
                    "api-write",
                    "npx-publisher",
                    "bun-publisher",
                    "helper-script",
                )
            ),
        )

    def test_secret_reference_marks_indirect_execution_as_publication(self) -> None:
        workflow = load_fixture(
            """
            name: secret-backed publication shapes
            on: push
            permissions:
              contents: read
            jobs:
              helper-script:
                env:
                  RELEASE_TOKEN: ${{ secrets.RELEASE_TOKEN }}
                steps:
                  - run: bash scripts/publish.sh
              third-party-action:
                steps:
                  - uses: example/package-publisher@v1
                    with:
                      token: ${{ secrets.RELEASE_TOKEN }}
            """
        )

        self.assertEqual(
            frozenset(job.job_id for job in workflow.publication_jobs),
            frozenset(("helper-script", "third-party-action")),
        )

    def test_top_level_write_permission_marks_every_job_as_publication(self) -> None:
        workflow = load_fixture(
            """
            name: inherited publication authority
            on: push
            permissions:
              contents: write
            jobs:
              arbitrary-action:
                steps:
                  - uses: example/unknown@v1
              arbitrary-script:
                steps:
                  - run: bash scripts/helper.sh
            """
        )

        self.assertEqual(
            frozenset(job.job_id for job in workflow.publication_jobs),
            frozenset(("arbitrary-action", "arbitrary-script")),
        )

    def test_missing_permissions_is_conservatively_publication_capable(self) -> None:
        workflow = load_fixture(
            """
            name: repository-default authority
            on: push
            jobs:
              opaque-helper:
                steps:
                  - run: bash scripts/helper.sh
            """
        )

        self.assertEqual(
            tuple(job.job_id for job in workflow.publication_jobs),
            ("opaque-helper",),
        )

    def test_release_tooling_uses_resolvable_action_refs(self) -> None:
        # setup-uv publishes no `v8` major alias, so the ref must stay a full
        # release tag; shortening it resolves to nothing and fails the gate.
        ci = load_workflow(CI)

        setup_uv_actions = tuple(
            step.action
            for step in ci.job("test").steps
            if step.action is not None and step.action.startswith("astral-sh/setup-uv@")
        )
        self.assertEqual(setup_uv_actions, (SETUP_UV_ACTION,))

    def test_only_the_tagged_release_attachment_holds_write_authority(self) -> None:
        ci = load_workflow(CI)
        release = load_workflow(RELEASE)

        self.assertEqual(ci.publication_jobs, ())
        self.assertEqual(
            frozenset(job.job_id for job in release.publication_jobs),
            frozenset(("release",)),
        )
        self.assertEqual(release.job("release").needs, frozenset(("build-deb",)))


__all__ = ("WorkflowAuthorityContractTests",)


if __name__ == "__main__":
    _ = unittest.main(verbosity=2)

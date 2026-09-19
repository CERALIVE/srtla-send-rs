# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic>=2,<3"]
# ///
# Run: uv run scripts/bench/apply_verdict_precondition.py
"""Resolve Todo 21's release default from the three independent evidence inputs."""

import argparse
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

Mode = Literal["classic", "enhanced", "rtt-threshold", "edpf", "adaptive"]


class Verdict(BaseModel):
    model_config = ConfigDict(frozen=True)
    ship_set: tuple[Mode, ...] = Field(min_length=1)
    base_mode: Mode


class M5(BaseModel):
    model_config = ConfigDict(frozen=True)
    gate: Literal["passed", "failed", "skipped", "pending"]
    reason: str | None = None


class Canary(BaseModel):
    model_config = ConfigDict(frozen=True)
    outcome: Literal["pass", "fail", "pending"]


class Resolution(BaseModel):
    model_config = ConfigDict(frozen=True)
    default: Mode
    branch: Literal["verdict", "fallback"]
    final_set: tuple[Mode, ...]


def resolve(verdict: Verdict, m5: M5, canary: Canary) -> Resolution:
    m5_ready = m5.gate == "passed" or (
        m5.gate == "skipped"
        and verdict.base_mode == "enhanced"
        and m5.reason == "base mode is enhanced"
    )
    use_verdict = m5_ready and canary.outcome == "pass"
    default: Mode = verdict.base_mode if use_verdict else "enhanced"
    return Resolution(
        default=default,
        branch="verdict" if use_verdict else "fallback",
        final_set=tuple(sorted(set(verdict.ship_set) | {default})),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verdict", type=Path, default=Path("docs/evidence/bpc/m4/verdict.json")
    )
    parser.add_argument(
        "--m5", type=Path, default=Path("docs/evidence/bpc/m5/skipped.json")
    )
    parser.add_argument(
        "--canary", type=Path, default=Path("docs/evidence/bpc/canary/canary.json")
    )
    args = parser.parse_args()
    result = resolve(
        Verdict.model_validate_json(args.verdict.read_bytes()),
        M5.model_validate_json(args.m5.read_bytes()),
        Canary.model_validate_json(args.canary.read_bytes()),
    )
    print(result.model_dump_json())


if __name__ == "__main__":
    main()

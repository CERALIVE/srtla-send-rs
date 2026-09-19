#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/m4_supplements.py RAW_ROOT OUTPUT_JSON

import sys
from pathlib import Path

from report import (
    Document,
    EvidenceError,
    Manifest,
    RunRecord,
    Sender,
    build_summary,
    publish,
)


class Bucket(Document):
    t_ms: int
    duration_ms: int
    bytes: int


class Health(Document):
    conn_id: str
    health: str | None = None


class Telemetry(Document):
    connections: tuple[Health, ...]


class Sample(Document):
    t_ms: int
    document: Telemetry


class SoakSender(Sender):
    stats_file_samples: tuple[Sample, ...]


class Raw(Document):
    stats_csv_path: Path
    sink_series: tuple[Bucket, ...]


class FullRun(RunRecord):
    raw: Raw
    sender: SoakSender


class Observation(Document):
    duration_seconds: float
    sender_stderr_lines: int
    sender_alive: bool
    receiver_alive: bool
    caller_alive: bool
    listener_alive: bool


class SoakResult(Document):
    candidate: str
    crashes: int
    unrecovered_links: int
    final_health: tuple[Health, ...]
    goodput_60s_buckets: tuple[float, ...]
    monotonic_goodput_decline: bool
    passed: bool


class LogRate(Document):
    scenario: str
    candidate: str
    run_index: int
    lines: int
    seconds: float
    lines_per_second: float
    passed: bool


class FecResult(Document):
    off_goodput_median: float
    on_goodput_median: float
    ratio: float | None
    passed: bool


class Supplements(Document):
    fec: FecResult
    soaks: tuple[SoakResult, ...]
    log_rates: tuple[LogRate, ...]


def soak_result(run: FullRun, observation: Observation) -> SoakResult:
    if run.window.end_ms - run.window.start_ms != 600000:
        raise EvidenceError("M8 requires the full 600s window")
    if not run.sender.stats_file_samples:
        raise EvidenceError("M8 final health is unavailable")
    final = max(run.sender.stats_file_samples, key=lambda sample: sample.t_ms)
    if final.t_ms < run.window.end_ms - 2000 or len(final.document.connections) != 3:
        raise EvidenceError("M8 final three-link telemetry is stale or incomplete")
    rates: list[float] = []
    for index in range(10):
        start = run.window.start_ms + index * 60000
        end = start + 60000
        covered = 0
        delivered = 0.0
        for bucket in run.raw.sink_series:
            overlap = max(
                0, min(end, bucket.t_ms) - max(start, bucket.t_ms - bucket.duration_ms)
            )
            if bucket.duration_ms <= 0:
                raise EvidenceError("invalid sink bucket")
            covered += overlap
            delivered += bucket.bytes * overlap / bucket.duration_ms
        if covered != 60000:
            raise EvidenceError("M8 incomplete or overlapping sink bucket coverage")
        rates.append(delivered * 8 / 60)
    decline = rates[-1] < rates[0] and all(
        right <= left for left, right in zip(rates, rates[1:])
    )
    crashes = sum(
        not alive
        for alive in (
            observation.sender_alive,
            observation.receiver_alive,
            observation.caller_alive,
            observation.listener_alive,
        )
    )
    unrecovered = sum(link.health != "healthy" for link in final.document.connections)
    return SoakResult(
        candidate=run.candidate.label,
        crashes=crashes,
        unrecovered_links=unrecovered,
        final_health=final.document.connections,
        goodput_60s_buckets=tuple(rates),
        monotonic_goodput_decline=decline,
        passed=crashes == 0 and unrecovered == 0 and not decline,
    )


def reduce(root: Path) -> Supplements:
    manifests = Path(__file__).parent / "manifests"
    matrix = Manifest.model_validate_json(
        (manifests / "m4a-ours-new.json").read_bytes()
    )
    summary = build_summary(matrix, (root / "a/results",))
    fec_groups = [g for g in summary.groups if "M4@fec-pair" in g.cell_id]
    if len(fec_groups) != 2 or any(g.cells["enhanced"].n != 3 for g in fec_groups):
        raise EvidenceError("FEC requires two complete N=3 controls")
    off = next(
        g.cells["enhanced"].goodput_median
        for g in fec_groups
        if g.cell_id.endswith("fec:off")
    )
    on = next(
        g.cells["enhanced"].goodput_median
        for g in fec_groups
        if g.cell_id.endswith("fec:on")
    )
    ratio = on / off if off > 0 else None
    fec = FecResult(
        off_goodput_median=off,
        on_goodput_median=on,
        ratio=ratio,
        passed=ratio is not None and ratio > 0.95,
    )
    soak_manifest = Manifest.model_validate_json(
        (manifests / "m4-soak.json").read_bytes()
    )
    build_summary(soak_manifest, (root / "soak/results",))
    soaks: list[SoakResult] = []
    logs: list[LogRate] = []
    for phase in ("a", "soak"):
        for path in sorted((root / phase / "results").rglob("run-*.json")):
            if path.name.endswith(".exhausted.json") or "stale" in path.parts:
                continue
            run = FullRun.model_validate_json(path.read_bytes())
            observation = Observation.model_validate_json(
                (run.raw.stats_csv_path.parent / "m4-observation.json").read_bytes()
            )
            if run.scenario.id == "M8":
                soaks.append(soak_result(run, observation))
            if run.scenario.id in ("G", "M7"):
                if (
                    observation.duration_seconds <= 0
                    or run.candidate.env.get("RUST_LOG") != "info"
                ):
                    raise EvidenceError(
                        "stderr rate requires positive elapsed time and RUST_LOG=info"
                    )
                rate = observation.sender_stderr_lines / observation.duration_seconds
                logs.append(
                    LogRate(
                        scenario=run.scenario.id,
                        candidate=run.candidate.label,
                        run_index=run.run_index,
                        lines=observation.sender_stderr_lines,
                        seconds=observation.duration_seconds,
                        lines_per_second=rate,
                        passed=rate <= 2,
                    )
                )
    if len(soaks) != 2 or {s.candidate for s in soaks} != {"enhanced", "adaptive"}:
        raise EvidenceError("two complete M8 soaks required")
    return Supplements(fec=fec, soaks=tuple(soaks), log_rates=tuple(logs))


if __name__ == "__main__":
    result = reduce(Path(sys.argv[1]))
    publish(Path(sys.argv[2]), result.model_dump_json(indent=2) + "\n")
    print(
        f"FEC pass={result.fec.passed}; M8 pass={all(s.passed for s in result.soaks)}; stderr pass={all(r.passed for r in result.log_rates)}"
    )

#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///

# ─── How to run ───
# 1. Install uv (if not installed):
#      curl -LsSf https://astral.sh/uv/install.sh | sh
# 2. Run directly (no venv, no pip install needed):
#      uv run scripts/bench/report.py --self-test
# 3. Or make executable and run:
#      chmod +x report.py && ./report.py --help
# ──────────────────

from __future__ import annotations

import argparse
import math
import subprocess
import sys
import tempfile
import unittest
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated, ClassVar, Final, Literal, assert_never, override

import numpy as np
from numpy.typing import NDArray
from pydantic import BaseModel, ConfigDict, Field, JsonValue, ValidationError

# allow: SIZE_OK — the task requires a single-file uv script including its schema
# boundary, bootstrap implementation and embedded tests; no local runtime imports.
RESAMPLES: Final = 10_000
SEED: Final = 20260913
type Count = Annotated[int, Field(ge=0, strict=True)]
type Finite = Annotated[float, Field(allow_inf_nan=False)]
type Positive = Annotated[Finite, Field(ge=0)]
type Rate = Annotated[Finite, Field(ge=0, le=1)]
type Number = float | Literal["+inf"] | None
type Hash = Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]


class Document(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(
        frozen=True, strict=True, allow_inf_nan=False
    )


class Cell(Document):
    candidate: str
    scenario: str
    receiver: str
    srt_profile: str
    runs: Annotated[int, Field(gt=0)]

    @property
    def id(self) -> str:
        return f"{self.candidate}--{self.scenario}--{self.receiver}--{self.srt_profile}"


class Candidate(Document):
    label: str
    args: tuple[str, ...] = ()
    env: dict[str, str] = Field(default_factory=dict)
    effective_config: JsonValue = None


class ManifestReceiver(Document):
    name: str
    kind: Literal["ceralive", "irlserver", "belabox"] | None = None


class Manifest(Document):
    campaign: Annotated[str, Field(min_length=1)]
    seed: Count
    cells: Annotated[tuple[Cell, ...], Field(min_length=1)]
    candidates: Annotated[tuple[Candidate, ...], Field(min_length=1)]
    receivers: Annotated[tuple[ManifestReceiver, ...], Field(min_length=1)]


class Envelope(Document):
    schema_version: Literal[1]
    campaign: str
    cell_id: str
    run_index: Count
    status: Literal["ok", "failed", "exhausted"]
    reason: str | None = None


class Scenario(Document):
    id: str
    hash: Hash


class RecordedCandidate(Document):
    label: str
    bin_sha256: Hash
    args: tuple[str, ...]
    env: dict[str, str]


class Receiver(Document):
    kind: str
    sha256: Hash


class Profile(Document):
    name: str
    latency_ms: Count
    lossmaxttl: Count


class Window(Document):
    start_ms: int
    end_ms: int


class Sender(Document):
    cpu_ms: Positive
    switch_count: Count | None = None
    effective_config: JsonValue = None


class Link(Document):
    iface: str
    share: Rate


class Episode(Document):
    event_index: Count
    horizon_ms: Count
    restore_ms: int | None
    graded: bool
    recovered: bool
    complete: bool
    failover_ms: Count | None
    recovery_ms: Count | None

    @property
    def duration(self) -> float:
        value = self.recovery_ms if self.restore_ms is not None else self.failover_ms
        return (
            float(value)
            if self.recovered and self.complete and value is not None
            else math.inf
        )


class LoadInterval(Document):
    start_ms: int
    end_ms: int
    offered_bps: Count
    target_bps: Positive
    graded: bool
    no_collapse: bool
    reached_ms: Count | None
    recovered: bool

    @property
    def reached(self) -> bool:
        return self.recovered and self.reached_ms is not None


class RunRecord(Envelope):
    scenario: Scenario
    candidate: RecordedCandidate
    receiver: Receiver
    srt_profile: Profile
    seed: Count
    fingerprint: Hash
    window: Window
    useful_goodput_bps: Positive
    viewer_loss_ratio: Rate
    diagnostics: dict[str, Positive | None]
    per_link: tuple[Link, ...]
    sender: Sender
    episodes: tuple[Episode, ...]
    load_intervals: tuple[LoadInterval, ...]
    loadavg_1m: Positive
    warnings: tuple[str, ...]


class Stats(Document):
    n: int
    missing: int
    mean: Number
    median: Number
    sd: Number
    ci_lower: Number
    ci_upper: Number
    nonfinite_rate: float | None


class Pair(Document):
    n: int
    dropped_indices: tuple[int, ...]
    ci_lower: float | None
    ci_upper: float | None
    viewer_loss_delta_pp: float
    recovery_ratio: float | None
    nonfinite_recovery: bool
    episode_mismatch: bool


class EpisodeStats(Document):
    event_index: int
    graded: bool
    horizon_ms: int
    failover_ms: Stats
    recovery_ms: Stats
    recovered_rate: float
    nonrecovered_rate: float


class LoadStats(Document):
    interval_index: int
    graded: bool
    offered_bps: int
    target_bps: float
    no_collapse_rate: float
    reached_ms: Stats
    recovered_rate: float
    failure_rate: float


class Evidence(Document):
    n: int
    run_indices: tuple[int, ...]
    goodput_median: float
    viewer_loss_median: float
    graded_episodes: int
    nonrecovered_rate: float | None
    switch_count: float | None
    comparisons: dict[str, Pair]
    integrity_errors: tuple[str, ...]
    checks: dict[str, float | None]
    metrics: dict[str, Stats]
    episodes: tuple[EpisodeStats, ...]
    load_intervals: tuple[LoadStats, ...]
    fingerprints: tuple[str, ...]


class Group(Document):
    campaign: str
    scenario: str
    receiver: str
    profile: str
    cells: dict[str, Evidence]


class Summary(Document):
    schema_version: Literal[1] = 1
    bootstrap_resamples: int = RESAMPLES
    bootstrap_seed: int = SEED
    bootstrap_statistic: str = "median; paired ratio of medians"
    groups: tuple[Group, ...]
    warnings: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class EvidenceError(Exception):
    detail: str

    @override
    def __str__(self) -> str:
        return self.detail


@dataclass(frozen=True, slots=True)
class LoadedRecords:
    by_cell: dict[str, tuple[RunRecord, ...]]
    warnings: tuple[str, ...]


def number(value: float) -> Number:
    return (
        "+inf"
        if math.isinf(value) and value > 0
        else value
        if math.isfinite(value)
        else None
    )


def statistics(values: Sequence[float | None]) -> Stats:
    data = np.array([value for value in values if value is not None], dtype=np.float64)
    n = len(data)
    if not n:
        return Stats(
            n=0,
            missing=len(values),
            mean=None,
            median=None,
            sd=None,
            ci_lower=None,
            ci_upper=None,
            nonfinite_rate=None,
        )
    indices = np.random.default_rng(SEED).integers(n, size=(RESAMPLES, n))
    medians = np.median(data[indices], axis=1)
    # Order-statistic quantiles avoid inf-inf interpolation for censored episodes.
    ordered: NDArray[np.float64] = np.sort(medians)
    return Stats(
        n=n,
        missing=len(values) - n,
        mean=number(float(np.mean(data))),
        median=number(float(np.median(data))),
        sd=float(np.std(data, ddof=1)) if n > 1 and np.isfinite(data).all() else None,
        ci_lower=number(ordered.item(math.ceil(RESAMPLES * 0.025) - 1)),
        ci_upper=number(ordered.item(math.ceil(RESAMPLES * 0.975) - 1)),
        nonfinite_rate=float(np.mean(~np.isfinite(data))),
    )


def paired(left: Sequence[RunRecord], right: Sequence[RunRecord]) -> Pair:
    a, b = {r.run_index: r for r in left}, {r.run_index: r for r in right}
    matched = sorted(a.keys() & b.keys())
    dropped = tuple(sorted(a.keys() ^ b.keys()))
    if not matched:
        return Pair(
            n=0,
            dropped_indices=dropped,
            ci_lower=None,
            ci_upper=None,
            viewer_loss_delta_pp=0,
            recovery_ratio=None,
            nonfinite_recovery=True,
            episode_mismatch=True,
        )
    x = np.array([a[i].useful_goodput_bps for i in matched])
    y = np.array([b[i].useful_goodput_bps for i in matched])
    indices = np.random.default_rng(SEED).integers(
        len(matched), size=(RESAMPLES, len(matched))
    )
    denominator = np.median(y[indices], axis=1)
    ci = (None, None)
    if np.all(denominator > 0):
        ratios = np.median(x[indices], axis=1) / denominator
        ordered: NDArray[np.float64] = np.sort(ratios)
        ci = (
            ordered.item(math.ceil(RESAMPLES * 0.025) - 1),
            ordered.item(math.ceil(RESAMPLES * 0.975) - 1),
        )
    recovery: list[float] = []
    nonfinite = False
    mismatch = False
    for i in matched:
        first, second = a[i], b[i]
        mismatch |= (
            first.seed != second.seed
            or first.scenario != second.scenario
            or first.receiver != second.receiver
            or first.srt_profile != second.srt_profile
            or first.window != second.window
        )
        ea = {e.event_index: e for e in first.episodes if e.graded and e.horizon_ms > 0}
        eb = {
            e.event_index: e for e in second.episodes if e.graded and e.horizon_ms > 0
        }
        mismatch |= ea.keys() != eb.keys()
        for event in ea.keys() & eb.keys():
            left_ms, right_ms = ea[event].duration, eb[event].duration
            mismatch |= ea[event].horizon_ms != eb[event].horizon_ms
            mismatch |= (ea[event].restore_ms is None) != (eb[event].restore_ms is None)
            nonfinite |= not math.isfinite(left_ms) and math.isfinite(right_ms)
            if left_ms == right_ms == 0:
                recovery.append(1.0)
            elif right_ms > 0 and math.isfinite(left_ms):
                recovery.append(left_ms / right_ms)
            else:
                recovery.append(math.inf)
    ratio = float(np.median(recovery)) if recovery else math.inf
    return Pair(
        n=len(matched),
        dropped_indices=dropped,
        ci_lower=ci[0],
        ci_upper=ci[1],
        viewer_loss_delta_pp=100
        * (
            float(np.median([r.viewer_loss_ratio for r in left]))
            - float(np.median([r.viewer_loss_ratio for r in right]))
        ),
        recovery_ratio=ratio if math.isfinite(ratio) else None,
        nonfinite_recovery=nonfinite,
        episode_mismatch=mismatch,
    )


def gini(record: RunRecord) -> float | None:
    shares = np.array([link.share for link in record.per_link], dtype=np.float64)
    total = float(np.sum(shares))
    if not total:
        return None
    differences: NDArray[np.float64] = np.abs(shares[:, None] - shares)
    return float(np.sum(differences)) / (2 * len(shares) * total)


def run_metrics(records: Sequence[RunRecord]) -> dict[str, Stats]:
    metrics = {
        "useful_goodput_bps": statistics([r.useful_goodput_bps for r in records]),
        "viewer_loss_ratio": statistics([r.viewer_loss_ratio for r in records]),
        "per_link_share_gini": statistics([gini(r) for r in records]),
        "cpu_ms_per_mb": statistics(
            [
                r.sender.cpu_ms
                / (r.useful_goodput_bps * (r.window.end_ms - r.window.start_ms) / 8e9)
                if r.useful_goodput_bps > 0
                else math.inf
                for r in records
            ]
        ),
        "switch_count": statistics([r.sender.switch_count for r in records]),
    }
    names = sorted({name for r in records for name in r.diagnostics})
    metrics.update(
        {
            f"diagnostics.{name}": statistics(
                [r.diagnostics.get(name) for r in records]
            )
            for name in names
        }
    )
    return metrics


def episode_statistics(records: Sequence[RunRecord]) -> tuple[EpisodeStats, ...]:
    results: list[EpisodeStats] = []
    for event in sorted({e.event_index for r in records for e in r.episodes}):
        observations = [
            e for r in records for e in r.episodes if e.event_index == event
        ]
        first = observations[0]
        passed = sum(e.recovered and e.complete for e in observations) / len(records)
        timings: list[Stats] = []
        for restored in (False, True):
            values = (
                e.recovery_ms if restored else e.failover_ms for e in observations
            )
            timings.append(
                statistics(
                    [
                        float(value)
                        if value is not None and e.recovered and e.complete
                        else math.inf
                        if not e.recovered or not e.complete
                        else None
                        for e, value in zip(observations, values, strict=True)
                    ]
                    + [math.inf] * (len(records) - len(observations))
                )
            )
        results.append(
            EpisodeStats(
                event_index=event,
                graded=first.graded,
                horizon_ms=first.horizon_ms,
                failover_ms=timings[0],
                recovery_ms=timings[1],
                recovered_rate=passed,
                nonrecovered_rate=1 - passed,
            )
        )
    return tuple(results)


def load_statistics(records: Sequence[RunRecord]) -> tuple[LoadStats, ...]:
    results: list[LoadStats] = []
    for index in range(max(len(r.load_intervals) for r in records)):
        intervals = [
            r.load_intervals[index] for r in records if index < len(r.load_intervals)
        ]
        first = intervals[0]
        passed = sum(i.reached for i in intervals) / len(records)
        results.append(
            LoadStats(
                interval_index=index,
                graded=first.graded,
                offered_bps=first.offered_bps,
                target_bps=first.target_bps,
                no_collapse_rate=sum(i.no_collapse for i in intervals) / len(records),
                reached_ms=statistics(
                    [
                        float(i.reached_ms)
                        if i.reached_ms is not None and i.reached
                        else math.inf
                        for i in intervals
                    ]
                    + [math.inf] * (len(records) - len(intervals))
                ),
                recovered_rate=passed,
                failure_rate=1 - passed,
            )
        )
    return tuple(results)


def checks(records: Sequence[RunRecord]) -> dict[str, float | None]:
    results: dict[str, list[bool]] = {}
    for record in records:
        match record.scenario.id:
            case "J":
                restored = [
                    e
                    for e in record.episodes
                    if e.horizon_ms > 0 and e.restore_ms is not None
                ]
                results.setdefault("post_restore_recovered_rate", []).append(
                    bool(restored)
                    and all(
                        e.recovered and e.complete and e.recovery_ms is not None
                        for e in restored
                    )
                )
            case "L":
                loads = record.load_intervals
                idle = [i for i, load in enumerate(loads) if load.offered_bps == 0]
                overload = [
                    load
                    for i, load in enumerate(loads)
                    if idle and i < idle[0] and load.graded
                ]
                burst = [
                    load
                    for i, load in enumerate(loads)
                    if idle and i > idle[-1] and load.graded
                ]
                results.setdefault("overload_no_collapse_rate", []).append(
                    bool(overload) and all(load.no_collapse for load in overload)
                )
                results.setdefault("burst_recovered_rate", []).append(
                    bool(burst) and all(load.reached for load in burst)
                )
            case _:
                # Scenario identifiers are open manifest strings, not a closed enum.
                continue
    return {name: sum(values) / len(records) for name, values in results.items()}


def summarize_cell(records: Sequence[RunRecord], expected: Candidate) -> Evidence:
    errors: list[str] = []
    if len({r.fingerprint for r in records}) != 1:
        errors.append("config mismatch: mixed fingerprints")
    if len({r.candidate.model_dump_json() for r in records}) != 1:
        errors.append("config mismatch: mixed candidate binaries/arguments/environment")
    if len({(r.scenario, r.receiver, r.srt_profile, r.window) for r in records}) != 1:
        errors.append("config mismatch: mixed scenario/receiver/profile/window")
    env = {"RUST_LOG": "info", "PATH": "/usr/bin:/bin", **expected.env}
    for record in records:
        if (
            record.sender.effective_config != expected.effective_config
            or record.candidate.args != expected.args
            or record.candidate.env != env
        ):
            errors.append(f"config mismatch: run_index={record.run_index}")
    episode_shapes = {
        tuple(
            (e.event_index, e.graded, e.horizon_ms, e.restore_ms is not None)
            for e in r.episodes
        )
        for r in records
    }
    if len(episode_shapes) != 1:
        errors.append("inconsistent episode shape")
    if (
        len(
            {
                tuple((i.graded, i.offered_bps, i.target_bps) for i in r.load_intervals)
                for r in records
            }
        )
        != 1
    ):
        errors.append("inconsistent load interval shape")
    graded = [e for r in records for e in r.episodes if e.graded and e.horizon_ms > 0]
    if records[0].scenario.id in ("C", "D", "E", "H", "K") and not graded:
        errors.append("missing graded episode evidence")
    metrics = run_metrics(records)
    return Evidence(
        n=len(records),
        run_indices=tuple(r.run_index for r in records),
        goodput_median=float(np.median([r.useful_goodput_bps for r in records])),
        viewer_loss_median=float(np.median([r.viewer_loss_ratio for r in records])),
        graded_episodes=len(graded),
        nonrecovered_rate=sum(not math.isfinite(e.duration) for e in graded)
        / len(graded)
        if graded
        else None,
        switch_count=float(
            np.median(
                [
                    r.sender.switch_count
                    for r in records
                    if r.sender.switch_count is not None
                ]
            )
        )
        if all(r.sender.switch_count is not None for r in records)
        else None,
        comparisons={},
        integrity_errors=tuple(errors),
        checks=checks(records),
        metrics=metrics,
        episodes=episode_statistics(records),
        load_intervals=load_statistics(records),
        fingerprints=tuple(sorted({r.fingerprint for r in records})),
    )


def load_records(
    manifest: Manifest, roots: Sequence[Path], *, smoke_coverage: bool = False
) -> LoadedRecords:
    if smoke_coverage:
        required = {
            (name, "A", "ceralive", "production", 2) for name in ("classic", "enhanced")
        }
        actual = {
            (c.candidate, c.scenario, c.receiver, c.srt_profile, c.runs)
            for c in manifest.cells
        }
        if (
            manifest.campaign != "smoke"
            or manifest.seed != 1
            or len(manifest.cells) != 2
            or actual != required
        ):
            raise EvidenceError(
                "smoke coverage requires the two canonical classic/enhanced A cells, runs=2, seed=1"
            )
    expected = {cell.id: cell for cell in manifest.cells}
    if len(expected) != len(manifest.cells):
        raise EvidenceError("duplicate manifest cells")
    receivers = {r.name: r for r in manifest.receivers}
    if len(receivers) != len(manifest.receivers) or any(
        c.receiver not in receivers for c in manifest.cells
    ):
        raise EvidenceError("duplicate/unresolved manifest receiver")
    presets = {"production": (2000, 40), "strict": (500, 10), "legacy-default": (0, 0)}
    for root in roots:
        if not root.is_dir():
            raise EvidenceError(f"results directory not found: {root}")
    records: dict[str, dict[int, RunRecord]] = {key: {} for key in expected}
    warnings: list[str] = []
    if smoke_coverage:
        warnings.append(
            "Smoke coverage calibration: at least one of two successes per required cell; original indices retained; not a performance or C1/C2 acceptance verdict"
        )
    paths = {
        path.resolve()
        for root in roots
        for path in root.rglob("*.json")
        if not {"stale", "artifacts", "raw"}.intersection(path.relative_to(root).parts)
        and path.name != "manifest.json"
    }
    for path in sorted(paths):
        text = path.read_text(encoding="utf-8")
        envelope = Envelope.model_validate_json(text)
        if envelope.campaign != manifest.campaign or envelope.cell_id not in expected:
            raise EvidenceError(
                f"unexpected cell {envelope.campaign}/{envelope.cell_id}: {path}"
            )
        match envelope.status:
            case "failed" | "exhausted":
                warnings.append(
                    f"{envelope.cell_id} run {envelope.run_index}: "
                    + f"{envelope.status} ({envelope.reason or 'unspecified'})"
                )
                continue
            case "ok":
                record = RunRecord.model_validate_json(text)
            case _:
                assert_never(envelope.status)
        cell = expected[record.cell_id]
        if smoke_coverage and record.run_index >= cell.runs:
            raise EvidenceError(f"unexpected run index {record.run_index} in {cell.id}")
        receiver = receivers[cell.receiver]
        kind = receiver.kind or (
            receiver.name
            if receiver.name in ("ceralive", "irlserver", "belabox")
            else None
        )
        if (
            record.candidate.label != cell.candidate
            or record.scenario.id != cell.scenario
            or record.srt_profile.name != cell.srt_profile
            or record.seed != manifest.seed
            or (record.srt_profile.latency_ms, record.srt_profile.lossmaxttl)
            != presets.get(cell.srt_profile)
            or kind is not None
            and record.receiver.kind != kind
            or record.window.end_ms <= record.window.start_ms
        ):
            raise EvidenceError(
                f"identity/window mismatch {cell.id}, run_index={record.run_index}"
            )
        if record.run_index in records[cell.id]:
            raise EvidenceError(
                f"duplicate ok run_index={record.run_index} in {cell.id}"
            )
        if len({e.event_index for e in record.episodes}) != len(record.episodes):
            raise EvidenceError(f"duplicate event_index in {cell.id}")
        records[cell.id][record.run_index] = record
    missing = [
        f"{cell.id}: n={len(records[cell.id])}, required={1 if smoke_coverage else cell.runs}, "
        + f"missing run_indices={sorted(set(range(cell.runs)) - records[cell.id].keys())}"
        for cell in manifest.cells
        if len(records[cell.id]) < (1 if smoke_coverage else cell.runs)
        or (not smoke_coverage and not set(range(cell.runs)) <= records[cell.id].keys())
    ]
    if missing:
        raise EvidenceError("missing/insufficient cell(s): " + "; ".join(missing))
    return LoadedRecords(
        {key: tuple(runs[i] for i in sorted(runs)) for key, runs in records.items()},
        tuple(warnings),
    )


def build_summary(
    manifest: Manifest, roots: Sequence[Path], *, smoke_coverage: bool = False
) -> Summary:
    loaded = load_records(manifest, roots, smoke_coverage=smoke_coverage)
    records = loaded.by_cell
    candidates = {c.label: c for c in manifest.candidates}
    if len(candidates) != len(manifest.candidates) or any(
        c.candidate not in candidates for c in manifest.cells
    ):
        raise EvidenceError("duplicate/unresolved manifest candidate")
    groups: list[Group] = []
    warnings = list(loaded.warnings)
    contexts = sorted({(c.scenario, c.receiver, c.srt_profile) for c in manifest.cells})
    for scenario, receiver, profile in contexts:
        arms = {
            c.candidate: records[c.id]
            for c in manifest.cells
            if (c.scenario, c.receiver, c.srt_profile) == (scenario, receiver, profile)
        }
        cells: dict[str, Evidence] = {}
        for name, runs in sorted(arms.items()):
            cell = summarize_cell(runs, candidates[name])
            comparisons = {
                other: paired(runs, right) for other, right in sorted(arms.items())
            }
            cells[name] = cell.model_copy(update={"comparisons": comparisons})
            label = f"{manifest.campaign}/{scenario}/{receiver}/{profile}/{name}"
            warnings.extend(f"{label}: {error}" for error in cell.integrity_errors)
            for other, pair in comparisons.items():
                if pair.dropped_indices:
                    warnings.append(
                        f"{label} vs {other}: dropped pairs {pair.dropped_indices}"
                    )
                if pair.episode_mismatch:
                    warnings.append(f"{label} vs {other}: config/episode mismatch")
            for record in runs:
                warnings.extend(
                    f"{label} run {record.run_index}: {w}" for w in record.warnings
                )
                if record.loadavg_1m > 2:
                    warnings.append(
                        f"{label} run {record.run_index}: loadavg>2 ({record.loadavg_1m})"
                    )
        groups.append(
            Group(
                campaign=manifest.campaign,
                scenario=scenario,
                receiver=receiver,
                profile=profile,
                cells=cells,
            )
        )
    return Summary(groups=tuple(groups), warnings=tuple(sorted(set(warnings))))


def markdown(summary: Summary) -> str:
    lines = [
        "# Scheduler benchmark report",
        "",
        f"Bootstrap: {RESAMPLES} resamples; seed {SEED}; 95% median CI.",
        "Censored failure durations are +inf; null means unavailable, not zero.",
    ]
    for group in summary.groups:
        lines.extend(
            (
                "",
                f"## {group.scenario} — {group.campaign} / {group.receiver} / {group.profile}",
                "",
                "| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |",
                "|---|---|---:|---:|---:|---:|---:|---|---:|",
            )
        )
        for name, cell in group.cells.items():
            metrics = dict(cell.metrics)
            for episode in cell.episodes:
                metrics[f"episode[{episode.event_index}].failover_ms"] = (
                    episode.failover_ms
                )
                metrics[f"episode[{episode.event_index}].recovery_ms"] = (
                    episode.recovery_ms
                )
            for interval in cell.load_intervals:
                metrics[f"load[{interval.interval_index}].reached_ms"] = (
                    interval.reached_ms
                )
            for metric, stat in metrics.items():
                lines.append(
                    f"| {name} | {metric} | {stat.n} | {stat.missing} | {stat.mean} | {stat.median} | "
                    + f"{stat.sd} | [{stat.ci_lower}, {stat.ci_upper}] | {stat.nonfinite_rate} |"
                )
        lines.extend(
            (
                "",
                "| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |",
                "|---|---|---:|---:|---:|",
            )
        )
        for name, cell in group.cells.items():
            for episode in cell.episodes:
                lines.append(
                    f"| {name} | episode[{episode.event_index}] graded={episode.graded} | — | "
                    + f"{episode.recovered_rate} | {episode.nonrecovered_rate} |"
                )
            for interval in cell.load_intervals:
                lines.append(
                    f"| {name} | load[{interval.interval_index}] graded={interval.graded} | "
                    + f"{interval.no_collapse_rate} | {interval.recovered_rate} | {interval.failure_rate} |"
                )
            for check, rate in cell.checks.items():
                lines.append(f"| {name} | {check} | — | {rate} | — |")
        lines.extend(
            (
                "",
                "| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |",
                "|---|---|---:|---|---:|---:|---|",
            )
        )
        for name, cell in group.cells.items():
            for other, pair in cell.comparisons.items():
                lines.append(
                    f"| {name} | {other} | {pair.n} | [{pair.ci_lower}, {pair.ci_upper}] | "
                    + f"{pair.viewer_loss_delta_pp} | {pair.recovery_ratio} | {pair.dropped_indices} |"
                )
    lines.extend(("", "## Warnings", "", *(f"- {w}" for w in summary.warnings)))
    if not summary.warnings:
        lines.append("None.")
    return "\n".join(lines) + "\n"


class ReportTests(unittest.TestCase):
    RECORD: Final = r"""{
      "schema_version":1,"campaign":"synthetic","cell_id":"adaptive--A--ceralive--production","run_index":0,"status":"ok",
      "scenario":{"id":"A","hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
      "candidate":{"label":"adaptive","bin_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","args":[],"env":{"PATH":"/usr/bin:/bin","RUST_LOG":"info"}},
      "receiver":{"kind":"ceralive","sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"},
      "srt_profile":{"name":"production","latency_ms":2000,"lossmaxttl":40},"seed":42,
      "fingerprint":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
      "window":{"start_ms":0,"end_ms":60000},"useful_goodput_bps":1000,"viewer_loss_ratio":0,
      "diagnostics":{"pkt_belated_sum":0,"reorder_distance_max":null},"per_link":[{"iface":"link0","share":0.5},{"iface":"link1","share":0.5}],
      "sender":{"cpu_ms":100,"switch_count":2},"episodes":[],"load_intervals":[],"loadavg_1m":0.5,"warnings":[],
      "raw":{"sink_series":[{"t_ms":40000,"duration_ms":1000,"bytes":125,"pkts":1}]}
    }"""

    def record(self, scenario: str = "A") -> RunRecord:
        return RunRecord.model_validate_json(
            self.RECORD.replace('"A"', f'"{scenario}"').replace(
                "--A--", f"--{scenario}--"
            )
        )

    def manifest(self, scenarios: tuple[str, ...] = ("A",)) -> Manifest:
        return Manifest(
            campaign="synthetic",
            seed=42,
            candidates=(Candidate(label="adaptive"),),
            receivers=(ManifestReceiver(name="ceralive"),),
            cells=tuple(
                Cell(
                    candidate="adaptive",
                    scenario=s,
                    receiver="ceralive",
                    srt_profile="production",
                    runs=1,
                )
                for s in scenarios
            ),
        )

    def test_pairing_preserves_correlation_when_runs_vary(self) -> None:
        # Given perfectly correlated paired measurements with very different loads.
        baseline = self.record()
        right = tuple(
            baseline.model_copy(update={"run_index": i, "useful_goodput_bps": v})
            for i, v in enumerate((100.0, 1000.0, 10000.0, 100000.0))
        )
        left = tuple(
            r.model_copy(update={"useful_goodput_bps": r.useful_goodput_bps * 0.96})
            for r in right
        )
        # When bootstrapping matched indices.
        pair = paired(left, right)
        # Then correlation cancels; an unpaired bootstrap would produce a wide CI.
        assert pair.ci_lower is not None and pair.ci_upper is not None
        self.assertAlmostEqual(pair.ci_lower, 0.96)
        self.assertAlmostEqual(pair.ci_upper, 0.96)

    def test_drops_pair_when_either_arm_is_missing(self) -> None:
        # Given disjoint missing indices in two arms.
        record = self.record()
        left = (record, record.model_copy(update={"run_index": 1}))
        right = (record, record.model_copy(update={"run_index": 2}))
        # When paired, then only the common run contributes.
        pair = paired(left, right)
        self.assertEqual((pair.n, pair.dropped_indices), (1, (1, 2)))

    def test_optional_switch_counter_when_sender_does_not_support_metrics(self) -> None:
        # Given the producer omits unsupported counters rather than writing null.
        raw = self.RECORD.replace(',"switch_count":2', "")
        # When the real JSON boundary parses it, then the counter remains unknown.
        self.assertIsNone(RunRecord.model_validate_json(raw).sender.switch_count)

    def test_failed_attempt_and_stale_archive_never_inflate_n(self) -> None:
        # Given one success, a failed attempt, and a stale copy of the success.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _ = (root / "run-0.json").write_text(self.RECORD, encoding="utf-8")
            _ = (root / "run-0.failed-1.json").write_text(
                self.RECORD.replace('"status":"ok"', '"status":"failed"'),
                encoding="utf-8",
            )
            (root / "stale").mkdir()
            _ = (root / "stale" / "run-0.json").write_text(
                self.RECORD, encoding="utf-8"
            )
            # When all results directories are loaded, then only the current success counts.
            result = build_summary(self.manifest(), (root,))
            self.assertEqual(result.groups[0].cells["adaptive"].n, 1)
            self.assertTrue(any("failed" in w for w in result.warnings))

    def test_duplicate_success_is_rejected(self) -> None:
        # Given two physical records claiming the same paired index.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("one.json", "two.json"):
                _ = (root / name).write_text(self.RECORD, encoding="utf-8")
            # When loaded, then duplicate measurements cannot inflate N.
            with self.assertRaisesRegex(EvidenceError, "duplicate ok run_index=0"):
                _ = build_summary(self.manifest(), (root,))

    def test_required_runs_are_per_explicit_cell(self) -> None:
        # Given one ok record but a cell requiring two runs.
        source = self.manifest()
        manifest = source.model_copy(
            update={"cells": (source.cells[0].model_copy(update={"runs": 2}),)}
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _ = (root / "run-0.json").write_text(self.RECORD, encoding="utf-8")
            # When summarized, then the authoritative cell's count blocks publication.
            with self.assertRaisesRegex(
                EvidenceError, "adaptive--A--ceralive--production.*required=2"
            ):
                _ = build_summary(manifest, (root,))

    def smoke_fixture(self, root: Path) -> Manifest:
        manifest = Manifest(
            campaign="smoke",
            seed=1,
            candidates=tuple(Candidate(label=name) for name in ("classic", "enhanced")),
            receivers=(ManifestReceiver(name="ceralive"),),
            cells=tuple(
                Cell(
                    candidate=name,
                    scenario="A",
                    receiver="ceralive",
                    srt_profile="production",
                    runs=2,
                )
                for name in ("classic", "enhanced")
            ),
        )
        for cell in manifest.cells:
            base = self.record(cell.scenario)
            record = base.model_copy(
                update={
                    "campaign": "smoke",
                    "seed": 1,
                    "cell_id": cell.id,
                    "run_index": 1,
                    "candidate": base.candidate.model_copy(
                        update={"label": cell.candidate}
                    ),
                }
            )
            _ = (root / f"{cell.id}.json").write_text(
                record.model_dump_json(), encoding="utf-8"
            )
        return manifest

    def test_smoke_coverage_preserves_original_nonzero_indices(self) -> None:
        # Given only index1 succeeds in each of the two required smoke cells.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.smoke_fixture(root)
            # When the explicit smoke coverage policy reduces the records.
            summary = build_summary(manifest, (root,), smoke_coverage=True)
            # Then every retained sample keeps its original index and actual count.
            for group in summary.groups:
                for cell in group.cells.values():
                    self.assertEqual(cell.n, 1)
                    self.assertEqual(cell.run_indices, (1,))

    def test_default_reporting_still_requires_both_smoke_indices(self) -> None:
        # Given the same partial data but no explicit calibration flag.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.smoke_fixture(root)
            # When the default reporter runs, then full manifest completeness still applies.
            with self.assertRaisesRegex(EvidenceError, "missing/insufficient"):
                _ = build_summary(manifest, (root,))

    def test_smoke_coverage_rejects_zero_coverage(self) -> None:
        # Given one valid sample per required cell.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.smoke_fixture(root)
            path = root / f"{manifest.cells[0].id}.json"
            path.unlink()
            # When a cell has no sample, then calibration cannot manufacture coverage.
            with self.assertRaisesRegex(EvidenceError, "missing/insufficient"):
                _ = build_summary(manifest, (root,), smoke_coverage=True)

    def test_smoke_coverage_rejects_unplanned_indices(self) -> None:
        # Given valid smoke data, when an index exceeds the planned pair, then reject it.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.smoke_fixture(root)
            path = root / f"{manifest.cells[0].id}.json"
            record = RunRecord.model_validate_json(path.read_text(encoding="utf-8"))
            _ = path.write_text(
                record.model_copy(update={"run_index": 2}).model_dump_json(),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(EvidenceError, "unexpected run index"):
                _ = build_summary(manifest, (root,), smoke_coverage=True)

    def test_smoke_coverage_is_unavailable_to_other_campaigns(self) -> None:
        # Given a non-smoke campaign, when requesting calibration, then refuse it.
        with (
            tempfile.TemporaryDirectory() as directory,
            self.assertRaisesRegex(EvidenceError, "smoke coverage"),
        ):
            _ = build_summary(self.manifest(), (Path(directory),), smoke_coverage=True)

    def test_smoke_coverage_cli_emits_actual_indices(self) -> None:
        # Given a canonical partial smoke fixture with only index1 in each cell.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            results = root / "results"
            results.mkdir()
            manifest = self.smoke_fixture(results)
            manifest_path = root / "manifest.json"
            _ = manifest_path.write_text(manifest.model_dump_json(), encoding="utf-8")
            summary_path = root / "summary.json"
            # When the actual CLI explicitly requests the smoke-only calibration.
            process = subprocess.run(
                [
                    sys.executable,
                    __file__,
                    "--smoke-coverage",
                    "--results",
                    str(results),
                    "--manifest",
                    str(manifest_path),
                    "--out",
                    str(root / "report.md"),
                    "--json",
                    str(summary_path),
                ],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Then publication preserves each measured index rather than renumbering.
            self.assertEqual(process.returncode, 0, process.stderr)
            summary = Summary.model_validate_json(
                summary_path.read_text(encoding="utf-8")
            )
            for group in summary.groups:
                for cell in group.cells.values():
                    self.assertEqual((cell.n, cell.run_indices), (1, (1,)))

    def test_mismatched_configuration_and_load_are_reported(self) -> None:
        # Given an observed override different from the manifest plus a busy host.
        record = self.record().model_copy(
            update={
                "loadavg_1m": 3.0,
                "sender": Sender(
                    cpu_ms=100,
                    switch_count=2,
                    effective_config={"adaptive_features": {"probe": False}},
                ),
            }
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _ = (root / "run-0.json").write_text(
                record.model_dump_json(), encoding="utf-8"
            )
            # When summarized, then warnings and the decision-facing integrity flag survive.
            summary = build_summary(self.manifest(), (root,))
            self.assertTrue(summary.groups[0].cells["adaptive"].integrity_errors)
            self.assertTrue(any("loadavg>2" in w for w in summary.warnings))
            self.assertTrue(any("config mismatch" in w for w in summary.warnings))

    def test_cpu_units_and_gini_when_payload_is_known(self) -> None:
        # Given 60 seconds at 1000 useful bits/s and 100ms CPU (0.0075 MB).
        record = self.record()
        # When metrics are derived, then decimal megabytes and equal link shares are used.
        metrics = run_metrics((record,))
        self.assertEqual(metrics["cpu_ms_per_mb"].median, 100 / 0.0075)
        self.assertEqual(metrics["per_link_share_gini"].median, 0)

    def test_missing_cell_cli_exits_without_summary(self) -> None:
        # Given an explicit A/J matrix with only A present.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            results = root / "results"
            results.mkdir()
            _ = (results / "run-0.json").write_text(self.RECORD, encoding="utf-8")
            manifest = root / "manifest.json"
            _ = manifest.write_text(
                self.manifest(("A", "J")).model_dump_json(), encoding="utf-8"
            )
            output = root / "summary.json"
            _ = output.write_text('{"old_success":true}', encoding="utf-8")
            # When the standalone CLI loads the incomplete matrix.
            process = subprocess.run(
                [
                    sys.executable,
                    __file__,
                    "--results",
                    str(results),
                    "--manifest",
                    str(manifest),
                    "--out",
                    str(root / "report.md"),
                    "--json",
                    str(output),
                ],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Then it names the missing cell and publishes no summary.
            self.assertNotEqual(process.returncode, 0)
            self.assertIn("adaptive--J--ceralive--production", process.stderr)
            self.assertFalse(output.exists())

    def test_summary_decision_roundtrip_preserves_j_and_l(self) -> None:
        # Given a recovered J post-restore episode and L's 1kbit/s trickling burst.
        j = self.record("J").model_copy(
            update={
                "episodes": (
                    Episode(
                        event_index=0,
                        horizon_ms=30000,
                        restore_ms=23000,
                        graded=False,
                        recovered=True,
                        complete=True,
                        failover_ms=5000,
                        recovery_ms=2000,
                    ),
                    Episode(
                        event_index=1,
                        horizon_ms=0,
                        restore_ms=None,
                        graded=False,
                        recovered=False,
                        complete=False,
                        failover_ms=None,
                        recovery_ms=None,
                    ),
                )
            }
        )
        load_record = self.record("L").model_copy(
            update={
                "load_intervals": (
                    LoadInterval(
                        start_ms=0,
                        end_ms=20000,
                        offered_bps=20000000,
                        target_bps=14400000,
                        graded=True,
                        no_collapse=True,
                        reached_ms=1000,
                        recovered=True,
                    ),
                    LoadInterval(
                        start_ms=20000,
                        end_ms=30000,
                        offered_bps=0,
                        target_bps=0,
                        graded=False,
                        no_collapse=False,
                        reached_ms=None,
                        recovered=False,
                    ),
                    LoadInterval(
                        start_ms=30000,
                        end_ms=60000,
                        offered_bps=10000000,
                        target_bps=9000000,
                        graded=True,
                        no_collapse=False,
                        reached_ms=None,
                        recovered=False,
                    ),
                )
            }
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            results = root / "results"
            results.mkdir()
            for record in (self.record(), j, load_record):
                _ = (results / f"{record.cell_id}.json").write_text(
                    record.model_dump_json(), encoding="utf-8"
                )
            source = root / "manifest.json"
            _ = source.write_text(
                self.manifest(("A", "J", "L")).model_dump_json(), encoding="utf-8"
            )
            summary_path, verdict_path = root / "summary.json", root / "verdict.json"
            # When serialized records traverse BOTH production CLIs.
            report = subprocess.run(
                [
                    sys.executable,
                    __file__,
                    "--results",
                    str(results),
                    "--manifest",
                    str(source),
                    "--out",
                    str(root / "report.md"),
                    "--json",
                    str(summary_path),
                ],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            self.assertEqual(report.returncode, 0, report.stderr)
            decision = subprocess.run(
                [
                    sys.executable,
                    str(Path(__file__).with_name("decide.py")),
                    "--summary",
                    str(summary_path),
                    "--out",
                    str(verdict_path),
                    "--rule",
                    "d1",
                    "--candidates",
                    "adaptive",
                    "--scenarios",
                    "A,J,L",
                    "--n",
                    "1",
                ],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Then J passes, the burst fails, and idle creates no decision obligation.
            self.assertEqual(decision.returncode, 0, decision.stderr)
            summary = Summary.model_validate_json(
                summary_path.read_text(encoding="utf-8")
            )
            by_scenario = {g.scenario: g.cells["adaptive"] for g in summary.groups}
            self.assertEqual(
                by_scenario["J"].checks, {"post_restore_recovered_rate": 1.0}
            )
            self.assertEqual(
                by_scenario["L"].checks,
                {"overload_no_collapse_rate": 1.0, "burst_recovered_rate": 0.0},
            )
            self.assertEqual(by_scenario["L"].load_intervals[2].recovered_rate, 0)
            self.assertEqual(
                by_scenario["L"].load_intervals[2].reached_ms.median, "+inf"
            )
            self.assertNotIn("idle", " ".join(by_scenario["L"].checks))

            class CheckVerdict(Document):
                reported_checks: dict[str, dict[str, dict[str, float | None]]]

            verdict = CheckVerdict.model_validate_json(
                verdict_path.read_text(encoding="utf-8")
            )
            self.assertEqual(
                verdict.reported_checks["synthetic/L/ceralive/production"]["adaptive"][
                    "burst_recovered_rate"
                ],
                0,
            )

    def test_bootstrap_is_deterministic_when_repeated(self) -> None:
        # Given nonconstant synthetic measurements.
        values = (1.0, 3.0, 9.0, 20.0)
        # When the fixed-seed bootstrap is repeated.
        first, second = statistics(values), statistics(values)
        # Then every statistic is identical.
        self.assertEqual(first, second)
        self.assertEqual(first.n, 4)
        self.assertEqual(first.median, 6)

    def test_nonrecovered_episode_remains_infinite(self) -> None:
        # Given one recovered observation and two censored failures.
        values = (1000.0, float("inf"), float("inf"))
        # When summarized.
        result = statistics(values)
        # Then failures are retained rather than dropped from the denominator.
        self.assertEqual(result.median, "+inf")
        self.assertEqual(result.nonfinite_rate, 2 / 3)


class Arguments(argparse.Namespace):
    self_test: bool = False
    smoke_coverage: bool = False
    results: list[Path] | None = None
    manifest: Path | None = None
    out: Path | None = None
    json: Path | None = None


def publish(path: Path, text: str) -> None:
    with tempfile.TemporaryDirectory(
        prefix=".bench-report-", dir=path.parent
    ) as directory:
        pending = Path(directory) / path.name
        _ = pending.write_text(text, encoding="utf-8")
        _ = pending.replace(path)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Manifest-complete paired benchmark statistics"
    )
    _ = parser.add_argument("--self-test", action="store_true")
    _ = parser.add_argument(
        "--smoke-coverage",
        action="store_true",
        help="Explicit one-of-two coverage for classic/A and enhanced/A only",
    )
    _ = parser.add_argument("--results", nargs="+", type=Path)
    _ = parser.add_argument("--manifest", type=Path)
    _ = parser.add_argument("--out", type=Path)
    _ = parser.add_argument("--json", type=Path)
    args = parser.parse_args(namespace=Arguments())
    if args.self_test:
        result = unittest.TextTestRunner(verbosity=2).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(ReportTests)
        )
        return int(not result.wasSuccessful())
    if (
        args.results is None
        or args.manifest is None
        or args.out is None
        or args.json is None
    ):
        parser.error("--results, --manifest, --out and --json are required")
    if len({args.manifest.resolve(), args.out.resolve(), args.json.resolve()}) != 3:
        parser.error("manifest and output paths must be distinct")
    try:
        args.json.unlink(missing_ok=True)
        manifest = Manifest.model_validate_json(
            args.manifest.read_text(encoding="utf-8")
        )
        summary = build_summary(
            manifest, args.results, smoke_coverage=args.smoke_coverage
        )
        publish(args.out, markdown(summary))
        publish(args.json, summary.model_dump_json(indent=2) + "\n")
    except (OSError, ValidationError, EvidenceError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

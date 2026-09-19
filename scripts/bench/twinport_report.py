# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.*", "pydantic==2.*"]
# ///
# Run: uv run scripts/bench/twinport_report.py RAW_ROOT OUTPUT_DIRECTORY
import sys
from pathlib import Path

from report import Document, EvidenceError, Hash, publish
from twinport_inputs import Attempt, Observation, digest, load
from twinport_models import Comparison, Spike, decide, paired_delta


class ProfileResult(Document):
    profile: str
    valid_indices_4002: tuple[int, ...]
    valid_indices_4003: tuple[int, ...]
    comparison: Comparison
    discarded_final_attempt_comparison: Comparison


class Summary(Document):
    groups: tuple[()] = ()
    profiles: tuple[ProfileResult, ...]
    observations: tuple[Observation, ...]
    decision: Spike
    input_sha256: dict[str, Hash]
    warnings: tuple[str, ...]


def require_coverage(rows: tuple[Observation, ...]) -> None:
    expected = {(profile, scenario, port, index)
                for profile in ("default", "legacy-l2") for port in (4002, 4003)
                for scenario in ("M1", "SLS") for index in range(3 if scenario == "M1" else 1)}
    observed = {(r.profile, r.scenario, r.port, r.run_index) for r in rows}
    if observed != expected:
        raise EvidenceError(f"incomplete/unexpected TWINPORT cells: {expected ^ observed}")
    for key in expected:
        attempts = sorted((r for r in rows if (r.profile, r.scenario, r.port, r.run_index) == key), key=lambda r: r.attempt)
        if [r.attempt for r in attempts] not in ([1], [1, 2]):
            raise EvidenceError(f"duplicate/missing attempt: {key}")
        if len(attempts) == 2 and attempts[0].reason != "player_leg_invalid":
            raise EvidenceError(f"unauthorized retry: {key}")
        if attempts[-1].reason == "player_leg_invalid" and len(attempts) != 2:
            raise EvidenceError(f"missing required rerun: {key}")


def compare(rows: tuple[Observation, ...], profile: str) -> ProfileResult:
    selected = [r for r in rows if r.profile == profile and r.scenario == "M1"]
    valid = {port: tuple(sorted(r.run_index for r in selected if r.port == port and r.accepted))
             for port in (4002, 4003)}
    comparison = Comparison(n=len(set(valid[4002]) & set(valid[4003])))
    if valid[4002] == valid[4003] == (0, 1, 2):
        arms = {port: tuple(r.measurement.delivered_fraction for r in sorted(selected, key=lambda r: r.run_index)
                            if r.port == port and r.accepted) for port in (4002, 4003)}
        comparison = paired_delta(arms[4003], arms[4002])
    final = {(r.port, r.run_index): r for r in sorted(selected, key=lambda r: r.attempt)}
    diagnostic = paired_delta(tuple(final[4003, i].measurement.delivered_fraction for i in range(3)),
                              tuple(final[4002, i].measurement.delivered_fraction for i in range(3)))
    if any(r.publisher_received_end is not None for r in selected):
        raise EvidenceError("new publisher denominator requires verified counter/window semantics; do not assume")
    return ProfileResult(profile=profile, valid_indices_4002=valid[4002], valid_indices_4003=valid[4003],
                         comparison=comparison, discarded_final_attempt_comparison=diagnostic)


def reduce(root: Path) -> Summary:
    paths = tuple(sorted(path for profile in ("default", "legacy-l2")
                         for path in (root / profile / "results").glob("*/run-*.json")
                         if ".exhausted." not in path.name))
    rows = tuple(load(path) for path in paths)
    require_coverage(rows)
    default, legacy = compare(rows, "default"), compare(rows, "legacy-l2")
    hashes = {str(path): digest(path) for path in paths}
    for row in rows:
        run = Attempt.model_validate_json(Path(row.source).read_bytes())
        directory = run.raw.stats_csv_path.parent
        for name in ("twinport.json", "player.csv", "sls-start.json", "sls-end.json", "sender-start.json",
                     "sender-end.json", "listener.log", "player.log", "request.json", "sls.conf"):
            artifact = directory / name
            hashes[str(artifact)] = digest(artifact)
    for profile in ("default", "legacy-l2"):
        manifest = root / profile / "results" / "manifest.json"
        hashes[str(manifest)] = digest(manifest)
    lock = Path(__file__).parent / "receivers.lock.json"
    hashes[str(lock)] = digest(lock)
    return Summary(profiles=(default, legacy), observations=rows,
                   decision=decide((default.comparison, legacy.comparison)), input_sha256=hashes,
                   warnings=("Publisher /stats exposes pktRcvDrop but no received-packet denominator: d_loss_pp is null, never NAK-derived.",
                             "Invalid player legs and unsettled outcomes do not count. Diagnostic final-attempt comparisons are NOT equivalence evidence.",
                             "Four synthetic conformance configurations are separate from twelve planned M1 indices."))


def markdown(summary: Summary) -> str:
    lines = ["# S-TWINPORT results", "", f"**Decision: {summary.decision.outcome.upper()}** — {summary.decision.selected_alternative}.",
             "", "See [predeclared method](method.md). These are SLS/attached-player measurements, not bare SLT sink measurements.",
             "", "## Paired post-settle inference", "",
             "Profile | Valid 4002 indices | Valid 4003 indices | Δfraction ×100 | 95% CI | Δpublisher loss pp",
             "---|---|---|---|---|---"]
    for profile in summary.profiles:
        c = profile.comparison
        lines.append(f"{profile.profile} | {profile.valid_indices_4002} | {profile.valid_indices_4003} | {c.d_goodput_pct} | [{c.ci_lower_pct}, {c.ci_upper_pct}] | {c.d_loss_pp}")
    lines += ["", "None/null means unidentifiable, NOT zero. A true-alias conclusion is not authorized.",
              "", "## Every attempt (including discarded player legs)", "",
              "Profile | Scenario | Port | Index | Attempt | Settled | Player valid | Player Mbit/s | Loss/drop | Delivered fraction | registered/carry/latency | Disposition",
              "---|---|---|---|---|---|---|---|---|---|---|---"]
    for row in sorted(summary.observations, key=lambda r: (r.profile, r.scenario, r.port, r.run_index, r.attempt)):
        m = row.measurement
        leg = m.player_leg
        assertions = "/".join("ok" if passed else "FAIL" for passed in
                              (row.assertions.registered, row.assertions.carry, row.assertions.latency))
        lines.append(f"{row.profile} | {row.scenario} | {row.port} | {row.run_index} | {row.attempt} | {m.settled} | {m.player_leg_valid} | {leg.mbps_recv_rate:.9f} | {leg.loss}/{leg.drop} | {m.delivered_fraction:.9f} | {assertions} | {row.reason or row.status}")
    lines += ["", "## Diagnostic-only final-attempt comparisons", "",
              "These retain the actual numbers from discarded runs for audit, not for the decision or a confidence claim about valid N=3 runs."]
    for profile in summary.profiles:
        c = profile.discarded_final_attempt_comparison
        lines.append(f"- {profile.profile}: Δfraction×100={c.d_goodput_pct:.9f}; diagnostic bootstrap [{c.ci_lower_pct:.9f}, {c.ci_upper_pct:.9f}].")
    lines += ["", "## Publisher field availability and override proof", "",
              "All live raw API responses retain publisher keys and both edge counters. No publisher packet-received denominator exists; no normalized publisher-loss rate can be computed. The player's receive denominator is NOT substituted.",
              "", "Both listener startup policy lines are checked on every attempt: default TTL200/freeze1/NAK1/gate1/FEC1; rollback TTL40/freeze1/NAK0/gate0/FEC0. Actual negotiated publisher latency is2000ms for M1,500ms for synthetic SLS. Actual player receive latency is200ms, verified from its CSV.",
              "", "## Caveats", ""]
    lines += [f"- {warning}" for warning in summary.warnings]
    return "\n".join(lines) + "\n"


def main() -> None:
    root, output = (Path(arg) for arg in sys.argv[1:])
    output.mkdir(parents=True, exist_ok=True)
    summary = reduce(root)
    publish(output / "report.md", markdown(summary))
    publish(output / "spike.json", summary.decision.model_dump_json(indent=2) + "\n")
    publish(output / "summary.json", summary.model_dump_json(indent=2) + "\n")


if __name__ == "__main__":
    main()

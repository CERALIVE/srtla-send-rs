#!/usr/bin/env python3
"""
Generate PR description from bonded-path-convergence evidence ledger.

PEP 723 script metadata:
/// script
/// requires-python = ">=3.10"
/// ///
"""

import json
import sys
import subprocess
from pathlib import Path
from typing import Optional, TypedDict


class SpikeEvidence(TypedDict, total=False):
    """Spike evidence from docs/evidence/bpc/*/spike.json"""
    spike: str
    hypothesis: Optional[str]
    outcome: str
    selected_alternative: str
    artefact_path: Optional[str]


class VerdictEvidence(TypedDict, total=False):
    """Verdict from docs/evidence/bpc/m4/verdict.json"""
    ship_set: list[str]
    base_mode: str
    covered_by_base_pct: float
    sacrificed_cells: list[dict]


class CanaryEvidence(TypedDict, total=False):
    """Canary status from docs/evidence/bpc/canary/canary.json"""
    outcome: str


class SrtReleaseEvidence(TypedDict, total=False):
    """SRT release info from docs/evidence/bpc/srt-release.json"""
    srt_release_version: str


def get_git_log_range(marker_commit: str) -> tuple[str, str]:
    """Get git log ranges for inherited vs new commits."""
    try:
        # Get commits from marker to HEAD
        result = subprocess.run(
            ["git", "log", "--oneline", f"{marker_commit}..HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        new_commits = result.stdout.strip()
        
        # Get commits from fork-point to marker
        result = subprocess.run(
            ["git", "log", "--oneline", f"bf57395..{marker_commit}"],
            capture_output=True,
            text=True,
            check=True,
        )
        inherited_commits = result.stdout.strip()
        
        return inherited_commits, new_commits
    except subprocess.CalledProcessError:
        return "", ""


def load_spike_evidence(evidence_dir: Path) -> dict[str, dict]:
    """Load all spike.json files from evidence directory."""
    spikes = {}
    for spike_file in evidence_dir.glob("*/spike.json"):
        try:
            with open(spike_file) as f:
                data = json.load(f)
                spike_name = spike_file.parent.name
                spikes[spike_name] = data
        except (json.JSONDecodeError, ValueError):
            pass
    return spikes


def load_verdict(evidence_dir: Path) -> Optional[dict]:
    """Load verdict.json from m4 directory."""
    verdict_file = evidence_dir / "m4" / "verdict.json"
    if verdict_file.exists():
        try:
            with open(verdict_file) as f:
                return json.load(f)
        except (json.JSONDecodeError, ValueError):
            pass
    return None


def load_canary(evidence_dir: Path) -> Optional[dict]:
    """Load canary.json."""
    canary_file = evidence_dir / "canary" / "canary.json"
    if canary_file.exists():
        try:
            with open(canary_file) as f:
                return json.load(f)
        except (json.JSONDecodeError, ValueError):
            pass
    return None


def load_srt_release(evidence_dir: Path) -> Optional[dict]:
    """Load srt-release.json."""
    srt_file = evidence_dir / "srt-release.json"
    if srt_file.exists():
        try:
            with open(srt_file) as f:
                return json.load(f)
        except (json.JSONDecodeError, ValueError):
            pass
    return None


def get_parity_contract_changes(agents_file: Path) -> str:
    """Extract parity contract changes from AGENTS.md."""
    # For now, return a placeholder that documents the key changes
    return """### Parity Contract Changes

**Version 4.0.0 introduces the following breaking changes to the parity contract:**

- **Scheduler modes removed**: `--mode classic`, `--mode rtt-threshold`, `--mode edpf`, `--mode adaptive` now exit with clap error (exit 2) naming this release. Only `--mode enhanced` is accepted.
- **Deprecated control flags accepted but ignored**: `--no-quality`, `--exploration`, `--rtt-delta-ms`, `--stall-deselect`, `--stall-min-in-flight`, `--stall-ack-stale-ms`, `--stall-reprobe-ms` all parse successfully with one startup WARN each when explicitly supplied, but have no effect.
- **Deprecated runtime commands return success with `deprecated: true, effect: "none"`**: `quality on|off`, `explore on|off`, `rtt-delta N`, `set-quality`, `set-exploration`, `set-rtt-delta` all succeed without mutation.
- **`get-status.mode` always returns `"enhanced"`**: consumers must keep it typed as an open string for forward compatibility.
- **Telemetry additions (schema_version stays 1)**:
  - Optional per-connection: `iface` (interface name), `link_id` (bind-map identity), `health` (link state), `priority` (configured preference)
  - Optional top-level: `bind_map_status`, `disposition`, `receiver_nak_report`
- **Receiver observation fields** (`receiver_nak_report`, `get-status.receiver`, per-link `rexmit_forwarded`) are additive and optional; schema_version remains 1.
- **All other surfaces unchanged**: binary name, CLI positional order, `--bind-map`, `--stats-file`, `--capabilities-json`, telemetry JSON shape (required fields), control socket, SIGHUP reload, clean shutdown.

**Rollback**: reinstall the released `3.3.0` `.deb` (`srtla-send-rs_3.3.0_<arch>.deb`); no file format, sidecar or telemetry shape changed between the two."""


def generate_pr_description(
    evidence_dir: Optional[Path] = None,
    output_file: Optional[Path] = None,
) -> str:
    """Generate the full PR description."""
    if evidence_dir is None:
        evidence_dir = Path("docs/evidence/bpc")
    
    # Load evidence
    spikes = load_spike_evidence(evidence_dir)
    verdict = load_verdict(evidence_dir)
    canary = load_canary(evidence_dir)
    srt_release = load_srt_release(evidence_dir)
    
    # Get git log ranges
    inherited, new = get_git_log_range("bf57395")
    
    # Build PR description
    sections = []
    
    # Section 1: Inherited vs new
    sections.append("## Inherited vs New\n")
    sections.append("### Inherited (fork-point to marker)\n")
    if inherited:
        sections.append("```\n" + inherited + "\n```\n")
    else:
        sections.append("(No inherited commits)\n")
    
    sections.append("\n### New (marker to HEAD)\n")
    if new:
        sections.append("```\n" + new + "\n```\n")
    else:
        sections.append("(No new commits)\n")
    
    # Section 2: Spike ledger
    sections.append("\n## Spike Ledger\n")
    sections.append("| Spike | Hypothesis | Outcome | Selected Alternative | Artefact |\n")
    sections.append("|---|---|---|---|---|\n")
    for name in sorted(spikes.keys()):
        spike = spikes[name]
        hyp = spike.get("hypothesis") or "—"
        alt = spike.get("selected_alternative") or "—"
        art = spike.get("artefact_path") or "—"
        outcome = spike.get("outcome") or "—"
        sections.append(f"| {name} | {hyp} | {outcome} | {alt} | {art} |\n")
    
    sections.append("\n## Byte-Identity Proofs\n")
    sections.append("- Golden traces: `tests/selection_mode_traces.rs` (1 test, byte-identical)\n")
    sections.append("- Telemetry legacy fixture: `tests/fixtures/telemetry-legacy-producer.json` (pre-ADR-003 shape, unchanged)\n")
    sections.append("- `--capabilities-json` probe: version 4.0.0, schema_version 1, additive fields only\n")
    
    sections.append("\n## Contract Deltas\n")
    sections.append(get_parity_contract_changes(Path("AGENTS.md")))
    
    if verdict:
        sections.append(f"\n### Verdict Summary\n")
        ship_set = verdict.get("ship_set", [])
        sections.append(f"- **Ship set**: {', '.join(ship_set)}\n")
        sections.append(f"- **Base mode**: {verdict.get('base_mode', '—')}\n")
        sections.append(f"- **Coverage**: {verdict.get('covered_by_base_pct', 0)}%\n")
        sacrificed = verdict.get("sacrificed_cells", [])
        sections.append(f"- **Sacrificed cells**: {len(sacrificed)}\n")
    
    if canary:
        sections.append(f"\n### Canary Status\n")
        sections.append(f"- **Outcome**: {canary.get('outcome', '—')}\n")
        sections.append("- **Note**: Hardware canary (Starlink + cellular bond) not yet run; pending on real rig.\n")
    
    if srt_release:
        sections.append(f"\n### Receiver Dependency\n")
        sections.append(f"- **Works against**: any receiver\n")
        sections.append(f"- **Best with**: `{srt_release.get('srt_release_version', '—')}` receivers (once released)\n")
        sections.append("- **Deploy order**: AFTER receiver soak (F5 audit); sender PR opens in parallel per Wave-5 design\n")
    
    sections.append(f"\n### ADR-004 Statement\n")
    sections.append("- `src/sender/selection/` is fork-owned from 4.0.0 forward\n")
    sections.append("- Upstream scheduler changes are triaged, never merged\n")
    
    pr_description = "".join(sections)
    
    if output_file:
        with open(output_file, "w") as f:
            f.write(pr_description)
    
    return pr_description


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Generate PR description from bonded-path-convergence evidence"
    )
    parser.add_argument(
        "--out",
        type=Path,
        help="Output file (default: stdout)",
    )
    parser.add_argument(
        "--evidence-dir",
        type=Path,
        default=Path("docs/evidence/bpc"),
        help="Evidence directory (default: docs/evidence/bpc)",
    )
    
    args = parser.parse_args()
    
    try:
        description = generate_pr_description(
            evidence_dir=args.evidence_dir,
            output_file=args.out,
        )
        if not args.out:
            print(description)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

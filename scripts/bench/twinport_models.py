import math
from typing import Literal

import numpy as np
from pydantic import ConfigDict

from report import RESAMPLES, SEED, Count, Document, EvidenceError, Positive


class Telemetry(Document):
    schema_version: Literal[1]
    last_updated_ms: Count
    bytes_sent_total: Count


class PlayerLeg(Document):
    loss: Count
    drop: Count
    mbps_recv_rate: Positive

    def valid(self, offered_bps: int) -> bool:
        return self.loss == 0 and self.drop == 0 and self.mbps_recv_rate * 1e6 >= 0.98 * offered_bps


class Measurement(Document):
    settled: bool
    player_leg_valid: bool
    player_leg: PlayerLeg
    player_latency_ms: Literal[200]
    offered_bps: Count
    player_bytes: Count
    sender_bytes: Count
    sender_start: Telemetry
    sender_end: Telemetry
    player_csv_start_ms: Count
    player_csv_end_ms: Count
    elapsed_ms: Positive
    sls_override: Literal["legacy-l2"] | None

    @property
    def delivered_fraction(self) -> float:
        if self.sender_bytes <= 0:
            raise EvidenceError("missing sender bytes")
        return self.player_bytes / self.sender_bytes


class Publisher(Document):
    model_config = ConfigDict(extra="allow", frozen=True, allow_inf_nan=False)
    pktRcvDrop: Count | None = None
    pktRecv: Count | None = None
    latency: Count


class ApiSnapshot(Document):
    status: Literal["ok"]
    publishers: dict[str, Publisher]


class Comparison(Document):
    n: Count
    d_goodput_pct: float | None = None
    ci_lower_pct: float | None = None
    ci_upper_pct: float | None = None
    d_loss_pp: float | None = None

    @property
    def delivery_equal(self) -> bool:
        return (self.n == 3 and self.ci_lower_pct is not None and self.ci_upper_pct is not None
                and self.ci_lower_pct <= 0 <= self.ci_upper_pct)

    @property
    def equivalent(self) -> bool:
        return self.delivery_equal and self.d_loss_pp is not None and abs(self.d_loss_pp) <= 0.05


class Spike(Document):
    spike: Literal["TWINPORT"] = "TWINPORT"
    alias_equivalent: bool
    d_goodput_pct: float | None
    d_loss_pp: float | None
    outcome: Literal["alias", "distinct"]
    selected_alternative: Literal["measure ship decisions on 4002 only", "run the reserved 4003 block in M4"]


def paired_delta(port4003: tuple[float, ...], port4002: tuple[float, ...]) -> Comparison:
    if len(port4003) != 3 or len(port4002) != 3:
        raise EvidenceError("TWINPORT needs all three valid paired indices")
    left, right = np.array(port4003), np.array(port4002)
    if not np.isfinite(left).all() or not np.isfinite(right).all():
        raise EvidenceError("nonfinite delivered fraction")
    indices = np.random.default_rng(SEED).integers(3, size=(RESAMPLES, 3))
    differences = 100 * (np.median(left[indices], axis=1) - np.median(right[indices], axis=1))
    ordered = np.sort(differences)
    return Comparison(n=3, d_goodput_pct=100 * float(np.median(left) - np.median(right)),
                      ci_lower_pct=float(ordered[math.ceil(RESAMPLES * 0.025) - 1]),
                      ci_upper_pct=float(ordered[math.ceil(RESAMPLES * 0.975) - 1]))


def decide(profiles: tuple[Comparison, Comparison]) -> Spike:
    equivalent = all(profile.equivalent for profile in profiles)
    return Spike(alias_equivalent=equivalent, d_goodput_pct=profiles[0].d_goodput_pct,
                 d_loss_pp=profiles[0].d_loss_pp, outcome="alias" if equivalent else "distinct",
                 selected_alternative="measure ship decisions on 4002 only" if equivalent
                 else "run the reserved 4003 block in M4")

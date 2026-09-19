use super::AdaptiveState;
use super::admission::srtt;
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::elapsed_ms;
use crate::connection::health::{HealthConstants, HealthState};

const SOLE_HOLD_MS: u64 = 2000;

pub(super) fn select(
    conns: &mut [SrtlaConnection],
    state: &mut AdaptiveState,
    now: u64,
) -> Option<usize> {
    let eligible = |i: usize| !matches!(state.targets[i].health, HealthState::Down);
    // Resolve the cached identity, not its old positional index after SIGHUP.
    let incumbent = state
        .sole_identity
        .and_then(|(id, generation)| {
            conns
                .iter()
                .position(|c| c.conn_id == id && c.delivery.socket_generation == generation)
        })
        .filter(|&i| eligible(i));
    let mut failed = None;
    if let (Some(i), Some((_, elected))) = (incumbent, state.sole_carrier) {
        let conn = &conns[i];
        let proof_anchor = conn
            .delivery
            .latest_data_proof_ms()
            .unwrap_or(elected)
            .max(elected);
        let proof_deadline = HealthConstants::default().stall_tau(srtt(conn)).max(2000.0);
        if elapsed_ms(now, proof_anchor) >= proof_deadline {
            conns[i].adaptive.sole_failed_until_proof = true;
            failed = Some(i);
        } else {
            let challenger = (0..conns.len())
                .filter(|&j| {
                    j != i
                        && eligible(j)
                        && !conns[j].adaptive.sole_failed_until_proof
                        && srtt(&conns[j])
                            .is_some_and(|rtt| rtt <= 0.5 * srtt(conn).unwrap_or(f64::INFINITY))
                })
                .min_by(|&a, &b| compare_rtt(conns, a, b));
            if now.saturating_sub(elected) < SOLE_HOLD_MS || challenger.is_none() {
                state.sole_carrier = Some((i, elected));
                return Some(i);
            }
            if let Some(j) = challenger {
                return Some(elect(conns, state, j, now));
            }
        }
    }
    let another = (0..conns.len()).any(|i| eligible(i) && Some(i) != failed);
    let choice = (0..conns.len())
        .filter(|&i| eligible(i) && (!another || Some(i) != failed))
        .min_by(|&a, &b| {
            let a_state = &conns[a].adaptive;
            let b_state = &conns[b].adaptive;
            a_state
                .sole_failed_until_proof
                .cmp(&b_state.sole_failed_until_proof)
                .then_with(|| {
                    if a_state.sole_failed_until_proof {
                        a_state
                            .last_sole_election_ms
                            .cmp(&b_state.last_sole_election_ms)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .then_with(|| compare_rtt(conns, a, b))
        });
    match choice {
        Some(i) => Some(elect(conns, state, i, now)),
        None => {
            state.sole_carrier = None;
            state.sole_identity = None;
            None
        }
    }
}

fn compare_rtt(conns: &[SrtlaConnection], a: usize, b: usize) -> std::cmp::Ordering {
    srtt(&conns[a])
        .unwrap_or(f64::INFINITY)
        .total_cmp(&srtt(&conns[b]).unwrap_or(f64::INFINITY))
        .then(a.cmp(&b))
}

// Four inputs are the two owned policy stores plus the elected index and clock.
fn elect(conns: &mut [SrtlaConnection], state: &mut AdaptiveState, i: usize, now: u64) -> usize {
    state.sole_carrier = Some((i, now));
    state.sole_identity = Some((conns[i].conn_id, conns[i].delivery.socket_generation));
    conns[i].adaptive.last_sole_election_ms = Some(now);
    i
}

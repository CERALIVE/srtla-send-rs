use super::{AdaptiveState, sole};
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::SelectionWeight;
use crate::sender::selection::SchedulerFeatures;
use crate::sender::selection::admission::Admission;

pub(crate) enum Selection {
    Ranked,
    Carrier(Option<usize>),
}

pub(crate) fn refresh(
    conns: &mut [SrtlaConnection],
    state: &mut AdaptiveState,
    now: u64,
    admission: &mut Admission,
) -> Selection {
    let mut has_candidate = false;
    if !admission.fallback {
        for offset in 0..admission.eligible.len() {
            let i = admission.eligible[offset];
            let conn = &mut conns[i];
            let weight = admission.compute_weight(i, conn);
            conn.adaptive.weight = Some(weight);
            has_candidate = true;
        }
    }
    if has_candidate {
        state.sole_carrier = None;
        state.sole_identity = None;
        return Selection::Ranked;
    }
    if admission.features.contains(SchedulerFeatures::SOLE) {
        if let Some(i) = sole::select(conns, state, now) {
            state.targets[i].sole_carrier = true;
            conns[i].adaptive.weight = Some(admission.compute_weight(i, &mut conns[i]));
            return Selection::Carrier(Some(i));
        }
    } else {
        state.sole_carrier = None;
        state.sole_identity = None;
    }
    // Preserve the connected-pool escape and its base-only ordering. Only its
    // chosen carrier is admitted; soft-held neighbours must still publish zero.
    let fallback = conns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.connected)
        .max_by(|(a, ca), (b, cb)| ca.get_score().cmp(&cb.get_score()).then(b.cmp(a)))
        .map(|(i, _)| i);
    if let Some(i) = fallback {
        conns[i].adaptive.weight = Some(SelectionWeight {
            base_score: conns[i].get_score(),
            quality_multiplier: 1.0,
            effective_multiplier: 1.0,
        });
    }
    Selection::Carrier(fallback)
}

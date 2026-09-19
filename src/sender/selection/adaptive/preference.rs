use crate::bind_map::Priority;
use crate::connection::health::HealthState;

/// Bounded Healthy-only ranking bias, multiplied into the admission weight.
///
/// Reached from `admission::compute_weight` (`admission.rs`), which folds this
/// value into `SelectionWeight::effective_multiplier`; the bias ramps in with the
/// congestion window and is `1.0` at or below 10000.
#[must_use]
pub fn preference_multiplier(priority: Option<Priority>, window: i32, health: HealthState) -> f64 {
    match health {
        HealthState::Healthy => {
            let ramp = ((f64::from(window) - 10_000.0) / 10_000.0).clamp(0.0, 1.0);
            1.0 + priority.map_or(0.0, Priority::get) * ramp
        }
        HealthState::Degraded
        | HealthState::Stalled
        | HealthState::Rejoining
        | HealthState::Down => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::bind_map::Priority;
    use crate::connection::health::HealthState;

    #[test]
    fn preference_multiplier_table() {
        // Given: bounded preferences, ramp boundaries, and every health state.
        use HealthState::{Degraded, Down, Healthy, Rejoining, Stalled};
        let cases = [
            (None, 20_000, Healthy, 1.0),
            (Some(0.2), i32::MIN, Healthy, 1.0),
            (Some(0.2), 10_000, Healthy, 1.0),
            (Some(0.2), 15_000, Healthy, 1.1),
            (Some(0.2), 20_000, Healthy, 1.2),
            (Some(0.2), i32::MAX, Healthy, 1.2),
            (Some(-0.2), 20_000, Healthy, 0.8),
            (Some(-0.2), 15_000, Healthy, 0.9),
            (Some(0.0), 20_000, Healthy, 1.0),
            (Some(0.2), 20_000, Rejoining, 1.0),
            (Some(0.2), 20_000, Degraded, 1.0),
            (Some(-0.2), 20_000, Stalled, 1.0),
            (Some(-0.2), 20_000, Down, 1.0),
        ];
        for (priority, window, health, expected) in cases {
            // When: evaluate the pure bias without invoking a selector.
            let actual = super::preference_multiplier(
                priority.map(|p| Priority::try_from(p).unwrap()),
                window,
                health,
            );
            // Then: only healthy links receive the clamped, symmetric bias.
            assert!(
                (actual - expected).abs() < 1e-12,
                "{priority:?}/{window}/{health:?}: {actual}"
            );
        }
    }
}

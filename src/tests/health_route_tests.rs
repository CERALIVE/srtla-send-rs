use super::transition_tests::{STATES, signals};
use super::{HealthConstants, HealthMachine, HealthState};
use crate::connection::route::RouteHealth;

#[test]
fn route_absence_is_an_independent_degraded_cause() {
    // Given: no measured loss or queue, but an observed missing default route.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(HealthState::Healthy, 0);
    let mut s = signals(100);
    s.loss_ewma = None;
    s.loss_cohort_ok = false;
    s.route_health = RouteHealth::NoDefaultRoute;
    // When: route absence persists well beyond the clearance dwell.
    for now in [100, 200, 1_200, 20_000] {
        s.now_ms = now;
        machine.step(&s, &k);
        // Then: route degradation never invents loss or clears itself.
        assert_eq!(machine.state(), HealthState::Degraded);
        assert!(machine.route_latched());
        assert!(!machine.loss_latched());
    }
}

#[test]
fn route_transition_table_preserves_hard_and_stall_precedence() {
    // Given: every state, absent route, otherwise healthy observations.
    for (state, expected) in STATES.into_iter().zip([
        HealthState::Degraded,
        HealthState::Degraded,
        HealthState::Stalled,
        HealthState::Degraded,
        HealthState::Rejoining,
    ]) {
        let mut machine = HealthMachine::new(state, 0);
        let mut s = signals(100);
        s.route_health = RouteHealth::NoDefaultRoute;
        // When: the independent route observation is applied.
        machine.step(&s, &HealthConstants::default());
        // Then: existing Down restoration and Stalled recovery contracts survive.
        assert_eq!(machine.state(), expected);
    }
}

#[test]
fn unknown_route_cannot_clear_a_latched_absence() {
    // Given: route-only degradation followed by an unreadable route table.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(HealthState::Healthy, 0);
    let mut s = signals(100);
    s.route_health = RouteHealth::NoDefaultRoute;
    machine.step(&s, &k);
    s.route_health = RouteHealth::Unknown;
    // When: unknown persists past dwell.
    for now in [200, 1_200, 20_000] {
        s.now_ms = now;
        machine.step(&s, &k);
    }
    // Then: unknown is not evidence of restoration.
    assert_eq!(machine.state(), HealthState::Degraded);
    assert!(machine.route_latched());
}

#[test]
fn restored_route_requires_uninterrupted_clearance_without_loss_evidence() {
    // Given: route-only degradation, then partial restoration interrupted by absence.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(HealthState::Healthy, 0);
    let mut s = signals(0);
    s.loss_ewma = None;
    s.loss_cohort_ok = false;
    for (now, route) in [
        (0, RouteHealth::NoDefaultRoute),
        (100, RouteHealth::DefaultRoutePresent),
        (1_099, RouteHealth::NoDefaultRoute),
        (1_100, RouteHealth::DefaultRoutePresent),
        (2_099, RouteHealth::DefaultRoutePresent),
    ] {
        s.now_ms = now;
        s.route_health = route;
        machine.step(&s, &k);
        assert_eq!(machine.state(), HealthState::Degraded);
    }
    // When: a full uninterrupted tau elapses after the latest restoration.
    s.now_ms = 2_100;
    machine.step(&s, &k);
    // Then: the link ramps with no fabricated loss sample required.
    assert_eq!(machine.state(), HealthState::Rejoining);
    assert!(!machine.route_latched());
}

#[test]
fn clearing_loss_and_queue_does_not_clear_an_absent_route() {
    // Given: all three independent degradation inputs, then measured recovery.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(HealthState::Healthy, 0);
    let mut s = signals(100);
    s.route_health = RouteHealth::NoDefaultRoute;
    s.loss_ewma = Some(0.5);
    s.queue_delay_ms = 100.0;
    machine.step(&s, &k);
    s.loss_ewma = Some(0.0);
    s.queue_delay_ms = 0.0;
    // When: loss/queue stay clear but the route remains absent.
    for now in [200, 1_200] {
        s.now_ms = now;
        machine.step(&s, &k);
    }
    // Then: the third cause still holds the machine Degraded.
    assert_eq!(machine.state(), HealthState::Degraded);
    assert!(machine.route_latched());
}

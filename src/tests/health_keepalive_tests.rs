use super::transition_tests::signals;
use super::{HealthConstants, HealthMachine, HealthSignals, HealthState};

#[test]
fn independent_control_silence_preserves_data_evidence_and_failure_precedence() {
    // Given: silent controls, fresh DATA, DATA-only failure, and unknown evidence.
    let cases = [
        (0, None, Some(2999), true, HealthState::Healthy),
        (0, None, Some(3000), true, HealthState::Stalled),
        (0, Some(999), Some(3000), true, HealthState::Healthy),
        (0, Some(1000), Some(3000), true, HealthState::Stalled),
        (31, Some(9000), None, true, HealthState::Healthy),
        (32, Some(1000), Some(0), true, HealthState::Stalled),
        (0, None, Some(3000), false, HealthState::Down),
    ];
    for (attempts, proof, keepalive, connected, expected) in cases {
        let mut machine = HealthMachine::new(HealthState::Healthy, 0);
        // When: the production FSM evaluates the competing evidence.
        machine.step(
            &HealthSignals {
                attempts_since_proof: attempts,
                proof_age_ms: proof,
                keepalive_silence_ms: keepalive,
                connected,
                ..signals(10_000)
            },
            &HealthConstants::default(),
        );
        // Then: DATA remains independent and hard failure keeps precedence.
        assert_eq!(machine.state(), expected);
    }
}

#[test]
fn keepalive_echo_does_not_rehabilitate_a_data_stalled_link() {
    // Given: Stalled with healthy controls but no successful DATA recovery rounds.
    let mut machine = HealthMachine::new(HealthState::Stalled, 0);
    // When: fresh keepalive evidence reaches the FSM.
    machine.step(
        &HealthSignals {
            keepalive_silence_ms: Some(0),
            ..signals(10_000)
        },
        &HealthConstants::default(),
    );
    // Then: keepalive liveness is not DATA recovery evidence.
    assert_eq!(machine.state(), HealthState::Stalled);
}

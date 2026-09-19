use super::*;

#[test]
fn scheduler_fields_follow_identity_when_present() {
    // Given: explicit optional scheduler observations.
    let conn = TelemetryConn {
        link_id: Some("modem-a".into()),
        health: Some("degraded"),
        priority: Some(0.15),
        ..Default::default()
    };
    // When: the production per-connection serializer runs.
    let json = serde_json::to_string(&ConnRecord::from(&conn)).unwrap();
    // Then: the additive tail follows link_id in producer order.
    assert!(json.ends_with(r#""link_id":"modem-a","health":"degraded","priority":0.15}"#));
}

#[test]
fn scheduler_fields_are_absent_when_unknown() {
    // Given: a legacy producer with neither observation.
    let conn = TelemetryConn::default();
    // When: serialized by the real document model.
    let json = serde_json::to_value(ConnRecord::from(&conn)).unwrap();
    // Then: omitted keys, not null or sentinel values.
    assert!(json.get("health").is_none());
    assert!(json.get("priority").is_none());
}

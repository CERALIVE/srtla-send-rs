use super::tests::capture_logs;
use super::*;
use crate::test_helpers::create_test_connections;

#[tokio::test]
async fn premature_nak_status_reports_each_links_count_including_zero() {
    // Given distinct diagnostic totals on two links.
    let mut conns = create_test_connections(2).await;
    conns[1].premature_nak_count = 7;
    // When the normal status report is emitted.
    let rendered = capture_logs(|| log_connection_status(&conns, None, &DynamicConfig::new()));
    // Then every link exposes its own count, including the untouched link's zero.
    assert!(rendered.contains("premature_naks=0"));
    assert!(rendered.contains("premature_naks=7"));
    assert_eq!(rendered.matches("premature_naks=").count(), 2);
}

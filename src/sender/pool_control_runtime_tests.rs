use std::time::Duration;

use super::pool_control::{LinkKey, PoolControlError, PoolControlRequest};
use super::{SenderPaths, TelemetrySinks, run_sender_with_config};
use crate::bind_map::Priority;
use crate::config::DynamicConfig;
use crate::stats::SharedStats;
use crate::subscription::SubscriptionManager;

#[tokio::test]
async fn running_sender_consumes_priority_requests() {
    // Given: the real sender entry point, one loopback uplink, initial snapshot readiness.
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    let stats = SharedStats::new();
    let subscriptions = SubscriptionManager::new();
    let snapshots = subscriptions.subscribe();
    let peer = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sender = run_sender_with_config(
        0,
        "127.0.0.1",
        peer.local_addr().unwrap().port(),
        SenderPaths {
            ips_file: ips.to_str().unwrap(),
            bind_map: None,
        },
        DynamicConfig::default(),
        stats.clone(),
        TelemetrySinks {
            file: None,
            subscriptions,
        },
    );
    let control = async {
        tokio::task::spawn_blocking(move || snapshots.recv_timeout(Duration::from_secs(3)))
            .await
            .unwrap()
            .unwrap();
        let handle = stats.pool_control().unwrap();
        let priority = Some(Priority::try_from(0.15).unwrap());
        let reply = handle
            .submit(PoolControlRequest::SetLinkPriority {
                key: LinkKey::ConnId(0),
                priority,
            })
            .unwrap();
        // When: the live event loop, not a test-side consumer, handles the message.
        let applied = tokio::time::timeout(Duration::from_secs(3), reply)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        // Then: it reports the changed live value and rejects an absent position.
        assert!(applied.applied);
        assert_eq!(applied.effective_priority, priority);
        let reply = handle
            .submit(PoolControlRequest::SetLinkPriority {
                key: LinkKey::ConnId(9),
                priority,
            })
            .unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(3), reply)
                .await
                .unwrap()
                .unwrap(),
            Err(PoolControlError::UnknownLink(LinkKey::ConnId(9)))
        );
    };
    tokio::select! {
        result = sender => panic!("sender exited before control completion: {result:?}"),
        () = control => {},
    }
}

#[tokio::test]
async fn bounded_queue_and_shutdown_return_errors_not_false_success() {
    // Given: a sender channel with its bounded queue filled, no consumer progress.
    let stats = SharedStats::new();
    let receiver = stats.attach_pool_control();
    let handle = stats.pool_control().unwrap();
    let mut replies = Vec::new();
    for _ in 0..64 {
        replies.push(
            handle
                .submit(PoolControlRequest::SetLinkPriority {
                    key: LinkKey::ConnId(0),
                    priority: None,
                })
                .unwrap(),
        );
    }
    // When: one more request exceeds the bound, followed by sender shutdown.
    assert!(matches!(
        handle.submit(PoolControlRequest::SetLinkPriority {
            key: LinkKey::ConnId(0),
            priority: None,
        }),
        Err(PoolControlError::Busy)
    ));
    drop(receiver);
    // Then: all waiting requests disconnect and subsequent submits fail closed.
    for reply in replies {
        assert!(reply.await.is_err());
    }
    assert!(matches!(
        handle.submit(PoolControlRequest::SetLinkPriority {
            key: LinkKey::ConnId(0),
            priority: None,
        }),
        Err(PoolControlError::Unavailable)
    ));
}

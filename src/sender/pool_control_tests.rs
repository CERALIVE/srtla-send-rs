use crate::bind_map::{LinkId, Priority};
use crate::connection::SrtlaConnection;
use crate::sender::pool_control::{LinkKey, PoolControlError, PoolControlRequest};
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection;

async fn pool() -> Vec<SrtlaConnection> {
    let mut a = create_test_connection().await;
    a.link_id = Some(LinkId::parse("modem-a").unwrap());
    a.priority_baseline = Some(Priority::try_from(-0.1).unwrap());
    vec![a, create_test_connection().await]
}

#[tokio::test]
async fn priority_request_mutates_the_live_pool_before_reply() {
    // Given: sender-owned real sockets and a handle obtained by a stats consumer.
    let stats = SharedStats::new();
    let mut receiver = stats.attach_pool_control();
    let handle = stats.pool_control().unwrap();
    let mut conns = pool().await;
    let value = Some(Priority::try_from(0.15).unwrap());
    let key = LinkKey::LinkId(LinkId::parse("modem-a").unwrap());
    let mut reply = handle
        .submit(PoolControlRequest::SetLinkPriority {
            key: key.clone(),
            priority: value,
        })
        .unwrap();
    assert!(matches!(
        reply.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));
    // When: the same consumer used by the live event loop applies the request.
    receiver.recv().await.unwrap().apply(&mut conns);
    // Then: acknowledgment contains the actual resulting priority and position.
    let applied = reply.await.unwrap().unwrap();
    assert!(applied.applied);
    assert_eq!(applied.key, key);
    assert_eq!(applied.link_id, conns[0].link_id);
    assert_eq!(applied.conn_id, 0);
    assert_eq!(applied.priority, value);
    assert_eq!(applied.effective_priority, value);
    assert_eq!(conns[0].effective_priority(), value);
    assert_eq!(conns[1].effective_priority(), None);
}

#[tokio::test]
async fn addressed_clear_exposes_only_the_next_layer() {
    // Given: distinct baseline, link and conn priorities.
    for (key, expected) in [
        (LinkKey::ConnId(0), 0.1),
        (LinkKey::LinkId(LinkId::parse("modem-a").unwrap()), 0.2),
    ] {
        let stats = SharedStats::new();
        let mut receiver = stats.attach_pool_control();
        let mut conns = pool().await;
        conns[0].priority_override_link = Some(Priority::try_from(0.1).unwrap());
        conns[0].priority_override_conn = Some(Priority::try_from(0.2).unwrap());
        let reply = stats
            .pool_control()
            .unwrap()
            .submit(PoolControlRequest::SetLinkPriority {
                key,
                priority: None,
            })
            .unwrap();
        // When: only the addressed override is cleared by the real consumer.
        receiver.recv().await.unwrap().apply(&mut conns);
        // Then: the remaining override, never the baseline, is exposed.
        assert_eq!(
            reply
                .await
                .unwrap()
                .unwrap()
                .effective_priority
                .unwrap()
                .get(),
            expected
        );
        assert_eq!(conns[0].effective_priority().unwrap().get(), expected);
    }
}

#[tokio::test]
async fn unknown_link_returns_typed_error_without_mutation() {
    // Given: a real pool and each kind of absent target.
    for key in [
        LinkKey::ConnId(99),
        LinkKey::LinkId(LinkId::parse("absent").unwrap()),
    ] {
        let stats = SharedStats::new();
        let mut receiver = stats.attach_pool_control();
        let mut conns = pool().await;
        let reply = stats
            .pool_control()
            .unwrap()
            .submit(PoolControlRequest::SetLinkPriority {
                key: key.clone(),
                priority: None,
            })
            .unwrap();
        // When: the sender resolves the target against its pool.
        receiver.recv().await.unwrap().apply(&mut conns);
        // Then: no successful application and no change to the existing link.
        assert_eq!(
            reply.await.unwrap(),
            Err(PoolControlError::UnknownLink(key))
        );
        assert_eq!(conns[0].effective_priority().unwrap().get(), -0.1);
    }
}

#[tokio::test]
async fn queued_positional_request_cannot_cross_reload() {
    // Given: a queued position-zero request and a pool about to reorder.
    let stats = SharedStats::new();
    let mut receiver = stats.attach_pool_control();
    let old = stats.pool_control().unwrap();
    let mut conns = pool().await;
    let request = PoolControlRequest::SetLinkPriority {
        key: LinkKey::ConnId(0),
        priority: Some(Priority::try_from(0.2).unwrap()),
    };
    let reply = old.submit(request.clone()).unwrap();
    // When: the sender closes this pool epoch before any reload await.
    receiver.close_for_reload();
    conns.swap(0, 1);
    let _replacement = stats.attach_pool_control();
    // Then: queued requests are rejected and even retained old handles stay closed.
    assert_eq!(reply.await.unwrap(), Err(PoolControlError::PoolReloaded));
    assert!(matches!(
        old.submit(request),
        Err(PoolControlError::Unavailable)
    ));
    assert_eq!(conns[0].effective_priority(), None);
    assert_eq!(conns[1].effective_priority().unwrap().get(), -0.1);
}

#[tokio::test]
async fn cancelled_request_never_mutates_the_pool() {
    // Given: an enqueued request whose caller abandoned its reply.
    let stats = SharedStats::new();
    let mut receiver = stats.attach_pool_control();
    let mut conns = pool().await;
    let reply = stats
        .pool_control()
        .unwrap()
        .submit(PoolControlRequest::SetLinkPriority {
            key: LinkKey::ConnId(0),
            priority: Some(Priority::try_from(0.2).unwrap()),
        })
        .unwrap();
    drop(reply);
    // When: the sender observes that cancellation before application.
    receiver.recv().await.unwrap().apply(&mut conns);
    // Then: no late mutation of the baseline occurs.
    assert_eq!(conns[0].effective_priority().unwrap().get(), -0.1);
}

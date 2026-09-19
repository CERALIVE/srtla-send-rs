use tokio::net::UdpSocket;

use super::*;
use crate::test_helpers::create_test_connection_to;

#[tokio::test]
async fn fresh_receiver_id_rebroadcasts_reg2_despite_old_group_grants() {
    // Given reconnect REG2 grants for an identity the restarted receiver no longer knows.
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    conns[0].connected = false;
    let mut reg = SrtlaRegistrationManager::new();
    reg.send_reg2_to(0, &mut conns[0]).await;
    let mut bytes = [0; 512];
    peer.recv_from(&mut bytes).await.unwrap();
    reg.send_reg1_to(0, &mut conns[0]).await;
    peer.recv_from(&mut bytes).await.unwrap();
    let mut fresh = reg.srtla_id;
    fresh[SRTLA_ID_LEN - 1] ^= 1;
    // When the receiver returns its new group identity and the real driver runs.
    reg.process_registration_packet(0, &create_reg2_packet(&fresh));
    reg.reg_driver_send_if_needed(&mut conns).await;
    // Then an old group's authorization cannot suppress this group's actual REG2 send.
    let (len, _) = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        peer.recv_from(&mut bytes),
    )
    .await
    .expect("new group REG2 must leave the socket immediately")
    .unwrap();
    assert_eq!(&bytes[..len], create_reg2_packet(&fresh));
}

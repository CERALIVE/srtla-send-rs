//! The ifindex-staleness lifecycle a modem replug drives, exercised against an
//! interface table the test controls — a real interface cannot be made to
//! disappear mid-test without privileges.

use std::collections::HashMap;
#[cfg(unix)]
use std::io;
use std::sync::Mutex;

use crate::bind_map::IfaceName;
use crate::connection::egress::{
    EgressFault, EgressLifecycle, EgressPoll, IfaceResolver, LinkState,
};

fn iface(name: &str) -> IfaceName {
    IfaceName::parse(name).expect("test interface name must be valid")
}

/// An interface table the test drives: a modem can be unplugged, or replugged
/// under the same name with a different ifindex.
#[derive(Default)]
struct FakeIfaces(Mutex<HashMap<String, u32>>);

impl FakeIfaces {
    fn with(name: &str, index: u32) -> Self {
        let table = Self::default();
        table.plug(name, index);
        table
    }

    fn plug(&self, name: &str, index: u32) {
        self.0.lock().unwrap().insert(name.to_string(), index);
    }

    fn unplug(&self, name: &str) {
        self.0.lock().unwrap().remove(name);
    }
}

impl IfaceResolver for FakeIfaces {
    fn ifindex(&self, name: &str) -> Option<u32> {
        self.0.lock().unwrap().get(name).copied()
    }
}

#[test]
fn every_bind_re_resolves_the_interface_by_name() {
    // Given: a mapped link bound while its interface had ifindex 7.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    assert_eq!(egress.resolve_for_bind(&table), Ok(Some(7)));

    // When: the device is replugged and re-enumerates under the same name.
    table.plug("wwan0", 12);

    // Then: the next bind resolves the NAME again and gets the new index — a
    // cached ifindex would have kept sending into the void.
    assert_eq!(egress.resolve_for_bind(&table), Ok(Some(12)));
}

#[test]
fn a_re_enumeration_invalidates_the_current_socket_rather_than_reusing_it() {
    // Given: a link bound against ifindex 7.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    egress.resolve_for_bind(&table).unwrap();
    assert!(!egress.needs_rebind());

    // When: the interface re-enumerates under the same name mid-run.
    table.plug("wwan0", 12);

    // Then: the periodic poll reports it and demands a rebind.
    assert_eq!(
        egress.poll(&table),
        EgressPoll::Reenumerated { from: 7, to: 12 }
    );
    assert!(
        egress.needs_rebind(),
        "the socket still holds ifindex 7 and must never be reused"
    );
}

#[test]
fn an_interface_disappearing_mid_run_removes_the_link_within_one_poll() {
    // Given: a live mapped link.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    egress.resolve_for_bind(&table).unwrap();
    assert_eq!(egress.state(), LinkState::Active);

    // When: the modem is unplugged mid-run.
    table.unplug("wwan0");

    // Then: ONE poll detects it — bounded detection, not a CONN_TIMEOUT wait.
    assert_eq!(egress.poll(&table), EgressPoll::Removed);
    assert_eq!(egress.state(), LinkState::Removed);
    assert!(
        !egress.needs_rebind(),
        "a removed link waits for a reload; rebinding would only burn the backoff"
    );
}

#[test]
fn a_removed_link_cannot_be_rebound_until_the_interface_returns() {
    // Given: a link whose interface disappeared.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    egress.resolve_for_bind(&table).unwrap();
    table.unplug("wwan0");
    egress.poll(&table);

    // When: a bind is attempted anyway.
    // Then: it is refused — no socket is created on a name the kernel does not
    // know, so no stale socket can be reused.
    assert_eq!(
        egress.resolve_for_bind(&table),
        Err(EgressFault::InterfaceGone)
    );
    assert_eq!(egress.state(), LinkState::Removed);

    // When: the modem comes back.
    table.plug("wwan0", 21);

    // Then: the link binds again, against the NEW index.
    assert_eq!(egress.resolve_for_bind(&table), Ok(Some(21)));
    assert_eq!(egress.state(), LinkState::Active);
}

#[cfg(unix)]
#[test]
fn an_enodev_send_removes_the_link_and_enetunreach_only_rebinds_it() {
    // Given: two live mapped links.
    let table = FakeIfaces::with("wwan0", 7);
    let mut gone = EgressLifecycle::for_iface(iface("wwan0"));
    let mut unreachable = EgressLifecycle::for_iface(iface("wwan0"));
    gone.resolve_for_bind(&table).unwrap();
    unreachable.resolve_for_bind(&table).unwrap();

    // When: each hits its respective send failure.
    let enodev = gone.note_send_error(&io::Error::from_raw_os_error(libc::ENODEV));
    let enetunreach = unreachable.note_send_error(&io::Error::from_raw_os_error(libc::ENETUNREACH));

    // Then: ENODEV means the device is gone (await a reload); ENETUNREACH means
    // it is merely down, so the socket is rebuilt rather than abandoned.
    assert_eq!(enodev, Some(EgressFault::InterfaceGone));
    assert_eq!(gone.state(), LinkState::Removed);
    assert_eq!(enetunreach, Some(EgressFault::NetworkUnreachable));
    assert_eq!(unreachable.state(), LinkState::Active);
    assert!(unreachable.needs_rebind());
}

#[cfg(unix)]
#[test]
fn an_unmapped_link_is_untouched_by_egress_errnos() {
    // Given: a legacy link with no interface binding at all.
    let mut egress = EgressLifecycle::unmapped();

    // When: a send returns an errno that WOULD be fatal for a device binding.
    let fault = egress.note_send_error(&io::Error::from_raw_os_error(libc::ENODEV));

    // Then: nothing happens — those errnos carry no ifindex meaning without
    // SO_BINDTODEVICE, and the legacy recovery path stays in charge.
    assert_eq!(fault, None);
    assert_eq!(egress.state(), LinkState::Active);
    assert!(!egress.needs_rebind());
}

#[test]
fn an_unmapped_link_never_resolves_or_polls_an_interface() {
    // Given: a legacy link and an interface table it must not consult.
    let table = FakeIfaces::default();
    let mut egress = EgressLifecycle::unmapped();

    // When/Then: bind resolution yields no ifindex and polling is a no-op, so
    // the legacy path cannot be removed by an interface it never named.
    assert_eq!(egress.resolve_for_bind(&table), Ok(None));
    assert_eq!(egress.poll(&table), EgressPoll::Unchanged);
    assert_eq!(egress.state(), LinkState::Active);
}

#[test]
fn a_reload_naming_a_different_interface_drops_the_previous_binding() {
    // Given: a link pinned to wwan0 and bound against its ifindex.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    egress.resolve_for_bind(&table).unwrap();

    // When: a reload moves the same link_id onto wwan1.
    table.plug("wwan1", 9);
    let changed = egress.adopt(Some(iface("wwan1")));

    // Then: the change is reported (the caller must recreate the socket) and no
    // ifindex-scoped state survives — the next poll sees a fresh binding.
    assert!(changed);
    assert_eq!(egress.iface().map(IfaceName::as_str), Some("wwan1"));
    assert_eq!(egress.poll(&table), EgressPoll::Unchanged);
    assert_eq!(egress.resolve_for_bind(&table), Ok(Some(9)));
}

#[test]
fn a_reload_is_how_a_removed_link_comes_back() {
    // Given: a link removed by an unplug.
    let table = FakeIfaces::with("wwan0", 7);
    let mut egress = EgressLifecycle::for_iface(iface("wwan0"));
    egress.resolve_for_bind(&table).unwrap();
    table.unplug("wwan0");
    egress.poll(&table);
    assert_eq!(egress.state(), LinkState::Removed);

    // When: a reload re-publishes the same interface (the operator replugged).
    let changed = egress.adopt(Some(iface("wwan0")));

    // Then: the link leaves the removed state even though the NAME is unchanged.
    assert!(!changed);
    assert_eq!(egress.state(), LinkState::Active);
}

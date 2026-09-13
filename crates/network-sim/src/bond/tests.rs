use super::*;

const NAT: LinkSpec = LinkSpec {
    carrier: CarrierMode::Nat,
    shared_ip_with: None,
};

#[test]
fn source_tables_avoid_kernel_reserved_tables_for_large_bonds() {
    // Given: the entire supported link range, including indices beyond the smoke.
    let tables = (0..253)
        .map(routing::source_table)
        .collect::<std::collections::HashSet<_>>();
    // When: reserving per-link source tables.
    // Then: no link can overwrite the kernel's default/main/local tables.
    assert!([253, 254, 255].iter().all(|table| !tables.contains(table)));
    assert_eq!(tables.len(), 253);
}

#[test]
fn shared_ip_requires_explicit_mapping_mode() {
    // Given: a shared-IP pair and an invalid namespace name as a side-effect tripwire.
    let links = [
        NAT,
        LinkSpec {
            shared_ip_with: Some(0),
            ..NAT
        },
    ];
    // When: no explicit shared-IP launch policy is supplied.
    let result = BondTopology::new("/must-not-create", &links, MappingMode::None);
    // Then: configuration is rejected, not a namespace operation attempted.
    let error = result.err().expect("reject missing mapping mode");
    assert_eq!(
        error.downcast_ref::<BondConfigError>(),
        Some(&BondConfigError::SharedIpRequiresExplicitMappingMode)
    );
}

#[test]
fn source_routes_include_every_nat_link_and_the_first_link() {
    // Given: three distinct source addresses as in the proven carrier spike.
    for idx in 0..3 {
        let subnet = idx + 1;
        let table = 101 + idx;
        let route = routing::SourceRoute {
            iface: "s0",
            subnet: &format!("10.30.{subnet}.0/24"),
            gateway: &format!("10.30.{subnet}.2"),
            ip: &format!("10.30.{subnet}.1"),
            table,
        };
        // When: rendering the source routing setup.
        let commands = route.commands();
        // Then: each source has a connected route, default and explicit source rule.
        assert_eq!(
            commands,
            [
                format!("route add 10.30.{subnet}.0/24 dev s0 table {table}"),
                format!("route add default via 10.30.{subnet}.2 dev s0 table {table}"),
                format!("rule add from 10.30.{subnet}.1/32 table {table}")
            ]
        );
    }
}

#[test]
fn shared_links_require_separate_nat_carriers() {
    // Given: a Direct link cannot disambiguate return traffic for a shared address.
    let links = [
        LinkSpec {
            carrier: CarrierMode::Direct,
            ..NAT
        },
        LinkSpec {
            shared_ip_with: Some(0),
            ..NAT
        },
    ];
    // When: explicitly selecting the legacy falsifiability control.
    let result = spec::validate(&links, &MappingMode::LegacyControl);
    // Then: sharing does not silently convert a Direct carrier into NAT.
    assert_eq!(
        result,
        Err(BondConfigError::SharedIpRequiresNat { index: 0 })
    );
}

#[test]
fn shared_link_must_reference_an_earlier_link() {
    // Given: a self reference cannot resolve a source address.
    let links = [LinkSpec {
        shared_ip_with: Some(0),
        ..NAT
    }];
    // When: validating a declared legacy control.
    let result = spec::validate(&links, &MappingMode::LegacyControl);
    // Then: the reference is rejected before allocation.
    assert_eq!(
        result,
        Err(BondConfigError::InvalidSharedIndex {
            index: 0,
            target: 0
        })
    );
}

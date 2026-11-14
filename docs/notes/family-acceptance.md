# Receiver family acceptance

Sources select matching-family receiver answers. IPv4-mapped IPv6 sources use
IPv4 sockets. A whole-bond DNS re-home removes sources without a matching
receiver family from the active pool; a later source-list reload can restore them.
The receiver's A and AAAA addresses must serve the same logical SRTLA group.
This is address compatibility, not reachability-based Happy Eyeballs.

## Kernel topology test

Run `bash scripts/netns_test_gate.sh --only netns_family` on Linux with
passwordless sudo, `ip`, `unshare`, `nsenter`, `mount`, `mknod`, `sysctl`, `nft`
and Python 3. Supply absolute executable paths in `SRTLA_RECEIVER` (dual-stack
[irlserver/srtla](https://github.com/irlserver/srtla) at
`877dc16e36d3fef421f9cffd345b54a526020732`, with `--srtla_port`,
`--srt_hostname`, `--srt_port`),
`SRTLA_IPV4_RECEIVER` ([BELABOX/srtla](https://github.com/BELABOX/srtla) at
`37862da3d0c13b46956efd3f88877053293d97d6`, positional-argument IPv4 receiver)
and `TAYGA_BIN` ([TAYGA](http://www.litech.org/tayga/) 0.9.2).
Without these explicit receiver prerequisites the Rust test reports a skip.
The gate allows 180 seconds and runs one test worker. All interfaces and TUN nodes
are created in private network, PID and mount namespaces; no host device is used.

The fixture sends thirty synthetic SRT-shaped markers per phase and checks
all thirty arrive through the receiver. It covers native IPv6, native IPv4,
a real CLAT/PLAT pair, mixed startup and same-process IPv6-to-IPv4 reload.
The reload case waits eight seconds to include the existing five-second grace
retry; it asserts eventual recovery rather than a prompt-rejoin deadline. The
translation destination route is removed before returning to native IPv4. These
markers do not qualify decoded media or hardware.

Open maintainer question: should matching-family destination selection become a
versioned parity contract, and if so which decision record should introduce it?
It is documented here as implementation behavior pending that decision.

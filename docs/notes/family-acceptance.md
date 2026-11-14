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
receiver with `--srtla_port`, `--srt_hostname`, `--srt_port`),
`SRTLA_IPV4_RECEIVER` (original positional-argument IPv4 receiver) and `TAYGA_BIN`.
Without these explicit receiver prerequisites the Rust test reports a skip.
The gate allows 180 seconds and runs one test worker. Run it in a transient
unit with UMask=0022, Nice=10 and CPUWeight=50. All interfaces and TUN nodes
are created in private network, PID and mount namespaces; no host device is used.

The fixture sends thirty synthetic SRT-shaped markers per phase and checks
all thirty arrive through the receiver. It covers native IPv6, native IPv4,
a real CLAT/PLAT pair and mixed startup. The translation destination route is
removed before returning to native IPv4. These markers do not qualify decoded media or hardware.

Open maintainer question: should matching-family destination selection become a
versioned parity contract, and if so which decision record should introduce it?
It is documented here as implementation behavior pending that decision.

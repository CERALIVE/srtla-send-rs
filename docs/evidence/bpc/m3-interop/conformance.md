# M3 foreign-sender conformance

Both directions captured before registration; zero kernel capture drops required.
Keepalive payload length/equality is measured, not inferred from protocol documentation.

| Sender | Check | Result | Evidence |
|---|---|---|---|
| belabox-c | registration | PASS | Both-link complete receiver group handshakes=2; group SHA256=('ec7981e34625728e8c59fcd064dba3f3a56f8c293cf4690fff70f47e981bce64', 'bcce5587c90bc668c6a30e76a2ec4216bf9210b8dfd0094de4d6502554ffcc5f') |
| belabox-c | keepalive_echo | FAIL | 10s idle supplement: requests/link=(9, 0); exact echoes/link=(0, 0); request lengths=(2,); unmatched/mismatched echoes=10; expected length=2. Loaded scenario-I requests/link=(0, 0), exact echoes/link=(0, 0) |
| belabox-c | receiver_restart | PASS | Restart event completed at measurement 20054ms (scheduled20000ms; completion lag upper bound=54ms); fresh receiver group=True; three-second sink recovery after completion=10946ms |
| irlserver-rust-classic | registration | PASS | Both-link complete receiver group handshakes=2; group SHA256=('528bbb0be65ddad102e39f8a9bcb0b0f3e6319142e79df734d32e3044579d91b', '053c886443d0d23591caa31e0cda106e96cb37f35e3e24592f286cf3ad64aacf') |
| irlserver-rust-classic | keepalive_echo | PASS | 10s idle supplement: requests/link=(8, 8); exact echoes/link=(8, 8); request lengths=(38,); unmatched/mismatched echoes=0; expected length=38. Loaded scenario-I requests/link=(53, 52), exact echoes/link=(48, 47) |
| irlserver-rust-classic | receiver_restart | PASS | Restart event completed at measurement 20135ms (scheduled20000ms; completion lag upper bound=135ms); fresh receiver group=True; three-second sink recovery after completion=11865ms |
| irlserver-rust-enhanced | registration | PASS | Both-link complete receiver group handshakes=2; group SHA256=('ac21e92f5185b602227396933f2a00bb0627409f0c0bdd17115bad39bdb3219f', 'e49ced1f18e20debd7f33e2ab4e3555d51056b1183f13893729ee4bb8a0e10cd') |
| irlserver-rust-enhanced | keepalive_echo | PASS | 10s idle supplement: requests/link=(9, 10); exact echoes/link=(9, 10); request lengths=(38,); unmatched/mismatched echoes=0; expected length=38. Loaded scenario-I requests/link=(52, 52), exact echoes/link=(47, 47) |
| irlserver-rust-enhanced | receiver_restart | PASS | Restart event completed at measurement 20059ms (scheduled20000ms; completion lag upper bound=59ms); fresh receiver group=True; three-second sink recovery after completion=11941ms |

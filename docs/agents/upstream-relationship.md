<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## UPSTREAM RELATIONSHIP

One permanent remote, `origin` (`https://github.com/CERALIVE/srtla-send-rs.git`). The
upstream remote is **transient**: added for a merge, removed before any push or PR.

- **Fork base:** upstream `df0b3938791ff24eced4aed8b29e3d49d0efb639` (v4.0.1). It stays
  an ancestor of `main` forever.
- **Last-merged upstream SHA:** `1481507ca1e278519fb77329a407074b2b62514b` (third 2026-11
  sync, ledger `docs/notes/upstream-sync-2026-11-1481507.md`; the sync before it, to `8677bb49`
  through `8db546c` and `b1cf1f2`, is `docs/notes/upstream-sync-2026-11-8db546c.md`, and the
  first, to `9e63d8f`, is `docs/notes/upstream-sync-2026-11.md`). The next sync's merge base
  is computed from it.
  The 2026-09 hard-fork closure ledger (bases across all four repos, ports/drops, A/B
  verdicts, erasure receipts, rollback) is `docs/notes/upstream-hardfork-2026-09.md`.
- **Merges are MANUAL and COMPAT-GATED.** No auto-sync, no scheduled merge, no bot PRs.
  Pull upstream deliberately, in a dedicated PR, when there is a reason to.
- Each merge runs the full gate (below) on the pinned toolchain and must not regress the
  parity contract. If upstream HEAD is red on formatting, green it with a mechanical
  `cargo fmt` + `clippy --fix` pass inside the merge PR; never import red state.
- Never `git fetch --tags` from upstream. The only release tags here are `v<version>`.

```bash
git remote add irlserver https://github.com/irlserver/srtla_send.git   # never 'upstream'
git fetch irlserver main:refs/remotes/irlserver/main
git rev-parse refs/remotes/irlserver/main            # pin-verify the SHA first
git merge refs/remotes/irlserver/main --no-ff -m "chore: merge upstream irlserver/srtla_send <SHA>"
git remote remove irlserver                          # BEFORE any push or PR
git remote -v                                        # must show origin only
```


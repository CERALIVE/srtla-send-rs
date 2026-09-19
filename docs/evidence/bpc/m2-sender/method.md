# M2 predeclared measurement protocol

Frozen before M2 measurements. Base: `0135515`; K=3. The referenced plan is
absent locally; the owner's Todo 17 request supplies the four terminal rules.

- One serial campaign, CPUs 4–27, existing whole-host measurement lock, seed
  20260913, one attempt per planned index. No performance retries or discarded
  settle timeouts. A settle timeout still measures the complete catalog window;
  infrastructure or incomplete metric records block reduction.
- 47 indices: in-flight 8 cells ×3, REXMIT 1×1, blindness 6×3, HSRSP 4×1.
  A–L definitions and their windows/load are unchanged. `S-HSRSP` is a separately
  named 20-second diagnostic (two 10-Mbit links, 5ms one-way, 1-Mbit offered),
  never an abbreviated A performance observation. REXMIT uses G plus a distinct
  noncovering capture variant. HSRSP and REXMIT are not ordinary verdict cells.
- All arms use the same release+test-internals sender. `enhanced-rule-disabled`
  differs from enhanced only by startup env `SRTLA_DISABLE_PREMATURE_NAK_RULE=1`.
  A production build never reads this variable. Invalid test-build values fail
  startup; absent/0 retain the shipped rule. No CLI option.
- Caller is the locked ours-new srt-live-transmit for every lineage, explicitly
  separate from the locked numeric irlserver sinks. Record its binary/DSO hashes.
  Old receiver retains genuine freeze-off behavior because its URI lacks support.
  New receiver uses M1 TTL*=200, freeze-on and periodic gate-on. Foreign numeric
  sinks use their locked 120=1 and 42=200 settings; do not force NAKREPORT to make
  the expected lineage observation pass.

## Frozen rules

Use the reporter's median bootstrap, 10,000 resamples, seed 20260913, 95% CI.
All three full-window outcomes enter each performance statistic, including failed
settling. Settle rate is separately the successful count /3.

1. **M2-INFLIGHT**: KEEP the rule. Report B1/C settle ON minus OFF. Trigger K=1
   iff A or G has ON median <0.95×OFF median AND ON CI upper <OFF CI lower.
   If triggered, change the production cap and repeat each triggering comparison
   once (both arms, N=3), preserving initial outcomes. Resolved means that same
   regression predicate is false on confirmation. Never remove the rule.
2. **M2-REXMIT**: one G/irlserver-prod/enhanced run. Start sender-loopback capture
   before the caller, destination UDP port5555; retain capture-drop log. Classify
   original/retransmission independently of the R bit: first occurrence vs repeat
   of (caller source port, SRT destination socket ID, 31-bit sequence). Require
   ≥1000 DATA, ≥1 repeated sequence, zero R=1 first occurrences and zero R=0
   repeats. Bit is message-word bit26, matching `packet.h` and get_srt_rexmit_flag.
   Zero kernel drops required for trustworthy classification. Checker exit0 means
   TRUSTED for this caller build, exit1 means `not visible on this encoder build`;
   execution/invalid-capture errors exit2 and block an evidentiary conclusion.
3. **M2-NAK-OFF**: G/F/A, irlserver-prod, enhanced/adaptive, N=3. Interpret the
   lower-confidence-bound comparison conservatively as adaptive G CI lower
   ≥1.05×enhanced G CI lower. “No regression” on A/F means adaptive median AND
   CI lower are each ≥enhanced's corresponding statistic (no invented tolerance).
   If all pass: `adaptive signals compensate`, ordinary verdict cells. Otherwise:
   `present`, known limitation with all numbers; do not assert causality from modes
   alone, because both now share delivery-proof admission signals.
4. **M2-HSRSP**: expected ours-old/on, ours-new/on, irlserver-prod/off,
   irlserver-next/on. Collect concrete `receiver: nak_report=` status lines,
   ignoring pre-handshake unknown. Exactly the expected observed value passes;
   absent/conflicting/mismatched observations record null (None, policy NAK-on).
   No parser changes or heuristics. Either decode result completes this diagnostic.

The numerical definitions above are fixed before any M2 data are inspected.

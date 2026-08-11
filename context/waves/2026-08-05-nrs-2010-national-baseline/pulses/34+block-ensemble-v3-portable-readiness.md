---
pulse: 34
wave: nrs-2010-national-baseline
date: 2026-08-11
status: complete
---

# Block-Ensemble v3 Portable Readiness

## Outcome

The mandatory prelaunch recheck stopped before admission or process creation.
The general v3 verifier passed with an empty active ledger, but the readiness
verifier rejected the wrapper binding after an ordinary branch switch on
Windows. The original readiness artifact had hashed a mixture of LF and CRLF
working-tree bytes, so semantically identical checked-out text could not
reproduce every recorded digest.

V3 readiness now hashes reviewable `.json`, `.md`, `.py`, and `.rs` custody
after canonical CRLF-to-LF normalization. Release executables and scientific
inputs remain byte-exact. Two focused tests prove that LF and CRLF text bind
identically while binary payloads do not.

The refreshed readiness record passes all 14 compiled probe replays, all input
audits, the pristine-package check, and the 8 GiB capacity formula. At the new
observation, 87,197,196,288 bytes were free against 8,589,934,592 required.

## Claim Boundary

This is a prelaunch custody repair. No admission record, runner resource
record, trace, ensemble draw, feasibility result, or convergence diagnostic was
created. Stage 0 remains at 0/6 preflights.

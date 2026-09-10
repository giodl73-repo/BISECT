# Hardened portability release record

Implementation revision: `5e3c668d306d93976da3b005bf14737d7f9ac5c7`.
Published branch: `fix/portable-vault-role-review-2026-09-10`.
Pull request: [#49](https://github.com/giodl73-repo/BISECT/pull/49).
This record does not imply the PR has merged or that pending CI has passed.

The immutable offline source bundle on M includes the implementation and its
full history. Its [receipt](../../../configs/data-vault/source-custody-portable-2026-09-10.json)
records the exact bytes and ref. A new clone was made from that bundle, with
the [lean sparse-checkout profile](../../LEAN_CHECKOUT.md), without sharing
objects or using the original checkout as its source.

That clone materialized 993 tracked files totaling **19,930,990 bytes**
(excluding `.git`, vault data and installed dependencies). Its isolated-environment
RI replay passed all six arms, reading the configured vault. The full indexed
tree at the implementation revision is 629,642,499 bytes, below the 650-MB budget.

## Acceptance evidence

- 63 local regression tests passed, including 28 vault/evidence tests.
- Fresh Windows Python 3.14.2 venv: dependency installation, canonical RI
  rebuilding, complete v2 admission receipt and six-arm replay passed.
- Both large ensemble traces recovered from an empty vault through pinned Git
  history, and independently from the main vault; hashes matched.
- Hydrated trace structure validation and byte-exact analysis replay passed.
- Real Windows link creation/relocation preserves source files and unrelated
  user directories. Dedicated Windows/Ubuntu CI is in the PR.
- `git bundle verify` passed; the bundle was actually cloned and replayed.

The [machine-readable remediation record](remediation.json) identifies the
tested code, runtime, packages and M-relative evidence locations. The original
validation record remains intact as a historical pre-hardening observation.

## Limits retained, not disguised as fixes

The [recovery matrix](../../DATA_RECOVERY_MATRIX.md) is the disposition of the
review's per-track coverage recommendation. Fresh 2000/2010 and all other-track
recovery are **not implemented** by this bounded 2020 CLI. National and non-Windows
geospatial rebuilding have not been established by the RI test.

The full old ensemble verifier detects pre-existing `block_trace.rs` source
drift on modern main. Its data hydration, trace validation and analysis replay
pass, but no claim is made that its full historical executable replay passed.

No data was destroyed: the two traces were removed from current Git tracking,
retained on M and hydrated back into ignored local paths. Historical Git objects
also retain them. Shared Git history was not rewritten. The removable drive
still needs a separate physical backup.

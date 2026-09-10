# NRS v0.3 Second-Laptop Verification

**Status:** clean-worktree vault-backed replay passed locally; ready for an
independent second-machine execution
**Purpose:** let a non-author verify the published three-cycle evidence or
reproduce all national baselines without private project knowledge

## Local custody-recovery replay

On 2026-09-09, the complete certified context corpus was recovered from
`M:\DATA_VAULT` and replayed from a clean detached worktree. All 150 governed
State-cycle assignment hashes matched, all three 50-State batches and national
summaries verified, both cross-census checks passed, and the independent
geographic verifier checked 231,765 rows. The compact machine-readable result
is `replication-record-2026-09-09.json`; the large generated packages and full
transcript remain in the data vault.

This closes the repository's local source-custody and clean-worktree replay
gap. It is not an independent second-laptop witness: the record identifies the
same project laptop, and a separate reviewer must still execute the protocol
below to support an external-replication claim.

## Independence protocol

Use a fresh clone on the second laptop. Do not copy this laptop's build
directory or generated `runs/` directory. Record the checked-out commit before
running anything. A failed or blocked run is evidence and must be preserved;
do not accept private corrections during the run.

The reviewer should return:

1. the JSON record produced by the selected script;
2. the transcript for a full replay;
3. the exact Git commit and operating-system/toolchain information;
4. every failure or deviation, including a different executable hash; and
5. a completed `replication-record-template.md`.

## Level 1: independent artifact verification

This takes a fresh clone, Python 3.10 or later, and PowerShell 7. It does not
need the 6.7 GiB Census context corpus and does not regenerate assignments.

```powershell
git status --porcelain
pwsh -File docs/external/nrs-v0.3-national-verification/verify-artifacts.ps1
```

The script independently verifies:

- the 2000, 2010, and 2020 publication manifests;
- all three compact node snapshots;
- the cross-census comparison manifest;
- the complete 231,765-row county/tract geographic audit;
- every national, pairwise, and per-State statistic through the separate
  Python implementations; and
- the frozen county-split counts (1,812, 1,819, 1,823) and tract-split counts
  (17,268, 18,800, 20,288).

Manifest checks accept raw bytes or the same UTF-8 text with only LF/CRLF
transport normalization. Binary files remain byte-exact, and the Python audit
still recomputes every reported statistic from parsed content.

A Level 1 pass verifies the committed evidence package. It does not prove that
the second laptop independently regenerated all 27,398,654 block
assignments.

The older v0.1 hostile challenge bundle is a separate historical gate and is
not included in this v0.3 fresh-clone result.

## Level 2: full national replay

Recommended minimums:

- 64-bit Windows, Linux, or macOS with PowerShell 7;
- Rust 1.95.0 and Cargo;
- Python 3.10 or later;
- 32 GiB RAM recommended; and
- at least 25 GiB free beyond the repository checkout.

The certified context corpus is not stored in Git. Supply exactly 50 RCTX
files for each cycle under:

```text
<data-root>/2000/certified/*.rctx   approximately 2.01 GiB
<data-root>/2010/certified/*.rctx   approximately 2.70 GiB
<data-root>/2020/certified/*.rctx   approximately 2.01 GiB
```

Transfer those inputs separately or reconstruct them from the published
source-custody process. The replay script verifies them against each cycle's
national RCTX package before generation. A direct external-drive copy is
acceptable, but preserve filenames and do not transfer this laptop's `target/`
or `runs/` directories. The RCTX files are inputs, not prior run outputs.

```powershell
pwsh -File docs/external/nrs-v0.3-national-verification/replay-national.ps1 `
  -DataRoot D:\nrs-certified-data
```

The script performs, for all three cycles:

1. a locked release build;
2. complete RCTX verification;
3. governed NRS v0.3 generation;
4. a separate `verify-nrs-batch --require-complete` pass;
5. national summary and proof-coverage publication;
6. compact node snapshot generation;
7. three-cycle comparison generation;
8. independent Python recomputation of the comparison;
9. verification of the committed county/tract audit; and
10. exact comparison of all 150 regenerated assignment hashes to the governed
    audit anchors.

Outputs and a transcript remain under `runs/nrs-v0.3/external-replay/`. Do not
delete them until the record has been reviewed.

On systems where Python 3 is named `python3`, add
`-PythonCommand python3` to either command.

## Expected national counts

| Cycle | Blocks | Population | States | Districts | Nodes | Expected failures |
|---|---:|---:|---:|---:|---:|---:|
| 2000 | 8,199,908 | 280,849,847 | 50 | 435 | 385 | 0 |
| 2010 | 11,071,790 | 308,143,815 | 50 | 435 | 385 | 0 |
| 2020 | 8,126,956 | 330,759,736 | 50 | 435 | 385 | 0 |

The expected comparison contains 120 all-cycle common node signatures and 18
all-cycle exact-topology States.

The expected geographic audit contains 231,765 State/level geography rows.
Maryland 2010 has a disclosed metadata-only tree-hash exception, but its
assignment hash must match exactly like every other State-cycle assignment.

## Claim boundary

A successful replay establishes independent reproducibility of the operational
assignments, package verification, and structural comparison on the tested
machine. It does not establish legal validity, VRA compliance, partisan
fairness, official adoption, or exact weighted-boundary/canonical optimality.
Exact boundary and canonical proof coverage remains 0/1,155 nodes.

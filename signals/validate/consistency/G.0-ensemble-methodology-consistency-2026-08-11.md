---
skill: validate-consistency
topic: G.0-ensemble-methodology
date: 2026-08-11
p1_count: 0
p2_count: 0
quantities_checked: 18
---

# G.0 Ensemble Methodology Consistency Check

## Scope

Post-Pulse-32 check of the full G.0 source, paper index, evidence inventory,
v2 terminal ledger, admission, resource record, and failure narrative. The
legacy paper has no `plan.md` or `_panel.yaml`, so the full post-write
orchestrator could not determine journal or stage; this focused consistency
check is the applicable fallback.

## Quantity and claim registry

| Q-ID | Quantity or claim | Value | Locations | Result |
|---|---|---|---|---|
| Q-01 | RI block units | 25,649 | G.0 section 5; retained RI package | PASS |
| Q-02 | RI kernels/chains | 4 Wilson + 4 Kruskal | abstract; section 5 | PASS |
| Q-03 | RI steps/burn-in | 2,000 / 500 | section 5; protocol posture | PASS |
| Q-04 | RI tolerance | 0.005 | section 5 | PASS |
| Q-05 | RI convergence rule | split R-hat <1.05, pooled ESS >=100 | section 5 | PASS |
| Q-06 | Wilson cut fraction | 0.0103331 | section 5 | PASS |
| Q-07 | Kruskal cut fraction | 0.00778270 | section 5 | PASS |
| Q-08 | Cut-fraction KS | 0.405833 | section 5 | PASS |
| Q-09 | Weighted cuts | 193,294,034 / 152,599,928 | section 5 | PASS |
| Q-10 | Weighted-cut KS | 0.31 | section 5 | PASS |
| Q-11 | Resource ceilings | 21 h; 2.25 GiB/process; 3 GiB retained/scratch | section 5; v2 protocol | PASS |
| Q-12 | v1 primary completion | 5/6 | abstract; section 5; conclusion; inventory | PASS |
| Q-13 | v1 terminal process | GA Kruskal disk exhaustion | same four locations | PASS |
| Q-14 | v1 governed replay | none | abstract; section 5; conclusion; inventory | PASS |
| Q-15 | v2 completion | 0 preflights; 0 governed chains | ledger; Pulse 32; inventory | PASS |
| Q-16 | v2 terminal process | first NH Wilson preflight | ledger; resource; Pulse 32 | PASS |
| Q-17 | v2 return/wall/RSS | 1; 1.3137548999511637 s; 4,722,688 bytes | resource; terminal narrative | PASS |
| Q-18 | v2 integration cause | execution class rejected; compiled validator retained predecessor seed | source; paper; terminal narrative | PASS |

## Cross-checks

- The abstract, section 5, conclusion, paper index, and evidence inventory all
  keep v1's partial scientific evidence distinct from v2's zero-draw
  implementation-integration failure.
- The observed error was `unsupported execution class`. Wording was amended so
  it does not claim that the process separately emitted a seed error; the source
  inspection establishes the additional predecessor-seed incompatibility.
- The readiness observation and process admission use different free-byte
  values because they were taken at different times; both exceed the same
  frozen 8 GiB requirement and are labeled separately.
- No text promotes v2 to feasibility, convergence, sampler comparison, or
  statistical evidence.

## Inconsistency register

No open P1, P2, or P3 inconsistency remains in the Pulse 32 additions.

## Amendments incorporated

1. Distinguished the observed execution-class rejection from the independently
   inspected predecessor-seed validator contract.
2. Added v2's zero-completion status to the paper inventory without changing
   the v1 bounded-evidence result.
3. Rebuilt G.0 after aligning the abstract, bridge, and conclusion.

# BOARD-REVISION-PLAN — Track H (Search Strategies)
**Source**: REVIEW_BOARD.md Round 1 (2026-05-07)
**Track score**: 8.4/10 → target 8.5+/10

## B1 Items (Blocking)

**B1.1 [ensemble-search: G + H]** — H.0 cites G.1 ensemble percentiles as unqualified
H.0's adversarial bar argument ("challenger must produce a plan more compact than 99.8% of all valid plans") depends on G.1 ensemble percentiles that are point estimates without ESS-based uncertainty bounds. Until G.1-A (ESS correction) and G.4-C (ESS formula) are complete, add the following qualification to H.0 §5.1: "This bar is computed from G.1 GerryChain ensemble data. The percentile figures are point estimates contingent on G.1's sampling adequacy; a more conservative statement is that the p=0.0 plan is more compact than approximately 99%+ of G.1 ensemble plans, with the exact percentile subject to sampling uncertainty." This is W9 from H.0's own revision plan.

## B2 Items (Important)

None unique to Track H at the board level. The B.11 cross-citation (H.1 → B.11) is correctly handled; no action needed.

## B3 Items (Nice-to-have)

**B3.H** —
- H.0: complete R3 revision (6 P1 prose items — all prose, no new experiments; 3–4 hours total)
- H.1: complete R3 revision (1 must-fix factual error GA/PA + 4 should-fix items)
- H.2: add R04 planarity sentence (1 sentence, 2 minutes) → then submit to USENIX
- H.3: generate review round (r1_*.md files); create _panel.yaml

## Critical Path to 8.5+/10
H.2 planarity sentence (today) → submit H.2 → H.0/H.1 R3 revisions (1–2 weeks) → H.3 review round → submit H.0, H.1, H.3 → board sign-off on Track H
Time estimate: Track H can complete all B3 items within 4 weeks and achieve board sign-off.
Track H is the program's most board-ready module.

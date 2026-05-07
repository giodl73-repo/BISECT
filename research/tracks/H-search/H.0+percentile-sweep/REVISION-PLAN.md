# REVISION PLAN — H.0 PercentileSweep (Round 2)

**Paper**: PercentileSweep: Statutory Choice of Legal Posture in Algorithmic Redistricting
**Round**: 2
**Date**: 2026-05-05
**Average score**: 3.4 / 4

## Score Summary

| Reviewer | R1 Score | R2 Score | Delta | Disposition |
|---|---|---|---|---|
| Karypis | 3/4 | 4/4 | +1 | Accept |
| Rodden | 3/4 | 3/4 | 0 | Accept with revisions |
| Duchin | 2/4 | 3/4 | +1 | Accept with revisions |
| Stephanopoulos | 3/4 | 3/4 | 0 | Accept with revisions |
| Liang | 3/4 | 4/4 | +1 | Accept |
| **Average** | **2.75 / 4** | **3.4 / 4** | **+0.65** | **Conditional accept** |

Round 2 target (≥ 3.2/4) **met**. Duchin's blocking concern (D-P1-A statute box) resolved.
Karypis and Liang are now Accept. Duchin moved from 2→3.

---

## Round 1 → Round 2 Resolution Summary

### W3 — RESOLVED (Duchin blocking)
Statute box §5.2 now contains the bisection-family qualification sentence. D-P1-A closed.

### W4 — RESOLVED (Liang, Rodden)
Abstract updated to "zero partisan seats change for four of six states; TX and CA show at most 0.5 seats variation." §4.4 retitled and restructured. L-P1-B closed.

### W1/W2 — PARTIALLY RESOLVED (Rodden, Liang)
TX and CA now explicitly labeled as B.11 extrapolations, not confirmed PS sweeps. Four-state confirmed claim separated from two-state conjecture. R-P1-B and L-P1-C closed. TX/CA actual sweeps deferred to H.2.

### W5 — RESOLVED (Karypis)
SHA-256 truncation: bytes 0–3 big-endian, masked to 2^31−1. K-P1-B closed.

### W6 — RESOLVED (implicit, Karypis)
Seed-chain vs. independent-hash explained in §3.2. K-P1-A closed.

---

## Open Items After Round 2

### P1 Items Remaining (block final publication)

| ID | Item | Reviewer | Round 1 ID |
|---|---|---|---|
| W3b | §5.1: Reorder paragraphs — lead with insensitivity arg before zero-sum arg | Duchin | D-P1-B |
| W7 | Add geographic-sorting acknowledgment + Rodden (2019) citation | Rodden | R-P1-A |
| W9 | Add G.1 ensemble-percentile-as-point-estimate qualification to §5.1 | Duchin | D-P1-C |
| W10 | Add structural-bias challenge route + B.0/B.17 cross-ref to §5 | Stephanopoulos | S-P1-B |
| W11 | Qualify Karcher analogy as advocacy-by-parallel, not precedent | Stephanopoulos | S-P1-A |
| W13 | Add VRA interaction paragraph to §5 (GA specifically; Callais cross-ref) | Duchin, Stephanopoulos | D-P1-D / S-P1-C |

### P2 Items (new from Round 2)

| ID | Item | Reviewer |
|---|---|---|
| W15 | Fix §1.3 introduction body — still says "at most 0.5 seats" (not updated to match abstract) | Karypis, Liang |
| W16 | Add forward reference to H.2 for $T$-sensitivity check (GA/NC at $T = 601$) | Karypis, Liang |
| W8 | Bootstrap/jackknife CI for $p = 1.0$ partisan outcome (GA, NC) | Rodden (softened to P2) |

---

## Round 3 Target

Resolve W3b, W7, W9, W10, W11, W13 (all prose-level, no new experiments).
Fix W15 and W16 (minor sentence-level).
Aim for average score ≥ 3.75 / 4.
Binding constraints: Rodden (W7), Stephanopoulos (W11, W10, W13), Duchin (W3b, W9, W13).

---

## Round 1 P1 Checklist — Final Status

### From Karypis

- [x] **K-P1-A**: Seed chain vs. independent seeds resolved in §3.2.
- [x] **K-P1-B**: SHA-256 truncation convention added (bytes 0–3 big-endian, masked to 2^31−1).
- [ ] **K-P1-C**: $T = 601$ GA/NC sensitivity check — deferred to H.2 (forward reference needed).

### From Rodden

- [ ] **R-P1-A**: Geographic-sorting paragraph still absent. Rodden (2019) not cited. **Blocking for Rodden.**
- [x] **R-P1-B**: TX/CA extrapolation clearly labeled; four-state claim confirmed.
- [ ] **R-P1-C**: Bootstrap CI not provided (softened to P2 by Rodden given narrowed scope).

### From Duchin

- [x] **D-P1-A**: Statute box §5.2 bisection-family qualification added. **Blocking concern RESOLVED.**
- [ ] **D-P1-B**: §5.1 paragraph order — insensitivity arg still buried after zero-sum arg.
- [ ] **D-P1-C**: G.1 ensemble percentile qualification not added.
- [ ] **D-P1-D**: VRA and non-compactness criteria not addressed in §5.

### From Stephanopoulos

- [ ] **S-P1-A**: Karcher-by-analogy qualifier not added to statute box or §5.1 prose.
- [ ] **S-P1-B**: Structural-bias challenge route not acknowledged; B.0/B.17 not cross-referenced in §5.
- [ ] **S-P1-C**: VRA posture omitted from taxonomy.

### From Liang

- [ ] **L-P1-A**: $T = 601$ sensitivity check not run (mitigated by scope narrowing; H.2 forward-ref needed).
- [x] **L-P1-B**: Abstract/§4.4 inconsistency resolved.
- [x] **L-P1-C**: TX/CA extrapolation correctly labeled throughout.

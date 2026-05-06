# Quickstart: Federal Statute (Districting Integrity Act)

**Who you are:** Congressional staff, a policy advocate, or anyone who wants to understand what the proposed Districting Integrity Act would require and how to verify compliance.

**What you'll have at the end:** A clear picture of how the DIA mandate connects to existing law, how to run the DIA-default plan for any state, and how to verify that any submitted plan matches its claimed inputs byte-for-byte.

**Time:** 5 minutes to read; 15–30 minutes to run the full 50-state DIA default.

---

## What is the Districting Integrity Act?

"Each State shall apportion its congressional districts by applying the ApportionRegions algorithm to its census tracts."

That is the operative mandate in one sentence. The full text is at `docs/legal/MODEL_FEDERAL_STATUTE.md`. The statute specifies the algorithm (recursive graph bisection), the inputs (TIGER shapefiles + PL 94-171 census data), the tolerance schedule (0.5% per district), the edge-weight rule (TIGER boundary lengths in metres), and the random seed (published 18 months before each census release by the Director of the Census).

The DIA is deliberately analogous to 2 U.S.C. § 2a, which mandates Huntington-Hill apportionment. Congress prescribes the method; states execute; any person can verify.

---

## How it connects to Huntington-Hill

The connection is not just analogical -- it is mathematical.

Huntington-Hill produces a priority sequence that assigns each new seat to the state with the largest H-H value. For a state with k seats, the sequence implies a prime factorisation of k: each prime factor corresponds to a bisection level in the recursive tree.

Example: North Carolina receives 14 seats.

```
14 = 7 x 2
```

The ApportionRegions algorithm bisects NC into 7 parts first (each receiving 2 seats), then bisects each of those into 2. The bisection tree is derivable from the Huntington-Hill priority order: no new law is needed to justify the structure, only the recognition that the existing apportionment statute already implies it.

This derivation is formalised in `docs/RECURSIVE_BISECTION.md` and defended in the B.11 paper (`docs/PAPERS.md`).

---

## Running the DIA default

The official configuration reads from `configs/official_proposal.yml`, which sets:
- Structure: `apportion-regions` (prime factorisation)
- Weights: `county-sticky` (TIGER boundary lengths with county-crossing bonus)
- Search: `convergence` sweep with T=600 seeds

```bash
# Run the DIA default for all 50 states, 2020 census
redist build official_2020 --year 2020
```

This is equivalent to:
```bash
redist run --year 2020 \
    --version official_2020 \
    --partition-mode apportion-regions \
    --weights county \
    --search convergence \
    --workers 12
```

Expected time: 15–30 minutes on a 4-core laptop (the `redist` binary is ~200x faster than the archived Python pipeline).

Expected output: `outputs/official_2020/2020/{state}/final_assignments.json` for all 50 states.

---

## Verifying any DIA run

The SHA chain in `manifest.json` proves that the output matches the inputs byte-for-byte. Run:

```bash
redist label-verify official_2020 --year 2020
```

This re-hashes every input file (TIGER shapefiles, PL 94-171 data) and compares against the SHA-256 values recorded in `manifest.json` at run time. A passing result means:

1. The input data has not changed since the run
2. The binary version matches the manifest's `redist_build_commit`
3. `algorithm_output.json` is byte-identical to what the reference implementation produces from those inputs

A failing result means the run is not reproducible from the stated inputs -- which is the core finding the DIA's private right of action would allow any citizen to litigate.

For a single state:
```bash
redist doctor --verify-manifest outputs/official_2020/2020/north_carolina/manifest.json
```

---

## Key results

The 50-state DIA default sweep (`configs/official_proposal.yml`, 2020 census) produces:

| Metric | DIA default | Enacted maps |
|--------|-------------|--------------|
| National seat total | 223D / 209R | 220D / 215R (approx enacted) |
| Mean Polsby-Popper | +22% vs enacted | baseline |
| NC outcome | 7D / 7R | 10R / 4D (enacted) |
| States with near-proportional outcomes | 34 / 50 | -- |

NC is the headline case: k=14=7x2 forces a symmetric bisection tree. The algorithm gives 7D/7R with a 0.7 percentage-point partisan gap -- as close to perfectly proportional as geography permits. The enacted map gives 10R/4D.

The 223D/209R national total is not a design target. It is the output of a partisan-blind, geographically compact algorithm applied to the 2020 population. The +3 Democratic seats relative to rough proportionality reflects the geographic sorting of the US population (Democrats cluster in cities; see `docs/PAPERS.md` B.11 for the Rodden-gap analysis).

---

## Where to read more

- Full statute text: `docs/legal/MODEL_FEDERAL_STATUTE.md`
- Policy memo (why this approach is constitutional): `docs/legal/STATUTE_RATIONALE.md`
- 90-second version: `docs/legal/STATUTE_ONE_PAGER.md`
- B.02 paper (recursive bisection algorithm): `docs/PAPERS.md`
- B.11 paper (prime factorisation and 50-state sweep): `docs/PAPERS.md`
- State-court companion strategy (post-Rucho): `docs/legal/FAIRNESS_DOCTRINE.md`
- SHA chain and manifest format: `docs/file-formats/manifests.md`

# Track R — Adversarial Robustness and Gaming Resistance

**Theme**: When a hostile actor knows the algorithm, can they game it to produce a partisan outcome while maintaining "algorithmic neutrality" as cover? Track R systematically catalogues gaming vectors and shows that bisect's architecture makes meaningful manipulation either detectable or ineffective.

**Core question per paper**: Can gaming vector X produce a partisan outcome undetectable in the algorithm's output, and if so, what is the audit mechanism that catches it?

## Track Chain

R.0 (gaming taxonomy) → R.1 (parameter gaming) → R.2 (input data manipulation)
                      → R.3 (geography/GEOID gaming) → R.4 (audit mechanisms)

## Papers

| Paper | Title | Gaming Vector | Defense Mechanism |
|-------|-------|--------------|------------------|
| R.0 | A Taxonomy of Gaming Vectors in Algorithmic Redistricting | Overview | Audit chain |
| R.1 | Parameter Gaming: Can Parameter Tuning Produce Partisan Plans? | Edge weights, tolerance | B.17 cross-reference; parameter disclosure |
| R.2 | Input Data Manipulation: Census Data, GEOID, and Shapefile Integrity | Population counts, adjacency | SHA-256 audit chain; Census Bureau provenance |
| R.3 | Geographic Gaming: Can Tract Definition Choices Produce Partisan Plans? | Resolution choice, adjacency definition | Resolution pre-registration; manifests |
| R.4 | The Audit Chain as a Gaming Defense: Provenance, Reproducibility, Tamper Detection | All of the above | bisect label-verify; SHA-256; manifest schema |

## Contracts

Every R.1–R.3 paper must:
- Define the gaming vector precisely (what exactly would a hostile actor change?)
- Implement the attack on NC/TX and report the maximum achievable partisan shift
- Show that the shift is either (a) detectable via the audit chain, or (b) smaller than sampling noise
- Propose a countermeasure that does not require trusting the map-drawer

## Key Architectural Defense

Bisect's gaming resistance rests on three structural properties:
1. **No partisan data ingested**: the algorithm cannot use partisan data to gerrymander, even if instructed to
2. **SHA-256 audit chain**: any modification of input data or parameters is cryptographically detectable
3. **Public data provenance**: all inputs (Census P.L. 94-171, TIGER/Line) are public-domain and independently verifiable

R.4 formalizes these as a theorem: given the three structural properties, the only gaming vectors that could produce undetectable partisan outcomes are those that corrupt the Census data at its source — which is outside the map-drawer's control.

## Expected Finding

R.1 will reproduce B.17's result: parameter gaming produces at most 0.3 D-seats of variation nationally (well within sampling noise). R.2 will show that Census data manipulation is detectable via SHA-256 hash of the P.L. 94-171 file. R.3 will show that resolution choice is the most consequential design decision, but only within the range established by F.3's threshold rule.

## Legal Utility

In litigation, the defense can point to Track R to respond to "the algorithm can be gamed" objections: "We've tested every gaming vector; here is the maximum achievable shift; here is why it's detectable." This converts an abstract concern into a quantified, bounded, auditable claim.

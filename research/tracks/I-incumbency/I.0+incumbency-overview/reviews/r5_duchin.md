# Review — I.0: Incumbency and Algorithmic Redistricting: Overview
**Reviewer**: Moon Duchin (Metric geometry, redistricting methodology)
**Round**: 1
**Score**: 2/4
**Verdict**: Major Revision

## Summary
The incumbency-neutral baseline is a good idea, but the implementation is incomplete. The baseline is described as "expected value under random redistricting given geographic constraints" — but the paper does not specify what distribution over plans "random redistricting" means. Is this the ReCom ensemble? Uniform over all valid plans? The bisection plan family? The baseline is underspecified enough to be unverifiable.

## Concerns
- **Baseline underspecification**: The paper derives an analytical expectation for pairing rates but the derivation assumes independent Bernoulli pairing, which ignores the correlation between pairings (two incumbents being in the same district is correlated with how the boundaries fall elsewhere). The formula $p_{\text{pair}} = 1/(k-1)$ holds only under a specific exchangeability assumption that is not validated.
- **Single-seed**: The single-seed limitation is acknowledged but not adequately addressed. All four substantive conclusions (NC pairing rate within range, WI within range, TX within range, enacted maps below range) are from one seed. A wrong seed could flip all four.
- **No validation against ensemble**: The paper should compute the same pairing metrics on the GerryChain ensemble for NC (available from G.1) and compare to both the algorithmic plan and the analytical baseline. This would validate whether the baseline is correctly specified.

## Required Changes
- **P1**: Validate the pairing independence assumption or replace the analytical formula with a simulation-based baseline from the ReCom ensemble.
- **P1**: Multi-seed results for at least NC before the paper can claim "within range."
- **P1**: Compare pairing rate in the algorithmic plan to the G.1 GerryChain ensemble, not only to the analytical baseline.
- **P2**: Clarify what distribution "random redistricting" refers to throughout.

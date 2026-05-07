# Review: G.0 — Ensemble Methodology
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.0 establishes the methodological framework for the G-series, distinguishing ensemble methods (sampling the plan space to characterise a distribution) from the ApportionRegions (AR) algorithm (computing one canonical minimum-edge-cut plan). The paper introduces the percentile framework, outlines convergence diagnostics, and connects the ConvergenceSweep (CS) stopping criterion to ensemble mixing diagnostics. The framework is logically sound and the two-goal distinction is the paper's strongest intellectual contribution.

## Strengths

The "Goal A vs. Goal B" framing in Section 1 is exactly the kind of clarifying distinction this literature needs. It correctly identifies that ensemble methods and deterministic partition algorithms answer different questions, and it sets up the G-series to use ensembles as corroboration rather than as a competing method. The summary table comparing Flip, ReCom, Short-Burst, and AR is well-constructed.

The percentile definition (Eq. 1) is cleanly stated. The decision to use empirical counts rather than normal approximation as the authoritative figure — made explicit in the NC discussion in later papers — is methodologically sound for skewed distributions.

The convergence diagnostics section correctly notes that ReCom typically achieves higher ESS than flip chains of the same length, which is important context.

## Weaknesses

**The CS-to-ensemble bridge (Section 5) conflates two different notions of robustness.** ESS measures the variance of an estimator over repeated draws from a distribution. The CS threshold T=600 measures stability of one specific objective (normalised edge cut) under seed perturbation. These are not analogous: ESS answers "how many independent samples do I effectively have?" whereas T=600 answers "have I found the best METIS seed?" The paper asserts these are "complementary certificates for different goals" but does not articulate what makes them comparable in any quantitative sense. A stronger bridge would require showing that the 600-seed tail is calibrated against the plan space in a way that parallels ESS.

**The claim that the AR plan falls at the "65th–75th percentile of the ReCom compactness distribution" in Section 5.2 is presented as an estimate but no derivation is shown.** G.1 eventually provides state-specific figures (61st–72nd), but this framing paper should either defer to G.1 explicitly or note that these are forward-referencing estimates pending G.1. As written, the numbers appear to be asserted rather than derived.

**The flip chain description (Section 2.1) contains an error of emphasis.** The paper says the flip chain produces "a sample from the stationary distribution of valid plans" and that this distribution is "uniform." The stationarity claim depends on the acceptance criterion, which the paper does not specify. A heat bath / Gibbs flip is uniform over plans; a Metropolis flip on a general state space may not be. This conflation could mislead readers about what the Herschlag 2020 ensemble is sampling.

## Minor Issues

- The Rhat formula in Section 4 includes a factor $(m+1)/(mn)$ for the between-chain term in $\hat{V}$. The standard Gelman-Rubin formula uses $1/n$. The $(m+1)/(mn)$ version appears in the Vehtari 2021 update and is correct, but the paper should cite Vehtari 2021 here rather than only the 1992 paper.
- The notation $\pi_f(P^*)$ (percentile) clashes with the common use of $\pi$ for the stationary distribution in MCMC contexts. Consider $\mathrm{pctl}_f(P^*)$ or $q_f(P^*)$.
- Table 1 lists the AR plan's "best evidentiary use" as "Canonical plan" — this is not a use in the ensemble sense and should perhaps be noted as "N/A (not a sample)" with a separate row for its evidentiary role.

## Recommendation

Accept with minor revisions. The framework is sound and the two-goal distinction is valuable. The bridge section needs tightening: either prove a formal analogy between the CS tail and ESS, or explicitly disclaim the analogy and describe them as independent certificates for different purposes.

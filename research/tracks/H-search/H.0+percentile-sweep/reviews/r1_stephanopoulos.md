---
reviewer: Nicholas Stephanopoulos
round: 1
score: 3
date: 2026-05-06
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting

### Strengths

The paper offers a genuinely useful legal framework. The mapping of compactness percentile to legal doctrine — Karcher for $p = 0.0$, representativeness norm for $p = 0.5$, ensemble benchmark for TargetedSweep — is a clear and novel contribution. The recommendation of $p = 0.0$ is well-motivated on three independent grounds (legal strength, determinism, partisan neutrality), and the statutory box format makes the posture practically implementable. The Karcher analogy is apt: that case's requirement that population deviations be justified by legitimate state objectives translates naturally to a requirement that departures from maximum compactness be justified, and the $p = 0.0$ plan eliminates that burden entirely.

The treatment of Rucho is accurate. The paper correctly notes that Rucho forecloses federal judicial review of partisan claims and positions the PercentileSweep framework as a statutory response that sidesteps the foreclosed federal terrain. The insensitivity finding is then precisely the argument that makes this sidestep work: because partisan outcomes are invariant to percentile choice, the legislature cannot be accused of making a partisan choice when it specifies a particular $p$.

The Callais citation (§3.3, via callais2026) is correctly positioned: the disentanglement requirement on race-conscious and partisan signals is satisfied at any percentile because the bisection objective is pure edge-cut minimisation with no racial or partisan inputs. This is a clean argument.

### P1 Items (must resolve before acceptance)

**P1-A: The Karcher analogy requires more precise doctrinal grounding.**
The paper uses Karcher for the proposition that "any departure from maximum compactness [must] be justified by a legitimate purpose." But Karcher is a population-equality case, not a compactness case. The holding is narrow: near-mathematical equality of population is required for congressional districts, and any deviation must be justified. The analogy to compactness is by structural parallel, not by precedent. There is no federal holding that establishes a similar strict-scrutiny standard for compactness. The paper should acknowledge that the Karcher analogy is an advocacy argument — one that has not yet been tested — not an established doctrine. The statute box for Compactness Posture should say "under the Karcher framework by analogy" rather than implying direct doctrinal support.

**P1-B: The recommendation of $p = 0.0$ needs engagement with the "compactness as a shield" critique.**
The paper argues that $p = 0.0$ is legally strongest because "any challenger who claims the plan is too compact must explain why a less compact plan would better serve any legitimate redistricting objective." This is correct as far as it goes, but it assumes that the only legal challenge is a compactness challenge. There is a distinct challenge route: the challenger claims the algorithm family itself is structured to produce a particular partisan outcome, and choosing $p = 0.0$ merely selects the most extreme example of that structural bias. Under this challenge theory, the insensitivity finding is not a defense — it is the evidence that the structure bakes in the partisan outcome regardless of $p$. The paper should acknowledge this challenge route and distinguish it from the over-optimising objection. The insensitivity finding is a defense against the latter (choice of percentile is not a partisan choice) but not necessarily against the former (choice of algorithm family is not addressed by H.0; it is addressed by B.0 and B.17, which should be cross-referenced here explicitly).

**P1-C: The three-posture taxonomy omits the VRA posture.**
Section 5 presents three legal postures mapped to three percentile choices. But any redistricting statute operating in a covered jurisdiction must also address Voting Rights Act compliance. The paper cites Callais but does not consider how the posture choice interacts with VRA requirements. For states like Georgia (§4.3), where the $p = 0.0$ plan produces 6D/8R with a 14-seat delegation and a significant Black voting-age population, the question of whether the compactness extremum creates or dilutes majority-minority districts is legally material. The paper should either (i) note that VRA compliance is evaluated independently of percentile choice (if the bisection objective does not interact with minority district formation), or (ii) acknowledge that a fourth posture — VRA-compliant compactness — may require a constrained percentile selection that is not $p = 0.0$.

### P2 Items (recommended improvements)

**P2-A:** The TargetedSweep posture (§5.3) is dismissed partly on grounds of "ensemble definition (which constraints are imposed, how plans are scored) must be specified in statute." This is correct but the paper should note that this specification problem is not unique to TargetedSweep: it applies to any algorithmic redistricting statute, including the DIA's PS specification. The critique is symmetric.

**P2-B:** The paper frames the insensitivity finding as making the choice of $p$ "primarily a legal and normative choice rather than an empirical one" (§5, opening paragraph). This framing is legally attractive but should be qualified: it holds only within the tested parameter space ($T = 101$, six states, 2020 data). A court might ask whether insensitivity holds across census years and state size ranges; the paper should note that B.17's 50-state sweep provides partial validation but that the full claim requires the pending TX and CA sweeps.

**P2-C:** The statute boxes use the phrase "Districting Administrator" without defining this office. If the paper is intended for DIA legislative drafters, this term should either be defined or replaced with a neutral placeholder like "the relevant official designated by statute."

### Score

**3 / 4** — Accept with revisions. The legal framework is the paper's primary contribution and is largely sound, but the Karcher analogy (P1-A) should be more carefully qualified, the structural-bias challenge route (P1-B) needs acknowledgment, and the VRA gap (P1-C) is a material omission for any state with significant minority populations.

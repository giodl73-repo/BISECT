---
reviewer: Moon Duchin
round: 1
score: 2
date: 2026-05-06
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting

### Strengths

The paper makes a genuine and important algorithmic contribution: by exposing the compactness percentile as a statutory parameter, it converts an implicit design choice into a visible, auditable, and legislatively debatable quantity. This is exactly the kind of transparency move that makes algorithmic redistricting defensible in adversarial legal settings. The three posture taxonomy — compactness doctrine, representativeness doctrine, full ensemble representativeness — is a useful organizing framework, and the recommendation of $p = 0.0$ is defensible given the paper's empirical findings.

The acknowledgment that the bisection-space distribution is not the ReCom ensemble is, in my view, the paper's most important contribution to the redistricting methods literature (§3.5). This distinction is frequently muddled in policy discussions. The paper states it clearly and correctly, and the MedianSweep subsection (§3.4) draws the implication explicitly: $p = 0.5$ is the median of the bisection family, not the median of all valid plans. This is correct and important.

### P1 Items (must resolve before acceptance)

**P1-A: The "median bisection plan" legal claim is more misleading than clarifying.**
The paper introduces $p = 0.5$ as the "representativeness posture" and offers the statutory language: the plan is "equally likely to be produced by any party running the algorithm with a uniformly random seed from the statutory seed space." This is true within the bisection family but will almost certainly be misread by courts and legislators as "equally likely to be produced by any party running any redistricting algorithm." The paper acknowledges the distinction in §3.5 and §3.4, but the statute box in §5.2 does not contain this qualification. A court reading only the statutory box would conclude that the $p = 0.5$ plan is representative of the space of all valid plans — which it is not. This is a legal malpractice risk. The statute box for Posture 2 must include a sentence explicitly stating that "this plan is representative of the bisection family, not of the full space of valid redistricting plans." Without that, the paper's clearest contribution (§3.5) is undermined by its own policy output.

**P1-B: The over-optimising objection response is circular for $p = 0.0$.**
Section §5.1 responds to the over-optimising objection by arguing: "Compactness is not a zero-sum quantity: maximising compactness does not require minimising anything else." This is not a legal argument — it is an assertion. The over-optimising objection as framed is that extreme compactness *within the bisection family* may be a form of manipulation, specifically that cherry-picking the very best seed from 600 tries privileges an outcome that no neutral, unoptimised draw would produce. The response should engage this framing directly. The correct response is the insensitivity finding itself: even if you cherry-pick the best seed, the partisan outcome is identical to a random draw. The paper makes this point in §5.1 ("the insensitivity finding confirms that this compactness concession does not affect partisan outcomes") but buries it after the circular argument. The paragraph order should be inverted: lead with the insensitivity argument, then add the "compactness is not zero-sum" observation as a secondary point.

**P1-C: The ensemble coverage assumption underlying G.1 is not validated.**
The entire legal analysis depends on the G.1 ensemble positions being reliable estimates of the true population percentiles. But G.1 (as referenced) provides no mixing-time analysis, no diagnostics for the ReCom chain, and no confidence intervals around the percentile estimates. The paper inherits these limitations and amplifies them: it uses the ensemble positions to argue that $p = 0.0$ sits at the "0.1st percentile" and that any challenger "must produce a plan more compact than 99.9% of all valid plans." If the G.1 ensemble under-samples the tails of the plan space — as ReCom chains often do near constraints — this bound is overstated. The paper should at minimum note that the ensemble percentiles are point estimates without confidence bounds and that the adversarial bar argument is contingent on G.1's sampling adequacy.

**P1-D: No discussion of non-compactness redistricting criteria.**
The legal posture analysis (§5) considers only compactness, population balance, and contiguity. But most state statutes require additional criteria: political subdivision preservation (county lines, municipality lines), community of interest, and increasingly, VRA compliance. Compactness maximisation at $p = 0.0$ may conflict with these criteria in ways that alter the legal posture substantially. For example, maximising compactness in a state with strong political subdivision preservation requirements may force the algorithm to split counties that a legislature would wish to keep whole. The paper cannot claim the $p = 0.0$ posture is "legally strongest" without at minimum acknowledging that other statutory criteria may impose constraints that the bisection family cannot satisfy while remaining at $p = 0.0$.

### P2 Items (recommended improvements)

**P2-A:** The TargetedSweep(50th, ReCom) posture (§5.3) is dismissed as "a research direction rather than an immediate statutory option" on grounds of operational complexity. But it is also the only posture that would be defensible to a judge familiar with the redistricting science literature, since it is the only one that targets the actual space of valid plans. The paper should be more precise about why it is deferred and what mixing-time guarantees would be needed to make it statutory-grade.

**P2-B:** The Duchin & Tenner (2020) reference is cited but not engaged. The discrete geometry framework in that paper directly addresses the geometry of the bisection-plan distribution vs. the full plan space. A one-paragraph engagement would strengthen the theoretical grounding of §3.5.

**P2-C:** The NC case is described as "geographic convergence" but this term is not defined. Clarify: geographic convergence means that the bisection-tree topology for $k = 14$ in NC constrains the leaf-level METIS output so severely that all valid bisections produce nearly the same plan boundary. This is a substantive claim that warrants a sentence of explanation rather than a cross-reference to G.1.

### Score

**2 / 4** — Major revisions required. The bisection/ensemble distinction is correctly stated but then violated in the statutory language (P1-A), which creates a significant legal risk. P1-B and P1-D are also substantive gaps. The paper is close to acceptable but requires a careful revision of §5 before it can be submitted.

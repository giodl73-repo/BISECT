# Review: NestSection — Moon Duchin

**Reviewer**: Moon Duchin (Tufts University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, gerrymandering detection, ensemble methods, election law
**Round**: 1
**Score**: 2/4 (major revision required)
**Recommendation**: Major Revision

---

## Summary

NestSection proposes a cross-chamber redistricting algorithm that constrains congressional, state senate, and state house maps to share a common bisection hierarchy. The central technical construct — the compatible factorization spine — is mathematically clean. The 50-state compatibility census is a useful empirical contribution. However, the paper's legal and normative framing, which is the section most likely to matter for this venue (Harvard Journal on Legislation), is substantially underdeveloped. The claim that spine-sharing is an "anti-gerrymandering mechanism by construction" is stated as a design principle but is not empirically validated, legally connected to existing doctrine, or even sketched at a level of rigor that would support a statutory proposal. I recommend major revision before this paper is ready for publication.

---

## Strengths

1. **Novel algorithmic contribution.** No prior redistricting paper, to my knowledge, addresses cross-chamber consistency as a computational objective. The CompatibleSpines construction is clean and the compatibility score is a useful summary statistic.

2. **Bimodal finding is surprising and worth documenting.** The complete absence of states in the 0 < sigma < 50 range is a genuine empirical observation that deserves explanation and wider awareness.

3. **Honest about single-seat triviality.** The paper correctly acknowledges in Section 5.2 that the 7 single-seat states (C=1) achieve sigma=0 for trivial reasons. This self-awareness is appropriate.

4. **Oregon case study is a good foundation.** The gcd(6,30,60)=6 example with its clean 1:5:10 ratio is the strongest argument in the paper for why this framework is worth developing.

---

## Major Concerns

### M1: The anti-gerrymandering claim is asserted, not established

The paper's central normative claim (Section 5.1, "Degrees-of-freedom reduction") is that NestSection reduces the feasible plan space available for partisan optimization. This claim is stated as a structural observation — "the partisan optimizer's effective search space is reduced by a factor related to the trunk-to-state ratio of geographic complexity" — but is never quantified or empirically tested.

This is exactly the kind of claim that the GerryChain/ensemble-methods literature has developed tools to evaluate. The paper itself cites ensemble diagnostics infrastructure (redist-analysis::ensemble_diagnostics). Why not use it? A comparison of the partisan outcome distribution under 100 NestSection seeds versus 100 independent GeoSection seeds for North Carolina would either support or undermine the central claim. Without this, the anti-gerrymandering argument is a conjecture.

The paper does flag this as future work ("Gerrymander resistance hypothesis" in §5.3), but it is listed as a paragraph in Future Work, not as a central result. For a paper making a statutory design proposal, an untested anti-gerrymandering mechanism is not sufficient basis for legislation.

**Required**: Either move the ensemble comparison to the current results, or reframe the paper as a "structural feasibility study" that establishes the mathematical foundation for a future empirical test, and reduce the statutory design proposal accordingly.

### M2: No connection to actual redistricting law

The paper's legal discussion (§5.1) cites a list of state supreme court cases (LWV v. Commonwealth, Harper v. Hall, Harkenrider v. Hochul) but never establishes what doctrine in any of these cases would support or enable a cross-chamber consistency mandate. These cases all addressed single-chamber maps — congressional or state legislative — under state constitutional provisions about compactness, contiguity, or partisan fairness. None of them addressed cross-chamber consistency.

The key legal question is: what constitutional or statutory hook would allow a court or legislature to require that congressional and state legislative maps share a bisection hierarchy? The paper does not identify one. The closest analog in existing law is the requirement in some state constitutions (e.g., Arizona, Indiana) that state house districts be subsets of state senate districts. But this is a two-chamber requirement (house-in-senate), not a three-chamber requirement including congressional districts.

Mandating that congressional maps comply with a state-level multi-chamber spine constraint almost certainly implicates the Supremacy Clause and the Elections Clause (Art. I, §4), since congressional redistricting is federal in scope. The paper gestures toward this with the Article I §2 mention in the plan file, but §5.1 never engages with it. This is a serious omission for a paper targeting a law review.

**Required**: Add a subsection addressing the constitutional hook for a spine-sharing mandate, distinguishing between: (a) voluntary adoption by a redistricting commission; (b) state-legislated requirement for state maps only; (c) a statutory requirement that would also bind the congressional map. Case (c) has significant constitutional complexity that must be addressed.

### M3: The "11 strictly compatible states" finding is almost entirely trivial

Of the 11 sigma=0 states, 7 have C=1 (AK, DE, ND, SD, VT, WV, WY). For these states, gcd(1, S, H) = 1 = min always, and sigma=0 is guaranteed by the formula regardless of S and H. The "strict nesting" of a single-seat congressional delegation consists of: the congressional district is the entire state, so all senate and house districts are nested inside it by definition. This imposes zero constraint on the state legislative maps.

The remaining 4 multi-seat strictly compatible states are MT (C=2), NH (C=2), AL (C=7), and OR (C=6). Among these, MT and NH also have small C. The paper acknowledges this limitation in §5.2, but the 11/50 headline finding is presented in the abstract and conclusion as a substantive discovery. A reader scanning the abstract gets the impression that 11 states have a meaningful compatibility property; a reader who reads to §5.2 learns that 7 of these are trivial.

**Required**: Either remove the single-seat states from the main compatibility table (reporting them separately as "trivially compatible") and present a headline finding of 4 non-trivially strictly compatible states, or prominently caveat the 11-state count in the abstract.

### M4: The Mode 2 definition is vestigial

The paper defines three nesting modes (Strict, Partial, Best-effort) based on sigma thresholds (0, 0-50, >=50). But Mode 2 (partial nesting) has zero empirical instances among current US states. The paper explains why (§4.2): in current apportionments, states either have g = min (strict) or g << min (incompatible), with nothing in between. Defining a mode for which no US state qualifies makes the taxonomy feel padded. Future apportionments might produce Mode 2 states, but this is speculative. Mode 2 should either be removed from the main taxonomy and moved to a "future states" discussion, or the paper needs to provide an example (possibly a hypothetical state or a historical apportionment) where Mode 2 occurs.

---

## Minor Issues

- The compatibility score formula sigma = (1 - g/min) * 100 is intuitive, but it is not the only reasonable measure. An alternative is sigma' = log(min) - log(g) (measuring log-scale factorization distance). The paper should at minimum acknowledge that the linear score is a design choice and note its properties relative to alternatives.

- The paper claims that NestSection's constraint is "symmetric and enforced at the algorithmic level, not at the political level" (§5.1). This is only true if the spine is fixed before the mapmakers optimize within trunk regions. If mapmakers control the GeoSection runs at the trunk level, they can still optimize the trunk boundaries to their advantage. The paper should explicitly state who fixes the trunk — a neutral algorithm running on a census graph with no partisan inputs — and include this in the legal argument.

- Section 5.3 mentions "Montana gained a seat in 2020 (going from C=1 to C=2)." Montana was actually reapportioned from 1 to 2 seats following the 2020 Census (first applied in 2022). This is correct; just verify the timing language is accurate.

- The proposed nestability threshold of g >= 5 (which qualifies only Oregon and Alabama) as the threshold for "mandatory strict nesting" is very narrow. The authors should consider g >= 2 as the minimum threshold for any statutory requirement, which would cover 22 states. A mandate covering 2 states is an unusual starting point for a federal or state statutory framework.

---

## Assessment

The algorithmic contribution is real and novel. The 50-state empirical census is a useful artifact that will be cited by future work. But the paper as submitted makes statutory design claims that are not supported by either empirical validation of the anti-gerrymandering claim or analysis of the legal framework that would enable the proposed mandate. For Harvard Journal on Legislation, the legal analysis needs to be substantially strengthened. The paper would benefit from repositioning as a computational feasibility study with a measured, speculative legal discussion, rather than a statutory design proposal with a conjecture as its empirical support.

**Score: 2/4 — Major revision required.**

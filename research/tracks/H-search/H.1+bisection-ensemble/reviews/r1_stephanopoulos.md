# Review 4: Nicholas Stephanopoulos (Election Law, Procedural Fairness)
**Paper**: H.1: BisectionEnsemble
**Round**: 1
**Recommendation**: Minor Revision

---

## Summary

Section 5 ("Legal Properties") is the paper's most directly relevant contribution for litigation deployment. My review focuses on whether the legal characterizations are accurate and defensible in a redistricting court proceeding. Overall the section is carefully drafted, but there are gaps in the procedural fairness argument and one claim that misstates the *Rucho* doctrine in a way that could be harmful if the paper is used as expert evidence.

## The "Pre-Committed Procedure" Argument: Mostly Sound, One Gap

The paper argues that BisectionEnsemble satisfies the "pre-committed procedure" standard because the hyperparameters $p$ and $T$ are fixed before plans are generated and no post-hoc adjustment is made based on partisan outcomes. This argument is correct and tracks how ensemble methods have been used successfully in redistricting litigation (e.g., *Common Cause v. Lewis* in North Carolina, *League of Women Voters v. Pennsylvania*).

The one gap: the paper treats the initial METIS seed as a fixed parameter in the audit log (Section 5.3, Disclosure requirements), but does not discuss how the METIS seed is selected. If the METIS seed is chosen by examining multiple seeds and selecting the one that produces the most favorable (or most compact) initial bisection, then the "pre-committed" argument breaks down: the seed selection step would itself require disclosure and would be subject to challenge as a hidden partisan lever.

The paper should either (a) specify that the METIS seed is drawn randomly from a hardware random number generator before any plans are computed, or (b) acknowledge that seed selection is part of the "committed procedure" and must be disclosed and defended alongside $p$ and $T$.

## The *Rucho* Citation: A Material Mischaracterization

Section 5.1 ("No partisan inputs") cites *Rucho v. Common Cause*, 588 U.S. 684 (2019), for the proposition that political blindness is required "for any map-drawing algorithm that is proposed as a procedurally neutral baseline." This misreads *Rucho*.

*Rucho* held that federal courts lack jurisdiction to adjudicate partisan gerrymandering claims under the Elections Clause and Equal Protection Clause — it did not articulate a "political blindness" standard for neutral algorithms. *Rucho* explicitly acknowledged that state courts retain jurisdiction over such claims (and indeed that state constitutions may impose neutrality requirements), but the Court did not itself define what a neutral algorithm must look like.

The appropriate citations for a "political blindness" requirement in redistricting algorithms are state court cases, not *Rucho*: *League of Women Voters v. Commonwealth* (Pa. 2018), *Common Cause v. Lewis* (N.C. 2019), and *Harper v. Hall* (N.C. 2022) all articulate standards for partisan neutrality in algorithmic map-drawing. If the paper is to be used in state court proceedings (where post-*Rucho* partisan gerrymandering claims are litigated), these are the operative authorities. Citing *Rucho* for a "political blindness requirement" inverts its holding and could be embarrassing if flagged by opposing counsel.

## Population Balance and *Wesberry*: Correct

Section 5.2 correctly characterizes *Wesberry v. Sanders* as requiring "nearly equal populations" and states that BisectionEnsemble enforces $\pm 0.5\%$ deviation. This is accurate. The paper also correctly notes that the tolerance condition is enforced at the leaf level, and that parent-level imbalances are bounded by $\varepsilon$ and absorbed by subtrees. This analysis is legally and mathematically sound.

## The Equal Protection / Percentile Choice Argument: Needs Strengthening

Section 5.3 ("Percentile Choice and Equal Protection") addresses the concern that different values of $p$ may produce different partisan outcomes, which an opponent might characterize as a hidden partisan thumb on the scale. The paper's response is two-pronged: (1) the NC results show the outcome is stable across $p$ values, and (2) a partisan actor exploiting $p$ would need the outcome to be sensitive to $p$, and stability of outcome is evidence against this.

This argument has a logical gap: the argument assumes that *if the outcome were sensitive to $p$, a partisan actor would exploit it*. It does not rule out that $p$ is insensitive to outcome in NC while being sensitive in other states. The paper has WI data showing a one-seat shift at $p=0.5$, which is direct evidence of $p$-sensitivity in at least one state. An opposing expert would lead with the WI result and argue that NC stability is not representative.

A stronger legal argument would be: "We commit to disclosing $p$, run sensitivity analysis over $p \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ for the challenged state, and show the distribution of outcomes across $p$ values. If no value of $p$ produces an outlier outcome relative to the distribution, the method is neutral regardless of which $p$ is selected." This forward-looking sensitivity disclosure is more legally robust than the retrospective NC stability claim.

## Disclosure Requirements: Well Drafted

Section 5.3's disclosure requirements (values of $p$, $T$, METIS seed, census release, acceptance rate per node) are comprehensive and appropriately detailed. The reference to the `redist-cli` JSONL audit log satisfies the reproducibility standard that courts have required in litigation involving algorithmic evidence.

One addition: the paper should specify whether the JSONL log includes the full plan at each accepted step (not just the final selected plan), since opposing experts may want to examine the distribution from which the percentile was selected. If the log only records the final plan, the ensemble distribution cannot be independently verified.

## Contiguity and the Article I §2 Reference

Section 5.2 states that contiguity is required "as implicit in Article I §2." This is a contested legal claim. Courts have found contiguity requirements in state redistricting statutes and state constitutions, but the federal constitutional basis is murky. *Article I §2* deals with apportionment and equal population, not contiguity. The correct federal authority is closer to *Evenwel v. Abbott* (2016) and the body of Voting Rights Act case law. A footnote acknowledging that contiguity is primarily a statutory requirement would be appropriate.

## Overall Assessment

Section 5 is the strongest legal properties section I have seen in a redistricting algorithms paper. The main issues are: (1) the *Rucho* citation needs correction; (2) the METIS seed selection protocol needs explicit treatment; (3) the percentile-sensitivity argument should be reframed as a prospective sensitivity analysis commitment rather than a retrospective NC stability claim; and (4) the audit log should specify whether ensemble distributions are preserved. These are targeted and achievable revisions.

# Review 4 (Round 2): Nicholas Stephanopoulos (Election Law, Procedural Fairness)
**Paper**: H.1: BisectionEnsemble
**Round**: 2
**Recommendation**: Accept with Minor Revisions

---

## Response to Round 1 Concerns

### Rucho Citation — Fully Addressed (Excellent)

The revised Section 5.1 completely replaces the Rucho-as-neutrality-standard framing with the correct post-Rucho framework. The passage now accurately states that Rucho "held federal courts lack jurisdiction over partisan gerrymandering claims" and directs readers to the operative state-court authorities: *League of Women Voters v. Commonwealth* (Pa. 2018), *Common Cause v. Lewis* (N.C. 2019), and *Harper v. Hall* (N.C. 2022). This is precisely the correction needed. A practitioner reading this paper will now have the correct jurisdictional framing. This was the most important fix in the paper and it has been done well.

### METIS Seed Protocol — Fully Addressed (Good)

The new "METIS seed protocol" paragraph in Section 5.3 specifies that the seed is drawn from the system CSPRNG before any plans are computed and logged before the first plan is generated. This directly addresses the "pre-committed" gap I identified. The explanation of why this matters ("If the METIS seed were chosen after inspecting results, the 'pre-committed' argument would break down") is appropriately explicit for a practitioner audience.

### Percentile Sensitivity — Fully Addressed (Excellent)

The prospective sensitivity-analysis protocol in Section 5.3 is well-drafted. The commitment to run $p \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ and disclose the seat-count distribution for any litigated state is exactly the right legal posture. The statement "a result that varies with $p$ must be disclosed as such" is appropriately strong. The explicit reference to the WI result as an example of required disclosure is a good concrete illustration.

### Ensemble Distribution Logging — Fully Addressed (Good)

The `--log-ensemble` flag and the updated Disclosure requirements list (now six items including the $p$-sweep distribution) adequately address my S-7 concern. The optional preservation of the full plan sequence for independent verification is the right design.

### Ergodicity — Adequately Addressed

The ergodicity claim is now correctly hedged ("to the extent the local chain mixes"). This is acceptable for a legal properties section, though I note that a practitioner using this in litigation should understand that the "median plan" interpretation relies on an unproven mixing assumption. A brief footnote acknowledging this to practitioner readers would be useful.

## Remaining Concerns

### Article I §2 and Contiguity — Not Addressed

I noted in Round 1 that "contiguity is required as implicit in Article I §2" is a contested legal claim. The revised paper retains this language unchanged: "This satisfies the contiguity requirement that courts have interpreted as implicit in Article I §2 and explicit in most state redistricting statutes." The Article I §2 basis for contiguity is indeed contested; the safer phrasing is "explicit in most state redistricting statutes and widely interpreted as a constitutional requirement in redistricting case law." This is a minor point but matters if the paper is used in litigation.

### GerryChain Version in Table 1 — Still Absent

The GerryChain comparison remains unconfigured. From a legal-evidence standpoint, an opposing expert can challenge the $<1\%$ acceptance rate simply by running a different GerryChain configuration. At minimum, a footnote specifying GerryChain version, $\varepsilon$, and ReCom variant would make Table 1 defensible. This was C-4 in the revision plan.

## Overall Assessment

The legal section is now in excellent shape. The Rucho correction eliminates the most serious error. The METIS seed protocol and sensitivity-analysis commitment are well-drafted and litigation-ready. The remaining issues (Article I §2 language, GerryChain config) are minor and can be addressed with targeted edits.

**Score**: 3.5/4

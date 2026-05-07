> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Round 1 Review — Percy Liang
**R1 Score: 3.0/4.0**

## Summary Assessment

This guide's primary purpose is to orient a non-technical audience, but it also makes specific reproducibility claims that a technically-minded reader will verify. My concerns center on those claims: the guide asserts that "any researcher with an internet connection can reproduce" the five headline numbers, and this standard deserves scrutiny. The document largely passes this test, but there are gaps in how reproducibility is characterized and one specific claim that does not survive verification.

## Reproducibility Claims: Traceability

The five headline numbers each carry a paper citation, which is good practice. The claims that these numbers are derived from "publicly available Census data using openly published code" and that they are reproducible by any researcher are the key statements I evaluate.

**The +22% claim.** As flagged by other reviewers, the compactness percentage cited (22%) does not match Paper B.2 (which reports 20% over enacted maps). This is a reproducibility failure: a researcher who reads this guide, checks B.2, and attempts to reproduce "22% improvement over enacted maps" will not find that number in the source. For a guide whose central claim is verifiability, this inconsistency undermines the document's own stated standard.

**The NC 7D/7R claim.** Paper B.11 is cited. A researcher who reads B.11 will find the NC result, but will also find the GA 5D/9R result that this guide does not mention. The reproducibility standard here is met — the number is real and traceable — but the guide's presentation of the finding is selective.

**The T=600 claim.** Paper B.16 is cited. The claim that T=600 gives a firm "stopping rule" is consistent with B.16's stated result ($P(\text{tail} > 600) < 0.001$). Reproducible and accurate.

**The 42% VRA threshold.** Paper D.1 is cited. The number is accurate and reproducible. No issue.

**The $O(n^{1.07})$ runtime.** Paper B.6 is cited. The empirical exponent $b = 1.07 \pm 0.03$ is consistent with B.6's reported result. Reproducible and accurate.

## Section 6: Dashboard Reproducibility

The three commands for reproducing figures:
```
redist fetch --year 2020
redist state --state NC --year 2020 --version v1
redist map   --state NC --year 2020 --version v1
```
are accurate representations of the CLI interface described in the repository's `docs/REDIST_CLI.md`. The output path (`outputs/v1/2020/north_carolina/`) is consistent with the CLI documentation. No issues here.

## "Any Researcher" Claim

The statement "Any researcher with an internet connection can reproduce them" (end of Section 3) is a strong claim. The guide should qualify this slightly: "any researcher with an internet connection and a modern laptop running Linux, macOS, or Windows" — the platform requirement is not trivial, and the `redist` binary has specific platform support that is documented in A.4 but not mentioned here. This is a minor qualification, but accuracy matters in reproducibility claims.

## Citation Structure

All five headline number citations link to specific papers within the portfolio. This is appropriate for an internal guide. External reproducers will need to find the portfolio repository; the guide correctly points to `github.com/giodl73-repo/REDIST` (in A.4) for this purpose. The guide itself should include one sentence pointing to the repository for the non-technical reader who wants to explore further.

## Recommendation

Accept with two required corrections: (1) change the compactness headline from 22% to 20% — this is a reproducibility failure, not just a precision issue — and (2) add a repository URL reference in Section 3 or the abstract for readers who want to verify the claims directly. The document is otherwise suitable for its purpose.

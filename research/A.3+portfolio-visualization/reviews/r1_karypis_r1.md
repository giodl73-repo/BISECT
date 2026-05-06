> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Round 1 Review — George Karypis
**R1 Score: 3.2/4.0**

## Summary Assessment

This visual guide fulfills its stated purpose: orient a non-technical audience to the portfolio and explain why algorithmic redistricting merits serious attention. For a synthesis/guide paper the bar is appropriately different from an algorithm paper, and by that standard the document is well-constructed. The TikZ diagram, table of tracks, and "What Goes In / What Comes Out" table are effective didactic tools. My concerns are primarily technical accuracy issues in how METIS and the bisection procedure are described.

## Algorithm Description: Accuracy

**METIS as "graph partitioner" — acceptable simplification.** The pizza analogy and the four-step description in Section 2 are broadly correct. The characterization of METIS as "the same tool used to distribute workloads across supercomputers" is accurate and an effective hook for a non-technical audience. No objection here.

**"Make the cut as short as possible" — edge-weighted bisection.** The description correctly identifies that the cut objective drives compactness. However, the paper does not distinguish between *unweighted* bisection (minimizing edge count) and *edge-weighted* bisection (minimizing sum of boundary lengths), which is the specific innovation of B.2. The current description implies the algorithm inherently produces compact districts, when in fact compactness depends on using boundary-length edge weights. A practitioner implementing the unweighted version would be misled. One sentence clarifying that edge weights encode boundary lengths would fix this without burdening a non-technical reader.

**Bisection round visualization.** The placeholder figures (Minnesota and Alabama) are an acceptable substitute for a guide paper, but the figure caption mentions "rounds 1–3" for Minnesota (8 districts), which would require only $\lceil\log_2 8\rceil = 3$ rounds — this is consistent. No issue here.

## Track Diagram: Completeness

The TikZ diagram includes all seven tracks (A through G) and the paper table lists the correct count (5, 18, 9, 6, 7, 6, 5 papers respectively). The arrow structure — Track B as the methodological source feeding all downstream tracks — is architecturally accurate. The arrow from A to B, but not from C–G back to A, is appropriate for a dependency diagram rather than a reading-order diagram.

**Minor concern**: The diagram labels Track G as "Ensemble" and places it below Track B, but Track G is conceptually adjacent to Track C (validation), not directly derived from Track B in the same way Tracks D–F are. The current placement is defensible, but a reader might infer that Track G is a direct B offspring in the same way as D. Not a critical issue for a guide paper.

## Headline Numbers: Accuracy Check

This is my primary concern. Section 3 reports "+22% compactness improvement" attributed to Paper B.2. However, Paper B.2's conclusion states: "20% improvement over enacted 2020 congressional districts (0.367 vs 0.305)." The 22% figure does not appear in B.2's text; the 56% improvement cited in B.2 refers to the improvement *over the unweighted baseline*, not over enacted maps. The synthesis/guide paper is inflating the headline number by 2 percentage points. For a practitioner audience that will fact-check citations, this discrepancy — small but attributable — is a credibility risk. The correct headline is **+20%**.

The remaining four numbers (7D/7R NC, T=600, 42% VRA threshold, $O(n^{1.07})$) are consistent with their source papers.

## Audience Routing

The "How to Use" section is effective. The routing for judges, legislators, researchers, and journalists is specific and actionable. The reference to `docs/quickstart/quickstart-callais-expert.md` for expert witnesses is appropriate.

## Recommendation

Accept with one required correction: change "+22%" to "+20%" in Section 3 (the compactness headline number), with the note that this is the improvement over enacted 2020 maps as reported in Paper B.2. No structural changes needed. This is a well-executed guide document.

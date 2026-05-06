# Review 4 — Nicholas Stephanopoulos
**Paper**: F.2: NestSection at Scale — Spine-Compatible Bicameral Redistricting for All 50 States
**Round**: R1
**Score**: 3/4

## Summary

F.2 addresses the constitutional nesting requirement in bicameral state legislatures, a topic with direct legal relevance. My review focuses on whether the constitutional characterisations are accurate, whether the California and Iowa examples are correctly described, and whether the litigation implications are appropriately stated.

## Strengths

The litigation implications section (Section 6.3) is the most legally useful part of the paper. The conclusion that NestSection can produce nested maps with ≤0.5% balance, +2.5% edge-cut penalty (compactness-equivalent), and ≤0.3 seat partisan shift directly addresses the feasibility question that courts and commissions would ask before requiring nesting. This is the right framing for a paper that intends to be useful in redistricting practice.

## Concerns

**C1 — California constitutional requirement.** The paper states (Introduction): "California's constitution, for example, requires that 'each Senate district shall consist of two complete, contiguous Assembly districts.'" This is a historical constitutional provision from before Proposition 11 (2008). California's current redistricting system, administered by the Citizens Redistricting Commission under the Fair Maps Act, does not require this relationship. The CRC draws house and senate maps independently, applying the same priority ordering of criteria to each. The two-senate-districts-per-assembly-district rule was part of the pre-commission constitutional structure and was eliminated as part of the reform. If this error is present here and carries through to F.4, it needs correction in both places.

**C2 — Iowa statutory vs. constitutional nesting.** The paper states that "Iowa's statute requires that senate districts shall consist of whole house districts wherever practicable." This is broadly accurate: Iowa Code § 42.4 requires senate districts to be formed from two house districts "wherever practicable." However, characterising this as a "statute" (in contrast to California's "constitution") is correct — Iowa's nesting requirement is statutory, not constitutional. The paper's discussion of constitutional alternatives for incompatible states (Section 6.3) would benefit from explicitly distinguishing constitutional from statutory requirements, since the barriers to change differ.

**C3 — "7 incompatible" vs. "9 incompatible" count.** Section 6.3 states: "For the 7 incompatible states (Missouri, Oklahoma, Texas, Hawaii, Pennsylvania, Connecticut, Rhode Island, Maine, Delaware), the nesting requirement can only be met by changing at least one chamber's district count." This list contains 9 states, not 7. This is the same error as noted in the other reviews: the paper lists 9 states with gcd=1 but claims "7 incompatible states." The incorrect count has a legal implication: if used in litigation, the incorrect number would be identified and could undermine the paper's credibility.

**C4 — Constitutional amendment characterisation for incompatible states.** Section 6.3 suggests that Texas could achieve compatibility by changing the Senate from 31 to 30 seats. Texas has 31 senators set by Article III, Section 3 of the Texas Constitution. Changing this would require a constitutional amendment, not merely a statute. The paper presents this as a "small adjustment" without acknowledging the constitutional barrier. For states where chamber size is constitutionally fixed (as in most states), the suggestion to adjust k for compatibility is not a practical option without constitutional amendment.

## Recommendation

Accept with revisions. C1 (California constitutional error) and C3 (incompatible state count) must be corrected. C4 (constitutional amendment barrier) should be acknowledged in the text.

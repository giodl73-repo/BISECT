# Review 2 — Jonathan Rodden
**Paper**: F.2: NestSection at Scale — Spine-Compatible Bicameral Redistricting for All 50 States
**Round**: R1
**Score**: 3/4

## Summary

F.2 is the most politically relevant paper in the F track after F.1. The partisan implications of nesting constraints — specifically the finding that nesting changes partisan outcomes by at most 0.3 seats across compatible states — is a significant and practically important result. My review focuses on the partisan analysis and on the connection to state constitutional requirements for nesting.

## Strengths

The partisan analysis (Section 6) is well-designed. The comparison of independent versus NestSection maps for 30 states with precinct data (Kuriwaki 2023) is the appropriate counterfactual. The finding that nesting changes outcomes in at most 1 state (Wisconsin: 43D → 42D) out of 30 with available data is striking and credible: the spine construction partitions the state into politically coherent super-regions, so within-super-region redistricting has little scope to change the super-region's partisan composition.

The constitutional context discussion (Section 6.3) is useful: the note that California, Iowa, and others have state-law requirements for senate-house nesting, and that NestSection achieves nesting with only a small compactness penalty and negligible partisan effect, directly addresses the practical legal question of whether nesting is feasible at scale.

## Concerns

**C1 — Wisconsin House: 43D vs. political expectations.** The partisan table (Table 3) shows Wisconsin House as 43D/56R under independent maps, 42D/57R under NestSection. Wisconsin has been a prominent gerrymandering state: the enacted 2011 map and subsequent litigation (Gill v. Whitford) drew national attention. The paper should note that both the independent and NestSection algorithmic maps produce minority-Democratic outcomes (43D out of 99 seats despite roughly 50/50 partisan statewide), consistent with Chen and Stephanopoulos's findings on geographic sorting. Whether this is the minimum achievable Democratic seat share under fair redistricting in Wisconsin — or whether the algorithmic result overstates the geographic constraint — is a question the paper should address.

**C2 — Partisan analysis limited to 30 states.** The precinct data comparison covers only 30 of 42 compatible states. The 12 compatible states without precinct data are not identified. Given that the partisan neutrality claim ("≤0.3 expected seat shift") is based on 30 states, it would be strengthened by identifying which 12 states are excluded and why (precinct data unavailability, small state size, etc.).

**C3 — The "7 incompatible" framing underestimates the problem.** Texas (H=150, S=31, gcd=1) is identified as incompatible. The paper notes that "small adjustments (e.g., Texas: H=150, S=31 → 30, gcd(150,30)=30) would achieve compatibility." This is correct arithmetically but politically naive: changing a chamber's district count requires a constitutional amendment in most states. The paper presents this as a practical suggestion without acknowledging the political barriers. A more precise framing: "these states cannot use NestSection without a constitutional amendment" (for states where chamber size is constitutional) or "without a statutory change" (for states where chamber size is statutory).

**C4 — "Nesting rarely changes partisan outcomes" causally overstated.** Section 6.2 explains why nesting rarely changes outcomes: "the partisan composition of each super-region is fixed; the choice between different within-super-region maps does not change which party controls the super-region's delegation." This is approximately correct but understates the within-super-region competitive races. In states with many evenly divided super-regions, the within-super-region optimisation could swing competitive districts. The paper should report how many super-regions are competitive (margin < 5%) to bound the scope of this effect.

## Recommendation

Accept with minor revisions. The partisan analysis is the strongest part of the paper. C1 (Wisconsin interpretation in gerrymandering context) and C3 (constitutional barriers to compatibility) should be addressed.

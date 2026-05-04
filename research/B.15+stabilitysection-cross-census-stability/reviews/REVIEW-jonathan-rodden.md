# Review: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Jonathan Rodden (Stanford University, Hoover Institution)
**Expertise**: Political geography, geographic sorting of partisan voters, urban-rural divide, unintentional gerrymandering, comparative electoral systems
**Date**: 2026-05-02

## Overall Assessment

This paper asks a genuinely important empirical question: does the geographic structure that GeoSection detects remain stable across the three decennial censuses covering 2000--2020? The 67% stability result for the 2000--2010 comparison is a meaningful finding. It aligns with what the political geography literature would predict: the deep urban-rural sorting that underlies most of the proportionality gaps documented in the paper has been building since the 1970s and is not a product of any single census cycle. A 67% stability rate tells us that for most states, the geography has been durable enough to produce the same algorithmic answer across a decade of demographic change.

My concerns come in two clusters. First, the paper's treatment of the *unstable* cases is diagnostically thin. Iowa shifting from $f = 0.18$ to $f = 0.49$ across a decade is the paper's most striking result, and it goes almost entirely unexplained. Why did rural decline and urban growth in Iowa produce such a dramatic shift? What geographic mechanism caused the algorithm to flip from a heavily asymmetric peel to a near-equal split? This is the paper's most scientifically interesting finding, and the current draft treats it as a data point rather than a case study. Second, the legal theory section is compelling in outline but needs to confront a serious objection: if 33% of states are *not* census-stable, what does that say for the claim that GeoSection maps are free of partisan contingency?

## Score: 3/4

**My score**: 3/4 --- Strong core finding; needs deeper engagement with instability cases and with the political geography literature on urban-rural sorting as a cause of both stability and instability.

## Major Strengths

1. **The right question for the political geography literature**: The paper correctly identifies that cross-census stability is the key empirical test of geographic determinism. My work on unintentional gerrymandering documents that compact, geography-only algorithms systematically produce Republican-advantaged maps due to the spatial concentration of Democratic voters in urban cores. This paper asks whether that structural bias is consistent across census cycles, which is the right follow-up question. A census-stable 5D/9R outcome in Wisconsin is a different claim from a single-year 5D/9R outcome.

2. **The metropolitan-fixity hypothesis is novel and correct**: The claim that cities with fixed geographic boundaries (Milwaukee, Chicago, Philadelphia) will produce stable ratios while cities with expanding boundaries (Phoenix, Las Vegas, Denver) will not is the right theoretical framework for predicting which states will be stable. The Milwaukee example --- Lake Michigan prevents eastern expansion, suburban incorporation limits prevent annexation --- is exactly the kind of mechanistic geographic argument that is missing from most redistricting papers. This is an original contribution.

3. **Iowa as the central puzzle**: The paper correctly identifies Iowa as the theoretically most interesting case. Iowa is a state I would have predicted to be highly stable: near-uniform agricultural density, no dramatic urban concentration. The fact that GeoSection shifts from $f = 0.18$ to $f = 0.49$ between 2000 and 2010 suggests that rural depopulation and the growth of Des Moines, Cedar Rapids, and Iowa City created a new population-geography equilibrium that the algorithm detects. This is a genuine scientific finding.

4. **The 2D stability picture (seed, census)**: The synthesis with B.7 is compelling. The prediction that the bottom-right cell (low seed stability, high census stability) should be empty is theoretically grounded and testable. States whose geographic structure is durable should be both easy to detect and consistently detected.

## Major Issues (Must Address)

### Issue 1: Iowa Deserves a Full Case Study
**Severity**: High
**Description**: The paper reports Iowa's $f$ shifting from 0.18 to 0.49 and attributes this to "dramatic population redistribution from rural decline and urban growth." This is the most striking finding in the paper and receives three sentences. A shift this large means GeoSection selected a heavily asymmetric peel in 2000 (peeling a small urban core from a large rural mass) and selected a near-equal geographic split in 2010. That is not a small perturbation; it is a qualitative change in the algorithm's reading of the state's geographic structure.

What happened to Iowa's population-geography Lorenz curve between 2000 and 2010? Did the Des Moines, Cedar Rapids, and Iowa City metro areas grow enough to flip the normalised edge-cut comparison at the 2:2 threshold? Or did rural depopulation reduce the population in half the state enough to make a 2:2 split iso-perimetrically optimal? These two mechanisms have different implications. The first (urban growth shifting $p^*$) implies Lorenz drift upward; the second (rural contraction reducing the edge-cut premium on the asymmetric peel) implies the isoperimetric gap between 1:3 and 2:2 narrowed to zero.

The paper has the data to answer this question. The Lorenz drift proxy section should include Iowa as a primary case study, with $p^*_{2000}$ and $p^*_{2010}$ computed explicitly.

**Recommendation**: Add a two-page Iowa case study to Section 4.3. Compute $p^*$ for both census years, show the normalised edge-cut curves, and identify the mechanism of the flip. This would transform the most puzzling result into the paper's most scientifically valuable contribution.

### Issue 2: The 33% Instability Rate Is Not Confronted Directly
**Severity**: High
**Description**: The paper's legal argument rests on the claim that census-stable GeoSection maps carry a "strongest possible geographic determinism claim." But 10 of 30 comparable states (33%) are *not* stable across the 2000--2010 decade at the 5% threshold. The paper's legal section argues as if the 67% finding is unambiguously good news, but a court could equally ask: for one in three states, the algorithm produces a different answer depending on which census is used. In those states, is the map really the "natural geographic partition"?

The political geography literature I have contributed to suggests that instability itself is informative. A state whose natural partition shifts between census cycles is a state where the urban-rural sorting process is still in flux. These are states where geographic determinism is weakest, and where the legal argument for a GeoSection map is correspondingly weaker.

The paper needs to confront this directly: the 67% finding means that for 33% of states, a two-census comparison would show different optimal partitions. What is the appropriate legal conclusion for these states?

**Recommendation**: Add a discussion in Section 5.1 distinguishing the legal argument for high-CSS states (strong geographic determinism claim) from the appropriate framing for low-CSS states (the algorithm detects genuine geographic reorganisation, which is itself informative but not a determinism claim). This would strengthen the paper by making its scope precise.

### Issue 3: No Partisan Data for 2000 and 2010 Means Seat Stability Cannot Be Computed
**Severity**: High
**Description**: The CSS formula assigns 50% weight to $s_{\text{seat}}$ (partisan seat stability), which requires knowing the partisan outcome in 2000 and 2010 as well as 2020. The paper acknowledges that election CSVs are not available for 2000 and 2010, but the CSS formula treats $s_{\text{seat}}$ as if it will be computable when those sweeps complete. For states where no precinct-level election data is available for 2000 and 2010, the 50% seat-stability component cannot be computed at all.

This is not a minor gap. Seat stability is the primary legal question: does the algorithm produce the same D/R outcome regardless of which census year's data is used? A CSS built entirely on ratio stability ($s_{\text{ratio}}$) and gap similarity ($s_{\text{gap}}$) would answer the geometric question but not the political one.

**Recommendation**: The paper should state explicitly that the current empirical findings cover only $s_{\text{ratio}}$ (and implicitly $s_{\text{gap}}$ for states with 2020 partisan data), that $s_{\text{seat}}$ requires 2000 and 2010 precinct-level data that is not yet available, and that the 67% stability result reported in Section 4.3 is a ratio-stability finding, not a full-CSS finding. The CSS formula as presented in the abstract and introduction overstates what the current data can support.

## Minor Issues

- **The 5% threshold is not justified**: The paper uses $|f_{2000} - f_{2010}| \leq 0.05$ as the stability threshold with no empirical or theoretical justification. Why 5%? What would change at 3% or 10%? For Iowa ($\Delta f = 0.31$), the threshold choice is irrelevant, but for states near the boundary, the classification is threshold-sensitive. A robustness check showing how many states change classification at 3%, 5%, and 10% thresholds would strengthen the result.

- **Alabama's instability deserves political geography analysis**: The paper reports Alabama shifting from $f = 0.46$ to $f = 0.40$ and attributes this to "Black Belt population share shifted." What specifically shifted? Did the Black Belt depopulate further? Did Birmingham's suburban growth change the weighting? For a state with active VRA litigation (*Allen v. Milligan*), the political geography of Alabama's instability is not a minor point.

- **The Lorenz drift proxy table (Table 4) mixes predictions with different data**: Several entries are flagged as "Medium (Milwaukee fixed)" or "Low (Phoenix growth)" based on the paper's metropolitan-fixity hypothesis rather than actual Lorenz drift measurements. The table should clearly distinguish entries where $\Delta p^*$ is computed from data from entries where stability is predicted from the fixity hypothesis.

- **Rodden 2019 connection should be made more specific**: My book documents the geographic sorting process state-by-state, and several of the high-CSS predicted states (Iowa, Kansas, Wisconsin) have case study chapters. The paper's predictions are consistent with those findings but does not engage with the state-specific geography in any depth.

## Questions for Authors

1. For Iowa, what is $p^*_{2000}$ versus $p^*_{2010}$? Is the flip explained by urban growth (rising $p^*$) or rural contraction (narrowing isoperimetric gap)?

2. Of the 10 unstable states in the 2000--2010 comparison, how many also show low seed stability in B.7? Is the instability correlated across dimensions as predicted by the 2D stability matrix?

3. For the Lorenz drift proxy: in the states where the 2000 sweep is complete, does the predicted stability class from the proxy match the observed ratio stability?

4. What is the paper's response to the argument that 67% stability is actually problematic for the legal claim --- that one in three states having an unstable map suggests geographic structure is not the only driver?

## Recommendation

This is an important paper that makes a meaningful empirical contribution to the redistricting literature and provides a new theoretical tool (CSS) that connects algorithmic stability to geographic determinism. The Iowa instability case study, the treatment of the 33% unstable states, and the seat-stability data availability problem need to be addressed before this reaches journal submission quality. Revise and resubmit.

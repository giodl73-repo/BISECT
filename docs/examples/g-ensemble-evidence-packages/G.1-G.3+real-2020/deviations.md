# Real Ensemble Experiment Deviations

## Deviation 01 - Wisconsin Baseline Ineligible

**Observed before production chains:** The Wisconsin benchmark assignment is
not contiguous under the same tract adjacency graph used by the samplers.
District 1 contains 177 tracts, but only 119 are reachable within its largest
connected component.

The existing `label-analyze --types contiguity` artifact reported only
`status: ok` and did not surface this district-level failure. ReCom requires
contiguous initial districts; the Rust pilot correctly failed when a merged
district-pair subgraph was disconnected.

**Disposition:**

- Wisconsin remains archived as a failed eligibility/negative-result case.
- No Wisconsin ensemble percentile will be computed.
- Iowa is added as a replacement medium-size state with four districts.
- The production state set is RI, IA, and NC, with WI reported separately.
- This replacement is based on a constitutional input invariant, not observed
  compactness or partisan outcomes.

## Deviation 02 - 2016 Election Coverage

The locally available 2016 tract estimates are keyed substantially to pre-2020
tract geography. Unmatched 2020 benchmark GEOIDs were:

- RI: 12 of 250;
- WI: 274 of 1,542;
- IA: 129 of 896;
- NC: 942 of 2,672.

No committed 2010-to-2020 election crosswalk was available. The 2016 seat
metric is therefore retained as an incomplete diagnostic but excluded from
primary partisan percentiles. The primary partisan metric uses the 2020
presidential estimates, which matched all RI and NC units and will be checked
for Iowa.

This is the documented election-selection rationale permitted by the pulse:
one nationally consistent election with complete current-tract coverage is
preferable to treating unmatched historical tracts as zero votes.

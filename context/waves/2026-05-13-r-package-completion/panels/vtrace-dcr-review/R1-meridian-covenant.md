# R1 MERIDIAN / COVENANT Review - VTRACE DCR Filing

### F-01 - WARN: Full-scale reproducibility needs an independent replay comparison rule
File: `docs\vtrace\DCRS.md`
Finding: DCR-007 records scope, environment, build features, source hashes, config hash, commands, seed/search metadata, and artifact generation, but does not explicitly require an independent clean replay or expected-hash comparison.
Consequence: DCR-007 could close as a well-documented production run rather than a reproducibility demonstration.
Fix: Require DCR-007 closure to compare a replay from a clean checkout/environment against expected hashes or to record exact divergences and dispositions.

### F-02 - WARN: Release-subset reproducibility scope needs claim-label discipline
File: `docs\vtrace\DCRS.md`
Finding: DCR-007 allows "all states/years or a specific release subset" and says public claims must cite the declared scope, but it does not require a visible claim label such as full-scale, release-subset, or smoke-only.
Consequence: A subset replay could be summarized as general release reproducibility in downstream docs.
Fix: Require DCR-007 evidence to label the reproducibility class and require public/docs references to use that same label.

## Role Summary

MERIDIAN/COVENANT accept DCR-007 as the right reproducibility gate. It needs clean replay comparison and scope-label discipline before closure.

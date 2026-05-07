---
reviewer: Percy Liang
round: 3
score: 4
date: 2026-05-05
---

## Summary

Round 3 directly addresses my primary P1 concern: the software availability statement. The new Data Availability paragraph provides a repository URL, specifies that redistricting outputs and adjacency graphs are available at the repository, and states that adjacency files for all 50 states are available as GitHub Release assets with SHA-256 hashes. This is a substantial improvement that moves the reproducibility claim from aspirational to actionable. I am upgrading my score from 3 to 4. One carry-forward P1 item (estimation model source for † cells) remains.

## R3 Change: Data Availability Statement

**P1.3 — Software availability statement: SUBSTANTIALLY RESOLVED.**

The new text provides:
- Repository URL: `https://github.com/apportionment-research/redist`
- Statement that redistricting outputs, adjacency graphs, and analysis scripts are at the repository
- Specific statement that "pre-computed adjacency files for all 50 states are available as GitHub Release assets, with SHA-256 hashes recorded in the plan manifests"

This is a structural improvement. The connection between "SHA-256-verified adjacency graphs" (which the paper's reproducibility claim depends on) and the public GitHub Release assets (where an independent team can obtain those graphs) is now explicit. An independent team can:
1. Download the adjacency files from the GitHub Release
2. Compute SHA-256 hashes of the downloaded files
3. Compare to the hashes in the plan manifests
4. Reproduce the partitioning using the stated parameters and the `redist` binary

This is a complete reproducibility chain. The remaining gap is the absence of a commit hash or release tag for the `redist` binary itself. The adjacency graph provenance is now documentable; the software version provenance is not. I flag this as P2: "Commit hash: [hash] at release [tag]" should appear in the Data Availability statement or as a software citation footnote.

**P1.1 — Dataset release: SUBSTANTIALLY ADDRESSED.**
The adjacency graphs and plan manifests are now stated to be publicly available. This resolves the primary reproducibility concern. The bakeoff-state TIGER/Line SHA-256 hashes and census_release_id strings I specifically requested in R1 are now covered by the broader statement that "SHA-256 hashes [are] recorded in the plan manifests" — assuming the plan manifests are part of the public release (which the text implies). This is sufficient for the current B-series context.

## Remaining P1 Item

**P1.2 — Estimation model source: NOT ADDRESSED.**
The estimated (†) cells still do not specify whether each estimate uses the B.8–B.9 theoretical relationship, interpolation from adjacent configurations, or a model-derived prediction. The ±1 seat / ±3pp uncertainty is quantified but unsourced. An independent team cannot reproduce the estimated values without knowing the estimation procedure. I carry this forward as a journal-submission condition. For the current B-series internal track, I accept it as a P2 item given that the confirmed results (no superscript) are now reproducible from the stated parameters and public adjacency files.

## Rodden-null Change

The denominator explanation is a positive addition. From a reproducibility standpoint, the key improvement is that the 34-state denominator is now explained through a two-step exclusion procedure that can be independently verified: identify all states with $k=1$ (7 states, enumerated), then identify states where all modes produce identical seat counts (9 states, not enumerated — see Duchin's request for enumeration). Both steps are in principle verifiable from the B-series experimental data.

## Score: 4 / 4 — Accept (with P1.2 and software version pin as journal conditions)

The Data Availability improvement resolves my primary R1/R2 concern. The reproducibility chain from stated parameters + public adjacency files + `redist` binary is now documentable. P1.2 (estimation model source) and the software version pin are journal-submission conditions that do not undermine the paper's current value as a B-series synthesis reference.

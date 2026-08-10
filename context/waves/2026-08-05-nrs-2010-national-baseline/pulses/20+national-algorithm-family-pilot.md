---
title: National algorithm-family pilot
status: complete
date: 2026-08-09
---

# Pulse 20 - National Algorithm-Family Pilot

## Frozen gate

Protocol `neutral-algorithm-family-national-bakeoff-v1` froze the 44-State
multi-district cohort, four structures, controls, failure policy, aggregation,
and claim boundary before governed multi-State execution. Its eight-State
pilot is RI, NE, CT, KY, SC, WI, AZ, and CA in that order: 32 cells spanning
two through 52 congressional districts. The pilot is an engineering gate, not
a probability sample or national ranking.

## Initial attempt and diagnosis

The retained initial attempt completed 27 of 32 cells and passed six of eight
States. Arizona standard-bisect and ratio-optimal produced disconnected final
districts. California standard-bisect reached a later contiguity-constrained
METIS call with an already disconnected recursive child, and
ratio-optimal-area produced disconnected final districts. California
ratio-optimal was interrupted during scoped process cleanup and is not counted
as an algorithm result.

Both source adjacency graphs are connected. Inspection showed that equal
two-way non-NRS splits still used recursive METIS without `Contig`/`MinConn`,
whereas only asymmetric splits used the contiguity-capable k-way path. An
early equal split could therefore fragment a recursive region, surfacing as a
disconnected final district or as invalid input to a later constrained split.

## Remediation and bounded validation

All non-NRS recursive splits now use k-way METIS with `Contig` and `MinConn`;
the frozen NRS v0.1 reference path is unchanged. A new equal-split connectivity
regression passed, the full `bisect-runner` library suite passed 278 tests, and
the release binary rebuilt. Focused seed-0 probes then passed for both failed
Arizona structures and all three relevant California structures, including
the previously interrupted ratio-optimal cell.

The pre-remediation package is retained at
`docs/experiments/neutral-algorithm-family-bakeoff-pilot-2020-pre-remediation/`.
The clean post-remediation package is being generated at
`docs/experiments/neutral-algorithm-family-bakeoff-pilot-2020/`.

## Result and decision

The clean governed matrix passed eight of eight States and 32 of 32 cells. All
native audits and population checks passed, the manifest flags agreed with the
audit certificates, and each State preserved its frozen common inputs. Across
the 48 within-State structure pairs, maximum-overlap assignment agreement
ranged from 49.0634 percent to 100 percent. Alternative edge-cut ratios to
standard-bisect ranged from 0.856479 to 5.441826; these descriptive within-State
values are not pooled performance estimates or a ranking.

Independent exact regeneration reran all 32 cells in a temporary package and
passed byte-identical normalized comparison. The pilot gate therefore passes
and authorizes the already frozen full phase. That phase is not started here:
its 176 cells require a separately accepted compute budget. The pilot remains
insufficient for national coverage, comparative ranking, or legal conclusions.

---
title: "Community Character Weighting: Framework and Legal Grounding"
series: M.0
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm
Defines w(u,v) = w_base × similarity(char_u, char_v). Similarity is cosine similarity of tract-level character trait vectors. The framework is modular — any tract-level signal can be a dimension. M.1–M.7 each contribute one or more dimensions. No specific data source; theoretical paper synthesizing the full M-track.

## Claims
1. Communities of interest can be operationalized as cosine similarity of tract-level character vectors without partisan data.
2. The framework is legally neutral under Shaw v. Reno — no racial, partisan, or incumbency data is required or consumed.
3. Administrative zone co-membership (M.6) is the strongest single legal proxy for community of interest, supported by property-tax fiscal bond doctrine.

## Data Sources
No external download. This paper synthesizes the data sources and methods defined in M.1–M.7. All empirical validation is delegated to the individual M-track papers.

## Layer
Framework paper defining the edge-weight modifier interface for the three-layer compositor. Each M-track paper implements a plug-in similarity function. The compositor blends w_base (geographic/boundary) with w_similarity via alpha blending: w(u,v) = alpha × w_boundary + (1-alpha) × w_char, alpha ∈ [0,1] tunable per plan config.

## Test Invariants (L0)
- similarity_symmetric: sim(u,v) == sim(v,u) for all tract pairs
- similarity_self_is_one: sim(u,u) == 1.0 for any tract u
- weight_stays_nonneg: w(u,v) >= 0.0 for all edges after blending

## Empirical Targets
No empirical pipeline run required for M.0. Targets are defined in M.1–M.7 and validated there. M.0 provides the legal grounding section and shared mathematical definitions that all other M papers cite.

## Legal References
- Shaw v. Reno, 509 U.S. 630 (1993) — racial predominance prohibition
- Thornburg v. Gingles, 478 U.S. 30 (1986) — VRA Section 2 compactness
- CA Prop 11 (2008) — communities of interest as explicit redistricting criterion
- CO Art. V §44 — independent commission communities of interest language
- AZ IRC criteria — communities of interest as one of six criteria
- Property-tax fiscal bond doctrine — administrative co-membership as legitimate community basis

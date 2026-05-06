---
reviewer: George Karypis
spec: redist-ensemble ReCom crate
round: 1
score: 3
date: 2026-05-06
---

## Summary
The spec proposes a Rust implementation of the ReCom MCMC sampler using Wilson's loop-erased random walk algorithm for uniform spanning tree generation. The algorithmic description is mostly correct but contains two technical inaccuracies that would cause correctness or performance problems in production.

## Strengths
- The use of Wilson's algorithm is the right choice: it produces a *uniformly* random spanning tree, which is the theoretical requirement for ReCom's correct measure-theoretic properties. Aldous-Broder also produces USTs but Wilson's is faster in practice for sparse graphs.
- The CSR graph reuse from `redist-metis` is sound engineering — adjacency structure is already built and validated, and reusing it avoids a redundant representation. The induced subgraph construction for the merge region is the correct framing.
- The performance table is structured in a way that makes the 2,500x claim falsifiable: specific states, tract counts, and step counts are given. This is the right kind of target specification.

## P1 — Required changes

- **O(|E|) complexity claim is wrong for Wilson's algorithm.** Wilson's algorithm runs in O(cover time) expected time, which for a general graph on n vertices can be O(n^3) in the worst case. For planar graphs (which census tract adjacency graphs are, approximately), the cover time is O(n log n) expected, making the practical complexity O(|V| log |V|) not O(|E|). The O(|E|) claim in the pseudocode comment must be corrected to "O(cover time), O(|V| log |V|) expected for planar graphs." Stating the wrong complexity in a published codebase creates legal and scientific credibility risk.

- **The balance check retry logic (50 samples from the same tree) is statistically biased.** The current implementation samples 50 random edges from the *same* spanning tree. For a balanced tree cut to exist, it must exist in the original graph — but the tree is fixed. If no balanced cut exists in the sampled tree, sampling more edges from the same tree cannot help. The retry loop should either (a) resample the spanning tree entirely on failure, or (b) enumerate *all* tree edges and find any balanced cut before falling back to rejection. The current 50-retry-from-same-tree approach will fail systematically for large k (e.g., TX k=38) where balanced cuts are rare, and will produce a subtly biased sampler because not all edges are tried with equal probability across repeated calls.

- **The `find_adjacent_pairs` call is O(|E|) per step and dominates at large k.** For TX (k=38, 5,265 tracts) with a dense adjacency graph, scanning all tract edges to find adjacent district pairs on every step is expensive. The spec should specify an incremental update: maintain a set of cut edges and adjacent-district pairs that is updated only for tracts whose district assignment changes at each step. Without this, the claimed 0.5s for 10K TX steps is implausible — a full scan of ~20K+ edges per step at 10K steps is 200M+ operations.

## P2 — Suggested improvements

- Specify the subgraph-induced adjacency construction more precisely. The `SubGraph::induced` call must preserve only edges where both endpoints are in the region — this is correct for contiguity but the spec does not state that the subgraph must be verified as connected before running Wilson's algorithm. Add a connectivity check (BFS/DFS, O(|V|)) before the spanning tree call; if the region is disconnected (which should be impossible given the district-adjacency precondition but can occur due to implementation bugs), Wilson's algorithm will silently loop forever.

- The performance targets should distinguish between the spanning tree cost (which scales with region size, typically 2k tracts for NC) and the full-chain cost (which also includes the `find_adjacent_pairs` scan). Separating these two costs in the performance table would make the targets more auditable.

## Score: 3/4
The algorithm is correctly motivated and the overall structure is sound, but the O(|E|) complexity claim is wrong and the retry logic is subtly biased in a way that would produce incorrect results at scale. These are fixable in one revision pass.

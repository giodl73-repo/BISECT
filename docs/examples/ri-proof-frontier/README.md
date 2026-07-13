# Rhode Island Proof Frontier

## Proven

The improved connected incumbent has district populations 548,689 and 548,690.
RoundingSat proves that no connected canonical split has scaled population
deviation below 1. VeriPB independently accepts the proof.

Therefore Rhode Island's first certified objective stage—population—is closed.
Because total population is odd, scaled deviation zero is arithmetically
impossible; the proof formally certifies that parity lower bound for the exact
instance.

## Not Proven

The prior boundary decision reached a time limit. Its incomplete 7.1 GB proof
log was deleted and is not treated as evidence.

A second 300-second search without proof logging also reached `TIMELIMIT`.
Those attempts used the prior cut 102,659,356. Zero-population cleanup has
since reduced the cut to 102,622,860, and three equal-population swaps further
reduced it to 102,193,710. The new boundary model is compiled but has not yet
been resolved.

The boundary model was split into exact right-population branches 548,689 and
548,690. Both branch searches reached 120-second time limits on the prior
102,193,710 incumbent.

Twenty-five population-preserving 1-to-2 exchanges have since reduced the cut
to 97,994,953. A deterministic 32-seed METIS ensemble then found seed 4, whose
full refinement reduced the cut to 64,132,468. All prior branch timeouts apply
to superseded incumbents. A fixed-core HiGHS search over the top-two METIS
disagreement band then found a connected, Rust-validated 49,081,395 incumbent.
A seed-4/28 pairwise band subsequently reduced the current connected incumbent
to 43,806,724. Nested one-, two-, and four-hop shell expansions then reduced the
current incumbent to 43,047,238. The new exact-population branches are compiled
and unsearched; no unrestricted boundary proof is claimed.

The elite-two fixed-core branch has a separate RoundingSat proof accepted by
VeriPB 3.0.2. It proves 49,081,395 is optimal only while 24,217 consensus-core
blocks remain fixed. The compressed local proof is 5.62 GB. Because assignments
outside those heuristic cores are not covered, this is not Rhode Island
boundary optimality.

## Custody

The 616 KB population proof is committed. Its 163 MB OPB input remains under
ignored local data custody; the exact input hash, compiler identity, discovery
identity, and replay commands are recorded in `provenance.json`.

## Claim Boundary

This package proves population optimality only. Rhode Island does not yet have
a complete root-cut certificate.

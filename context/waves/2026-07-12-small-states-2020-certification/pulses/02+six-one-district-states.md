---
pulse: 02
title: Six one-district States
status: done
depends_on: 01
wave: small-states-2020-certification
validation_level: L2 real block data
---

# Pulse 02 - Six One-District States

Build connected 2020 block contexts and complete one-district certificates for
Alaska, Delaware, North Dakota, South Dakota, Vermont, and Wyoming.

## Deliverables

- [x] six block-level RCTX packages;
- [x] deterministic island bridging where required;
- [x] six all-unit district-0 certificates;
- [x] Rust package verification;
- [x] independent Python verification;
- [x] aggregate coverage report; and
- [x] hash-bound local custody.

## Result

All six packages verify with complete unit coverage, one connected component,
and weighted inter-district boundary cut zero. Alaska requires 1,778
deterministic bridges; the other five State graphs are connected without
bridges.

## Claim Boundary

These are complete one-district operational certificates. They do not validate
the accuracy of Census source files beyond recorded hashes.

---
pulse: 01
title: One-district certificate contract
status: done
wave: small-states-2020-certification
validation_level: L1 synthetic and package verification
---

# Pulse 01 - One-District Certificate Contract

Define the wall-to-wall certificate for States with one congressional district.

## Deliverables

- [x] one-district package schema;
- [x] deterministic assignment of every canonical unit to district 0;
- [x] population total and unit-universe binding;
- [x] connectivity verification under the RCTX graph;
- [x] manifest and replay commands;
- [x] positive synthetic fixture; and
- [x] hostile omitted, nonzero-label, and disconnected fixtures.

## Result

The Rust builder/verifier and independent Python verifier accept the connected
grid fixture and reject all hostile cases. Boundary and canonical objectives
are structurally trivial at `k=1`.

## Claim Boundary

A one-district package proves complete assignment and connectivity. Boundary and
canonical optimization are trivial because no inter-district cut exists.

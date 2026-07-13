---
pulse: 01
title: National data and context inventory
status: done
wave: nationwide-2020-operational-certification
validation_level: data custody and prerequisite matrix
---

# Pulse 01 - National Data And Context Inventory

## Deliverables

- [x] 50-State TIGER block source inventory;
- [x] 50-State PL 94-171 source inventory;
- [x] existing connected RCTX inventory;
- [x] State block-count and district-count ranking;
- [x] estimated build storage/runtime;
- [x] resumable batch order.

## Gate

No nationwide context build starts until every State has both source families or
an explicit missing-data blocker.

## Result

All 50 States have both source families. The inventory counts 8,126,956 blocks,
11 existing connected contexts, and approximately 2.02 GiB for all 50 RCTX
files.

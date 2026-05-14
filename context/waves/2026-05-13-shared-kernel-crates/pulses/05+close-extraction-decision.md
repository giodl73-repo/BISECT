---
wave: shared-kernel-crates
pulse: 05
status: done
depends_on: [03, 04]
governing_roles:
  - LEDGER
  - COVENANT
  - TRENCH
---

# Pulse 05 - Wave Close And Extraction Decision

## Mission

Close the first shared-kernel wave with an explicit extraction decision and
follow-up plan for `rstat-core`.

## Deliverables

- [x] Update the source spec with API decisions made during implementation.
- [x] Decide whether `rgraph-core` remains incubated or moves toward a shared
  repository.
- [x] Record `rstat-core` as a separate future wave if still warranted.
- [x] Run final validation and close the wave.

## Completion Notes

- Decision: keep `rgraph-core` incubated in this workspace until ROUTE can consume
  it through a portable shared dependency.
- `rstat-core` remains a future separate wave, seeded by the source spec.


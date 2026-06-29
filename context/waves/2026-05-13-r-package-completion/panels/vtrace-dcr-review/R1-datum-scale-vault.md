# R1 DATUM / SCALE / VAULT Review - VTRACE DCR Filing

### F-01 - WARN: Public evidence package contract needs a versioned contract artifact
File: `docs\vtrace\DCRS.md`
Finding: DCR-004 requires a public evidence package contract, but its acceptance criteria do not explicitly require a named versioned schema or contract file.
Consequence: Downstream consumers could cite "the contract" without a stable version, making breaking changes and evidence comparisons ambiguous.
Fix: Require DCR-004 closure to publish or reference a versioned contract artifact with required fields, optional fields, compatibility rules, and change-control trigger.

### F-02 - WARN: Custody retention and immutability expectations are not explicit
File: `docs\vtrace\DCRS.md`
Finding: DCR-004 requires hashes, limitations, and custody disposition, but it does not state retention, immutability, or replacement rules for public bundles.
Consequence: A public evidence package could be replaced or partially regenerated without clear downstream notice.
Fix: Add closure criteria for release-bundle immutability or replacement notices, including hash manifest retention and supersession rules.

## Role Summary

DATUM/SCALE/VAULT agree that DCR-004 is the right gate for public artifacts. It should close only with a versioned, custody-aware contract.

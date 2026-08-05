# Pulse 06 Internal Custody Review

**Lane:** VAULT-style internal review

**Decision:** `accepted_with_disclosed_historical_limitation`

**Public gate:** open

The national assignment evidence is suitable for an internal release candidate
because tree hashes, normative context hashes, complete assignments,
connectivity, recursive schedules, and arithmetic population floors are
independently replayable.

Forty legacy packages lack matching retained builder bytes. The search and
disposition are recorded in
`docs/experiments/nationwide-2020/BUILDER_CUSTODY_DISPOSITION.md`. This review
does not describe those packages as clean execution replays and does not waive
the concrete external VAULT/public-claim gate required before publication.

All future packages must embed immutable source snapshots. The four recovered
128-seed Rust packages already satisfy that rule.

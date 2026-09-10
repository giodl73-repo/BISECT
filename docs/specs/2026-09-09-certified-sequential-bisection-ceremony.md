# Certified Sequential Bisection Ceremony

**Status:** implemented  
**Schema:** `certified-bisection-ceremony-v1`  
**Normative implementation:** `crates/bisect-ilp/src/ceremony.rs`  
**Operator CLI:** `crates/bisect-cli/src/ceremony_cmd.rs`

## Purpose

The ceremony makes recursive bisection a sequence of public, irreversible,
locally reviewable decisions rather than a single free-form statewide map
selection. It does not claim that politically blind geography guarantees a
partisan-symmetric result. It establishes that the published plan follows a
precommitted rule and that no later round exchanges units across an already
certified cut.

## Preconditions

Before the first cut, the authority publishes and externally timestamps:

1. the rule-profile hash;
2. the root certified-instance hash;
3. the district count and canonical `floor/ceil` tree;
4. the minimum review interval;
5. the required number and kind of independent publication witnesses; and
6. the challenge, halt, correction, and restart policy.

The genesis publication carries its own witness receipts. The same review
interval must elapse between genesis and the root-cut announcement; this makes
the rule itself reviewable before the first geographic decision is released.

The rule-profile file is the governance bundle whose hash is stored in the
genesis configuration. It fixes Census vintage and units, population field,
adjacency, island bridges, edge-weight units and rounding, population
objective, subdivision policy, prohibited inputs, canonical tie-break, witness
validation policy, and the challenge, halt, correction, and restart policy.
Search engines may discover candidates but do not determine which candidate
wins.

## Round protocol

Round zero publishes the certified root cut. Round `d` publishes every
non-leaf cut at tree depth `d`, ordered by binary node path. A later round is
ineligible until at least `minimum_review_seconds` after publication of the
previous round.

The reference API exposes `start_ceremony` and `announce_next_round`. The
announcement transition is transactional: a proposal that fails schedule,
hash-chain, witness, timing, or tree-binding verification does not mutate the
accepted transcript. The last eligible round changes status to `completed`
and binds the certified tree ID automatically.

Each round publishes:

- its index and publication time;
- the previous round ID;
- for every cut, its node path and complete certified split artifacts
  (instance, certificate, and proof);
- independent transparency-log or trusted-timestamp receipts; and
- a canonical round ID covering all fields above.

The receipt strings in the BISECT artifact are bindings to external evidence.
The BISECT verifier checks their presence and uniqueness, not the external
service's signature or clock. A production deployment must add adapters that
verify the selected witness systems.

## Review and challenge window

During the interval, any observer can verify the disclosed split, reproduce
its objective, inspect the proof, and confirm that its two child universes are
exactly the units assigned by the parent. Descendant results must not be
published early.

A valid challenge halts the ceremony. A halted transcript remains public and
records a nonempty reason. It cannot be edited into a successful transcript.
Correction requires either an objectively verifiable packaging correction
that leaves the certified decision unchanged, or a new ceremony with a new
configuration hash and an explicit link to the halted ceremony. The authority
may not substitute a politically preferable cut inside the same ceremony.

## Completion

A completed transcript contains every non-leaf tree depth, binds the final
certified tree ID, and passes both the tree verifier and ceremony verifier. An
open transcript must be a strict prefix and cannot claim a final tree ID.

## Operator workflow

The CLI writes a new immutable transcript at every transition. It refuses to
overwrite an earlier public artifact.

```powershell
bisect ceremony start `
  --tree certified-bisection-tree.json `
  --rule-profile canonical-rule-v1.json `
  --genesis-unix-seconds 1788979200 `
  --review-seconds 604800 `
  --minimum-witnesses 2 `
  --witness transparency-log:GENESIS-A `
  --witness timestamp-authority:GENESIS-B `
  --out ceremony/00-genesis.json

bisect ceremony announce `
  --tree certified-bisection-tree.json `
  --transcript ceremony/00-genesis.json `
  --published-unix-seconds 1789584000 `
  --witness transparency-log:ROOT-A `
  --witness timestamp-authority:ROOT-B `
  --out ceremony/01-root.json

bisect ceremony verify `
  --transcript ceremony/01-root.json
```

Repeat `announce` with the latest transcript and a new output filename after
each review interval. Use `ceremony halt --reason TEXT` when a challenge or
operational failure prevents continuation.

An open or halted prefix is independently verifiable without the unreleased
tree. Each cut announcement contains its instance, certificate, and proof.
Supply `--tree certified-bisection-tree.json` when verifying a completed
ceremony so the verifier can check the final tree binding and leaf coverage.

## Bakeoff integration

Prospective comparisons use
[`certified-ceremony-bakeoff-extension-v1`](2026-09-09-certified-ceremony-bakeoff-extension.md).
That extension keeps map/computational outcomes and procedural evidence on
separate scoreboards. Frozen historical bakeoffs are not amended or
retroactively treated as ceremonies.

## Security and claim boundary

Hash chains expose mutation; they do not prove honest wall-clock publication.
Self-asserted Unix timestamps are not trusted timestamps. At least one
independent append-only public log is required operationally, and multiple
witnesses are recommended for an official process.

The completed claim is:

> This plan is the unique output of the frozen certified sequential rule. Each
> eligible bisection round was published only after the preceding review
> interval, and the final tree contains no cross-boundary revision of an
> earlier certified cut.

The ceremony does not by itself establish VRA compliance, legal adoption,
partisan fairness, community representation, or the absence of political
effects.

## Acceptance checks

- A valid two-round, four-district ceremony verifies.
- A descendant round published one second early is rejected.
- A substituted certificate is rejected even if envelope hashes are rebuilt.
- Missing or duplicate witness receipts are rejected.
- Completed, open, and halted status rules are enforced.
- The full `bisect-ilp` unit and hostile-corpus suite remains green.

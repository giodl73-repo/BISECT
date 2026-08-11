# Block-Ensemble Host-Capacity Admission

**Contract ID:** `nrs-block-ensemble-host-capacity-v1`

**Status:** future-protocol hardening; does not reopen Pulse 25

## Purpose

Prevent a governed ensemble process from starting unless the filesystem that
will hold its trace can reserve the declared scratch ceiling, all remaining
retained-evidence capacity, and independent host safety headroom.

## Admission Formula

Immediately before every future governed process launch, require:

```text
free bytes >= scratch ceiling
            + (retained ceiling - retained bytes already in custody)
            + host safety reserve
```

The default future-protocol values are 3 GiB scratch, 3 GiB retained evidence,
and a 2 GiB safety reserve. At zero retained custody this requires 8 GiB free
on the evidence volume. Negative values and retained custody already above its
ceiling are contract errors.

## Semantics

- Admission runs against the actual package volume, not an unrelated system
  volume or an estimate captured earlier in the session.
- A rejection occurs before runner launch. It is neither a chain failure nor a
  statistical result, and cleanup followed by a repeated admission check is
  allowed because no governed seed stream began.
- The checker never deletes files. Operators must identify and remove only
  independently disposable data, then run admission again.
- After runner launch, the protocol's process memory, wall, scratch, retained,
  schedule, and failure rules remain independently binding.
- Closed protocols and their historical tools/evidence are not mutated or
  resumed by this contract.

## Pulse 25 Postmortem

Pulse 25 began GA Kruskal with only about 72 MB free on the host volume. The
runner exited on `os error 112` before producing a trace. A subsequent audit
found 16.97 GB of temporary diagnostic output and 11.08 GB of WSL crash dumps,
plus other disposable temporary artifacts. The registered no-retry rule was
honored; this admission contract applies only to newly frozen future work.

## Claim Boundary

Passing admission establishes only contemporaneous filesystem headroom. It
does not predict final trace size, prove portable performance, authorize a
scientific protocol, or weaken any governed stopping rule.

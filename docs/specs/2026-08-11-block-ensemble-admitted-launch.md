# Block-Ensemble Capacity-Admitted Launch Boundary

**Contract ID:** `nrs-block-ensemble-admitted-launch-v1`

**Status:** reusable future-protocol control; no governed launch authorized

## Contract

Immediately before a future block-ensemble process is created, the adapter
must:

1. resolve the evidence package and ledger paths;
2. measure free bytes on the filesystem that contains the package;
3. read `retained_bytes` from the protocol ledger;
4. apply `nrs-block-ensemble-host-capacity-v1`;
5. create, without overwriting, a JSON admission record; and
6. create the requested process only when
   `process_launch_authorized` is `true`.

The command is passed as an argument vector directly to the operating system.
The adapter does not use a shell, delete data, mutate the ledger, retry a
rejected launch, or reinterpret the child process return code.

## Interface

```text
python scripts/research/launch_block_ensemble_admitted.py \
  --package PATH \
  --ledger PATH/ledger.json \
  --admission-record PATH/admission.json \
  --cwd REPOSITORY_ROOT \
  -- RUNNER [RUNNER_ARGS...]
```

Scratch, retained, and safety limits default to 3 GiB, 3 GiB, and 2 GiB,
respectively. A future frozen protocol may set explicit values through the
corresponding CLI options, but those values remain part of that protocol's
precommitment.

## Failure Semantics

- Invalid paths, ledgers, limits, or empty commands fail before process
  creation.
- An existing admission-record path fails closed so prior custody cannot be
  silently replaced.
- A capacity rejection writes its evidence record, returns `1`, and creates no
  child process.
- An authorized launch returns the child process's exit status unchanged.

The small interval between measuring capacity, writing the custody record, and
creating the process is the defined admission boundary. The adapter makes no
claim that free space cannot change concurrently afterward.

## Claim Boundary

This contract proves only that the recorded capacity formula authorized a
specific process-creation boundary. It is not a resource monitor, scientific
protocol, execution verifier, mixing diagnostic, or authorization to reopen
any closed experiment.

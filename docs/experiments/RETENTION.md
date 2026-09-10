# Experiment Evidence Retention

BISECT keeps evidence needed to inspect claims without requiring every checkout
to carry every superseded generated payload.

## Active-tree contract

Keep these files for completed experiments:

- a concise README with scope, status, claim boundaries, and recovery details;
- the protocol manifest and aggregate analysis;
- compact CSV or JSON summaries used by published findings;
- representative accepted and failure fixtures needed by tests;
- the canonical output package for the currently supported result.

Move superseded bulk outputs out of the active tree once a canonical successor
exists. Record the source commit and Git tree object before removal so the exact
payload remains recoverable. Do not replace diffable summaries with binary
documents.

## Regeneration and recovery

Experiment commands and frozen inputs must remain sufficient to regenerate the
canonical package. Historical payloads can be inspected without restoring them
to the working tree:

```powershell
git ls-tree -r <commit> -- <path>
git archive <commit> <path> -o evidence.zip
```

New experiments should declare their retention class before execution:

- `canonical`: retained in full while it is the supported result;
- `compact-witness`: aggregate outputs and representative fixtures retained;
- `transient`: regenerated locally and never committed.

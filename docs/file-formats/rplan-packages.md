# RPLAN Golden Packages

An RPLAN golden package is the smallest public artifact bundle that lets a
reader verify a final plan without understanding the upstream algorithm that
produced it.

## Directory Shape

```text
package-name/
  plan.rplan
  context.rctx
  audit-certificate.json
  manifest.json
  method-transcript.json        # optional
  solve-report.json             # optional
  search-report.json            # optional
```

## Required Files

| File | Role | Verification meaning |
|---|---|---|
| `plan.rplan` | Final district assignment and plan identity | Supplies the plan hash checked by the certificate |
| `context.rctx` | Unit order, graph, populations, source hashes, and optional context | Supplies the context hash checked by the certificate |
| `audit-certificate.json` | Profile-scoped audit result | Verifies declared checks, hashes, source hashes, and lineage |
| `manifest.json` | Package inventory | Records file names, SHA-256 file hashes, roles, and verification command |

## Optional Files

| File | Role |
|---|---|
| `method-transcript.json` | Method-level status transcript for construction/search/evolutionary families |
| `solve-report.json` | Solver report for exact families such as branch-and-cut or branch-and-price |
| `search-report.json` | Search/improvement report for local search, ensemble, or evolutionary outputs |

Optional files are upstream evidence. They do not replace the package
certificate. A package is verifier-facing only when the certificate, plan, and
context agree.

## Manifest V1

Golden package manifests are currently example-local documentation artifacts,
not a crate-owned stable schema. Use this shape until the corpus has enough
examples to justify moving the manifest into `rplan-*` or `bisect-report`.

```json
{
  "schema_version": "u20-public-example-manifest-v1",
  "example_id": "grid3x3-valid",
  "paper": "U.20+plan-audit-certificates",
  "files": [
    {
      "path": "plan.rplan",
      "sha256": "354470c7a18e0407c6498a9eb6518f7369d241cdfc7754a520b968e4e7b00b27",
      "role": "plan"
    }
  ],
  "verification_command": "cargo run -p rplan-cli -- verify-certificate ..."
}
```

Required `files[*].role` values for the core package are `plan`, `context`, and
`certificate`. Optional roles should name the evidence type, such as
`method-transcript`, `solve-report`, or `search-report`.

## Verification Layers

The package contract intentionally separates four ideas:

| Layer | Checks | Does not claim |
|---|---|---|
| File integrity | Manifest SHA-256 values match package files | That the plan is valid |
| Certificate verification | Certificate content hash, plan hash, context hash, source hashes, and lineage are coherent | That the plan is fair or legally sufficient |
| Audit result | Declared profile checks pass, fail, or pass with warnings | That unrequested checks were evaluated |
| Policy/legal interpretation | External users interpret the verified evidence | Any automatic legal safe harbor |

The default command for a complete package is:

```powershell
cargo run -p rplan-cli -- verify-certificate `
  --certificate path/to/audit-certificate.json `
  --plan path/to/plan.rplan `
  --context path/to/context.rctx
```

`rplan verify-certificate` is the neutral verifier surface. `bisect verify`
may additionally verify BISECT run manifests, solver summaries, assignment
similarity, and report-sidechain artifacts.

## Current Public Examples

- `docs/examples/u20-plan-audit-certificates/grid3x3-valid/`

## Negative Fixture Catalog

The U.20 corpus should include examples or tests for:

- missing context for a contextual certificate
- plan assignment tamper
- context/source hash change
- canonical unit-order mismatch
- profile mismatch
- stale RCTX context
- unsupported constraint or missing-input result
- broken lineage or reserved lineage-field attempt

# RCTX AI Context Package Example

RCTX is domain-generic: it records context units, sources, graph records,
crosswalks when needed, and claim boundaries. CROP can use it for replayable AI
context packs without importing redistricting assumptions.

The `rctx-core::synthetic_ai_context_package_fixture()` helper demonstrates the
shape:

| RCTX field | AI context interpretation |
|------------|---------------------------|
| `source_index` | Markdown files, repo files, issue exports, chat logs, or notes with hashes. |
| `units` | Stable chunk, note, issue, commit, claim, or decision identifiers. |
| `graphs` | Evidence-neighborhood graph records used by context cropping. |
| `crosswalks` | Optional mappings between chunking schemes or source versions. |
| `claim_boundary.proves` | What the package verifier actually checked. |
| `claim_boundary.does_not_prove` | Model-answer correctness, semantic completeness, and embedding quality. |

Minimal CROP-style pattern:

```text
source: repo/README.md
source: notes/architecture.md
unit:   unit:repo-readme:overview
unit:   unit:note-architecture:bridge
unit:   unit:issue-17:decision
graph:  graph:crop-evidence-neighborhood
```

The helper verifies through `rctx-core::verify_package`, including manifest hash,
context unit index, source index, graph context/source references, and claim
boundary presence.


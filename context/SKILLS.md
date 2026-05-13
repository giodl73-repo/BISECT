# Local Skills

This repo keeps Claude/Codex workflow skills under `.claude/skills/`.

## Wave Skills

The R-package completion work uses these skills:

| Skill | Purpose |
|---|---|
| `/r-wave` | Manage active wave status, next pulse, and closeout. |
| `/r-fork` | Materialize a pulse into a single fork context file. |
| `/r-pulse` | Execute one pulse end to end with scout, implementation, docs, and validation. |
| `/r-review` | Run role review and write panel findings. |

Source of truth:

- `context/waves/PHASES.md`
- `context/waves/{active}/WAVE.md`
- `context/waves/{active}/pulses/`

## Legacy Skills

The existing enhancement and testing skills remain available. Use wave skills
for multi-pulse package or architecture goals, and enhancement skills for older
single-feature workflows.


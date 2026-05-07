# Plan: Rename `redist` → `bisect`

**Date**: 2026-05-07  
**Status**: Draft — pending review  
**Motivation**: The R `redist` package (alarm-redist.org) shares our name, won the 2022 Statistical Software Award, and is the academic standard. Having two `redist` projects creates confusion in court filings, academic citations, and web search. `bisect` is short, references the core recursive-bisection algorithm, and is unambiguous in the redistricting context.

---

## Name decision

| Item | Old | New |
|------|-----|-----|
| CLI binary | `redist` | `bisect` |
| Project name | Redist / redist | Bisect |
| GitHub repo | `giodl73-repo/REDIST` | `giodl73-repo/bisect` (new) |
| Old repo | stays | README forward to new repo |
| Internal crates | `redist-*` | Phase 1: keep; Phase 2: `bisect-*` |
| CLI commands | `redist state ...` | `bisect state ...` |

---

## Phase 1 — Soft rename (low risk, ~2 hours)

**Goal**: Get the new repo live with a working `bisect` binary. Internal crate names unchanged.

### 1.1 Rename CLI binary

In `redist/crates/redist-cli/Cargo.toml`:
```toml
[[bin]]
name = "bisect"          # was: redist
path = "src/main.rs"
```

Update any CI scripts that call `./redist` or `cargo run --bin redist`.

### 1.2 Update user-facing documentation

- `README.md`: s/`redist`/`bisect`/ in all CLI examples
- `CLAUDE.md`: update core commands section
- `docs/REDIST_CLI.md` → `docs/BISECT_CLI.md` (or update in-place)
- `docs/PAPERS.md`: no changes needed (paper codes B.xx, G.xx are not affected)
- All `docs/concepts/` and `docs/quickstart/` guides: update CLI examples

### 1.3 Create new GitHub repo

1. Create `github.com/giodl73-repo/bisect` (new empty repo)
2. Push current `main` branch there
3. Set `bisect` as the primary repo going forward
4. All future work happens in `bisect`

### 1.4 Update old repo with forward notice

In `giodl73-repo/REDIST` README, replace content with:
```markdown
# ⚠️ This project has moved

**redist** has been renamed to **bisect** and is now maintained at:

👉 **https://github.com/giodl73-repo/bisect**

This repository is archived. All issues, PRs, and discussions have moved to the new repository.
```

Archive the old repo (GitHub Settings → Archive this repository).

### 1.5 CI / workflow updates

- Update `.github/workflows/*.yml` if they reference the binary name
- Update any `cargo build --bin redist` → `cargo build --bin bisect`

### 1.6 Phase 1 verification

```bash
cargo build -p redist-cli
./target/debug/bisect --version   # must work
./target/debug/bisect state --state VT --year 2020  # smoke test
```

---

## Phase 2 — Full rename (thorough, ~1 day)

**Goal**: Rename all internal crates from `redist-*` to `bisect-*`, update all Rust imports, publish to crates.io.

### 2.1 Crate renames

| Old | New |
|-----|-----|
| `redist-cli` | `bisect-cli` |
| `redist-core` | `bisect-core` |
| `redist-data` | `bisect-data` |
| `redist-apportion` | `bisect-apportion` |
| `redist-metis` | `bisect-metis` |
| `redist-ensemble` | `bisect-ensemble` |
| `redist-smc` | `bisect-smc` |
| `redist-ilp` | `bisect-ilp` |
| `redist-multiscale` | `bisect-multiscale` |
| `redist-pareto` (planned) | `bisect-pareto` |
| `redist-analysis` | `bisect-analysis` |
| `redist-map` | `bisect-map` |
| `redist-report` | `bisect-report` |

### 2.2 Code changes

For each crate rename:
1. Update `[package] name` in `Cargo.toml`
2. Update all `path` dependencies in sibling `Cargo.toml` files
3. Update all `use redist_*::` imports to `use bisect_*::` in Rust source
4. Update `extern crate` references if any

Script approach (sed across all .rs and .toml files):
```bash
find redist/ -name "*.rs" -o -name "*.toml" | xargs sed -i 's/redist_cli/bisect_cli/g'
# etc. for each crate
```

### 2.3 Spec and documentation updates

All accepted specs reference CLI flags — update:
- `redist state --state NC` → `bisect state --state NC`
- `redist ensemble --method smc` → `bisect ensemble --method smc`
- `redist fetch --year 2020` → `bisect fetch --year 2020`
- All docs/specs/*.md files
- All research paper LaTeX (CLI examples in algorithm sections)

### 2.4 Publish to crates.io

- Register crates.io account under `bisect` organization
- Publish: `bisect-core`, `bisect-metis`, `bisect-ensemble`, `bisect-smc`, `bisect-cli`
- Add `Cargo.toml` metadata: license, description, repository, documentation URLs

### 2.5 Python bindings rename

`redist_py` → `bisect_py` (if/when the Python binding is published)

---

## Timeline estimate

| Phase | Effort | Risk |
|-------|--------|------|
| Phase 1: binary + docs + new repo | 2 hours | Low |
| Phase 2: crate renames + imports | 6-8 hours | Medium (many sed substitutions) |
| Phase 2: specs/papers update | 3-4 hours | Low |
| Phase 2: crates.io publish | 1 hour | Low |

Phase 1 can be done in one session. Phase 2 is a full day but low-risk with automated substitution.

---

## Open questions

1. **Org name**: publish under personal account (`giodl73`) or create a GitHub org (`bisect-rs`)?
2. **crates.io namespace**: is `bisect` taken? (Check before committing to name)
3. **Python package**: rename to `bisect` on PyPI or `bisect-redistricting` to avoid collision with Python stdlib `bisect`?
4. **Backwards compatibility**: any external users of the `redist` binary who need a migration path?
5. **Domain**: acquire `bisect.dev` or similar for project website?

---

## Review checklist

- [ ] Phase 1 plan is complete and low-risk
- [ ] Phase 2 crate rename list is exhaustive
- [ ] CI workflow changes identified
- [ ] crates.io `bisect` namespace availability confirmed
- [ ] Python stdlib collision addressed (`bisect` is stdlib — PyPI package should be `bisect-redistricting` or `bisect-rs`)
- [ ] Old repo archival plan approved

# Repo Map Standard

`repo-map.toml` is the source of truth for local multi-repo development. It tells contributors which sibling repositories they need, where those checkouts live locally, and which Cargo git dependencies should be patched to local paths.

Cargo cannot read arbitrary mapping files directly, so each repo uses a generated local `.cargo/config.toml` file. That generated file is ignored by git and should be recreated from `repo-map.toml` whenever mapped paths change.

## Required files

| File | Commit? | Purpose |
|---|---:|---|
| `repo-map.toml` | yes | Shared checkout/dependency map. |
| `tools/repo_map.py` | yes | Generates and checks local Cargo patches. |
| `.cargo/config.toml` | no | Generated local Cargo `[patch]` overrides. |

## Workflow

```powershell
python tools\repo_map.py list
python tools\repo_map.py clone-commands
python tools\repo_map.py write-cargo-config
python tools\repo_map.py check
cargo metadata --no-deps
```

## Cargo dependency rule

External repo dependencies stay as normal `git` dependencies in `Cargo.toml`. Local development uses generated `[patch]` entries from `repo-map.toml`; do not hard-code one developer's checkout path in a Cargo manifest.

Example:

```toml
# Cargo.toml
[workspace.dependencies]
example-core = { git = "https://github.com/example/EXAMPLE.git", package = "example-core", branch = "main" }
```

```toml
# repo-map.toml
[repos.example]
relative = "../tracker/repos/tools-infra/example"
clone_url = "https://github.com/example/EXAMPLE.git"
branch = "main"
cargo_patches = [
  { package = "example-core", source = "https://github.com/example/EXAMPLE.git", path = "crates/example-core" },
]
```

Then run:

```powershell
python tools\repo_map.py write-cargo-config
```

The generated `.cargo/config.toml` makes Cargo use the local checkout.

## Field standard

| Field | Required | Meaning |
|---|---:|---|
| `relative` or `absolute` | yes | Local checkout path. Prefer `relative` from the current repo root. |
| `canonical` | recommended | Human-readable canonical repo name. |
| `clone_url` | recommended | URL used by `clone-commands`. |
| `branch` | optional | Expected branch when cloning. |
| `required_by` | recommended | Why this repo is needed. |
| `cargo_patches` | optional | Cargo package overrides generated into `.cargo/config.toml`. |

Each `cargo_patches` row requires:

| Field | Meaning |
|---|---|
| `package` | Cargo package name to patch. |
| `source` | Exact git source URL used by the dependency being patched. |
| `path` | Package path inside the mapped repository. |

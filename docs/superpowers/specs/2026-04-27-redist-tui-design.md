# redist TUI — Design Spec
**Date:** 2026-04-27  
**Status:** Approved for implementation planning

---

## Problem

The redist CLI has 18 subcommands and ~30 flags. Practitioners — commission staff, researchers, legal teams, international electoral officers — must memorise flag syntax to do their work. The TUI eliminates that barrier without dumbing the tool down: informed users navigate freely; wizards are available but never forced.

---

## Personas & Key Findings

Six personas were interviewed across 10 questions. Design decisions trace directly to their answers.

| Persona | Primary need | Deal-breaker |
|---------|-------------|--------------|
| Commission Aide | Plan browser, guided run form | Can't see where outputs went |
| Academic Researcher | Command palette (`:run WA house 2020`), raw log, export | Latency; can't pipe to jq |
| Legal/Court Staff | `[v] Verify` front and center; PASS/FAIL large | Verify buried; output looks unofficial |
| Data Journalist | Sort/filter by splits; comparison export CSV | Can't get maps or CSV |
| International Officer | Location recognised (IE, MT-PARLIAMENT); non-US jargon | Forced US year conventions |
| Graduate Student | Inline metric glossary; actionable errors | Jargon in error messages |

**12 non-negotiable design decisions from the panel:**

1. Plan browser is the home screen — sortable, filterable
2. Verify is a top-level quick action, not buried in a menu
3. `:` command palette for power users + menu navigation for others
4. Errors show summary by default, expandable to raw with one key
5. Policy/Doctor in a side panel overlay, never full-screen takeover
6. Legal needs PASS/FAIL in a large box, not a table row
7. Empty state shows data status (adjacency files, years, location config)
8. Session config in `~/.config/redist/tui.toml`, editable manually
9. Legal gets `--no-session` for fresh-start sessions
10. US jargon softened — "TIGER file" → "census geometry file"
11. Graduate Student gets inline metric explanations via `[?]`
12. proof canvas for metrics cards — sparklines, mini-bars, value formatting

---

## Architecture

### New crate: `redist-tui`

Binary crate at `redist/crates/redist-tui/`. Added to the workspace. Zero changes to `redist-cli` or its 792 tests. `redist-cli` gains a `tui` subcommand that execs `redist-tui`.

```
redist-tui/src/
├── main.rs          — terminal setup, event loop, screen router
├── app.rs           — App state (current screen, selected plan, session)
├── screens/
│   ├── home.rs      — plan browser + quick-action bar
│   ├── run.rs       — run form + live progress
│   ├── compare.rs   — side-by-side metrics
│   ├── verify.rs    — manifest → PASS/FAIL
│   ├── doctor.rs    — pre-flight check view
│   └── overlay.rs   — command palette + side panels
└── canvas/
    └── regions.rs   — proof API → String → ratatui Paragraph
```

### Dependency stack

```
redist-tui
  ├── ratatui + crossterm      — terminal event loop, layout, key events
  ├── proof (git submodule)    — canvas, sparklines, mini-bars, value formatting
  ├── mdpath (git submodule)   — stable URIs (future: plan element addressing)
  ├── redist-cli (lib)         — LocationRegistry, chamber_district_count, policy
  ├── redist-report (lib)      — PlanManifest, RPLAN reading
  └── serde_json               — reading analysis JSON files
```

### Proof/mdpath packaging

proof and mdpath live as sibling directories to the apportionment repo (`C:\src\proof` and `C:\src\mdpath`). `redist-tui/Cargo.toml` references them as path dependencies:

```toml
[dependencies]
proof = { path = "../../../../proof" }
mdpath = { path = "../../../../mdpath" }
```

Only proof's canvas/element/dashboard functionality is used — not the full proof CLI surface. Future: extract `proof-canvas` as a standalone publishable crate to remove the sibling-directory requirement.

### Key invariant

`redist-tui` reads output files directly (`final_assignments.json`, `analysis/*.json`, `manifest.json`) for the browser, compare, and verify screens. Only the Run screen spawns a subprocess, piping `STATUS:` output back to drive the live progress view.

### Session persistence

`~/.config/redist/tui.toml`:
```toml
[session]
location = "WA"
chamber = "house"
year = "2020"
version = "v1"
output_base = "outputs"
resolution = "tract"

[ui]
sort_column = "splits"
sort_direction = "asc"
show_metric_glossary = true

[international]
adjacency_override = ""
seats_per_district = 1
```

`redist tui --no-session` ignores this file for Legal's clean-slate sessions.

---

## Screen Map & Navigation

```
                    ┌─────────────────────┐
                    │   HOME (plan browser)│
                    │  r a c v d  / :      │
                    └──────────┬──────────┘
         ┌──────────┬──────────┼──────┬──────────┐
         ▼          ▼          ▼      ▼          ▼
      ┌──────┐  ┌───────┐  ┌──────┐ ┌──────┐ ┌──────┐
      │ RUN  │  │ANALYZE│  │COMPARE│ │VERIFY│ │DOCTOR│
      │form+ │  │       │  │metrics│ │PASS/ │ │check │
      │progress│        │  │table │ │ FAIL │ │list  │
      └──────┘  └───────┘  └──────┘ └──────┘ └──────┘
```

**Three navigation modes — coexist, never conflict:**

| Mode | Keys | Primary users |
|------|------|---------------|
| Arrow keys + Enter | Standard menu nav | Aide, Legal, Grad Student |
| Single-key shortcuts | `r` `a` `c` `v` `d` | Everyone once learned |
| `:` command palette | `:run WA house 2020` | Researcher, Journalist |

**Universal keys:**
- `Escape` / `q` — back one level, always
- `?` — inline metric glossary (Graduate Student)
- `p` — policy side panel for current location
- `e` — expand raw log/error
- `Ctrl+S` — copy current view to clipboard

**Status bar (always visible):**
```
[WA · house · 2020 · v1]  plans: 12  │  last run: 4m ago  │  ? help  q quit
```

---

## Screens

### Home

Two-panel layout: plan list (65%) + selected plan detail (35%).

```
┌─ redist tui ──────────────────────────────────────────────────────────────────┐
│  [r] Run  [a] Analyze  [c] Compare  [v] Verify  [d] Doctor  [/] Search  [:] │
├───────────────────────────────────────────────┬───────────────────────────────┤
│  Plans  12 total   filter: ▌                  │  wa_house_seed_42             │
│  Label                St  Chamber  Yr   D  Sp ✓│  Washington · house · 2020  │
│ ▶ wa_house_seed_42   WA  house   2020  98   8 ✓│  98 districts · seed 42    │
│   wa_house_seed_17   WA  house   2020  98  12 ✗│                            │
│   wa_senate_v1       WA  senate  2020  49   4 ✓│  PP (mean)  ████████░░ .31 │
│   ca_congressional   CA  congress 2020 52  31 ✓│  Balance    ████░░░░░░ 3.2%│
│   ie_dail_2022       IE  parl    2022  39   2 ✓│  Splits     ██░░░░░░░░  8  │
│                                               │  Contiguous  ✓ all 98      │
│  [↑↓] navigate  [s] sort  [/] filter         │                            │
│                                               │  [Enter] open  [a] analyze │
│                                               │  [c] compare   [x] export  │
├───────────────────────────────────────────────┴───────────────────────────────┤
│  WA · house · 2020 · v1    plans: 12    last run: 4m ago    ? help  q quit  │
└───────────────────────────────────────────────────────────────────────────────┘
```

**Detail panel:** proof canvas renders the three metric mini-bars. proof's `mini-bar` element sized to panel width at runtime.

**Column behaviour:**
- `✗` in contiguous column — highlighted red
- Splits > threshold (from location_policy.json) — highlighted amber
- `[s]` cycles sort: splits↑ splits↓ PP↑ PP↓ deviation↑

**Empty state:**
```
│  No plans found in outputs/v1/2020/                                          │
│  Data:  WA adjacency 2020 ✓   Demographics 2020 ✓   Election data 2020 ✓  │
│  Press [r] to run your first plan, or [d] for a pre-flight check            │
```

### Run — Form state

```
┌─ Run New Plan ────────────────────────────────────────────────────────────────┐
│  Location     WA  · Washington                                               │
│  Chamber      [ house ▼ ]   → 98 districts  (from policy)                  │
│  Year         [ 2020  ▼ ]                                                   │
│  Resolution   [ tract ▼ ]   ⚠ block_group recommended for 98D             │
│  Seed         [ 42      ]   leave blank for random                         │
│  Label        [ wa_house_seed_42     ]                                      │
│  Version      [ v1      ]                                                   │
│  Balance tol  [ 5.0%    ]   from policy · override or leave                │
│                                                                              │
│  ─── Doctor (pre-flight) ──────────────────────────────────────────────── │
│  ✓  Location registered  ✓  Year valid  ✓  Adjacency found               │
│  ⚠  Granularity: 98D at tract resolution may hit balance floor             │
│     → switch resolution to block_group above to fix                        │
│                                                                              │
│  Command: redist state --state WA --chamber house --year 2020 --seed 42   │
│                                                                              │
│  [Enter] Run    [Tab] next field    [Esc] back                              │
└───────────────────────────────────────────────────────────────────────────── ┘
```

Doctor runs silently as fields change — warnings appear inline, no separate step. Equivalent command shown for Researcher copy-paste.

### Run — Live progress state

```
┌─ Running · wa_house_seed_42 ──────────────────────────────────────────────────┐
│  Washington · house · 2020 · 98 districts · seed 42                          │
│                                                                               │
│  Depth 1   ████████████████████  2 / 2    done                               │
│  Depth 2   █████████████░░░░░░░  3 / 4    running                           │
│  Depth 3   ░░░░░░░░░░░░░░░░░░░░  0 / 8    waiting                           │
│  Depth 4   ░░░░░░░░░░░░░░░░░░░░  0 / 16   waiting                           │
│  Depth 5   ░░░░░░░░░░░░░░░░░░░░  0 / 32   waiting                           │
│  Depth 6   ░░░░░░░░░░░░░░░░░░░░  0 / 64   waiting                           │
│                                                                               │
│  Assigned   ██████░░░░░░░░░░░░  22 / 98                                      │
│  Elapsed    0:42                                                               │
│  Balance    ✓ all within 5.0%                                                │
│                                                                               │
│  ─ Log ──────────────────────────────────────────────────── [e] full log    │
│  WA: depth 2 · node 10 · 12 districts                                       │
│  WA: depth 2 · node 00 · 12 districts  done                                 │
│                                                                               │
│  [Esc] cancel gracefully                                                     │
└───────────────────────────────────────────────────────────────────────────── ┘
```

Depth bars are proof `mini-bar` elements. STATUS: lines from the subprocess update `App.state`; ratatui re-renders on each new line.

### Run — Completion card

```
┌─ Done · wa_house_seed_42 · 2:34 ─────────────────────────────────────────────┐
│  ✓  98 districts  ·  all contiguous  ·  balance OK                          │
│                                                                              │
│  Compactness (PP)   ████████░░  0.31                                        │
│  Balance (max dev)  ████░░░░░░  3.2%                                        │
│  County splits      ██░░░░░░░░  8                                           │
│                                                                              │
│  [a] Analyze     [r] Report     [c] Compare     [Enter] Back to plans      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Compare

Plan B accepts label, `.rplan` path, or `.csv` (DRA format).

```
┌─ Compare ─────────────────────────────────────────────────────────────────────┐
│  Plan A  wa_house_seed_42                                                    │
│  Plan B  [ enter label or path to .rplan / .csv...   ]  [Go]               │
├───────────────────────────────────────────────────────────────────────────── │
│  Similarity  ████████████████░░░░  Jaccard 0.847   [?] what is this        │
│                                                                              │
│  Metric            Plan A           Plan B           Δ                      │
│  ────────────────────────────────────────────────────────────────────────   │
│  Compactness (PP)  0.31  ████░░░░   0.28  ███░░░░   +0.03  ▲ better        │
│  Balance (max dev) 3.2%  ████░░░░   4.1%  █████░░   -0.9%  ▼ worse         │
│  County splits       8   ██░░░░░░    12   ███░░░░   -4     ▼ worse         │
│  Contiguous         ✓  all 98       ✗  7 issues    ──     ✗ worse          │
│  VRA districts       2               2             ──     same              │
│                                                                              │
│  ─── Most-changed districts ───────────────────────────── [e] full list    │
│  District  7   ████████████░░░░  67% tracts moved                          │
│  District 23   ████████░░░░░░░░  52% tracts moved                          │
│  District 41   █████░░░░░░░░░░░  38% tracts moved                          │
│                                                                              │
│  [x] Export CSV    [m] Map diff (opens browser)    [Esc] back              │
└───────────────────────────────────────────────────────────────────────────── ┘
```

Δ column: green ▲ = better, red ▼ = worse, grey ── = neutral. `[e]` expands most-changed list.

### Verify

Designed around Legal's court-submission workflow.

```
┌─ Verify ──────────────────────────────────────────────────────────────────────┐
│  Manifest  [ path/to/manifest.json              ]  [Browse]  [Go]           │
│                                                                              │
│          ┌──────────────────────────────────────┐                           │
│          │                                      │                           │
│          │           ✓  PASS                    │                           │
│          │                                      │                           │
│          │    Jaccard similarity:  0.9987        │                           │
│          │    wa_house_seed_42  ·  WA  ·  2020  │                           │
│          │                                      │                           │
│          └──────────────────────────────────────┘                           │
│                                                                              │
│  ─── Chain of custody ──────────────────────────────────────────────────── │
│  Binary     v0.1.0   sha256: aaa…bbb   ✓  matches manifest                 │
│  METIS      5.1.0    ✓  recorded in manifest                               │
│  Adjacency  wa_adjacency_2020.adj.bin  sha256: ccc…ddd  ✓                  │
│  TIGER      tl_2020_53_tract.zip       sha256: eee…fff  ✓                  │
│  Seed       42   ✓  recorded                                               │
│                                                                              │
│  [p] Export PDF    [x] Export audit.json    [Esc] back                     │
└───────────────────────────────────────────────────────────────────────────── ┘
```

FAIL state: same box in red, followed by "Likely causes" list (METIS version diff, adjacency sha256 mismatch, missing seed).

### Doctor

Standalone screen identical to the inline pre-flight in Run, with `[r]` to launch Run pre-filled.

```
┌─ Doctor · WA · house · 2020 ──────────────────────────────────────────────────┐
│  ✓  Location registered: Washington (WA)                                     │
│  ✓  Year 2020 valid. Available: 2000, 2010, 2020                            │
│  ✓  98 districts (house, from policy)                                       │
│  ✓  Adjacency file found: wa_adjacency_2020.adj.bin                         │
│  ⚠  Granularity: 98D at tract resolution — use block_group                  │
│  ✓  Balance tolerance: 5.0% (WA house, from policy)                        │
│  ℹ  Compactness standard: RCW 44.05.090 — as compact as practicable        │
│  ℹ  Nesting: senate_contains_two_house (2:1)                               │
│                                                                              │
│  [r] Run with these settings    [Esc] back                                  │
└───────────────────────────────────────────────────────────────────────────── ┘
```

---

## Cross-Cutting Concerns

### Command palette (`:`)

Overlay from any screen. Tab-completes location codes from LocationRegistry. Arrow-key history. Escape dismisses, returns to prior context.

```
┌───────────────────────────────────────────┐
│  : run WA house 2020 seed=42▌             │
│  ─────────────────────────────────────── │
│  ► run WA house 2020 seed=42             │
│    run WA house 2020                     │
│    run WA senate 2020                    │
└───────────────────────────────────────── ┘
```

### Error display

```
│  ─── Error ──────────────────────────── [e] full log   [c] copy  │
│  Balance check failed: district 7 at 6.2% exceeds 5.0% limit     │
│  → Try --resolution block_group  (current: tract)                 │
│  → Or increase --balance-tolerance 7.0 (WA house max: ~10%)      │
```

Summary by default. `[e]` expands to raw stderr. `[c]` copies. No "TIGER", "pkl", or "gpmetis" in user-facing messages.

### proof canvas responsibility split

| Region | Renderer | Reason |
|--------|----------|--------|
| Plan browser rows | ratatui `Table` | Interactive — sortable, selectable, scrollable |
| Detail panel metric bars | proof `mini-bar` | Rich formatting, value alignment |
| Progress depth bars | proof `mini-bar` | Live-updating blocks |
| Comparison Δ bars | proof `mini-bar` | Side-by-side alignment |
| PASS/FAIL box | ratatui `Block` | Terminal colour (green/red) |
| Chain-of-custody rows | ratatui `Table` | Interactive |
| Status bar | ratatui `Paragraph` | Always present, tied to layout |
| Command palette | ratatui popup | Needs input focus |

---

## Testing Strategy

- **Unit tests:** App state transitions (screen routing, session load/save, filter logic)
- **Snapshot tests:** proof canvas output for each metric card — assert exact ASCII string for given input
- **Integration tests:** Run form → subprocess spawn → STATUS: parsing → App state update
- **Persona walkthrough tests:** Scripted key sequences for each persona's primary workflow, assert final screen state

---

## Out of Scope (v1)

- Real-time map rendering inside the TUI
- Multi-pane tiling (single active screen at a time)
- Mouse support
- Remote/SSH-specific optimisations
- `proof-canvas` extracted as standalone crate (planned for v2)

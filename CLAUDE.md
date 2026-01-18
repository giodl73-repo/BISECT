# Claude Code Assistant Guide

This document provides context and guidelines for AI assistants working on the Congressional Redistricting codebase.

**Last Updated**: January 17, 2026

## Project Overview

This is a congressional redistricting implementation using METIS recursive bisection algorithm to generate 435 districts across all 50 US states based on 2000, 2010, and 2020 Census data.

**Core Goal:** Algorithmically generate compact, population-balanced congressional districts using only geographic and demographic constraints (no gerrymandering).

## Key Technologies

- **Algorithm:** METIS graph partitioning (recursive bisection)
- **Language:** Python 3.13+
- **GIS:** GeoPandas, Shapely
- **Visualization:** Matplotlib
- **Data:** Census tract-level population, demographics, election results
- **Web:** Static HTML/JS dashboard

## Critical Files & Directories

### Configuration
- `scripts/config_2020.py` - State district counts for 2020 apportionment
- `scripts/config_2010.py` - State district counts for 2010 apportionment
- `scripts/config_2000.py` - State district counts for 2000 apportionment

### Core Algorithm
- `src/apportionment/partition/recursive_bisection.py` - Main redistricting algorithm
- `src/apportionment/partition/metis_wrapper.py` - METIS interface
- `src/apportionment/data/adjacency.py` - Tract adjacency graph generation

### Pipeline Scripts (Executable Entry Points)
- `scripts/pipeline/run_complete_redistricting.py` - **Main orchestrator** for 50-state pipeline (parallel multi-year execution)
- `scripts/pipeline/process_nation.py` - National post-processing orchestrator (9 parallel tasks)
- `scripts/pipeline/run_state_redistricting.py` - Single-state redistricting wrapper
- `scripts/pipeline/process_single_state.py` - Core single-state logic (emits STATUS messages)

### Progress Coordination (Multi-Year Parallel)
- `scripts/utils/progress_coordinator.py` - Hierarchical progress display coordinator
- `scripts/utils/terminal_utils.py` - Terminal utilities (progress bars, tree connectors, formatting)

### Analysis Scripts
- `scripts/political/` - Partisan lean analysis using 2020 election data
- `scripts/demographic/` - Demographic composition analysis
- `scripts/compactness/` - Polsby-Popper and Reock compactness scores

### Batch Files (User Entry Points)
- `run_redistricting.bat` - Main wrapper for running pipeline
- `deploy_web.bat` - Generate and open interactive dashboard
- `CANCEL.bat` - Kill all running Python processes

### Web Dashboard
- `web/dashboard.html` - Single-file interactive dashboard (HTML/CSS/JS)
- `scripts/web/generate_dashboard.py` - Bakes district data into static HTML

## Data Exclusions

**IMPORTANT:** The following are excluded from git (see `.gitignore`):
- `data/` - All census tract shapefiles, demographics, election data (~40GB)
- `outputs/` - All generated maps, CSVs, district assignments (~20GB per run)
- `*.png`, `*.jpg`, `*.pdf` - All images (except docs/)

**Never commit data or output files!**

## Project Structure

```
src/apportionment/         # Python package (library code)
├── partition/             # Core algorithms (import from scripts)
├── data/                  # Data loading utilities
└── visualization/         # Visualization helpers

scripts/                   # Executable scripts (use the library)
├── pipeline/              # Main pipeline orchestration
├── political/             # Political analysis
├── demographic/           # Demographic analysis
├── compactness/           # Compactness analysis
├── web/                   # Dashboard generation
├── figures/               # Shared figure generation
├── config_2020.py         # Configuration
└── config_2010.py         # Configuration

web/                       # Dashboard templates
├── dashboard.html         # Individual run dashboard (per-run)
└── master_dashboard.html  # Cross-run dashboard (master)

artifacts/                 # Academic outputs (LaTeX sources)
├── compile.bat            # Master compilation script
├── papers/                # Academic papers
│   ├── 01_recursive_bisection/
│   ├── 02_edge_weighted_bisection/
│   └── 03_combined_recursive_bisection/
├── presentations/         # Conference presentations
│   └── edge_weighted_bisection/
└── guides/                # Educational guides
    └── edge_weighted_bisection/

docs/                      # Documentation
├── archive/               # Historical session notes
├── enhancements/          # Enhancement tracking (INDEX.md for overview)
│   ├── completed/         # Archived completed enhancements
│   ├── active/            # In-progress and planned enhancements
│   └── templates/         # Enhancement templates
└── ENHANCEMENTS_2026.md   # Redirect to enhancements/INDEX.md
```

## Anthropic Skills

**Phase 1 Skills**: ✅ 12 skills (Enhancement, Pipeline & Testing)
**Phase 2 Skills**: ✅ 7 skills (Visualization & Documentation)
**Phase 3 Skills**: ✅ 6 skills (Research & Analysis)
**Phase 4 Skills**: ✅ 3 skills (Code Organization)
**Phase 5 Skills**: ✅ 3 skills (Editorial)

**Total: 31 skills implemented** in `.claude/skills/`

**All phases complete!** Claude Code automatically discovers and offers to use these skills when appropriate. You don't need to explicitly invoke them - Claude will suggest using a skill when your request matches its description.

**Available Skills (Phase 1 - Enhancement, Pipeline & Testing)**:
- `/enhancement-plan` - Create enhancement specifications following project patterns
- `/enhancement-implement` - Execute enhancements with todo tracking and testing
- `/enhancement-document` - Complete all documentation for finished enhancements
- `/create-skill` - Create new skills following established patterns
- `/run-redistricting` - Execute full 50-state redistricting pipeline
- `/run-analysis-only` - Regenerate analysis without redistricting
- `/pipeline-debug` - Systematically debug pipeline failures
- `/census-download` - Download census data for specific year/state
- `/adjacency-build` - Build adjacency graphs from tract data
- `/data-validate` - Validate data completeness before running pipeline
- `/run-tests` - Execute test suite with intelligent filtering and reporting
- `/debug-tests` - Systematically debug test failures with guided troubleshooting

**Available Skills (Phase 2 - Visualization & Documentation)**:
- `/create-state-map` - Generate state-level visualization maps
- `/create-national-map` - Generate national-level maps with AK/HI insets
- `/create-pedagogical-example` - Create educational algorithm examples
- `/generate-dashboard` - Generate static HTML dashboard
- `/update-docs` - Systematically review and update all documentation
- `/create-session-archive` - Archive session notes with rationale
- `/create-architecture-diagram` - Create/update Mermaid diagrams

**Available Skills (Phase 3 - Research & Analysis)**:
- `/create-presentation-figures` - Generate figures for research presentations
- `/compile-latex` - Compile LaTeX documents (papers, presentations)
- `/run-statistical-analysis` - Perform quantitative analysis of results
- `/run-experiment` - Test algorithm variants and compare results
- `/parameter-sweep` - Test algorithm with different parameter values
- `/validate-compactness` - Validate redistricting maintains/improves compactness

**Available Skills (Phase 4 - Code Organization)**:
- `/reorganize-directory-structure` - Restructure directories following best practices
- `/consolidate-scripts` - Merge duplicate or similar scripts
- `/refactor-for-pattern` - Refactor code to follow established patterns

**Available Skills (Phase 5 - Editorial)**:
- `/edit-paper` - Edit academic papers for journal submission (proofreading, condensing, copyediting)
- `/edit-presentation` - Edit conference presentations (Beamer slides, one idea per slide, time targeting)
- `/edit-guide` - Edit educational guides for general audiences (layman's guides, tutorials, explainers)

**How to use**: Simply describe what you want to do naturally. Examples:
- "I want to plan a new feature" → Claude offers `/enhancement-plan`
- "Create a new skill for editing papers" → Claude offers `/create-skill`
- "Run redistricting for 2020" → Claude offers `/run-redistricting`
- "Create a map for California" → Claude offers `/create-state-map`
- "Update the documentation" → Claude offers `/update-docs`
- "Run all tests" → Claude offers `/run-tests`
- "Why are my tests failing?" → Claude offers `/debug-tests`
- "Edit my paper for submission" → Claude offers `/edit-paper`
- "Condense my presentation to 15 minutes" → Claude offers `/edit-presentation`
- "Edit my layman's guide for readability" → Claude offers `/edit-guide`
- "Generate figures for my presentation" → Claude offers `/create-presentation-figures`
- "Compare edge-weighted vs unweighted mode" → Claude offers `/run-experiment`
- "Validate compactness improvements" → Claude offers `/validate-compactness`
- "Reorganize the data directories" → Claude offers `/reorganize-directory-structure`
- "Consolidate these similar scripts" → Claude offers `/consolidate-scripts`
- "Make this code follow project patterns" → Claude offers `/refactor-for-pattern`

**For full documentation**: See `docs/SKILLS.md`

## Enhancement Manager Tool

A web-based tool for viewing, filtering, searching, and editing enhancement files.

**Location**: `tools/enhancement_manager/`

**Features**:
- View all enhancements in filterable grid (status, search)
- Edit enhancement markdown inline with live preview
- Update status with automatic INDEX.md synchronization
- Statistics dashboard (totals, completion rate, complexity)

**Usage**:
```bash
cd tools/enhancement_manager
run.bat  # Opens http://localhost:5000 in browser
```

**Why use it**:
- Faster than opening individual markdown files
- Quick filtering by status (Completed/In Progress/Planned)
- Search across all enhancements
- Inline editing with validation
- Auto-updates INDEX.md on status changes

**Documentation**: See `tools/enhancement_manager/README.md`

## Coding Patterns & Conventions

**For comprehensive coding patterns, see `docs/CODING_PATTERNS.md`**

### Quick Reference

**Progress Reporting**: Child processes use `STATUS:position:message` protocol
```python
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0:
    print(f"STATUS:{position}:{msg}", flush=True)
```

**Key Conventions**:
- State names: lowercase with underscores (`california`, `new_york`)
- Use `Path` objects from `pathlib`, not string concatenation
- Scripts import from library: `from apportionment.partition.recursive_bisection import ...`
- Scripts import config: `from scripts.config_2020 import STATE_CONFIG_2020`

**See `docs/CODING_PATTERNS.md` for**:
- Detailed progress bar integration patterns
- Scope-based analysis pattern (state vs national)
- File naming conventions and structure
- Path handling best practices
- Testing guidelines

## Common Tasks

### Add a New Analysis Type

**Modern Approach (Scope-Based Pattern):**
Follow the scope-based pattern documented in `docs/CODING_PATTERNS.md` Section 7:
1. Create single script with `--scope state|national` parameter
2. Implement both `analyze_state()` and `visualize_national()` functions
3. Integrate into `process_single_state.py` (per-state) and `run_complete_redistricting.py` (post-processing)
4. Add tab to dashboard in `web/dashboard.html`

**See `docs/CODING_PATTERNS.md` for complete implementation template and integration guide.**

### Add a New Command-Line Parameter
1. Add to `argparse` in relevant script
2. Pass through pipeline hierarchy if needed
3. Update batch files if user-facing

### Update Dashboard
1. Edit `web/dashboard.html` (single template file)
2. Run `deploy_web.bat` to regenerate with data
3. Output goes to `outputs/us_{year}_{version}/index.html`

## Enhancement Workflow

**For detailed enhancement workflow, see [`docs/ENHANCEMENT_WORKFLOW.md`](docs/ENHANCEMENT_WORKFLOW.md)**

Quick summary of the 6-phase process:
1. **Research**: Review docs, archives, dependencies
2. **Planning**: Create enhancement spec, break into phases
3. **Implementation**: Use TodoWrite, follow patterns, test incrementally
4. **Testing**: Print-only → small states → all years → full pipeline
5. **Documentation**: Update all affected docs (CRITICAL!)
6. **Completion**: Git commit, archive session notes

See the full document for detailed guidance on each phase, common patterns, pitfalls, and examples.

## Testing & Running

### Run Full Pipeline
```bash
run_redistricting.bat --year 2020 --version v1 --dpi 150
```

### Generate Dashboard
```bash
deploy_web.bat --year 2020 --version v1
```

### Print-Only Mode (Dry Run)
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --print-only
```

## Important Notes

### Git Workflow
- Fresh repo created Jan 2026 with clean history
- Old repo with data files archived in `.git.old/`
- Always verify no data/outputs files staged before commit

### Performance
- 50-state full run: ~2-4 hours (parallel mode)
- Single state: 30 seconds - 5 minutes depending on size
- Dashboard generation: ~5 seconds

### Windows-Specific

**IMPORTANT: This project runs on Windows** (as indicated in `<env>Platform: win32</env>`)

**Console Output Encoding:**
- Windows cmd/PowerShell uses CP1252 encoding, NOT UTF-8
- **NEVER use Unicode characters** (✓, ✗, →, •, etc.) in print() statements
- **ALWAYS use ASCII alternatives**: `[OK]`, `[FAIL]`, `[WARN]`, `->`, `-`, etc.
- Violation causes: `UnicodeEncodeError: 'charmap' codec can't encode character`
- This applies to ALL scripts that print to console

**ASCII Alternatives for Common Symbols:**
```python
# DON'T use Unicode:
print(f"✓ Success")      # Crashes on Windows
print(f"✗ Failed")       # Crashes on Windows
print(f"→ Next step")    # Crashes on Windows

# DO use ASCII:
print(f"[OK] Success")   # Works everywhere
print(f"[FAIL] Failed")  # Works everywhere
print(f"-> Next step")   # Works everywhere
```

**Other Windows Specifics:**
- METIS binary: `bin/gpmetis.exe` (Windows build)
- Batch files use Windows-style paths with backslashes
- Line endings: CRLF (Windows) - Git auto-converts
- File paths: Use `Path` objects from `pathlib` for cross-platform compatibility

### Algorithm Constraints
- Equal population: Districts within ±0.5% of target population
- Contiguity: All districts must be geographically contiguous
- Compactness: Optimized via METIS edge-cut minimization
- No political/racial considerations (purely algorithmic)

## Documentation Files - When to Read What

### Start Here
- **`CLAUDE.md`** (this file) - AI assistant guide, project overview, quick reference

### Understand the System
- **`README.md`** - User-facing project description, setup instructions, usage examples
  - Read when: Understanding what the project does, how users interact with it

- **`docs/ARCHITECTURE.md`** - System design, data flow, component relationships, technical decisions
  - Read when: Understanding how components interact, modifying core architecture, adding major features

### Development Guidelines
- **`docs/CODING_PATTERNS.md`** - Detailed coding conventions, naming patterns, progress reporting protocol
  - Read when: Writing new code, ensuring consistency with existing patterns

- **`docs/CONTRIBUTING.md`** - Development workflow, git practices, code review guidelines
  - Read when: Making contributions, understanding development process

### Data & Setup
- **`docs/DATA_FORMATS.md`** - File formats, CSV schemas, data structures, column definitions
  - Read when: Working with data files, understanding input/output formats

- **`docs/DEPENDENCIES.md`** - Required packages, installation instructions, environment setup
  - Read when: Setting up development environment, debugging dependency issues

### History & Changes
- **`docs/CHANGELOG.md`** - Version history, feature additions, bug fixes
  - Read when: Understanding what changed between versions, tracking feature history

- **`docs/archive/`** - Historical session notes from previous Claude conversations
  - Read when: Understanding why specific decisions were made, detailed implementation history

### Quick Decision Tree

**User asks about setup/installation?** → Read `README.md` + `docs/DEPENDENCIES.md`

**Need to understand how something works?** → Read `docs/ARCHITECTURE.md`

**Writing new code?** → Read `docs/CODING_PATTERNS.md`

**Working with data files?** → Read `docs/DATA_FORMATS.md`

**Understanding recent changes?** → Read `docs/CHANGELOG.md`

**Making modifications/contributions?** → Read `docs/CONTRIBUTING.md`

**General orientation/quick reference?** → Read `CLAUDE.md` (this file)

## Common Pitfalls

1. **Don't commit data files** - Always excluded via .gitignore
2. **Config imports** - Use `from scripts.config_2020 import ...` (not from root)
3. **Dashboard paths** - Use `getBasePath()` which returns `.` for relative paths
4. **Progress bars** - Child processes must use STATUS protocol, not direct tqdm
5. **Line endings** - Git auto-converts LF to CRLF on Windows
6. **State names** - Always lowercase with underscores in code

## Recent Major Changes (Jan 2026)

- **Parallel Multi-Year Pipeline with Hierarchical Progress** (Enhancement 37 - Jan 17, 2026): Complete parallel execution across 3 census years (2020, 2010, 2000) with real-time hierarchical progress visualization showing year-level progress bars and worker-level status; STATUS message protocol for parent-child process communication; `.states_complete` marker files enable fast iteration (skip state processing, rerun national post-processing in minutes instead of hours); parallel national post-processing (each year launches 9 tasks immediately after completing states, no waiting); seamless worker transition from state → national tasks; changed defaults to `--year all` (multi-year by default) and `--workers 12`; `--skip-states` flag properly works in multi-year mode; eliminated all idle time and bottlenecks; clean in-place updates with ANSI escape codes; expected 60-70% time reduction (7-13 hours → 2-4 hours)
- **Test Execution & Debugging Skills** (Enhancement 34 - Jan 16, 2026): Created `/run-tests` and `/debug-tests` skills for intelligent test execution and systematic debugging; automatic detection of 6 common failure patterns (imports, mocks, assertions, Playwright, etc.); guided troubleshooting with specific fix suggestions; 50-70% reduction in debugging time; total skills: 29 → 31
- **Test Suite Complete** (Enhancements 30, 31, 33 - Jan 16, 2026): Comprehensive test coverage with 151 tests (110 unit, 21 integration, 20 E2E dashboard) running in ~18 seconds; mock data generators for all pipeline stages; artifact validation tests catch pipeline failures; 90%+ code coverage across all components; fully automated E2E dashboard testing with mock runs
- **Artifacts Directory Organization** (Enhancement 29, Jan 16, 2026): Reorganized papers, presentations, and guides into top-level artifacts/ directory; implemented master artifacts/compile.bat with --reset and --skip-figures flag threading; fixed all visualization \\n literal text issues; removed yellow stats boxes; reduced district label font sizes; added master dashboard Artifacts tab with PDF viewer
- **Algorithm Formalization & Figure Quality** (Jan 15, 2026): Formalized recursive bisection algorithm with RBA notation and mathematical set theory; increased all figure fonts for better readability; improved boundary label logic and removed redundant graph labels when both panels shown
- **Create-Skill Meta-Skill** (Enhancement 19): Automated skill creation following established patterns, reducing creation time from 30-60 minutes to 5-10 minutes (80-90% time savings)
- **Edit-Paper Skill** (Enhancement 20): Journal editor for academic papers - proofread, condense, copyedit, page targeting for submission
- **Edit-Presentation Skill** (Enhancement 21): Conference editor for Beamer presentations - one idea per slide, time targeting, visual clarity
- **Phase 5 (Editorial) Skills**: New skill category for academic writing and presentation editing
- **Scope-Based Analysis Pattern**: Unified per-state and national analysis into single scripts
- **Parallel Pipeline Integration**: Analysis now runs per-state (parallel), not batch (sequential)
- **Performance Optimization**: Eliminated 300+ minute sequential bottleneck
- **Directory Unification** (Enhancement 13): Merged year-specific paths into single directory structure
- **Pipeline Validation Framework** (Enhancement 14): Comprehensive output validation across all stages
- **Multi-Year Support** (Enhancement 15): Full 2000, 2010, 2020 census pipeline support
- **Artifact Naming Standardization** (Enhancement 17): Clean, consistent naming conventions across all outputs
- **Figure Quality Improvement** (Enhancement 18): Enhanced real census tract examples with strict validation (0.5% ratio tolerance, 0.25 Polsby-Popper compactness, 25 retry attempts)
- Added `--reset` flag for fresh runs, `--skip-analysis` for legacy batch mode
- Integrated political and demographic national maps into post-processing
- Fixed parameter threading for census year vs election year
- Added compactness visualization pipeline (Polsby-Popper, Reock)
- Created static dashboard generator with district data baking
- Fresh git repo (removed 240MB of data from history)

## Future Enhancements

See `docs/enhancements/INDEX.md` for detailed specifications of all enhancements:

**Completed:**
- ✅ Enhancement 1: Compactness Integration (Jan 10, 2026)
- ✅ Enhancement 2: D/R Seat Totals (Jan 11, 2026)
- ✅ Enhancement 3: National Maps (Jan 11, 2026)
- ✅ Enhancement 4: Urban Metro Area Maps (Jan 12, 2026)
- ✅ Enhancement 5: National Round Progression Maps (Jan 12, 2026)
- ✅ Enhancement 6: System Architecture Diagrams (Jan 12, 2026)
- ✅ Enhancement 7: Edge-Weighted Recursive Bisection (Jan 12, 2026)
- ✅ Enhancement 9: Parallel Per-State Analysis Integration (Jan 12, 2026)
- ✅ Enhancement 13: Directory Unification (Jan 14, 2026)
- ✅ Enhancement 14: Pipeline Output Validation Framework (Jan 14, 2026)
- ✅ Enhancement 15: Multi-Year Pipeline Support (Jan 14, 2026)
- ✅ Enhancement 17: Artifact Naming Standardization (Jan 14, 2026)
- ✅ Enhancement 18: Figure Quality Improvement (Jan 15, 2026)
- ✅ Enhancement 29: Artifacts Dashboard Tab (Jan 16, 2026)
- ✅ Enhancement 37: Parallel Multi-Year Pipeline (Jan 17, 2026)

**In Progress:**
- 🔄 Enhancement 8: Block-Level Data Support (Phase 0 Complete for 2010, Partial for 2000)

**Planned:**
- 📋 Enhancement 10: Per-State Urban Area Processing
- 📋 Enhancement 16: 2000 Census Metro Area Maps

For implementation details, timelines, technical specifications, and status updates, refer to the comprehensive enhancement document.

## Questions or Issues?

Check `docs/archive/` for historical session notes and implementation details from previous development sessions.

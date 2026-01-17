# Claude Code Assistant Guide

This document provides context and guidelines for AI assistants working on the Congressional Redistricting codebase.

**Last Updated**: January 16, 2026

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
- `scripts/pipeline/run_complete_redistricting.py` - **Main orchestrator** for 50-state pipeline
- `scripts/pipeline/run_state_redistricting.py` - Single-state redistricting wrapper
- `scripts/pipeline/process_single_state.py` - Core single-state logic

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

**Phase 1 Skills**: ✅ 10 skills (Enhancement & Pipeline)
**Phase 2 Skills**: ✅ 7 skills (Visualization & Documentation)
**Phase 3 Skills**: ✅ 6 skills (Research & Analysis)
**Phase 4 Skills**: ✅ 3 skills (Code Organization)
**Phase 5 Skills**: ✅ 3 skills (Editorial)

**Total: 29 skills implemented** in `.claude/skills/`

**All phases complete!** Claude Code automatically discovers and offers to use these skills when appropriate. You don't need to explicitly invoke them - Claude will suggest using a skill when your request matches its description.

**Available Skills (Phase 1 - Enhancement & Pipeline)**:
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

When implementing enhancements to the codebase, follow this systematic process. This workflow has been refined through 13+ enhancements and captures critical learnings from past implementations.

### Standard Enhancement Structure

Every enhancement follows this template (see `docs/enhancements/templates/enhancement_template.md`):
- **Current State**: What exists today
- **Goal**: What we want to achieve
- **Implementation Plan**: Phases with specific tasks
- **Files to Modify/Create**: Explicit file list
- **Testing Plan**: How to validate
- **Benefits**: Why we're doing this
- **Success Criteria**: How we know it's done
- **Estimated Complexity**: Time/effort estimate

### 1. Research Phase

**Review Documentation:**
- Read `docs/ARCHITECTURE.md` for system design and data flow
- Read `docs/CODING_PATTERNS.md` for implementation patterns
- Read `docs/DATA_FORMATS.md` if working with data files
- Review `docs/enhancements/INDEX.md` for related enhancements

**Review Enhancement Archives:**
- Check `docs/archive/` for related enhancement session notes
- Look for similar implementations using `Grep` tool
- Understand what worked and what didn't in past attempts

**Understand Dependencies:**
- Check for data format differences between census years (2000, 2010, 2020)
- Identify conditional paths (what if data doesn't exist?)
- Consider multi-year path compatibility during transitions
- Review any existing test cases or validation scripts

**Critical Research Questions:**
- Does this enhancement require data availability checks?
- Are there year-specific field names or formats?
- Will this work for all census years or just some?
- Are there Windows-specific considerations (Unicode, paths)?

### 2. Planning Phase

**Create Enhancement Specification:**
- Create new file in `docs/enhancements/active/` using the template
- Define clear phases (typically 3-6 phases)
- List all files to modify/create explicitly
- Estimate complexity and time

**Plan Implementation Phases:**
- **Phase 0**: Data preparation (if needed)
- **Phase 1**: Core implementation
- **Phase 2**: Integration with pipeline
- **Phase 3**: Configuration and CLI options
- **Phase 4**: Testing and validation
- **Phase 5**: Documentation updates

**Identify Test Matrix:**
- Which states to test? (Small: VT/DE/WY, Medium: AL, Large: CA/TX)
- Which years to test? (2000, 2010, 2020)
- Which modes to test? (if applicable)
- Print-only first, then real runs

**Consider Backward Compatibility:**
- Will existing workflows break?
- Can we support both old and new patterns during transition?
- Do we need migration scripts?

### 3. Implementation Phase

**Use TodoWrite Tool:**
- Track progress through phases
- Mark tasks as in_progress → completed
- Only one task in_progress at a time

**Follow Coding Patterns:**
- See `docs/CODING_PATTERNS.md` for detailed patterns
- Use ASCII characters for console output (Windows compatibility)
- Support both flat and year-specific paths during transitions
- Add informative skip messages when data unavailable

**Key Implementation Practices:**
- Make changes incrementally, one component at a time
- Prefer safe, manual edits for critical changes
- Use `Path` objects from `pathlib`, not string concatenation
- Keep config imports conditional even if data paths are unified
- Add print-only/dry-run support to new scripts

**Progress Bar Integration:**
- Use `TQDM_POSITION` environment variable
- Send STATUS messages: `print(f"STATUS:{position}:{msg}", flush=True)`
- `leave=True` for parent bars, `leave=False` for child bars
- Tree-style formatting with Unicode box-drawing

**Scope-Based Pattern (for analysis scripts):**
```python
# Single script handles both state and national scopes
parser.add_argument('--scope', choices=['state', 'national'], default='state')

if args.scope == 'state':
    # Per-state processing (runs in parallel)
    process_single_state(args)
elif args.scope == 'national':
    # National aggregation (runs in post-processing)
    aggregate_all_states(args)
```

### 4. Testing Phase

**Test in This Order:**

1. **Print-Only Mode First** (catches parameter threading issues fast)
   ```bash
   python script.py --print-only --year 2020 --version test
   ```

2. **Small States** (quick validation, 30 seconds - 2 minutes)
   ```bash
   python script.py --state VT --year 2020 --version test
   python script.py --state DE --year 2010 --version test
   ```

3. **All Supported Census Years** (ensure multi-year compatibility)
   ```bash
   # Test 2020, 2010, 2000 if enhancement affects all years
   ```

4. **Quantitative Validation** (if applicable)
   - Compare metrics before/after (e.g., compactness scores)
   - Document improvements with specific numbers
   - Test statistical significance if needed

5. **Connectivity Validation** (for graph changes)
   ```bash
   python scripts/data/geography/check_graph_connectivity.py --year 2020
   ```

6. **Full Pipeline Test** (spot-check, not required for every enhancement)
   ```bash
   run_redistricting.bat --year 2020 --version test --states "VT,DE,WY"
   ```

**Common Testing Pitfalls to Avoid:**
- Don't skip print-only testing - it catches issues fast
- Don't run 50-state pipeline to test changes - use 1-2 small states first
- Don't change defaults without quantitative validation
- Don't assume GEOID fields are strings (they may parse as integers)

**User Validation:**
- Joint validation of results with user
- User confirms metrics look reasonable
- User verifies outputs match expectations

### 5. Documentation Phase

**CRITICAL**: Always update documentation after completing enhancement. This is non-negotiable.

**Required Documentation Updates:**

1. **Enhancement file** (in `docs/enhancements/active/` or `completed/`)
   - Mark enhancement as complete with date
   - Add "Completion Date" and "Implementation Summary"
   - Document quantitative results if applicable
   - Update status from 📋 PLANNED → ✅ COMPLETED
   - Move file from `active/` to `completed/` if applicable
   - Update `docs/enhancements/INDEX.md`

2. **`docs/ARCHITECTURE.md`** (if applicable)
   - Update system design diagrams
   - Update data flow descriptions
   - Update code examples with new patterns
   - Add new component descriptions

3. **`docs/CODING_PATTERNS.md`** (if applicable)
   - Document new patterns introduced
   - Update existing pattern examples
   - Add "Don't Do This" anti-patterns

4. **`docs/DATA_FORMATS.md`** (if applicable)
   - Update file format specifications
   - Update directory structure diagrams
   - Update field name references
   - Document year-specific differences

5. **`CLAUDE.md`** (this file)
   - Update "Recent Major Changes" section
   - Update project structure if changed
   - Update quick reference examples
   - Add to "Common Pitfalls" if lessons learned

6. **`docs/CHANGELOG.md`**
   - Add entry with date, description, files changed
   - Include quantitative improvements if applicable

7. **Enhancement Status Document** (if created)
   - Mark all phases complete
   - Update completion summary
   - Document final statistics

**Documentation Update Checklist:**
- [ ] Update code examples to reflect new patterns
- [ ] Update file path references if structure changed
- [ ] Update command examples if CLI changed
- [ ] Update directory structure diagrams if applicable
- [ ] Add new sections for new features/capabilities
- [ ] Mark enhancement as complete in tracking documents
- [ ] Document quantitative before/after metrics
- [ ] Update "Last Updated" dates in modified docs

### 6. Completion Phase

**Final Review:**
- Review all modified files one final time
- Verify no data/output files are staged for commit
- Run `git status` to check staging area
- Ensure `.gitignore` patterns cover new files

**Git Commit:**
- Create clear commit message describing the enhancement
- Reference enhancement number if applicable
- List key files modified

**Archive Session Notes:**
- Create dated archive file in `docs/archive/` for significant sessions
- Format: `YYYY-MM-DD_enhancement_N_description.md`
- Include rationale for decisions, not just what changed

**Consider Default Changes:**
- Only change defaults after overwhelming evidence
- Document the evidence (e.g., "52.8% improvement in Polsby-Popper")
- Ensure non-default modes remain accessible

### Common Patterns from Past Enhancements

**Enhancement #1 (Compactness Integration) - Jan 10, 2026:**
- Integrated existing standalone functionality into main pipeline
- Added new columns to district_summary.csv
- No new scripts, just integration into existing workflow

**Enhancement #2 (D/R Seat Totals) - Jan 11, 2026:**
- Simple text annotation to existing visualizations
- Quick win: 20 minutes, high readability impact

**Enhancement #3 (National Maps) - Jan 11, 2026:**
- Created national-scope visualizations from per-state data
- Handled Alaska/Hawaii as insets
- Medium complexity: 2 hours

**Enhancement #4 (Metro Area Maps) - Jan 12, 2026:**
- Downloaded external boundary data (Census CBSA)
- Generated focused maps for top 20 MSAs
- Organized by state for dashboard integration

**Enhancement #5 (National Round Progression) - Jan 12, 2026:**
- Visualized recursive bisection at national scale
- Aggregated round data from all 50 states
- Handled states completing at different rounds

**Enhancement #6 (Architecture Diagrams) - Jan 12, 2026:**
- Visual documentation using Mermaid
- Embedded in ARCHITECTURE.md for easy viewing
- Critical for understanding complex system

**Enhancement #7 (Edge-Weighted Bisection) - Jan 12, 2026:**
- Validated with single-state test (Alabama) first
- Documented quantitative results: 52.8% PP improvement, 22.2% perimeter reduction
- Changed defaults only after overwhelming evidence
- Renamed misleading options (normal → unweighted)
- Output directory naming reflects mode
- Edge weights must be integers (scaled to centimeters)
- Water adjacencies use median land boundary length
- Point adjacencies get minimal weight (0.1m)

**Enhancement #8 (Historical Tract Data) - Jan 13, 2026:**
- Phase 0: Tract aggregation before block-level work
- Created year-specific parsing scripts for different formats
- 2010 complete via direct Census API
- 2000 requires manual NHGIS download (external dependency)
- Built conditional skip logic for missing data
- Prioritized getting one year working fully before next

**Enhancement #9 (Per-State Analysis Refactoring) - Jan 12, 2026:**
- Scope-based analysis pattern (--scope state|national)
- Per-state analysis runs in parallel during pipeline
- National visualization runs in post-processing
- Eliminated 300-minute sequential bottleneck
- Progress reporting protocol for coordinated output
- Explicit --state parameter (no path parsing)

**Enhancement #13 (Directory Unification) - Jan 14, 2026:**
- Moved files first, then updated scripts, then docs
- Used manual editing for safety on critical path changes
- Created UNIFICATION_STATUS.md to track 4-phase progress
- Preserved intentional conditionals (config imports)
- Removed ~80 lines of year-specific path logic
- Tested all three census years after changes

### Technical Challenges & Solutions (From Archives)

**Data Format Issues:**
- **Unicode crashes on Windows** → Use ASCII ([OK]/[X]) not checkmarks (✓/✗)
- **GEOID type mismatches** → Force string type when reading CSVs: `dtype={'GEOID': str}`
- **Fixed-width census parsing** → Document exact character positions, test carefully
- **Field names vary by year** → Research before implementing (GEOID vs GEOID10 vs CTIDFP00)

**METIS Integration:**
- **Requires integer edge weights** → Scale floats: `int(meters * 100)` for centimeters
- **CSR format codes** → 000 (unweighted), 011 (edge-weighted)
- **Point adjacencies** → Assign minimal weight (0.1m), not zero

**Graph Construction:**
- **Must explicitly add ALL nodes** → Isolated nodes won't be added by edges alone
  ```python
  # CRITICAL: Add nodes first
  for i in range(num_nodes):
      graph.add_node(i)  # Don't skip this!
  # Then add edges
  for i, neighbors in enumerate(adjacency):
      for j in neighbors:
          graph.add_edge(i, j)
  ```
- **Water adjacencies need heuristic** → Use median land boundary length
- **Connectivity must be validated** → Check single connected component before METIS

**Path Resolution:**
- **Use absolute paths for subprocesses** → `Path(__file__).parent`, `sys.executable`
- **Support both structures during transitions** → Check new path, fall back to old
- **Year parameter must cascade** → Config loading, file paths, output directories

**Performance Optimization:**
- **Computation time remains reasonable** → Edge-weighted only ~2x slower than uniform
- **Block-level is 10-100x slower** → Use tract aggregation for routine work
- **Parallel processing** → Build adjacency graphs by county, merge

### Key Learnings

1. **Always update documentation** - Future assistants need context, not just code
2. **Test incrementally** - Print-only → small state → all years → full run
3. **Follow existing patterns** - Consistency reduces cognitive load
4. **Manual is safer for critical paths** - Batch operations risk unexpected changes
5. **User validation is essential** - Metrics can look right but be wrong
6. **Print-only mode FIRST** - Catches parameter threading before expensive runs
7. **Single-state test THEN 50-state** - Alabama/Delaware good quick tests
8. **Connectivity validation is CRITICAL** - Must form single component before METIS
9. **Document quantitative improvements** - "52.8% better" > "much better"
10. **Support graceful degradation** - Missing optional data shouldn't crash pipeline
11. **Windows-specific testing matters** - Unicode, paths, line endings differ
12. **Multi-year compatibility requires research** - Field names, formats, APIs differ
13. **Keep intentional conditionals** - Config imports are conditional for good reason
14. **Archive significant decisions** - Future you needs to know WHY, not just WHAT

### What To Do (Examples)

```python
# ✅ DO: Check data availability before adding analysis
election_data_available = (args.year == '2020' and election_data_file.exists())
if election_data_available:
    steps.append(('Political Analysis', f'{sys.executable} scripts/political/analyze_districts.py ...'))
else:
    print(f"  [SKIP] Political analysis (no 2020 election data for {args.year})")

# ✅ DO: Support both paths during transitions
graph_file_new = Path(f'data/adjacency/{year}/{state}_adjacency_{year}.pkl')
graph_file_old = Path(f'data/adjacency/{state}_adjacency_{year}.pkl')
graph_file = graph_file_new if graph_file_new.exists() else graph_file_old

# ✅ DO: Use ASCII for console output
print(f"[OK] Using edge-weighted mode")  # Not: print(f"✓ Using edge-weighted mode")

# ✅ DO: Dynamic config loading
if args.year == '2010':
    from scripts.config_2010 import STATE_CONFIG_2010 as STATE_CONFIG
elif args.year == '2020':
    from scripts.config_2020 import STATE_CONFIG_2020 as STATE_CONFIG

# ✅ DO: Explicitly add all nodes including isolated ones
graph = nx.Graph()
for i in range(num_nodes):
    graph.add_node(i)  # CRITICAL: isolated nodes need explicit add
for i, neighbors in enumerate(adjacency):
    for j in neighbors:
        graph.add_edge(i, j)

# ✅ DO: Force GEOID as string type
tracts = pd.read_csv(file, dtype={'GEOID': str})
```

### What NOT To Do (Anti-Patterns)

```python
# ❌ DON'T: Hardcode census year
tracts_file = f'data/raw/{state}_tracts_2020.parquet'  # Wrong
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'  # Right

# ❌ DON'T: Fail on missing optional data
if not election_data_file.exists():
    raise FileNotFoundError("Election data not found")  # Wrong - crashes 2010

# ❌ DON'T: Rely on edges to add nodes (isolated nodes never added!)
for i, neighbors in enumerate(adjacency):
    for j in neighbors:
        graph.add_edge(i, j)  # Missing: graph.add_node(i) first

# ❌ DON'T: Use Unicode in console output
print(f"✓ Complete")  # Crashes on Windows
print(f"[OK] Complete")  # Works everywhere

# ❌ DON'T: Assume field names are consistent across years
df = pd.read_csv(file)
geoid = df['GEOID']  # Might be GEOID10 or CTIDFP00 in other years
```

**Additional DON'Ts:**
- Don't skip print-only testing
- Don't run 50-state pipeline to test changes
- Don't change defaults without quantitative validation
- Don't mix tabs and spaces in fixed-width parsing
- Don't assume GEOID fields are always strings

### Risk Mitigation

**Low-Risk Approach:**
- Phased implementation with testing between phases
- Keep old scripts until validated (mark deprecated)
- Use test version flag (`--version test`) to avoid overwriting production
- Mark new features as non-critical in pipeline
- Full rollback capability via git

**Validation Before Deletion:**
- Test old and new side-by-side
- Compare outputs byte-for-byte if possible
- Get user sign-off before deleting old implementation

### Complexity Estimates

Based on past enhancements:
- **Low** (20-60 min): Simple integrations, text annotations, small refactors
- **Medium** (2-4 hours): New visualizations, modest refactoring, single-file additions
- **Medium-High** (4-8 hours): Multi-file refactoring, new analysis types, scope-based patterns
- **High** (8-15 hours): New algorithms, major architectural changes, performance optimization
- **Very High** (15+ hours): Multi-year data, block-level support, complex optimizations

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

**In Progress:**
- 🔄 Enhancement 8: Block-Level Data Support (Phase 0 Complete for 2010, Partial for 2000)

**Planned:**
- 📋 Enhancement 10: Per-State Urban Area Processing
- 📋 Enhancement 16: 2000 Census Metro Area Maps

For implementation details, timelines, technical specifications, and status updates, refer to the comprehensive enhancement document.

## Questions or Issues?

Check `docs/archive/` for historical session notes and implementation details from previous development sessions.

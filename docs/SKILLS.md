# Claude Code Skills for Congressional Redistricting Project

**Last Updated**: January 15, 2026

This document defines reusable skills that AI assistants can use to work effectively on the Congressional Redistricting codebase. Skills encapsulate common workflows, patterns, and best practices learned through 18+ enhancements.

## What Are Skills?

Skills are specialized, reusable capabilities that:
- Encapsulate domain knowledge and best practices
- Follow established coding patterns consistently
- Include validation and error handling
- Are invoked via `/skill-name` commands
- Can be composed together for complex tasks

## Implementation Status

**Phase 1 Skills**: ✅ IMPLEMENTED (12 skills - January 16, 2026)
**Phase 2 Skills**: ✅ IMPLEMENTED (7 skills - January 15, 2026)
**Phase 3 Skills**: ✅ IMPLEMENTED (6 skills - January 15, 2026)
**Phase 4 Skills**: ✅ IMPLEMENTED (3 skills - January 15, 2026)
**Phase 5 Skills**: ✅ IMPLEMENTED (3 skills - January 15, 2026)

**Total: 31 skills implemented** as formal Anthropic MCP Skills in `.claude/skills/`

**All phases complete!** Claude Code automatically discovers these skills at startup and offers to use them when your requests match their descriptions.

**Implemented skills** (Phase 1 - High Priority - Enhancement, Pipeline & Testing):
- `/enhancement-plan` - Create enhancement specifications
- `/enhancement-implement` - Execute enhancements with todo tracking
- `/enhancement-document` - Complete documentation for finished enhancements
- `/create-skill` - Create new skills following established patterns
- `/run-redistricting` - Execute full 50-state redistricting pipeline
- `/run-analysis-only` - Run analysis without redistricting
- `/pipeline-debug` - Debug pipeline failures systematically
- `/census-download` - Download census data for specific year/state
- `/adjacency-build` - Build adjacency graphs from tract data
- `/data-validate` - Validate data completeness and quality
- `/run-tests` - Execute test suite with intelligent filtering and reporting
- `/debug-tests` - Systematically debug test failures with guided troubleshooting

**Implemented skills** (Phase 2 - Visualization & Documentation):
- `/create-state-map` - Generate state-level visualization maps
- `/create-national-map` - Generate national-level maps with AK/HI insets
- `/create-pedagogical-example` - Create educational algorithm examples
- `/generate-dashboard` - Generate static HTML dashboard
- `/update-docs` - Systematically review and update all documentation
- `/create-session-archive` - Archive session notes with rationale
- `/create-architecture-diagram` - Create/update Mermaid diagrams

**Implemented skills** (Phase 3 - Research & Analysis):
- `/create-presentation-figures` - Generate figures for research presentations
- `/compile-latex` - Compile LaTeX documents (papers, presentations)
- `/run-statistical-analysis` - Perform quantitative analysis of results
- `/run-experiment` - Test algorithm variants and compare results
- `/parameter-sweep` - Test algorithm with different parameter values
- `/validate-compactness` - Validate redistricting maintains/improves compactness

**Implemented skills** (Phase 4 - Code Organization):
- `/reorganize-directory-structure` - Restructure directories following best practices
- `/consolidate-scripts` - Merge duplicate or similar scripts
- `/refactor-for-pattern` - Refactor code to follow established patterns

**Implemented skills** (Phase 5 - Editorial):
- `/edit-paper` - Edit academic papers for journal submission
- `/edit-presentation` - Edit conference presentations (Beamer slides)
- `/edit-guide` - Edit educational guides for general audiences

### Using Skills

**You don't type skill names** - Claude automatically offers to use skills when appropriate.

**Examples**:
- Say: "I want to plan a new feature" → Claude offers to use `/enhancement-plan`
- Say: "Run redistricting for 2020" → Claude offers to use `/run-redistricting`
- Say: "Create a map for California" → Claude offers to use `/create-state-map`
- Say: "Update the documentation" → Claude offers to use `/update-docs`

**To see available skills**: Ask Claude "What skills are available?"

**Implementation details**: Each skill is a `SKILL.md` file in `.claude/skills/` with YAML frontmatter + markdown instructions. Claude reads the description to determine when to activate each skill.

## Skill Categories

1. [Enhancement Workflow Skills](#enhancement-workflow-skills)
2. [Data Management Skills](#data-management-skills)
3. [Pipeline Execution Skills](#pipeline-execution-skills)
4. [Testing & Validation Skills](#testing--validation-skills)
5. [Visualization Skills](#visualization-skills)
6. [Documentation Skills](#documentation-skills)
7. [Research & Academic Skills](#research--academic-skills)
8. [Experiment & Analysis Skills](#experiment--analysis-skills)
9. [Code Organization Skills](#code-organization-skills)

---

## Enhancement Workflow Skills

### `/enhancement-plan`
**Purpose**: Create a new enhancement specification following project patterns

**When to use**: User requests a new feature, improvement, or system change

**What it does**:
1. Reads `docs/CODING_PATTERNS.md` and `docs/ARCHITECTURE.md` for context
2. Reviews similar enhancements in `docs/ENHANCEMENTS_2026.md`
3. Creates enhancement specification with:
   - Current State
   - Goal
   - Implementation Plan (phases)
   - Files to Modify/Create
   - Testing Plan
   - Benefits
   - Success Criteria
   - Estimated Complexity
4. Adds to `docs/ENHANCEMENTS_2026.md` with status: 📋 PLANNED
5. Gets user approval before proceeding

**Inputs**:
- Enhancement description
- Affected components/systems

**Outputs**:
- Enhancement entry in ENHANCEMENTS_2026.md
- Implementation plan with phases

---

### `/enhancement-implement`
**Purpose**: Execute an enhancement following the standard workflow

**Prerequisites**: Enhancement must be planned (via `/enhancement-plan`)

**What it does**:
1. Creates TodoWrite task list from enhancement phases
2. Follows implementation phases sequentially:
   - Phase 1: Core implementation
   - Phase 2: Pipeline integration
   - Phase 3: Configuration/CLI
   - Phase 4: Testing & validation
   - Phase 5: Documentation updates
3. Uses STATUS protocol for progress reporting
4. Marks each phase complete before moving to next
5. Tests incrementally (print-only → small state → full validation)

**Follows patterns from**:
- `docs/CODING_PATTERNS.md` - Implementation patterns
- `CLAUDE.md` - Enhancement workflow section

**Outputs**:
- Modified/created files per enhancement spec
- Updated status in ENHANCEMENTS_2026.md

---

### `/enhancement-document`
**Purpose**: Complete documentation for a finished enhancement

**When to use**: After enhancement implementation and testing complete

**What it does**:
1. Updates `docs/ENHANCEMENTS_2026.md`:
   - Mark as ✅ COMPLETED with date
   - Add "Completion Date" and "Implementation Summary"
   - Document quantitative results if applicable
2. Updates `docs/CHANGELOG.md` with dated entry
3. Updates `CLAUDE.md`:
   - "Recent Major Changes" section
   - Completed enhancements list
   - Update "Last Updated" date
4. Updates `docs/ARCHITECTURE.md` if system design changed
5. Updates `docs/CODING_PATTERNS.md` if new patterns introduced
6. Updates `docs/DATA_FORMATS.md` if data formats changed

**Validation checklist**:
- [ ] All 6 doc files checked for updates
- [ ] Quantitative metrics documented
- [ ] Files modified list accurate
- [ ] Commit message drafted

---

## Data Management Skills

### `/census-download`
**Purpose**: Download census data for a specific year

**When to use**: Need tract/block data for 2000, 2010, or 2020

**What it does**:
1. Determines year-specific data source:
   - 2020: Census API (SF1)
   - 2010: Census API (SF1)
   - 2000: NHGIS (manual download required)
2. For each state:
   - Downloads tract-level data (population, demographics)
   - Downloads tract boundaries (TIGER/Line shapefiles)
   - Validates required fields (GEOID, POP100, etc.)
3. Saves to `data/tracts/{year}/{state}_tracts_{year}.parquet`
4. Creates data inventory CSV

**Handles**:
- Year-specific field names (GEOID vs GEOID10 vs CTIDFP00)
- API rate limiting
- Missing/incomplete data
- Data format conversions

**Prerequisites**: Census API key (if using API)

---

### `/adjacency-build`
**Purpose**: Build adjacency graphs for census tracts

**When to use**: After census data download, before redistricting

**What it does**:
1. Loads tract geometries for state and year
2. Builds spatial index for efficient queries
3. Identifies neighboring tracts (shared boundaries)
4. Calculates edge weights (boundary lengths in meters)
5. Validates connectivity (single connected component)
6. Saves graph to `data/adjacency/{year}/{state}_adjacency_{year}.pkl`
7. Reports statistics:
   - Number of nodes (tracts)
   - Number of edges (adjacencies)
   - Average degree
   - Connected components (should be 1)

**Handles**:
- Point adjacencies (water boundaries) - minimal weight
- Island tracts - water adjacencies to mainland
- Invalid geometries - repairs with buffer(0)

**Related**: `scripts/data/adjacency/build_adjacency_graph.py`

---

### `/data-validate`
**Purpose**: Validate data completeness for a census year

**What it does**:
1. Checks for all 50 states + DC:
   - Tract data files exist
   - Adjacency graphs exist
   - Required fields present (GEOID, population, demographics)
2. Validates data quality:
   - No missing GEOIDs
   - Population > 0 for all tracts
   - Geometries valid
   - Graphs connected
3. Reports completeness percentages per stage
4. Lists missing files

**Related**: `scripts/validation/validate_pipeline_outputs.py`

---

## Pipeline Execution Skills

### `/run-redistricting`
**Purpose**: Execute the full 50-state redistricting pipeline

**What it does**:
1. Validates prerequisites (census data, adjacency graphs)
2. Runs `scripts/pipeline/run_complete_redistricting.py` with args:
   - `--year`: Census year (2000/2010/2020)
   - `--version`: Output version tag
   - `--mode`: Edge-weighted vs unweighted
   - `--dpi`: Map resolution
3. Monitors progress via STATUS protocol
4. Handles errors gracefully (skip unavailable data)
5. Generates dashboard at completion
6. Validates outputs (optional with `--validate`)

**Execution modes**:
- `--print-only`: Dry run (parameter validation)
- `--states "VT,DE"`: Test subset of states
- `--reset`: Fresh run (delete existing outputs)
- `--force`: Regenerate all outputs

**Typical workflow**:
1. Print-only first (validate params)
2. Small state test (VT or DE)
3. Full 50-state run

---

### `/run-analysis-only`
**Purpose**: Run analysis stages without redistricting

**When to use**: Redistricting already complete, need to regenerate analysis

**What it does**:
1. Checks that district assignments exist
2. Runs analysis in parallel:
   - Political lean (if 2020 election data available)
   - Demographic composition
   - Compactness metrics
3. Runs post-processing:
   - National maps
   - Metro area maps (if CBSA data available)
   - Round progression maps
4. Regenerates dashboard

**Related**: Use `--skip-redistricting` flag

---

### `/pipeline-debug`
**Purpose**: Debug pipeline failures

**What it does**:
1. Reads error messages and tracebacks
2. Identifies failure stage:
   - Data loading
   - Graph construction
   - METIS partitioning
   - Analysis
   - Visualization
3. Checks common issues:
   - Missing data files
   - GEOID type mismatches
   - Graph connectivity failures
   - Unicode encoding errors (Windows)
4. Suggests fixes based on past issues
5. Tests fix with small state

**Common fixes**:
- Missing data → Run `/census-download` or skip analysis
- GEOID type → Force `dtype={'GEOID': str}`
- Unicode → Replace with ASCII ([OK] not ✓)
- Connectivity → Rebuild adjacency with water connections

---

## Testing & Validation Skills

### `/run-tests`
**Purpose**: Execute test suite with intelligent filtering and reporting

**When to use**:
- "Run tests"
- "Run all tests"
- "Run unit tests with coverage"
- Before committing code changes
- When validating new features

**What it does**:
1. Asks user what to run:
   - All tests (151 tests, ~18 seconds)
   - Unit tests (110 tests, ~7 seconds)
   - Integration tests (21 tests, ~3 seconds)
   - E2E dashboard tests (20 tests, ~8 seconds)
   - Specific component (redistricting, political, demographic, etc.)
2. Builds appropriate pytest command:
   - With markers: `-m redistricting`
   - With coverage: `--cov=apportionment --cov-report=html`
   - With options: `--lf` (failed first), `--tb=long` (verbose)
3. Executes tests with Bash
4. Parses results for statistics (passed/failed/skipped)
5. Reports summary with clear formatting
6. Suggests next steps based on results

**Test categories**:
- **By type**: unit, integration, e2e
- **By component**: redistricting, political, demographic, compactness, visualization, dashboard
- **By speed**: Fast (unit only) vs Full (all tests)

**Coverage reporting**:
- Generates HTML report in `htmlcov/`
- Shows line-by-line coverage
- Identifies uncovered code sections
- Typical coverage: 90%+ for most components

**Quick commands**:
```bash
pytest tests/ -v                 # All tests
pytest tests/unit/ -v            # Unit tests only
pytest tests/ -m political -v    # Political tests
pytest tests/ --cov=apportionment --cov-report=html -v  # With coverage
```

**Next steps**:
- If all pass: Ready to commit
- If some fail: Use `/debug-tests` to investigate
- If coverage low: Write additional tests

---

### `/debug-tests`
**Purpose**: Systematically debug test failures with guided troubleshooting

**When to use**:
- "Debug tests"
- "Why are my tests failing?"
- "Fix test failures"
- After `/run-tests` reports failures
- When tests fail in CI/CD

**What it does**:
1. Gathers test failure information:
   - Reads recent pytest output
   - Or offers to run tests
2. Categorizes failures by pattern:
   - **Import Errors**: ModuleNotFoundError, ImportError
   - **Mock Data Errors**: FileNotFoundError in fixtures/outputs
   - **Assertion Failures**: assert X == Y
   - **Playwright Errors**: TimeoutError, browser issues
   - **File Not Found**: Data files missing
   - **AttributeError**: API mismatches
3. Provides guided debugging steps for each category
4. Checks common issues automatically:
   - PYTHONPATH set correctly
   - Package importable
   - Mock data exists
   - Pytest plugins installed
   - Browser installed (for E2E)
5. Suggests specific fixes with commands
6. Offers targeted re-test with `--lf` or specific tests

**Common failure patterns detected**:
- **Import errors** → Check/set PYTHONPATH
- **Mock data missing** → Run `generate_mock_run.py`
- **Browser not installed** → Run `playwright install chromium`
- **Assertion failures** → Update test expectations or fix implementation
- **API mismatches** → Update mocks or test code

**Debugging workflow**:
```
1. Identify failure category
2. Check common causes automatically
3. Provide step-by-step debugging guide
4. Suggest specific fixes
5. Re-run failed tests
6. Verify fixes resolved issues
```

**Advanced debugging**:
- Interactive: `pytest --pdb` (stop on failure)
- Verbose: `pytest --tb=long` (full tracebacks)
- Visible browser: `pytest tests/e2e/ --headed` (see Playwright)
- Slow motion: `pytest tests/e2e/ --slowmo=500` (debug UI)

**Example patterns**:
```python
# Import error fix
export PYTHONPATH="${PYTHONPATH}:$(pwd)"

# Mock data fix
python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test

# Browser fix
playwright install chromium

# Re-run failed only
pytest --lf -v
```

**Benefit**: 50-70% reduction in debugging time through pattern recognition and guided troubleshooting

---

## Visualization Skills

### `/create-state-map`
**Purpose**: Generate a state-level visualization

**What it does**:
1. Loads state geography and district assignments
2. Creates map with:
   - Thin white tract boundaries
   - Thick black district boundaries
   - Color scheme (district colors, political lean, demographics, etc.)
3. Adds annotations (district numbers, totals)
4. Saves to appropriate output directory
5. Uses DPI from config

**Map types**:
- District assignment
- Political lean
- Demographic composition
- Compactness scores
- Round progression

**Related**: `scripts/visualization/` modules

---

### `/create-national-map`
**Purpose**: Generate a national-level visualization (all 50 states)

**What it does**:
1. Loads data from all 50 states
2. Creates US map with Alaska/Hawaii insets
3. Applies consistent color scheme
4. Adds national statistics
5. Saves to `outputs/us_{year}_{version}/maps/`

**Handles**:
- Alaska scaling (30% of actual size)
- Hawaii positioning
- Missing data for some states (graceful skip)

**Map types**:
- All 435 districts
- Political lean national
- Demographic composition national
- Compactness national
- Round progression (by round)

---

### `/create-pedagogical-example`
**Purpose**: Create educational example like real_tracts_examples

**When to use**: Need to demonstrate algorithm with small, clear example

**What it does**:
1. Selects small tract cluster (typically 12 tracts)
2. Runs METIS partition with target ratio
3. Validates result:
   - Ratio accuracy (within tolerance)
   - Compactness (both regions)
   - Visual clarity
4. Implements retry logic (test multiple starting locations)
5. Creates dual visualization:
   - Left: Geographic map with tract labels
   - Right: Abstract graph representation
6. Adds annotations (boundary lengths, populations, percentages)

**Key parameters**:
- `n_tracts`: Number of tracts (12 recommended for clarity)
- `target_ratio`: Population split (e.g., 1:1, 3:2, 5:6)
- `tolerance`: Ratio accuracy (0.005 = 0.5%)
- `min_compactness`: Polsby-Popper threshold (0.25)
- `max_attempts`: Retry count (26 = 25 retries)

**Related**: `presentations/edge_weighted_bisection/create_appendix_examples.py`

---

### `/generate-dashboard`
**Purpose**: Create static HTML dashboard with all visualizations

**What it does**:
1. Reads template: `web/dashboard.html`
2. Bakes in data:
   - State list with district counts
   - File paths for all maps/CSVs
   - Summary statistics
3. Generates: `outputs/us_{year}_{version}/index.html`
4. Opens in browser

**Related**: `scripts/web/generate_dashboard.py`

---

## Documentation Skills

### `/update-docs`
**Purpose**: Ensure all documentation is current

**What it does**:
1. Reviews all `docs/*.md` files
2. Checks for outdated information:
   - File paths that changed
   - Command examples that changed
   - Status updates needed
3. Updates "Last Updated" dates
4. Verifies consistency across documents

**Documents checked**:
- CLAUDE.md
- ARCHITECTURE.md
- CODING_PATTERNS.md
- DATA_FORMATS.md
- CHANGELOG.md
- ENHANCEMENTS_2026.md
- CONTRIBUTING.md

---

### `/create-session-archive`
**Purpose**: Archive detailed session notes for future reference

**When to use**: After significant work session or enhancement completion

**What it does**:
1. Creates dated file: `docs/archive/YYYY-MM-DD_enhancement_N_description.md`
2. Documents:
   - Problem statement
   - Decisions made and WHY
   - Challenges encountered
   - Solutions implemented
   - Lessons learned
3. Includes code snippets, error messages, examples
4. Cross-references related enhancements

**Focus**: Document rationale, not just changes

---

### `/create-architecture-diagram`
**Purpose**: Create or update Mermaid diagrams in ARCHITECTURE.md

**What it does**:
1. Analyzes system component relationships
2. Creates Mermaid diagram:
   - Flowcharts for data flow
   - Sequence diagrams for processes
   - Component diagrams for architecture
3. Embeds in ARCHITECTURE.md
4. Validates rendering (Mermaid syntax)

**Diagram types**:
- Pipeline flow
- Data dependencies
- Component relationships
- Analysis workflow

---

## Research & Academic Skills

### `/create-presentation-figures`
**Purpose**: Generate figures for research presentations

**What it does**:
1. Reads figure specifications from presentation directory
2. Generates visualizations:
   - Schematic diagrams (algorithm concepts)
   - Real census tract examples
   - Round progression examples
   - Statistical comparisons
3. Follows presentation style guidelines
4. Saves high-res outputs (DPI 300+)

**Related**: `presentations/edge_weighted_bisection/create_figures.py`

---

### `/compile-latex`
**Purpose**: Compile LaTeX documents (papers, presentations)

**What it does**:
1. Checks for LaTeX dependencies (\input, \include files)
2. Determines if recompilation needed (source newer than PDF)
3. Runs LaTeX compilation:
   - Multiple passes if needed
   - BibTeX if references changed
4. Handles compilation errors:
   - Extracts meaningful errors from .log
   - Points to problematic lines
5. Validates output PDF exists

**Skip logic**: PDF newer than all sources → skip compilation

**Related**: Planned compilation infrastructure (Enhancement 19)

---

### `/run-statistical-analysis`
**Purpose**: Perform quantitative analysis of redistricting results

**What it does**:
1. Loads district metrics (compactness, partisan lean, demographics)
2. Computes statistics:
   - Means, medians, standard deviations
   - Comparisons across years/modes
   - Statistical significance tests
3. Generates comparison tables
4. Creates statistical plots
5. Documents results in paper format

**Analysis types**:
- Compactness improvements
- Partisan fairness metrics
- Demographic representation
- Algorithm performance

---

## Experiment & Analysis Skills

### `/run-experiment`
**Purpose**: Test algorithm variants and compare results

**What it does**:
1. Defines experiment parameters:
   - States to test
   - Algorithm variants (edge-weighted, unweighted, etc.)
   - Metrics to track
2. Runs redistricting for each variant
3. Collects metrics:
   - Compactness (Polsby-Popper, Reock)
   - Partisan lean
   - Computation time
4. Performs statistical comparison:
   - Paired t-tests
   - Effect sizes
   - Confidence intervals
5. Generates comparison report

**Example**: Compare edge-weighted vs unweighted for 10 states

---

### `/parameter-sweep`
**Purpose**: Test algorithm with different parameter values

**What it does**:
1. Defines parameter space:
   - Edge weight scaling factors
   - Population tolerance
   - Minimum tract populations
2. Runs redistricting for each parameter combination
3. Tracks metrics vs parameters
4. Identifies optimal parameter values
5. Visualizes parameter sensitivity

**Example**: Test edge weight scales 0.1x, 0.5x, 1.0x, 2.0x, 5.0x

---

### `/validate-compactness`
**Purpose**: Validate redistricting maintains/improves compactness

**What it does**:
1. Loads district geometries
2. Computes compactness metrics:
   - Polsby-Popper: (4π × area) / perimeter²
   - Reock: area / area of minimum bounding circle
3. Compares to baselines:
   - Current congressional districts
   - Historical districts
   - Random partitions
4. Reports improvements/regressions
5. Identifies outlier districts

**Thresholds**:
- Polsby-Popper: 0.25+ considered compact
- Reock: 0.4+ considered compact

---

## Code Organization Skills

### `/reorganize-directory-structure`
**Purpose**: Restructure directories following best practices

**When to use**: Directory structure has grown organically and needs cleanup

**What it does**:
1. Analyzes current structure
2. Proposes new structure following patterns:
   - Year-specific data in `data/{type}/{year}/`
   - Analysis outputs in `outputs/{year}_{version}/{analysis_type}/`
   - Scripts organized by function
3. Creates migration plan:
   - Phase 1: Create new directories
   - Phase 2: Copy files (preserves originals)
   - Phase 3: Update all path references
   - Phase 4: Test all paths
   - Phase 5: Delete old structure
4. Updates all scripts with new paths
5. Documents changes in CHANGELOG.md

**Safety**: Never deletes files until all paths validated

**Related**: Enhancement 13 (Directory Unification)

---

### `/consolidate-scripts`
**Purpose**: Merge duplicate or similar scripts

**What it does**:
1. Identifies scripts with similar functionality
2. Analyzes differences:
   - Same logic, different parameters?
   - Different modes of same operation?
3. Proposes consolidation:
   - Single script with mode flags
   - Shared library functions
4. Implements refactoring:
   - Extract common code to library
   - Add CLI parameters for variants
   - Maintain backward compatibility during transition
5. Tests old and new side-by-side
6. Deprecates old scripts after validation

**Pattern**: Scope-based pattern (--scope state|national)

---

### `/refactor-for-pattern`
**Purpose**: Refactor code to follow established coding pattern

**When to use**: Code doesn't follow patterns in CODING_PATTERNS.md

**What it does**:
1. Reads CODING_PATTERNS.md for target pattern
2. Identifies code sections not following pattern:
   - Progress reporting
   - Path handling
   - Error handling
   - Dual output modes
3. Refactors to match pattern
4. Tests before/after equivalence
5. Documents pattern in code comments

**Common refactorings**:
- Add STATUS protocol support
- Add --scope state|national parameter
- Add skip logic for missing data
- Replace Unicode with ASCII

---

## Skill Composition Examples

Skills can be composed to accomplish complex tasks:

### New Enhancement Workflow
```
User: "I want to add block-level data support"
1. /enhancement-plan          # Create specification
2. /enhancement-implement     # Execute implementation
3. /run-experiment            # Validate improvements
4. /enhancement-document      # Complete documentation
```

### Data Update Workflow
```
User: "Update to use 2020 census data"
1. /census-download --year 2020
2. /adjacency-build --year 2020
3. /data-validate --year 2020
4. /run-redistricting --year 2020 --version v1 --states "VT,DE"
5. /run-redistricting --year 2020 --version v1  # Full 50 states
```

### Paper Preparation Workflow
```
User: "Prepare figures for paper submission"
1. /create-presentation-figures --style paper
2. /run-statistical-analysis
3. /compile-latex --document paper_main.tex
4. /validate-compactness --mode comparison
```

### Code Cleanup Workflow
```
User: "Clean up the analysis scripts directory"
1. /consolidate-scripts --directory scripts/analysis
2. /refactor-for-pattern --pattern scope-based
3. /reorganize-directory-structure --target scripts/analysis
4. /update-docs
```

---

## Skill Development Guidelines

When creating new skills:

### 1. Skill Naming
- Use kebab-case: `/my-skill-name`
- Start with verb: `/create-`, `/run-`, `/validate-`
- Be specific: `/create-state-map` not `/make-map`

### 2. Skill Structure
Every skill should document:
- **Purpose**: One-line summary
- **When to use**: Trigger conditions
- **What it does**: Step-by-step workflow
- **Inputs**: Required/optional parameters
- **Outputs**: Files/data created
- **Prerequisites**: What must exist first
- **Related**: Links to relevant code/docs

### 3. Error Handling
Skills must:
- Validate prerequisites before starting
- Handle missing data gracefully
- Report clear error messages
- Suggest fixes for common issues
- Leave system in consistent state on failure

### 4. Documentation
Skills must:
- Follow patterns in CODING_PATTERNS.md
- Update relevant documentation on completion
- Log what they did for audit trail
- Be idempotent (safe to run multiple times)

### 5. Testing
Skills should:
- Test incrementally (print-only → small → full)
- Validate outputs before declaring success
- Compare to baselines when applicable
- Report quantitative improvements

---

## Implementation Notes

### For Skill Developers

Skills are implemented as:
1. **Skill definition file** (this document): Specifies what skill does
2. **Skill implementation**: Code/prompts that execute the skill
3. **Skill registration**: Makes skill available via `/skill-name`

### For AI Assistants Using Skills

When a user invokes `/skill-name`:
1. Read the skill definition from this document
2. Follow the documented workflow step-by-step
3. Use existing tools (Read, Edit, Write, Bash, etc.)
4. Apply patterns from CODING_PATTERNS.md
5. Update documentation as specified
6. Report completion with summary

### Skill vs Direct Implementation

**Use a skill when**:
- Task matches a documented workflow exactly
- Want consistency across multiple invocations
- Task is common and will be repeated
- Need to ensure patterns are followed

**Implement directly when**:
- Task is novel or exploratory
- Skill would need heavy customization
- One-off task unlikely to repeat
- Skill doesn't exist yet

---

## Future Skill Ideas

Skills to consider implementing:

### Data Skills
- `/census-compare` - Compare data across years
- `/data-export` - Export results in various formats
- `/data-import` - Import external datasets

### Algorithm Skills
- `/algorithm-benchmark` - Performance benchmarking
- `/algorithm-profile` - Detailed profiling
- `/algorithm-visualize` - Visualize algorithm decisions

### Quality Skills
- `/code-review` - Automated code review
- `/test-coverage` - Check test coverage
- `/performance-check` - Check for regressions

### Collaboration Skills
- `/create-pr-description` - Generate PR description
- `/explain-changes` - Explain what changed and why
- `/review-enhancement` - Review enhancement spec

---

## Questions or Issues?

- **For skill usage questions**: Ask the AI assistant
- **For skill bugs**: Document in session notes, consider skill improvement
- **For new skill ideas**: Add to "Future Skill Ideas" section above
- **For skill improvements**: Update this document with learnings

**Remember**: Skills evolve. After using a skill, consider whether it worked well and document improvements.

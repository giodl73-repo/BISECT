# Claude Code Skills

**Updated**: 2026-01-17

**Status**: 31 skills implemented (all phases complete)

Skills are specialized workflows invoked via `/skill-name`. Claude Code auto-discovers them and suggests when appropriate.

## Implementation Status

✅ **Phase 1** (12 skills): Enhancement, Pipeline & Testing
✅ **Phase 2** (7 skills): Visualization & Documentation
✅ **Phase 3** (6 skills): Research & Analysis
✅ **Phase 4** (3 skills): Code Organization
✅ **Phase 5** (3 skills): Editorial

## Skills by Category

### Enhancement Workflow (3)

**`/enhancement-plan`** - Create enhancement specs (docs/enhancements/active/)
- Reviews ARCHITECTURE.md, CODING_PATTERNS.md, existing enhancements
- Breaks into phases, lists files, estimates complexity
- Identifies test requirements (unit/integration/e2e/dashboard)

**`/enhancement-implement`** - Execute enhancements with todo tracking
- Follows 6-phase workflow (research → plan → implement → test → document → complete)
- Uses TodoWrite for progress, STATUS protocol for reporting
- Tests incrementally (print-only → small state → multi-year)

**`/enhancement-document`** - Complete documentation after implementation
- Updates enhancement file, INDEX.md, CHANGELOG.md
- Runs /update-docs for consistency
- Lists tests added, documents quantitative results

### Data Management (3)

**`/census-download`** - Download tract data for year/state
- Handles year-specific sources (2000 NHGIS manual, 2010/2020 Census API)
- Downloads tracts + places, processes to parquet
- Forces GEOID string, validates completeness

**`/adjacency-build`** - Build adjacency graphs from tracts
- Detects shared boundaries (spatial indexing)
- Handles water boundaries (median length), islands (point adjacency)
- Validates connectivity (single component required for METIS)

**`/data-validate`** - Validate data completeness before pipeline
- Checks tracts, adjacency graphs, required fields for all states/years
- Reports missing data, suggests fixes
- Used before running redistricting

### Pipeline Execution (3)

**`/run-redistricting`** - Execute 50-state pipeline with validation
- Multi-year parallel (default) or single year
- Prerequisite checks (data, graphs, configs)
- Progress tracking, error recovery
- Flags: `--year {2020|2010|2000|all}`, `--version`, `--workers`, `--dpi`, `--reset`, `--skip-states`

**`/run-analysis-only`** - Run analysis without redistricting
- When district assignments exist but need updated analysis/maps
- Regenerates political/demographic/compactness analysis + maps
- Faster than full pipeline (minutes vs hours)

**`/pipeline-debug`** - Debug pipeline failures
- Analyzes error messages, checks common issues
- Suggests fixes (missing data, GEOID types, connectivity, METIS errors)
- Systematic troubleshooting workflow

### Testing & Validation (2)

**`/run-tests`** - Execute test suite with intelligent filtering
- Runs all (18s), unit (7s), integration (5s), or e2e (8s)
- Clear summaries, coverage options, failure guidance
- Filters by pattern, shows only relevant output

**`/debug-tests`** - Debug test failures systematically
- Detects 6 common patterns: imports, mocks, assertions, Playwright, fixtures, teardown
- Step-by-step debugging guidance
- Suggests specific fixes based on failure type

### Visualization (4)

**`/create-state-map`** - Generate state-level maps
- Districts, political lean, demographics, compactness, round progression
- Customizable colors, DPI, labels
- Output: `outputs/us_{year}_{version}/states/{state}/maps/`

**`/create-national-map`** - Generate US maps (435 districts)
- Alaska/Hawaii as insets
- All states aggregated
- Output: `outputs/us_{year}_{version}/maps/us_*.png`

**`/create-pedagogical-example`** - Educational algorithm examples
- Small, clear tract clusters for teaching
- Dual viz (geographic map + abstract graph)
- Strict quality validation (contiguity, compactness)

**`/generate-dashboard`** - Static HTML dashboard
- Bakes district data, maps, CSVs into single file
- Zero dependencies, works offline
- Output: `outputs/index.html` or `outputs/us_{year}_{version}/index.html`

### Documentation (3)

**`/update-docs`** - Review and update all docs for accuracy
- Checks: CLAUDE.md, README.md, ARCHITECTURE.md, CODING_PATTERNS.md, DATA_FORMATS.md, CHANGELOG.md, enhancements
- Verifies: File paths, commands, cross-references, examples
- Updates "Last Updated" dates

**`/create-session-archive`** - Historical documentation of sessions
- Captures rationale, decisions, challenges, solutions
- Archives to docs/archive/ for future reference
- Used after major enhancements or significant decisions

**`/create-architecture-diagram`** - Mermaid diagrams for ARCHITECTURE.md
- Visualizes system design, data flow, components
- Updates existing or creates new diagrams
- Improves documentation clarity

### Research & Academic (6)

**`/compile-latex`** - Compile papers/presentations
- pdflatex + bibtex, multiple passes
- Handles bibliography, cleanup
- Output: PDF in artifacts/{papers,presentations}/

**`/edit-paper`** - Edit academic papers for journal submission
- Proofreading, condensing, copyediting
- Page limit targeting
- Ensures clarity, technical accuracy, consistent notation

**`/edit-presentation`** - Edit presentation slides (Beamer LaTeX)
- Clear, concise (one idea/slide)
- Visually balanced, fits time constraints
- Targets 15-20 slides for 15-20min talks

**`/edit-guide`** - Edit educational guides for general audiences
- Layman's guides, tutorials, explainers
- Clarity, conciseness, accessibility
- No jargon, clear examples

**`/create-presentation-figures`** - Generate figures for papers/talks
- Schematic diagrams, round progression maps, census tract examples
- Copies from pipeline outputs or generates fresh
- High quality (300 DPI)

**`/consolidate-scripts`** - Merge duplicate/similar scripts
- Identifies scripts with similar functionality
- Proposes consolidation (mode flags, shared libs)
- Maintains backward compatibility, tests side-by-side

### Experiment & Analysis (3)

**`/run-experiment`** - Test algorithm variants, compare results
- Defines parameters (states, variants, metrics)
- Runs redistricting for each variant
- Statistical comparison (paired t-tests, effect sizes, confidence intervals)

**`/run-statistical-analysis`** - Quantitative analysis of results
- Computes stats (means, medians, std devs)
- Comparisons across years/modes
- Generates comparison tables, statistical plots

**`/validate-compactness`** - Verify compactness maintained/improved
- Loads district geometries, computes metrics (Polsby-Popper, Reock)
- Compares to baselines (current districts, historical, random)
- Reports improvements/regressions, identifies outliers

### Code Organization (3)

**`/refactor-for-pattern`** - Refactor code to follow established patterns
- Reads CODING_PATTERNS.md for target pattern
- Identifies non-compliant code (progress reporting, path handling, error handling)
- Refactors, tests equivalence, documents pattern

**`/reorganize-directory-structure`** - Restructure directories
- Analyzes current structure, proposes improvements
- Migration plan with validation
- Updates all path references, documents changes

**`/parameter-sweep`** - Test algorithm with different parameters
- Defines parameter space (edge weights, pop tolerance, min tract pop)
- Runs redistricting for each combo
- Tracks metrics vs parameters, identifies optimal, visualizes sensitivity

### Editorial (3)

**`/edit-paper`** - Journal submission editing (detailed in Research)
**`/edit-presentation`** - Conference talk editing (detailed in Research)
**`/edit-guide`** - Educational guide editing (detailed in Research)

## Skill Composition Examples

**Full enhancement workflow**:
```
/enhancement-plan → /enhancement-implement → /run-tests → /enhancement-document
```

**New feature with data**:
```
/census-download → /adjacency-build → /data-validate → /run-redistricting → /generate-dashboard
```

**Research paper workflow**:
```
/run-experiment → /run-statistical-analysis → /create-presentation-figures → /compile-latex → /edit-paper
```

**Bug fix workflow**:
```
/pipeline-debug → /debug-tests → /run-tests → /update-docs
```

**Visualization update**:
```
/run-analysis-only → /create-state-map → /create-national-map → /generate-dashboard
```

## Skill Development Guidelines

**Creating new skills** (use `/create-skill`):
1. Clear, specific use case (not too broad)
2. Follows established patterns (STATUS protocol, skip logic, GEOID handling)
3. Includes validation and error handling
4. Documents prerequisites, when to use, what you get
5. 5-10 min creation time (automated via /create-skill)

**Skill characteristics**:
- **Specialized**: Domain-specific knowledge
- **Reusable**: Applies to multiple situations
- **Validated**: Error checking, prerequisites
- **Documented**: Clear description, examples
- **Composable**: Works with other skills

## Implementation Notes

**Location**: `.claude/skills/{skill-name}/` (YAML frontmatter + Markdown)
**Auto-discovery**: Claude Code loads on startup
**Invocation**: `/skill-name` or natural language matching description
**Permissions**: Some skills request tool permissions (Bash for tests/pipeline)

**Tool Permissions**:
- Skills can request prompt-based permissions for Bash commands
- Example: "run tests", "install dependencies", "build project"
- Session-scoped, cleared on exit
- User sees requested permissions when approving skill execution

## Future Skill Ideas

Potential additions (not yet implemented):
- `/compare-algorithms` - Compare METIS vs other partitioning methods
- `/optimize-parameters` - Auto-tune algorithm parameters
- `/benchmark-performance` - Profile pipeline, identify bottlenecks
- `/export-for-qgis` - Convert outputs to QGIS-compatible formats
- `/create-interactive-viz` - Generate D3.js interactive maps
- `/analyze-fairness` - Compute fairness metrics (efficiency gap, mean-median)

## Questions or Issues?

**Skill usage**: Skills are automatically suggested when relevant to your request
**Creating skills**: Use `/create-skill` to generate new skill following patterns
**Skill catalog**: This document (SKILLS.md)
**Enhancement workflow**: See [ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md)

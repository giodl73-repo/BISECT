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

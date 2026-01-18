# Enhancement Workflow

**Structure**: Current State → Goal → Implementation Plan → Files → Testing → Benefits → Success Criteria → Complexity
**Template**: `enhancements/templates/enhancement_template.md`

## 1. Research Phase

**Review**:
- `ARCHITECTURE.md` - System design/data flow
- `CODING_PATTERNS.md` - Implementation patterns
- `DATA_FORMATS.md` - Data structures (if data work)
- `enhancements/INDEX.md` - Related enhancements
- `archive/` - Similar implementations

**Dependencies Check**:
- Census year differences (2000/2010/2020)? Field names? Formats?
- Conditional paths? Data availability checks?
- Multi-year compatibility?
- Windows-specific (Unicode, paths)?

## 2. Planning Phase

**Create Spec** (`enhancements/active/NN_name.md`):
- Define 3-6 phases: Data prep (0) → Core (1) → Integration (2) → Config/CLI (3) → Testing (4) → Docs (5)
- List ALL files to modify/create
- Estimate complexity

**Test Requirements** (MANDATORY):
```
Function/class?      → [ ] Unit tests (tests/unit/)
Component interact?  → [ ] Integration tests (tests/integration/)
Pipeline workflow?   → [ ] E2E tests (tests/e2e/)
Visualization?       → [ ] Dashboard tests (tests/e2e/)
Census years?        → [ ] 2000/2010/2020 coverage
```

**Manual Testing Plan**:
- States: Small (VT/DE/WY), Medium (AL), Large (CA/TX)
- Years: 2000/2010/2020
- Modes: Print-only first, then real runs

**Backward Compatibility**:
- Existing workflows break? Support both patterns during transition? Migration scripts?

## 3. Implementation Phase

**Use TodoWrite**: Track phases, one task in_progress at a time

**Key Practices**:
- Incremental changes (one component at a time)
- **Write tests as you code** (unit tests immediately after functions)
- Manual edits for critical paths
- Use `Path` objects (not string concat)
- Add print-only/dry-run support

**TDD (Recommended)**:
1. Write failing test
2. Implement to pass
3. Refactor (keep green)
4. Add integration/E2E when components work

**Coding Standards**:
```python
# STATUS Protocol (progress reporting)
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)

# Scope-based pattern (analysis scripts)
parser.add_argument('--scope', choices=['state', 'national'], default='state')
if args.scope == 'state': process_single_state(args)
elif args.scope == 'national': aggregate_all_states(args)
```

**Windows**: ASCII ONLY (`[OK]`/`[FAIL]`/`->`) - NO Unicode (✓✗→•)

## 4. Testing Phase

### A. Automated Tests (MANDATORY)

**Coverage Checklist**:
- [ ] Unit tests: New functions/classes (>80% coverage)
- [ ] Integration tests: Component interactions (main path + failures)
- [ ] E2E tests: Complete workflows (≥1 Vermont test)
- [ ] Dashboard tests: HTML generation + data baking
- [ ] Edge cases: Error conditions + boundaries
- [ ] Multi-year: 2000/2010/2020 support

**Decision Tree**:
```
Add/modify function/class?      YES → tests/unit/
                                 NO  ↓
File/config/component interact?  YES → tests/integration/
                                 NO  ↓
Complete pipeline workflow?      YES → tests/e2e/
                                 NO  ↓
Visualization/dashboard?         YES → tests/e2e/
                                 NO  → Done
```

**Quality Standards**:
- Unit: >80% coverage, fast (ms), mock I/O
- Integration: Real files (test fixtures), mock expensive ops (METIS)
- E2E: Full workflows, Vermont data, verify outputs
- All: Reliable (no flaky tests)

**Run Tests**:
```bash
pytest tests/ -v                          # All (18s)
pytest tests/{unit,integration,e2e}/ -v   # By type
pytest tests/unit/partition/ -v -k "bisection"  # Subset
```

### B. Manual Pipeline Testing

**Order** (after automated tests pass):
1. **Print-only**: `--print-only` (catches param threading fast)
2. **Small states**: VT/DE (30s-2min validation)
3. **Multi-year**: Test 2000/2010/2020 if enhancement affects all
4. **Quantitative**: Compare metrics before/after, document improvements
5. **Connectivity**: `check_graph_connectivity.py` (if graph changes)
6. **Full pipeline**: `--states "VT,DE,WY"` (spot-check, not required)

**Don't**:
- Skip automated tests (catch regressions)
- Skip print-only (catches param issues)
- Run 50-state to test (use 1-2 small states)
- Change defaults without quantitative validation
- Write tests dependent on specific output contents

**User Validation**: Joint review, confirm metrics/outputs reasonable

## 5. Documentation Phase (CRITICAL)

**Required Updates**:
1. **Enhancement file** → Mark complete, add date/summary, move active/ → completed/, update INDEX.md
2. **ARCHITECTURE.md** → System design, data flow, components (if applicable)
3. **CODING_PATTERNS.md** → New patterns, examples, anti-patterns (if applicable)
4. **DATA_FORMATS.md** → File formats, schemas, year diffs (if applicable)
5. **CLAUDE.md** → Recent changes, structure, quick ref, pitfalls
6. **CHANGELOG.md** → Date, description, files, quantitative metrics
7. **Enhancement doc** → All phases complete, final stats, **tests added**

**Checklist**:
- [ ] Code examples updated
- [ ] File path refs current
- [ ] Command examples work
- [ ] Directory diagrams match
- [ ] Enhancement marked complete
- [ ] Quantitative metrics documented
- [ ] "Last Updated" dates refreshed
- [ ] **Tests documented** (list files/coverage)

## 6. Completion Phase

**Review**:
- All modified files reviewed
- No data/output files staged (`git status`)
- `.gitignore` covers new files

**Commit**: Clear message, reference enhancement #, list key files

**Archive**: Create `archive/YYYY-MM-DD_enhancement_N_desc.md` for significant sessions (rationale, decisions, not just what)

**Defaults**: Only change with overwhelming evidence (e.g., "52.8% PP improvement"), keep non-default accessible

## Past Enhancement Patterns

**#1 (Compactness)**: Integrated existing standalone → pipeline, added CSV columns
**#2 (D/R Seats)**: Text annotations, 20min quick win
**#3 (National Maps)**: National-scope viz, AK/HI insets, 2h medium
**#4 (Metro)**: External CBSA data, top 20 MSAs by state
**#5 (Round Progression)**: Aggregated round data, handled varying completion
**#6 (Diagrams)**: Mermaid in ARCHITECTURE.md, critical for understanding
**#7 (Edge-Weighted)**: AL test first, quantitative (52.8% PP, 22.2% perimeter), changed defaults, edge weights=integers (cm scale), water=median, point=0.1m
**#8 (Historical)**: Phase 0 tracts first, year-specific parsers, 2010 Census API, 2000 NHGIS manual, conditional skips, one year fully working before next
**#9 (Per-State)**: Scope pattern (--scope state|national), parallel per-state, national post-processing, eliminated 300min bottleneck, STATUS protocol, explicit --state
**#13 (Unification)**: Move files → update scripts → docs, manual for critical paths, UNIFICATION_STATUS.md, preserved intentional conditionals, removed ~80 lines, tested all years

## Technical Challenges & Solutions

**Data**:
- Unicode Windows → ASCII (`[OK]`/`[X]`) not (✓/✗)
- GEOID types → `dtype={'GEOID': str}`
- Fixed-width → Document exact positions
- Field names vary → Research (GEOID vs GEOID10 vs CTIDFP00)

**METIS**:
- Integer edge weights → Scale: `int(meters * 100)` (cm)
- CSR codes → 000 (unweighted), 011 (edge-weighted)
- Point adjacencies → 0.1m (not zero)

**Graph**:
- **CRITICAL: Add ALL nodes explicitly** (isolated nodes won't auto-add)
  ```python
  for i in range(num_nodes): graph.add_node(i)  # Don't skip!
  for i, neighbors in enumerate(adjacency):
      for j in neighbors: graph.add_edge(i, j)
  ```
- Water adjacencies → Median land boundary
- Connectivity → Single component before METIS

**Paths**:
- Absolute for subprocesses → `Path(__file__).parent`, `sys.executable`
- Support both during transitions → Check new, fallback old
- Year param cascades → Config, paths, outputs

**Performance**:
- Edge-weighted ~2x slower (reasonable)
- Block-level 10-100x slower → Use tract aggregation
- Parallel → Build by county, merge

## Key Learnings

1. **Systematically add tests EVERY enhancement** (unit/integration/e2e/dashboard) - MANDATORY, use decision tree
2. **Always update docs** - Future assistants need context
3. **Test incrementally** - Automated → print-only → small → multi-year → full
4. **Follow patterns** - Consistency reduces load
5. **Manual safer for critical paths** - Batch risks unexpected changes
6. **User validation essential** - Metrics can look right but be wrong
7. **Print-only FIRST** - Catches param threading before expensive runs
8. **Single-state THEN 50-state** - AL/DE good quick tests
9. **Connectivity CRITICAL** - Must form single component before METIS
10. **Quantitative improvements** - "52.8% better" > "much better"
11. **Graceful degradation** - Missing optional data shouldn't crash
12. **Windows testing matters** - Unicode, paths, line endings differ
13. **Multi-year requires research** - Field names, formats, APIs differ
14. **Keep intentional conditionals** - Config imports conditional for reason
15. **Archive significant decisions** - Need WHY, not just WHAT

## Do This ✅

```python
# Check data availability
election_data_available = (args.year == '2020' and election_data_file.exists())
if election_data_available:
    steps.append(('Political Analysis', f'{sys.executable} scripts/political/analyze_districts.py ...'))
else:
    print(f"  [SKIP] Political analysis (no 2020 election data for {args.year})")

# Support both paths during transitions
graph_file_new = Path(f'data/adjacency/{year}/{state}_adjacency_{year}.pkl')
graph_file_old = Path(f'data/adjacency/{state}_adjacency_{year}.pkl')
graph_file = graph_file_new if graph_file_new.exists() else graph_file_old

# ASCII for console
print(f"[OK] Using edge-weighted mode")  # Not: print(f"✓ Using edge-weighted mode")

# Dynamic config
if args.year == '2010':
    from scripts.config_2010 import STATE_CONFIG_2010 as STATE_CONFIG
elif args.year == '2020':
    from scripts.config_2020 import STATE_CONFIG_2020 as STATE_CONFIG

# Explicitly add all nodes (CRITICAL: isolated need explicit add)
graph = nx.Graph()
for i in range(num_nodes): graph.add_node(i)
for i, neighbors in enumerate(adjacency):
    for j in neighbors: graph.add_edge(i, j)

# Force GEOID string
tracts = pd.read_csv(file, dtype={'GEOID': str})
```

## Don't Do This ❌

```python
# ❌ Hardcode year
tracts_file = f'data/raw/{state}_tracts_2020.parquet'  # Wrong
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'  # Right

# ❌ Fail on missing optional data (crashes other years)
if not election_data_file.exists():
    raise FileNotFoundError("Election data not found")

# ❌ Rely on edges to add nodes (isolated never added!)
for i, neighbors in enumerate(adjacency):
    for j in neighbors: graph.add_edge(i, j)  # Missing: graph.add_node(i) first

# ❌ Unicode console (crashes Windows)
print(f"✓ Complete")  # Crashes
print(f"[OK] Complete")  # Works

# ❌ Assume field names consistent across years
df = pd.read_csv(file)
geoid = df['GEOID']  # Might be GEOID10 or CTIDFP00 in other years
```

**Additional Don'ts**:
- **Skip automated tests** (every enhancement needs unit/integration/e2e/dashboard)
- **Only manual test** (automated catch regressions, document behavior)
- **Write tests after "done"** (write during implementation)
- Skip print-only
- Run 50-state to test
- Change defaults without quantitative validation
- Mix tabs/spaces in fixed-width parsing
- Assume GEOID always strings

## Risk Mitigation

**Low-Risk**:
- Phased with testing between
- Keep old until validated (mark deprecated)
- Use `--version test` (avoid overwrite production)
- Mark new features non-critical
- Full rollback via git

**Validation Before Deletion**:
- Test old/new side-by-side
- Compare outputs byte-for-byte if possible
- User sign-off before deleting old

## Complexity Estimates

- **Low** (20-60min): Simple integrations, text annotations, small refactors
- **Medium** (2-4h): New viz, modest refactor, single-file additions
- **Medium-High** (4-8h): Multi-file refactor, new analysis, scope-based patterns
- **High** (8-15h): New algorithms, major architecture, performance optimization
- **Very High** (15+h): Multi-year data, block-level support, complex optimizations

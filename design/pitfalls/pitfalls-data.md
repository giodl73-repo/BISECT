# Data Pitfalls (DP-01..DP-02)

Structural vulnerabilities in census data handling. Data errors are silent — the pipeline runs, outputs look plausible, and only downstream analysis or careful auditing reveals the problem.

---

## DP-01: Population metric ambiguity across legal and algorithmic contexts

**Pattern:** Two population metrics (total population and voting-age population) have different legal meanings, different values, and different appropriate uses. Systems that use one but describe results using the other's terminology produce findings that look valid but are legally misframed. When the metric isn't explicitly stated, consumers assume the legally correct one.

**Domain:** Any redistricting analysis touching VRA compliance. The Gingles preconditions (Section 2 VRA) use voting-age population (VAP). One-person-one-vote uses total population. Using total population for VRA analysis may produce different district-level minority percentages than VAP — especially in communities with large non-citizen or under-18 populations.

**Why it survives:** The code runs correctly for whichever metric it uses. The error is in the framing, not the computation. Papers that don't state their metric look rigorous.

**Structural solution:** Explicit metric declaration in every paper section that reports minority percentages. System documentation should state which metric is used and why. Ideally, provide both — total population for apportionment context, VAP for VRA context.

**Status:** OPEN — papers D.0-D.3 do not explicitly state total population vs. VAP
**Test:** None yet

---

## DP-02: Module context loss across subprocess boundary

**Pattern:** A script that works correctly when run from the project root fails when called as a subprocess because the module resolution context isn't inherited. The parent added the project root to `sys.path`; the child subprocess starts fresh. Import errors are raised in the child, which the parent may suppress or misinterpret.

**Domain:** Any Python project where scripts are called as subprocesses and import from project-level packages. The project root must be on `sys.path` for imports to resolve — but subprocesses don't inherit the parent's `sys.path`, only the environment's `PYTHONPATH`.

**Why it survives:** Running the script directly from the command line (with the correct working directory) works fine. The bug only appears when called as a subprocess from another script — a code path that's hard to test without integration tests.

**Structural solution:** Every script that imports from project-level packages must add the project root to `sys.path` in its own `__init__` block, independent of how it's invoked.

```python
_project_root = Path(__file__).resolve().parents[N]
if str(_project_root) not in sys.path:
    sys.path.insert(0, str(_project_root))
```

**Status:** SOLVED for `parse_pl94171_tracts_2010.py` and `parse_pl94171_tracts_2000.py`
**Test:** 2010/2000 pipeline runs complete without ModuleNotFoundError

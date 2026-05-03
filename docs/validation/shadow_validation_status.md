# Shadow Validation Status

**Status**: Wired and ready — feature flag active, compilation verified.

**Date**: 2026-05-02  
**Branch**: `feature/redist-metis-rust-port`  
**Task**: #28 (in_progress) — Run 50-state × 3-year shadow validation

## Readiness Summary

### Shadow Mode: Feature-Complete
- **Location**: `redist/crates/redist-apportion/src/split.rs` (lines 237-250, 290-303)
- **Design**: Rust METIS partitions normally; C METIS runs as quality oracle in parallel
- **Comparison**: Logs to stderr if Rust edge cut exceeds C cut by >20%
- **Format**: `[shadow-metis] k={k}: Rust cut {rust_cut} > C cut {c_cut} ({%} over)`
- **Thresholds**:
  - Gate: All states must show `rust_cut <= c_cut * 1.20` before C dep removal (Task #29)
  - Both `split()` and `split_weighted()` overrides included for asymmetric (prime-k) splits

### Compilation Status: Clean
```
redist-apportion (default features with shadow-metis):  PASS
redist-apportion (--no-default-features):              PASS
```
No regressions. Feature flag toggles C METIS at compile time via Cargo.toml dependency:
```toml
[features]
shadow-metis = ["dep:metis"]
default = ["shadow-metis"]   # keep C dep during validation phase
```

### Pipeline Data: Available
- **Adjacency graphs**: Present at `outputs/data/2020/adjacency/` (built 2026-02-08)
- **Unit data**: Present at `outputs/data/2020/units/`
- **Entry point**: `redist` binary not present in PATH; `run` alias not configured in environment
  - Workaround: Build via `cargo build -p redist-cli` in `redist/` directory
  - Impact: Cannot run full validation in current environment; documented below

## Blockers

### Hard Blocker: Rust `redist` Binary Not Available
The user's environment does not have:
- `redist` binary on PATH
- `run` or `runtest` doskey aliases configured (see `setup_env.bat`)

**Impact**: Cannot execute the shadow validation sweep (would use `run -y 2020 -v shadow_test -st VT`).

**Resolution for Next Session**:
1. Run `C:\src\apportionment\setup_env.bat` to configure `run`/`runtest` aliases
2. Ensure `cargo build -p redist-cli --release` has been run and binary is accessible
3. Execute: `run -y 2020 -v shadow_test -st VT 2>&1 | tail -20` to test on Vermont
4. For full sweep: `run -y 2020 -v shadow_test 2>&1 | tee shadow_validation_2020.log`

## Validation Plan (When Data Runs)

### Phase 1: Single-State Validation
```powershell
# Vermont (smallest, ~255 tracts) — quick sanity check
run -y 2020 -v shadow_test -st VT

# Expected output: 11 districts × (equal binary splits + prime-k asymmetric splits)
# Shadow warnings to stderr if any rust_cut > c_cut * 1.20
```

### Phase 2: Representative Multi-State
```powershell
# Mix of sizes: CA (largest), TX, NY, VT, DE
run -y 2020 -v shadow_test -st CA TX NY VT DE

# Aggregate across partitions to identify problem k values or edge cases
```

### Phase 3: Full 50-State Sweep
```powershell
# Production validation (~1-4h depending on workers)
run -y 2020 -v shadow_test

# Post-process: grep for [shadow-metis] warnings, aggregate by state
# Expected: 0-5 warnings across full run (minor variance in partition quality is acceptable)
```

## Success Criteria

1. **Compilation**: Both `shadow-metis` feature on and off compile cleanly ✓
2. **Code Review**: Shadow mode logic is sound (lines 237-250, 290-303) ✓
3. **Data Availability**: Adjacency graphs present in `outputs/data/2020/adjacency/` ✓
4. **Validation Runs**: (pending environment setup)
   - All 50 states complete without error
   - Final report: `outputs/shadow_test/2020/shadow_diffs.json` (if logging implemented)
   - Warnings: Track by state and k value

## Gate for C Dependency Removal (Task #29)

**Requirement**: Validation results show:
- No systematic Rust degradation (rust_cut within 20% of C cut across all k values)
- Prime-k splits (asymmetric) balanced correctly using `split_weighted()` override
- No contiguity or population-balance regressions

**Conclusion Options**:
- **PASS**: Remove `metis` dependency, disable shadow mode by default, commit
- **WARN**: Identify specific k values or states with issues; add targeted fixes before cutover
- **FAIL**: Rust METIS not production-ready; defer cutover until resolved

## Next Steps (For Next Session)

1. **Setup Environment**:
   ```powershell
   C:\src\apportionment\setup_env.bat
   cd C:\src\apportionment\redist
   cargo build -p redist-cli --release
   ```

2. **Run Vermont Validation**:
   ```powershell
   run -y 2020 -v shadow_test -st VT 2>&1 | tail -50
   ls outputs\shadow_test\2020\
   ```

3. **Capture Shadow Warnings**:
   ```powershell
   run -y 2020 -v shadow_test 2>&1 | tee shadow_log_2020.txt
   Select-String '\[shadow-metis\]' shadow_log_2020.txt | Measure-Object
   ```

4. **Archive Results**:
   ```powershell
   if (Test-Path outputs\shadow_test\2020\) {
     Copy-Item outputs\shadow_test\2020\* docs\validation\results_2020\ -Recurse
   }
   ```

5. **Update This Document** with results and gate decision.

## References

- **Shadow Mode Implementation**: `redist/crates/redist-apportion/src/split.rs` (lines 88-307)
- **Feature Flag**: `redist/crates/redist-apportion/Cargo.toml` (lines 17-19)
- **Validation Task**: Task #28 (in_progress)
- **Cutover Task**: Task #29 (pending) — Remove C metis dep
- **Task #21**: Validation gate + cutover

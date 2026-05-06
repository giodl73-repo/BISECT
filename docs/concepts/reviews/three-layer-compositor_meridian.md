# Review: three-layer-compositor.md
**Reviewer**: MERIDIAN (computational-geographer)
**Date**: 2026-05-05

---

## What's accurate

The core orthogonality claim is correct and well-stated: the three layers are genuinely independent composition axes, and the separation of structure/weights/search is a faithful description of how the Rust CLI dispatches work. The description of `ratio-optimal` (GeoSection) is algorithmically precise — the isoperimetric normalisation denominator `sqrt(min(i, k-i))` is accurately stated and correctly motivated as the correction for the geometric fact that smaller-fraction splits have intrinsically shorter boundaries. The METIS engine table is accurate: `c-ffi` links libmetis via C FFI with static linkage, `gpmetis` is correctly flagged as reserved/not yet implemented, and `redist-metis` is correctly described as the pure-Rust reimplementation. The description of `convergence` mode and T=600 as the statutory stopping criterion matches the implementation in ConvergenceSweep (B.16). The statement that the content-derived seed is publicly derivable from the Census Bureau release identifier is correct and important.

---

## P1 — Required fixes

**Section "Layer 2 — Weights", `vra-aligned` row**: The guide states minority-to-minority edges "receive 5-10x weight, tapered by the statewide minority fraction." This description is imprecise about the actual formula and direction of tapering. The implementation in `redist-core/src/vra.rs` computes `alpha = max(3.0, 10.0 * (1.0 - 0.7 * f_minority))`, where `f_minority` is the fraction of tracts above the minority threshold. The floor is 3.0x, not 5x. The stated range "5-10x" is wrong on the lower bound and omits the 3.0x floor that applies in high-minority-density states (e.g., Mississippi, Alabama, Louisiana). A reviewer using this guide to validate VRA weight outputs would accept a 3.5x boost as incorrect when it is in fact the expected result for high-density states.

**Section "Layer 3 — Search", `convergence` row**: The description says "Stops after T=600 consecutive seeds produce no improvement." The T=600 is described as the statutory stopping criterion from B.16. However, the guide presents this as the only stopping criterion. It should clarify that the algorithm runs from a content-derived seed `s_0` and walks forward through seeds, not sampling randomly — the seed sequence is deterministic from s_0, which is critical for reproducibility claims.

**Section "CLI examples"**: The flag `--partition-mode` shown in the examples maps to the structure layer, but the config YAML uses `structure:` as the key. The CLI and config namespaces differ in ways that will confuse first-time users. The guide should explicitly state that `--partition-mode ratio-optimal` on the CLI corresponds to `structure: ratio-optimal` in the YAML.

---

## P2 — Suggested improvements

The description of `prime-factor` would benefit from clarifying that the bisection tree for prime k uses a floor/ceil binary fallback at the top level only — subsequent levels may be non-binary if sub-problems are composite. The current text says "k=17 (prime) falls back to a 9+8 binary split" but does not clarify what happens to the sub-problems (9=3x3 would use ternary; 8=2^3 would use binary). A worked example through two levels for a prime k would prevent misunderstanding.

The METIS backend section would benefit from noting that a verifier using the `redist-metis` (pure Rust) engine may obtain different local optima from `c-ffi` even with the same seed, since the two implementations may differ in tie-breaking. This matters for statutory certification.

---

## Score: 3/4

The guide is algorithmically sound on all points except the VRA weight range, which is factually wrong. Fixing the 3.0x floor and clarifying the seed-walk determinism would bring this to publishable quality.

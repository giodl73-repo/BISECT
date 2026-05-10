---
title: "M.9-impl — Economic Character Weighter: cosine_similarity + EconomicCharacterWeighter"
series: M.9-impl
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — EdgeWeighter plugin)
---

## Purpose

Implementation spec for the M.9 economic character edge weights.
Extends the existing `EdgeWeighter` trait pipeline in
`crates/bisect-cli/src/edge_weights.rs`.

## Architecture fit

The existing `build_edge_weights()` in `runner.rs` already composes
`GeographicWeighter → MinorityOverrideWeighter → PartisanOverrideWeighter → SubdivisionWeighter`
via a `ComposedWeighter`. The new `EconomicCharacterWeighter` is a fifth
optional step in this pipeline, added when `WeightMode::EconomicCharacter`
is selected.

## New components

### 1. `EconChar` struct (`lodes.rs`)
```rust
pub struct EconChar {
    pub commercial_intensity: f64,  // (CNS07+CNS09+CNS10+CNS11) / C000
    pub industrial_fraction: f64,   // (CNS01+CNS02+CNS05+CNS08) / C000
    pub jobs_per_resident: f64,     // C000 / tract_population (capped at 10.0)
}
```
Zero-job tracts: `EconChar { 0.0, 0.0, 0.0 }` (pure residential — high similarity with other residential tracts).

### 2. `load_lodes_wac_tract(state_name, year) -> HashMap<Geoid, EconChar>` (`lodes.rs`)
Reads `data/{year}/lodes/{state_name}_wac_tract.csv`.
Returns empty map (not error) when file absent — falls back to uniform char.

### 3. `cosine_similarity(a: &EconChar, b: &EconChar) -> f64`
```
dot  = a.ci*b.ci + a.if_*b.if_ + a.jpr*b.jpr
|a|  = sqrt(a.ci² + a.if_² + a.jpr²)
|b|  = sqrt(b.ci² + b.if_² + b.jpr²)
sim  = if |a|==0 && |b|==0 { 1.0 }   // both zero → both residential → similar
       else if |a|==0 || |b|==0 { 0.5 } // one residential, one not → neutral
       else { (dot / (|a|*|b|)).clamp(0.0, 1.0) }
```
Range: [0.0, 1.0]. Symmetric. Defined for zero vectors.

### 4. `EconomicCharacterWeighter` (`edge_weights.rs`)
```rust
pub struct EconomicCharacterWeighter {
    chars: HashMap<usize, EconChar>,  // node_idx → EconChar
    alpha: f64,                        // blend: alpha*existing + (1-alpha)*sim
}

impl EdgeWeighter for EconomicCharacterWeighter {
    fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights.into_iter().map(|((u,v), w)| {
            let sim = cosine_similarity(
                self.chars.get(&u).unwrap_or(&ZERO_CHAR),
                self.chars.get(&v).unwrap_or(&ZERO_CHAR),
            );
            let w_new = self.alpha * w + (1.0 - self.alpha) * sim * w;
            ((u,v), w_new)
        }).collect()
    }
}
```
When alpha=0.5: w_new = 0.5*w_geographic + 0.5*sim*w_geographic
→ dissimilar tracts (sim≈0): edge halved; similar (sim≈1): edge unchanged.
When geographic weighting is OFF (unweighted mode): w passed in = 1.0 uniform.

### 5. Wire into `build_edge_weights()` (`runner.rs`)
```rust
// Step 5: Economic character (M.9/M.1) — similarity-weighted edges.
if spec.economic_character {
    let lodes_chars = lodes::load_lodes_wac_tract(state_name, year, position)?;
    let node_chars = align_lodes_to_adjacency(&lodes_chars, &graph.index_to_geoid, graph.n_vertices);
    composer = composer.push(EconomicCharacterWeighter::new(node_chars, spec.econ_alpha));
}
```

### 6. `WeightSpec` extension (`runner.rs`)
```rust
pub economic_character: bool,   // true when WeightMode::EconomicCharacter
pub econ_alpha: f64,            // blend factor [0.0, 1.0], default 0.5
```

### 7. `WeightMode::EconomicCharacter` → `WeightSpec` mapping (`runner.rs`)
```rust
WM::EconomicCharacter => WeightSpec {
    geographic: true,
    economic_character: true,
    econ_alpha: 0.5,
    ..WeightSpec::default()
},
```

## L0 test invariants

- `cosine_sim_both_zero_returns_one` — two zero chars (both residential) → 1.0
- `cosine_sim_one_zero_returns_half` — one residential, one non-zero → 0.5
- `cosine_sim_identical_returns_one` — same char vector → 1.0
- `cosine_sim_orthogonal_returns_zero` — perpendicular → 0.0
- `cosine_sim_symmetric` — sim(a,b) == sim(b,a)
- `econ_weighter_leaves_similar_tracts_unchanged` — alpha=0.5, sim=1.0 → weight unchanged
- `econ_weighter_halves_dissimilar_edges` — alpha=0.5, sim=0.0 → weight halved
- `econ_weighter_preserves_edge_ordering` — if w_geo(u,v) > w_geo(x,y) and both pairs
  have the same similarity, ordering is preserved
- `load_lodes_missing_file_returns_empty` — absent CSV returns Ok(empty map)
- `align_lodes_to_adjacency_zero_for_missing` — tracts not in LODES get zero EconChar

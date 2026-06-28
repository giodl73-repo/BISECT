//! Engine support types shared with the bisection runner.
//!
//! Extracted from `bisect-cli::runner` so the engine (`bisection_runner`) no
//! longer depends back on the CLI orchestration module. The CLI re-exports these
//! items from `bisect_runner::runner` to preserve its existing `crate::runner::*`
//! paths.

/// AreaSection warm-start strategy.
///
/// `RatioOptimal` uses the existing internal Lorenz-filtered ratio heuristic.
/// `MovingKnife` calls `split_subgraph_mka_direction()` first to obtain theta*
/// (the Reock-maximising cut angle), then converts it to a directional penalty
/// applied to edge weights before the METIS ratio search. Requires tract centroids;
/// falls back to `RatioOptimal` with a warning if centroid data is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaSectionInit {
    /// Current default — internal Lorenz heuristic, no directional pre-bias.
    RatioOptimal,
    /// Use MKA theta* as the directional penalty angle for the METIS ratio search.
    MovingKnife,
}

/// Validate a multiscale (fine, coarse) census-level pair.
///
/// Valid orderings (fine -> coarse):
///   (bg, tract), (bg, county), (tract, county)
///
/// Returns Ok(()) for valid pairs; Err with a descriptive message for invalid ones.
pub fn validate_multiscale_levels(fine: &str, coarse: &str) -> Result<(), String> {
    let rank = |level: &str| match level {
        "bg" | "block_group" => Ok(0usize),
        "tract" => Ok(1usize),
        "county" => Ok(2usize),
        other => Err(format!(
            "unknown multiscale level '{other}'. Valid values: bg, tract, county"
        )),
    };
    let fr = rank(fine)?;
    let cr = rank(coarse)?;
    if fr < cr {
        Ok(())
    } else {
        Err(format!(
            "--multiscale-fine {fine} is not finer than --multiscale-coarse {coarse}.\n\
             Valid orderings (fine -> coarse): bg->tract, bg->county, tract->county."
        ))
    }
}

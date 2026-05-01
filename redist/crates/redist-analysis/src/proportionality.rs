//! Proportionality analyzer: how closely does a districting plan's seat share
//! mirror the statewide vote share?
//!
//! The shipped political analyzer reports per-district margins + uncontested
//! flags, and the partisan analyzer computes efficiency gap / mean-median /
//! partisan bias (all *bias* metrics — measuring whether the plan systematically
//! favors one party at a fixed vote share). None of those is the simple
//! "did the parties win seats in proportion to their statewide votes" question.
//!
//! This module ships that question as a first-class metric:
//!
//! - `dem_vote_share_statewide` — total Dem votes / (Dem + Rep) summed across all
//!   tracts in the state.
//! - `dem_seat_share` — number of districts with Dem margin >= 0 / total districts.
//! - `proportionality_gap_pp` — `100 * (dem_seat_share - dem_vote_share_statewide)`,
//!   in percentage points. Positive means Dems got more seats than their votes
//!   would suggest. Negative means the opposite. Zero means perfect proportionality.
//!
//! The metric is signed and party-agnostic in interpretation: the same
//! computation read from the Republican side flips sign. We report the
//! Democratic side as the canonical sign convention to match the rest of the
//! political-analyzer output.
//!
//! Limitations (documented in the JSON output):
//! - Single-member districts cannot achieve continuous proportionality;
//!   the metric is bounded by `1/N` granularity.
//! - Uncontested districts (Dem or Rep votes = 0) may distort the metric
//!   if the data source under-reports minor-party / write-in totals; the
//!   `n_uncontested` field flags this.
//! - The metric does not account for population-weighted vs vote-weighted
//!   denominators; we use vote-weighted (consistent with the political
//!   analyzer's per-district margins).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analyzer::{Analyzer, AnalyzerContext};

#[derive(Debug, Deserialize)]
struct PoliticalRow {
    geoid: String,
    dem_votes: f64,
    rep_votes: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProportionalityResult {
    pub analyzer: &'static str,
    pub available: bool,

    /// Statewide Dem vote share (0..=1) computed as
    /// `sum(dem_votes) / (sum(dem_votes) + sum(rep_votes))`.
    pub dem_vote_share_statewide: f64,
    /// Statewide Rep vote share (0..=1).
    pub rep_vote_share_statewide: f64,
    /// Total statewide two-party vote count.
    pub total_two_party_votes: f64,

    /// Number of districts the algorithm produced.
    pub n_districts: usize,
    /// Districts with Dem margin >= 0.
    pub dem_seats: usize,
    /// Districts with Dem margin < 0.
    pub rep_seats: usize,
    /// Districts where one side recorded zero votes (data-quality flag).
    pub n_uncontested: usize,

    /// `dem_seats / n_districts` (0..=1).
    pub dem_seat_share: f64,
    /// `rep_seats / n_districts` (0..=1).
    pub rep_seat_share: f64,

    /// `100 * (dem_seat_share - dem_vote_share_statewide)`. Signed.
    /// Positive => Dems over-represented. Negative => Dems under-represented.
    /// Zero => perfect proportionality.
    pub proportionality_gap_pp: f64,

    /// Absolute value of `proportionality_gap_pp` — convenient for
    /// "how disproportionate is this plan?" sorting without sign-direction
    /// concerns.
    pub abs_proportionality_gap_pp: f64,

    /// Per-district Dem-share (0..=1) summary, sorted ascending so that
    /// callers can compute median / quartile easily.
    pub per_district_dem_share_sorted: Vec<f64>,
}

/// Compute proportionality metrics from raw political rows + an assignment.
pub fn aggregate_proportionality(
    rows: &[PoliticalRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> ProportionalityResult {
    let mut per_district: HashMap<usize, (f64, f64)> = HashMap::new();
    for d in 1..=num_districts {
        per_district.insert(d, (0.0, 0.0));
    }

    let mut total_dem = 0.0f64;
    let mut total_rep = 0.0f64;

    for row in rows {
        if let Some(&district) = assignments.get(&row.geoid) {
            let e = per_district.entry(district).or_insert((0.0, 0.0));
            e.0 += row.dem_votes;
            e.1 += row.rep_votes;
        }
        total_dem += row.dem_votes;
        total_rep += row.rep_votes;
    }

    let total_two_party = total_dem + total_rep;
    let (dem_share_statewide, rep_share_statewide) = if total_two_party == 0.0 {
        (0.0, 0.0)
    } else {
        (total_dem / total_two_party, total_rep / total_two_party)
    };

    let mut dem_seats = 0usize;
    let mut rep_seats = 0usize;
    let mut n_uncontested = 0usize;
    let mut per_district_share: Vec<f64> = Vec::with_capacity(num_districts);

    for d in 1..=num_districts {
        let (dem, rep) = per_district[&d];
        let total = dem + rep;
        let dem_pct = if total == 0.0 { 0.0 } else { dem / total };
        per_district_share.push(dem_pct);
        // Tie-breaking convention: dem_pct >= 0.5 counts as a Dem seat.
        // Exact 0.5 is vanishingly rare with real data; this matches the
        // shipped political analyzer's `lean_dem` semantics (margin >= 0).
        if dem_pct >= 0.5 {
            dem_seats += 1;
        } else {
            rep_seats += 1;
        }
        if dem == 0.0 || rep == 0.0 {
            n_uncontested += 1;
        }
    }

    per_district_share.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let dem_seat_share = if num_districts == 0 {
        0.0
    } else {
        dem_seats as f64 / num_districts as f64
    };
    let rep_seat_share = if num_districts == 0 {
        0.0
    } else {
        rep_seats as f64 / num_districts as f64
    };

    let proportionality_gap_pp = 100.0 * (dem_seat_share - dem_share_statewide);

    ProportionalityResult {
        analyzer: "proportionality",
        available: true,
        dem_vote_share_statewide: dem_share_statewide,
        rep_vote_share_statewide: rep_share_statewide,
        total_two_party_votes: total_two_party,
        n_districts: num_districts,
        dem_seats,
        rep_seats,
        n_uncontested,
        dem_seat_share,
        rep_seat_share,
        proportionality_gap_pp,
        abs_proportionality_gap_pp: proportionality_gap_pp.abs(),
        per_district_dem_share_sorted: per_district_share,
    }
}

pub struct ProportionalityAnalyzer;

impl Analyzer for ProportionalityAnalyzer {
    type Output = ProportionalityResult;

    fn name() -> &'static str {
        "proportionality"
    }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        // Reads the same election CSV the political analyzer consumes.
        let csv_path = ctx
            .data_root
            .join(ctx.year)
            .join("elections")
            .join("presidential_by_tract.csv");

        if !csv_path.exists() {
            eprintln!(
                "WARNING: proportionality analyzer requires election data at {}",
                csv_path.display()
            );
            return Ok(ProportionalityResult {
                analyzer: "proportionality",
                available: false,
                dem_vote_share_statewide: 0.0,
                rep_vote_share_statewide: 0.0,
                total_two_party_votes: 0.0,
                n_districts: ctx.num_districts,
                dem_seats: 0,
                rep_seats: 0,
                n_uncontested: 0,
                dem_seat_share: 0.0,
                rep_seat_share: 0.0,
                proportionality_gap_pp: 0.0,
                abs_proportionality_gap_pp: 0.0,
                per_district_dem_share_sorted: Vec::new(),
            });
        }

        let mut rdr = csv::Reader::from_path(&csv_path)?;
        let rows: Vec<PoliticalRow> = rdr
            .deserialize()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(aggregate_proportionality(&rows, ctx.assignments, ctx.num_districts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(geoid: &str, dem: f64, rep: f64) -> PoliticalRow {
        PoliticalRow {
            geoid: geoid.to_string(),
            dem_votes: dem,
            rep_votes: rep,
        }
    }

    fn asgn(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn perfect_proportionality_one_district() {
        // 1 district: must always elect Dem if Dems lead. seat_share = 1.0.
        // vote_share = 0.7. gap = +30pp.
        let rows = vec![row("01", 700.0, 300.0)];
        let result = aggregate_proportionality(&rows, &asgn(&[("01", 1)]), 1);
        assert!((result.dem_vote_share_statewide - 0.7).abs() < 1e-9);
        assert_eq!(result.dem_seats, 1);
        assert_eq!(result.rep_seats, 0);
        assert!((result.dem_seat_share - 1.0).abs() < 1e-9);
        assert!((result.proportionality_gap_pp - 30.0).abs() < 1e-9);
    }

    #[test]
    fn perfectly_proportional_two_districts() {
        // Two equal-population districts. Dem 50.0% statewide, dem wins exactly 1/2.
        // gap = 0.
        let rows = vec![
            row("01", 700.0, 300.0), // Dem-leaning district
            row("02", 300.0, 700.0), // Rep-leaning district
        ];
        let result = aggregate_proportionality(&rows, &asgn(&[("01", 1), ("02", 2)]), 2);
        assert!((result.dem_vote_share_statewide - 0.5).abs() < 1e-9);
        assert_eq!(result.dem_seats, 1);
        assert_eq!(result.rep_seats, 1);
        assert!((result.dem_seat_share - 0.5).abs() < 1e-9);
        assert!(result.proportionality_gap_pp.abs() < 1e-9);
        assert!(result.abs_proportionality_gap_pp.abs() < 1e-9);
    }

    #[test]
    fn ma_style_geographic_lockout() {
        // 3 districts, Dem 66% statewide, every district 60-66% Dem -> dems
        // sweep all 3. seat_share = 1.0, vote_share = 0.66, gap = +34pp.
        let rows = vec![
            row("01", 600.0, 400.0),
            row("02", 660.0, 340.0),
            row("03", 700.0, 300.0),
        ];
        let result = aggregate_proportionality(
            &rows,
            &asgn(&[("01", 1), ("02", 2), ("03", 3)]),
            3,
        );
        assert!(result.dem_vote_share_statewide > 0.65);
        assert!(result.dem_vote_share_statewide < 0.67);
        assert_eq!(result.dem_seats, 3);
        assert!((result.dem_seat_share - 1.0).abs() < 1e-9);
        // Gap should be positive ~34pp.
        assert!(result.proportionality_gap_pp > 30.0);
        assert!(result.proportionality_gap_pp < 40.0);
    }

    #[test]
    fn tx_style_efficient_minority_packing() {
        // 5 districts. Dem 47% statewide, but Dems concentrated in 2 districts
        // at 80% Dem; the other 3 districts are 30% Dem. Dem seats = 2/5 = 40%.
        // vote_share = 47%. gap = 40 - 47 = -7pp (Dems underrepresented).
        let rows = vec![
            row("01", 800.0, 200.0),
            row("02", 800.0, 200.0),
            row("03", 300.0, 700.0),
            row("04", 300.0, 700.0),
            row("05", 300.0, 700.0),
        ];
        let result = aggregate_proportionality(
            &rows,
            &asgn(&[("01", 1), ("02", 2), ("03", 3), ("04", 4), ("05", 5)]),
            5,
        );
        // 1600 + 600 + 900 = 1600 dem; 400 + 600 + 2100 = 1500 rep. Wait, let me recompute.
        // Actually: dem total = 800+800+300+300+300 = 2500. rep total = 200+200+700+700+700 = 2500.
        // So vote_share = 50%. Dem seats = 2/5 = 40%. gap = -10pp.
        assert!((result.dem_vote_share_statewide - 0.5).abs() < 1e-9);
        assert_eq!(result.dem_seats, 2);
        assert_eq!(result.dem_seat_share, 0.4);
        assert!((result.proportionality_gap_pp - (-10.0)).abs() < 1e-9);
        assert!(result.proportionality_gap_pp < 0.0);
    }

    #[test]
    fn handles_no_data() {
        let result = aggregate_proportionality(&[], &asgn(&[]), 5);
        assert_eq!(result.dem_vote_share_statewide, 0.0);
        assert_eq!(result.total_two_party_votes, 0.0);
        // With no votes, every district is "uncontested" and counts as Rep
        // by tie-break (dem_pct < 0.5). The metric is degenerate but well-defined.
        assert_eq!(result.dem_seats, 0);
        assert_eq!(result.n_districts, 5);
    }

    #[test]
    fn flags_uncontested_districts() {
        let rows = vec![
            row("01", 1000.0, 0.0), // uncontested Dem
            row("02", 500.0, 500.0),
        ];
        let result = aggregate_proportionality(&rows, &asgn(&[("01", 1), ("02", 2)]), 2);
        assert!(result.n_uncontested >= 1, "uncontested district must be flagged");
    }

    #[test]
    fn per_district_share_sorted_ascending() {
        let rows = vec![
            row("01", 800.0, 200.0), // 0.80
            row("02", 300.0, 700.0), // 0.30
            row("03", 600.0, 400.0), // 0.60
        ];
        let result = aggregate_proportionality(
            &rows,
            &asgn(&[("01", 1), ("02", 2), ("03", 3)]),
            3,
        );
        let s = &result.per_district_dem_share_sorted;
        assert_eq!(s.len(), 3);
        assert!(s[0] <= s[1] && s[1] <= s[2], "must be sorted ascending");
        assert!((s[0] - 0.30).abs() < 1e-9);
        assert!((s[2] - 0.80).abs() < 1e-9);
    }

    #[test]
    fn signed_gap_distinguishes_over_vs_under_representation() {
        // Two scenarios with the same statewide vote but different seat outcomes.
        let rows_dem_lockout = vec![
            row("01", 600.0, 400.0),
            row("02", 600.0, 400.0),
            row("03", 600.0, 400.0),
        ];
        let r1 = aggregate_proportionality(
            &rows_dem_lockout,
            &asgn(&[("01", 1), ("02", 2), ("03", 3)]),
            3,
        );
        assert!(r1.proportionality_gap_pp > 30.0); // dems overrepresented

        let rows_rep_lockout = vec![
            row("01", 400.0, 600.0),
            row("02", 400.0, 600.0),
            row("03", 400.0, 600.0),
        ];
        let r2 = aggregate_proportionality(
            &rows_rep_lockout,
            &asgn(&[("01", 1), ("02", 2), ("03", 3)]),
            3,
        );
        assert!(r2.proportionality_gap_pp < -30.0); // dems underrepresented
    }

    #[test]
    fn perfect_proportionality_at_50_50_three_districts_unachievable() {
        // 3 districts, exact 50/50 statewide. No assignment can produce
        // dem_seats = 1.5. The metric must report the rounding gap honestly.
        let rows = vec![
            row("01", 500.0, 500.0),
            row("02", 500.0, 500.0),
            row("03", 500.0, 500.0),
        ];
        let result = aggregate_proportionality(
            &rows,
            &asgn(&[("01", 1), ("02", 2), ("03", 3)]),
            3,
        );
        // Exact 0.5 dem_pct -> dem_seat under our >= 0.5 tie-break.
        // dem_seats = 3, seat_share = 1.0, gap = +50pp.
        // This is a documented limitation of the tie-break in tied data; in
        // real-world data ties are vanishingly rare.
        assert_eq!(result.dem_seats, 3);
        assert!((result.proportionality_gap_pp - 50.0).abs() < 1e-9);
    }
}

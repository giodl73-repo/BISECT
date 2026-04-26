use std::collections::HashMap;
use serde::Serialize;

use crate::analyzer::{Analyzer, AnalyzerContext};
use crate::demographic::{DemographicAnalyzer, DemographicDistrict};
use crate::political::{PoliticalAnalyzer, PoliticalDistrict};
use crate::urban::{UrbanAnalyzer, UrbanDistrict};

#[derive(Debug, Clone, Serialize)]
pub struct SummaryDistrict {
    pub district: usize,
    pub total_pop: Option<i64>,
    pub ideal_pop: Option<i64>,
    pub pop_deviation_pct: Option<f64>,
    pub pop_balance_ok: Option<bool>,   // |deviation| <= 0.005
    pub pct_minority: Option<f64>,
    pub is_majority_minority: Option<bool>,
    pub dem_pct: Option<f64>,
    pub lean_dem: Option<bool>,
    pub largest_city: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryResult {
    pub analyzer: &'static str,
    pub population_balance_valid: bool,
    pub districts: Vec<SummaryDistrict>,
}

/// Merge per-district data from sub-analyzers into a summary district.
pub fn merge_district(
    district: usize,
    demo: Option<&DemographicDistrict>,
    pol: Option<&PoliticalDistrict>,
    ideal_pop: Option<i64>,
    urban: Option<&UrbanDistrict>,
) -> SummaryDistrict {
    let total_pop = demo.map(|d| d.total_pop);
    let pop_deviation_pct = total_pop.and_then(|tp| ideal_pop.map(|ip| {
        if ip == 0 { 0.0 } else { (tp - ip).abs() as f64 / ip as f64 }
    }));
    let pop_balance_ok = pop_deviation_pct.map(|dev| dev <= 0.005);

    SummaryDistrict {
        district,
        total_pop,
        ideal_pop,
        pop_deviation_pct,
        pop_balance_ok,
        pct_minority: demo.map(|d| d.pct_minority),
        is_majority_minority: demo.map(|d| d.is_majority_minority),
        dem_pct: pol.map(|p| p.dem_pct),
        lean_dem: pol.map(|p| p.lean_dem),
        largest_city: urban.and_then(|u| u.largest_city.clone()),
    }
}

pub struct SummaryAnalyzer;

impl Analyzer for SummaryAnalyzer {
    type Output = SummaryResult;

    fn name() -> &'static str { "summary" }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        // Run sub-analyzers (best-effort: political/urban may be unavailable)
        let demo_result = DemographicAnalyzer::run(ctx).ok();
        let pol_result = PoliticalAnalyzer::run(ctx).ok();
        let urban_result = UrbanAnalyzer::run(ctx).ok();

        // Build lookup maps
        let demo_map: HashMap<usize, &DemographicDistrict> = demo_result.as_ref()
            .map(|r| r.districts.iter().map(|d| (d.district, d)).collect())
            .unwrap_or_default();

        let pol_map: HashMap<usize, &PoliticalDistrict> = pol_result.as_ref()
            .filter(|r| r.available)
            .map(|r| r.districts.iter().map(|d| (d.district, d)).collect())
            .unwrap_or_default();

        let urban_map: HashMap<usize, &UrbanDistrict> = urban_result.as_ref()
            .filter(|r| r.available)
            .map(|r| r.districts.iter().map(|d| (d.district, d)).collect())
            .unwrap_or_default();

        // Compute ideal_pop from demographic totals
        let state_total_pop: i64 = demo_map.values().map(|d| d.total_pop).sum();
        let ideal_pop = if ctx.num_districts > 0 {
            Some(state_total_pop / ctx.num_districts as i64)
        } else {
            None
        };

        let mut districts: Vec<SummaryDistrict> = (1..=ctx.num_districts).map(|d| {
            merge_district(
                d,
                demo_map.get(&d).copied(),
                pol_map.get(&d).copied(),
                ideal_pop,
                urban_map.get(&d).copied(),
            )
        }).collect();
        districts.sort_by_key(|d| d.district);

        let population_balance_valid = districts.iter()
            .all(|d| d.pop_balance_ok.unwrap_or(true));

        Ok(SummaryResult {
            analyzer: "summary",
            population_balance_valid,
            districts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demographic::DemographicDistrict;
    use crate::political::PoliticalDistrict;
    use crate::urban::UrbanDistrict;

    fn make_demo(district: usize, total_pop: i64, pct_minority: f64) -> DemographicDistrict {
        DemographicDistrict {
            district,
            total_pop,
            pct_white: 1.0 - pct_minority,
            pct_black: 0.0,
            pct_asian: 0.0,
            pct_hispanic: 0.0,
            pct_other: 0.0,
            pct_minority,
            is_majority_minority: pct_minority >= 0.50,
            pop_basis: "total_population",
        }
    }

    fn make_pol(district: usize, dem_pct: f64) -> PoliticalDistrict {
        let rep_pct = 1.0 - dem_pct;
        PoliticalDistrict {
            district,
            total_votes: 1000.0,
            dem_votes: dem_pct * 1000.0,
            rep_votes: rep_pct * 1000.0,
            dem_pct,
            rep_pct,
            margin: dem_pct - rep_pct,
            lean_dem: dem_pct >= 0.5,
            is_uncontested: false,
        }
    }

    fn make_urban(district: usize, city: Option<&str>, pop: u64) -> UrbanDistrict {
        UrbanDistrict {
            district,
            largest_city: city.map(|s| s.to_string()),
            largest_city_pop: pop,
            num_places: if city.is_some() { 1 } else { 0 },
        }
    }

    #[test]
    fn test_summary_pop_balance_computed() {
        // 2 districts with pops 1000 and 1010 → ideal=1005, deviation OK
        let d1 = make_demo(1, 1000, 0.1);
        let d2 = make_demo(2, 1010, 0.1);
        let ideal = Some((1000 + 1010) / 2);

        let s1 = merge_district(1, Some(&d1), None, ideal, None);
        let s2 = merge_district(2, Some(&d2), None, ideal, None);

        assert_eq!(s1.ideal_pop, ideal);
        assert!(s1.pop_balance_ok.unwrap_or(false));
        assert!(s2.pop_balance_ok.unwrap_or(false));
    }

    #[test]
    fn test_summary_imbalanced_districts_flagged() {
        // 2 districts 1000 and 1100 → deviation 9.5% → pop_balance_ok=false
        let d1 = make_demo(1, 1000, 0.1);
        let d2 = make_demo(2, 1100, 0.1);
        let ideal = Some((1000 + 1100) / 2);  // 1050

        let s1 = merge_district(1, Some(&d1), None, ideal, None);
        let s2 = merge_district(2, Some(&d2), None, ideal, None);

        assert!(!s1.pop_balance_ok.unwrap_or(true));
        assert!(!s2.pop_balance_ok.unwrap_or(true));

        // population_balance_valid = false
        let population_balance_valid = [&s1, &s2].iter()
            .all(|d| d.pop_balance_ok.unwrap_or(true));
        assert!(!population_balance_valid);
    }

    #[test]
    fn test_merge_district_with_all_sub_results() {
        let demo = make_demo(1, 1000, 0.05);
        let pol = make_pol(1, 0.55);
        let urban = make_urban(1, Some("Burlington"), 45000);
        let s = merge_district(1, Some(&demo), Some(&pol), Some(1005), Some(&urban));
        assert_eq!(s.district, 1);
        assert!((s.dem_pct.unwrap() - 0.55).abs() < 1e-6);
        assert_eq!(s.largest_city.as_deref(), Some("Burlington"));
    }

    #[test]
    fn test_merge_district_partial_inputs_no_panic() {
        // Only demographic provided
        let demo = make_demo(1, 800, 0.2);
        let s = merge_district(1, Some(&demo), None, None, None);
        assert!(s.dem_pct.is_none());
        assert!(s.largest_city.is_none());
        assert_eq!(s.total_pop, Some(800));
    }
}

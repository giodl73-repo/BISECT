use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Context;

use crate::analyzer::{Analyzer, AnalyzerContext};

#[derive(Debug, Deserialize)]
struct PlaceRow {
    #[serde(rename = "GEOID")]
    geoid: String,
    place_name: Option<String>,
    place_pop: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrbanDistrict {
    pub district: usize,
    pub largest_city: Option<String>,
    pub largest_city_pop: u64,
    pub num_places: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrbanResult {
    pub analyzer: &'static str,
    pub available: bool,
    pub districts: Vec<UrbanDistrict>,
}

pub fn aggregate_urban(
    rows: &[PlaceRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> UrbanResult {
    // For each district, track best (largest_city_pop, largest_city_name) and num_places
    let mut district_places: HashMap<usize, Vec<(Option<String>, u64)>> = HashMap::new();
    for d in 1..=num_districts {
        district_places.insert(d, vec![]);
    }

    let mut unmatched = 0usize;
    for row in rows {
        if let Some(&district) = assignments.get(&row.geoid) {
            district_places.entry(district).or_default().push((row.place_name.clone(), row.place_pop));
        } else {
            unmatched += 1;
        }
    }

    if unmatched > 0 {
        eprintln!("WARNING: {unmatched} place rows had no assignment match");
    }

    let mut districts: Vec<UrbanDistrict> = district_places.into_iter().map(|(district, places)| {
        let num_places = places.len();
        let largest = places.iter().max_by_key(|(_, pop)| *pop);
        let (largest_city, largest_city_pop) = match largest {
            Some((name, pop)) => (name.clone().filter(|n| !n.is_empty()), *pop),
            None => (None, 0),
        };
        UrbanDistrict { district, largest_city, largest_city_pop, num_places }
    }).collect();
    districts.sort_by_key(|d| d.district);

    UrbanResult {
        analyzer: "urban",
        available: true,
        districts,
    }
}

pub struct UrbanAnalyzer;

impl Analyzer for UrbanAnalyzer {
    type Output = UrbanResult;

    fn name() -> &'static str { "urban" }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        let state_lower = ctx.state_code.to_lowercase();
        // output_root is already outputs/{version}; places data lives at data/{year}/places/
        // Also try state_name (vermont) and state_code_lower (vt) prefixes
        let state_name_lower = ctx.state_name.replace(' ', "_");
        let places_dir = ctx.output_root.join("data").join(ctx.year).join("places");
        let csv_path_candidates = [
            places_dir.join(format!("{state_name_lower}_places_{}.csv", ctx.year)),
            places_dir.join(format!("{state_lower}_places_{}.csv", ctx.year)),
        ];
        let csv_path = csv_path_candidates.iter().find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| csv_path_candidates[0].clone());

        if !csv_path.exists() {
            eprintln!("WARNING: urban places data not found at {}", csv_path.display());
            return Ok(UrbanResult {
                analyzer: "urban",
                available: false,
                districts: vec![],
            });
        }

        let mut rdr = csv::Reader::from_path(&csv_path)
            .with_context(|| format!("cannot open places CSV: {}", csv_path.display()))?;

        let rows: Vec<PlaceRow> = rdr.deserialize()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to parse places CSV rows")?;

        Ok(aggregate_urban(&rows, ctx.assignments, ctx.num_districts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_place_row(geoid: &str, place_name: Option<&str>, pop: u64) -> PlaceRow {
        PlaceRow {
            geoid: geoid.to_string(),
            place_name: place_name.map(|s| s.to_string()),
            place_pop: pop,
        }
    }

    fn hashmap(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_urban_largest_city() {
        // Burlington (45000) beats Montpelier (8000)
        let rows = vec![
            make_place_row("50001", Some("Burlington"), 45000),
            make_place_row("50002", Some("Montpelier"), 8000),
        ];
        let assignments = hashmap(&[("50001", 1), ("50002", 1)]);
        let result = aggregate_urban(&rows, &assignments, 1);
        assert_eq!(result.districts[0].largest_city.as_deref(), Some("Burlington"));
        assert_eq!(result.districts[0].largest_city_pop, 45000);
        assert_eq!(result.districts[0].num_places, 2);
    }

    #[test]
    fn test_urban_missing_csv_returns_unavailable() {
        use std::path::PathBuf;
        let assignments = hashmap(&[]);
        let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let ctx = AnalyzerContext {
            assignments: &assignments,
            state_name: "vermont",
            state_code: "VT",
            year: "2020",
            version: "v1",
            num_districts: 1,
            data_root: &nonexistent,
            output_root: &nonexistent,
        };
        let result = UrbanAnalyzer::run(&ctx);
        assert!(result.is_ok());
        let ur = result.unwrap();
        assert!(!ur.available);
        assert!(ur.districts.is_empty());
    }
}

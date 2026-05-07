/// PopulationSource enum + CSV loading utilities.
///
/// Spec 1: `--population-source total|vap|cvap` controls which CSV column
/// is used as the tract weight for METIS partitioning.
///
/// Total: default, uses total census population (P001001)
/// Vap: Voting Age Population — eligible voters, not residents
/// Cvap: Citizen Voting Age Population — tightest legal standard
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PopulationSource {
    #[default]
    Total,
    Vap,
    Cvap,
}

impl std::fmt::Display for PopulationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PopulationSource::Total => write!(f, "total"),
            PopulationSource::Vap => write!(f, "vap"),
            PopulationSource::Cvap => write!(f, "cvap"),
        }
    }
}

impl PopulationSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "total" => Some(Self::Total),
            "vap" => Some(Self::Vap),
            "cvap" => Some(Self::Cvap),
            _ => None,
        }
    }

    pub fn column_name(&self) -> &'static str {
        match self {
            PopulationSource::Total => "total",
            PopulationSource::Vap => "vap",
            PopulationSource::Cvap => "cvap",
        }
    }
}

/// Load population weights from a demographics CSV for the given source.
///
/// CSV must have: geoid,total[,vap][,cvap] columns.
/// Returns HashMap<geoid, weight> for the requested geoids.
///
/// Errors clearly reference `--population-source` flag when a column is missing.
pub fn load_population_weights(
    source: PopulationSource,
    csv_bytes: &[u8],
    geoids: &[String],
) -> Result<HashMap<String, u64>, String> {
    let col = source.column_name();
    let geoid_set: std::collections::HashSet<&str> =
        geoids.iter().map(|s| s.as_str()).collect();

    let mut reader = csv::Reader::from_reader(csv_bytes);
    let headers = reader
        .headers()
        .map_err(|e| format!("CSV header error: {e}"))?
        .clone();

    // Find column indices
    let geoid_idx = headers
        .iter()
        .position(|h| h == "geoid")
        .ok_or_else(|| "CSV missing 'geoid' column".to_string())?;

    let col_idx = headers
        .iter()
        .position(|h| h == col)
        .ok_or_else(|| {
            format!(
                "CSV missing '{}' column required by --population-source {}. \
                 Available columns: {}",
                col,
                col,
                headers.iter().collect::<Vec<_>>().join(", ")
            )
        })?;

    let mut weights = HashMap::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let geoid = record
            .get(geoid_idx)
            .ok_or("missing geoid field in row")?
            .to_string();
        if geoid_set.is_empty() || geoid_set.contains(geoid.as_str()) {
            let val: u64 = record
                .get(col_idx)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            weights.insert(geoid, val);
        }
    }
    Ok(weights)
}

/// Check a single district's population balance at a given tolerance.
pub fn check_balance(district_pop: usize, ideal: usize, tolerance: f64) -> Result<(), String> {
    let dev = (district_pop as f64 - ideal as f64).abs() / ideal as f64;
    if dev > tolerance {
        Err(format!(
            "district population {} deviates {:.3} from ideal {} (tolerance ±{:.3})",
            district_pop, dev, ideal, tolerance
        ))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests — Task 4
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_source_vap_loads_from_demographics_csv() {
        let csv_content = "geoid,total,vap,cvap\n53033000100,1000,750,600\n";
        let weights = load_population_weights(
            PopulationSource::Vap,
            csv_content.as_bytes(),
            &["53033000100".to_string()],
        )
        .unwrap();
        assert_eq!(weights["53033000100"], 750);
    }

    #[test]
    fn test_population_source_cvap_loads_from_demographics_csv() {
        let csv_content = "geoid,total,vap,cvap\n53033000100,1000,750,600\n";
        let weights = load_population_weights(
            PopulationSource::Cvap,
            csv_content.as_bytes(),
            &["53033000100".to_string()],
        )
        .unwrap();
        assert_eq!(weights["53033000100"], 600);
    }

    #[test]
    fn test_population_source_vap_missing_column_errors() {
        let csv_content = "geoid,total\n53033000100,1000\n";
        let result = load_population_weights(
            PopulationSource::Vap,
            csv_content.as_bytes(),
            &["53033000100".to_string()],
        );
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("vap"), "Error must name the missing column");
        assert!(
            msg.contains("--population-source"),
            "Error must reference the flag"
        );
    }

    #[test]
    fn test_population_source_total_is_default() {
        assert_eq!(PopulationSource::default(), PopulationSource::Total);
    }

    #[test]
    fn test_balance_tolerance_5pct_passes_where_halfpct_fails() {
        let ideal = 1000usize;
        let district_pop = 1030usize; // 3% above ideal
        assert!(check_balance(district_pop, ideal, 0.05).is_ok());
        assert!(check_balance(district_pop, ideal, 0.005).is_err());
    }

    #[test]
    fn test_population_source_from_str() {
        assert_eq!(PopulationSource::from_str("total"), Some(PopulationSource::Total));
        assert_eq!(PopulationSource::from_str("vap"), Some(PopulationSource::Vap));
        assert_eq!(PopulationSource::from_str("cvap"), Some(PopulationSource::Cvap));
        assert_eq!(PopulationSource::from_str("VAP"), Some(PopulationSource::Vap));
        assert_eq!(PopulationSource::from_str("unknown"), None);
    }
}

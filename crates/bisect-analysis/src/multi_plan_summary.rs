//! Native multi-label comparison projection and exact table/CSV formatting.
use serde_json::Value;
use std::fmt::Write;
#[derive(Debug)]
pub struct PlanRow {
    pub label: String,
    pub mean_pp: Option<f64>,
    pub max_dev_pct: Option<f64>,
    pub county_splits: Option<u64>,
    pub contiguous: Option<bool>,
    pub balance_tol: Option<f64>,
}

pub fn project(label: &str, summary: Option<&Value>, compactness: Option<&Value>, splits: Option<&Value>, contiguity: Option<&Value>, balance_tol: Option<f64>) -> PlanRow {
        // summary.json → population deviation
        let max_dev_pct = summary.and_then(|v| {
            v.get("max_deviation_pct")
                .and_then(|d| d.as_f64())
                .or_else(|| {
                    v.get("population_max_deviation_pct")
                        .and_then(|d| d.as_f64())
                })
        });

        // compactness.json → mean polsby_popper
        let mean_pp = compactness.and_then(|v| {
            v.get("districts").and_then(|d| d.as_array()).map(|arr| {
                let vals: Vec<f64> = arr
                    .iter()
                    .filter_map(|d| d.get("polsby_popper").and_then(|p| p.as_f64()))
                    .collect();
                if vals.is_empty() {
                    return f64::NAN;
                }
                vals.iter().sum::<f64>() / vals.len() as f64
            })
        });

        // splits.json → total splits count
        let county_splits = splits.and_then(|v| {
            v.get("counties")
                .and_then(|c| c.get("split"))
                .and_then(|s| s.as_u64())
        });

        // contiguity.json → all_contiguous
        let contiguous = contiguity
            
            .and_then(|v| v.get("all_contiguous").and_then(|c| c.as_bool()));


    PlanRow { label: label.into(), mean_pp, max_dev_pct, county_splits, contiguous, balance_tol }
}
pub fn render(rows: &[PlanRow], is_csv: bool) -> String {
    let mut output=String::new();
    if is_csv {
        writeln!(&mut output,"Label,Mean PP,Max Dev%,County Splits,Bal Tol%,Contiguous").unwrap();
        for row in rows {
            let pp: String = row
                .mean_pp
                .map_or_else(|| "N/A".to_string(), |v| format!("{:.3}", v));
            let dev: String = row
                .max_dev_pct
                .map_or_else(|| "N/A".to_string(), |v| format!("{:.1}%", v));
            let splits: String = row
                .county_splits
                .map_or_else(|| "N/A".to_string(), |v| v.to_string());
            let tol: String = row
                .balance_tol
                .map_or_else(|| "-".to_string(), |v| format!("{:.1}%", v));
            let cont: String = row.contiguous.map_or_else(
                || "N/A".to_string(),
                |v| {
                    if v {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    }
                },
            );
            writeln!(&mut output,"{},{},{},{},{},{}", row.label, pp, dev, splits, tol, cont).unwrap();
        }
    } else {
        // Table format
        let col1 = rows
            .iter()
            .map(|r| r.label.len())
            .max()
            .unwrap_or(5)
            .max(23);
        writeln!(&mut output,
            "{:<col1$} | Mean PP | Max Dev% | Splits | Bal Tol% | Contiguous",
            "Label",
            col1 = col1
        ).unwrap();
        writeln!(&mut output,
            "{:-<col1$}-+---------+----------+--------+----------+------------",
            "",
            col1 = col1
        ).unwrap();
        for row in rows {
            let pp: String = row
                .mean_pp
                .map_or_else(|| " N/A ".to_string(), |v| format!(" {:.3} ", v));
            let dev: String = row
                .max_dev_pct
                .map_or_else(|| "  N/A  ".to_string(), |v| format!("  {:.1}% ", v));
            let splits: String = row
                .county_splits
                .map_or_else(|| "  N/A ".to_string(), |v| format!("  {:3}  ", v));
            let tol: String = row
                .balance_tol
                .map_or_else(|| "   -    ".to_string(), |v| format!("  {:.1}%  ", v));
            let cont: String = row.contiguous.map_or_else(
                || "  N/A ".to_string(),
                |v| {
                    if v {
                        "  Yes ".to_string()
                    } else {
                        "  No  ".to_string()
                    }
                },
            );
            writeln!(&mut output,
                "{:<col1$} |{pp}|{dev}|{splits}|{tol}|{cont}",
                row.label,
                col1 = col1
            ).unwrap();
        }
    }

    output
}

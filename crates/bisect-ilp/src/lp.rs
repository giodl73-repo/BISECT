//! Deterministic CPLEX-LP export for the U.16 branch-and-cut master problem.
//!
//! This is the solver-facing master formulation: assignment, population
//! balance, and edge-cut linearization. Connectivity is enforced by lazy or
//! iterative cuts, so it is intentionally not baked into this base LP file.

use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LpExportError {
    #[error("population length {pop_len} does not match adjacency length {n_vertices}")]
    PopulationLengthMismatch { n_vertices: usize, pop_len: usize },
    #[error("district count k must be positive")]
    EmptyDistrictSet,
    #[error("adjacency contains out-of-range edge {from}->{to} for {n_vertices} vertices")]
    InvalidNeighbor {
        from: usize,
        to: usize,
        n_vertices: usize,
    },
}

pub fn master_lp_string(
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    pop_tolerance: f64,
) -> Result<String, LpExportError> {
    if pop.len() != adjacency.len() {
        return Err(LpExportError::PopulationLengthMismatch {
            n_vertices: adjacency.len(),
            pop_len: pop.len(),
        });
    }
    if k == 0 {
        return Err(LpExportError::EmptyDistrictSet);
    }

    let n = adjacency.len();
    let edges = undirected_edges(adjacency)?;
    let total_pop: i64 = pop.iter().sum();
    let ideal = total_pop as f64 / k as f64;
    let lower = ideal * (1.0 - pop_tolerance);
    let upper = ideal * (1.0 + pop_tolerance);

    let mut out = String::new();
    writeln!(&mut out, "\\ BISECT U.16 branch-and-cut master LP").unwrap();
    writeln!(
        &mut out,
        "\\ Connectivity is enforced by lazy/iterative cuts, not this base master."
    )
    .unwrap();
    writeln!(&mut out, "Minimize").unwrap();
    write!(&mut out, " obj:").unwrap();
    if edges.is_empty() {
        writeln!(&mut out, " 0").unwrap();
    } else {
        for (idx, &(from, to)) in edges.iter().enumerate() {
            if idx == 0 {
                write!(&mut out, " z_{}_{}", from, to).unwrap();
            } else {
                write!(&mut out, " + z_{}_{}", from, to).unwrap();
            }
        }
        writeln!(&mut out).unwrap();
    }

    writeln!(&mut out, "Subject To").unwrap();
    for vertex in 0..n {
        write!(&mut out, " cover_{}:", vertex).unwrap();
        for district in 0..k {
            if district == 0 {
                write!(&mut out, " x_{}_{}", vertex, district).unwrap();
            } else {
                write!(&mut out, " + x_{}_{}", vertex, district).unwrap();
            }
        }
        writeln!(&mut out, " = 1").unwrap();
    }

    for district in 0..k {
        write!(&mut out, " pop_lower_{}:", district).unwrap();
        write_population_expr(&mut out, pop, district);
        writeln!(&mut out, " >= {}", format_lp_number(lower)).unwrap();

        write!(&mut out, " pop_upper_{}:", district).unwrap();
        write_population_expr(&mut out, pop, district);
        writeln!(&mut out, " <= {}", format_lp_number(upper)).unwrap();
    }

    for &(from, to) in &edges {
        for district in 0..k {
            writeln!(
                &mut out,
                " cut_pos_{}_{}_{}: x_{}_{} - x_{}_{} - z_{}_{} <= 0",
                from, to, district, from, district, to, district, from, to
            )
            .unwrap();
            writeln!(
                &mut out,
                " cut_neg_{}_{}_{}: x_{}_{} - x_{}_{} - z_{}_{} <= 0",
                from, to, district, to, district, from, district, from, to
            )
            .unwrap();
        }
    }

    writeln!(&mut out, "Binary").unwrap();
    for vertex in 0..n {
        for district in 0..k {
            writeln!(&mut out, " x_{}_{}", vertex, district).unwrap();
        }
    }
    for &(from, to) in &edges {
        writeln!(&mut out, " z_{}_{}", from, to).unwrap();
    }
    writeln!(&mut out, "End").unwrap();

    Ok(out)
}

fn undirected_edges(adjacency: &[Vec<usize>]) -> Result<Vec<(usize, usize)>, LpExportError> {
    let n = adjacency.len();
    let mut edges = Vec::new();
    for (from, neighbors) in adjacency.iter().enumerate() {
        for &to in neighbors {
            if to >= n {
                return Err(LpExportError::InvalidNeighbor {
                    from,
                    to,
                    n_vertices: n,
                });
            }
            if from < to {
                edges.push((from, to));
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    Ok(edges)
}

fn write_population_expr(out: &mut String, pop: &[i64], district: usize) {
    for (vertex, value) in pop.iter().enumerate() {
        if vertex == 0 {
            write!(out, " {} x_{}_{}", value, vertex, district).unwrap();
        } else if *value >= 0 {
            write!(out, " + {} x_{}_{}", value, vertex, district).unwrap();
        } else {
            write!(out, " - {} x_{}_{}", value.abs(), vertex, district).unwrap();
        }
    }
}

fn format_lp_number(value: f64) -> String {
    if (value.fract()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        let text = format!("{value:.9}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_4_adjacency() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2]]
    }

    #[test]
    fn master_lp_contains_objective_balance_and_binary_sections() {
        let lp = master_lp_string(&path_4_adjacency(), &[100, 100, 100, 100], 2, 0.005).unwrap();
        assert!(lp.contains("Minimize\n obj: z_0_1 + z_1_2 + z_2_3"));
        assert!(lp.contains("cover_0: x_0_0 + x_0_1 = 1"));
        assert!(lp.contains("pop_lower_0: 100 x_0_0 + 100 x_1_0"));
        assert!(lp.contains("cut_pos_0_1_0: x_0_0 - x_1_0 - z_0_1 <= 0"));
        assert!(lp.contains("Binary\n x_0_0"));
        assert!(lp.ends_with("End\n"));
    }

    #[test]
    fn master_lp_is_deterministic() {
        let a = master_lp_string(&path_4_adjacency(), &[100, 100, 100, 100], 2, 0.005).unwrap();
        let b = master_lp_string(&path_4_adjacency(), &[100, 100, 100, 100], 2, 0.005).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn master_lp_rejects_bad_population_length() {
        let err = master_lp_string(&path_4_adjacency(), &[100, 100], 2, 0.005).unwrap_err();
        assert!(matches!(
            err,
            LpExportError::PopulationLengthMismatch {
                n_vertices: 4,
                pop_len: 2
            }
        ));
    }

    #[test]
    fn master_lp_rejects_out_of_range_neighbor() {
        let err = master_lp_string(&[vec![1], vec![2]], &[100, 100], 2, 0.005).unwrap_err();
        assert!(matches!(
            err,
            LpExportError::InvalidNeighbor {
                from: 1,
                to: 2,
                n_vertices: 2
            }
        ));
    }
}

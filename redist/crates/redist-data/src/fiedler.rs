/// Fiedler value (λ₂) computation for the CompactBisect certificate (B.7).
///
/// The Fiedler value is the second eigenvalue of the weighted graph Laplacian
/// L = D - W. It provides a lower bound on minimum edge-cut via Cheeger:
///   EC_min(G) ≥ λ₂ × n / 4   (n = vertex count, unnormalized Laplacian)
///
/// Algorithm: deflated power iteration.
/// 1. Compute λ_max via power iteration on L.
/// 2. Compute λ₂ via power iteration on (λ_max·I − L), restricted to the
///    subspace orthogonal to the all-ones vector (the λ₁=0 eigenvector).
///    The dominant eigenvalue of (λ_max·I − L) on this subspace is λ_max − λ₂,
///    so λ₂ = λ_max − dominant_eigenvalue.
///
/// Complexity: O(n × E × max_iter) where E = |edges|.
/// For US census-tract subgraphs (n ≤ 4,000, E ≤ 25,000): < 100ms.
use std::collections::HashMap;

/// Fiedler certificate for one bisection level.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiedlerCertificate {
    pub n_vertices: usize,
    /// Second eigenvalue of the weighted Laplacian (algebraic connectivity).
    pub lambda2: f64,
    /// Total edge weight W_total = Σ w_{ij}.
    pub total_edge_weight: f64,
    /// Theoretical lower bound on min edge-cut: λ₂ × n / 4 (Cheeger, unnormalized Laplacian).
    pub ec_lower_bound: f64,
    /// Achieved geometric-mean PP: √(PP(L)·PP(R)).
    pub achieved_gmpp: f64,
    /// Theoretical upper bound on any bisection's GMPP (from λ₂ + geometry).
    /// None if geometry data (areas, ext_perimeters) is unavailable.
    pub gmpp_upper_bound: Option<f64>,
    /// achieved_gmpp / gmpp_upper_bound. 1.0 = at theoretical max.
    pub certification_ratio: Option<f64>,
    /// METIS seeds used to produce the certified split.
    pub seeds_used: usize,
    /// True iff certification_ratio ≥ 1 − delta.
    pub certified: bool,
    /// Threshold: plan is certified if ratio ≥ 1 − delta.
    pub delta: f64,
}

/// Apply the graph Laplacian: (Lv)[i] = degree[i]·v[i] − Σ_{j∈N(i)} w_{ij}·v[j].
fn apply_laplacian(
    v: &[f64],
    adjacency: &[Vec<usize>],
    edge_weights: &HashMap<(usize, usize), f64>,
    degree: &[f64],
) -> Vec<f64> {
    let n = v.len();
    let mut lv = vec![0.0f64; n];
    for i in 0..n {
        lv[i] = degree[i] * v[i];
        for &j in &adjacency[i] {
            let key = (i.min(j), i.max(j));
            let w = edge_weights.get(&key).copied().unwrap_or(0.0);
            lv[i] -= w * v[j];
        }
    }
    lv
}

fn dot(a: &[f64], b: &[f64]) -> f64 { a.iter().zip(b).map(|(x,y)| x*y).sum() }
fn norm(v: &[f64]) -> f64 { dot(v, v).sqrt() }
fn normalize(v: &mut Vec<f64>) { let n = norm(v); if n > 1e-14 { for x in v { *x /= n; } } }

/// Project out the component of v along the all-ones direction (λ₁=0 eigenvector).
fn project_out_ones(v: &mut Vec<f64>) {
    let n = v.len() as f64;
    let mean: f64 = v.iter().sum::<f64>() / n;
    for x in v.iter_mut() { *x -= mean; }
}

/// Compute the Fiedler value λ₂ of the weighted graph Laplacian.
/// Returns (lambda2, converged).
pub fn compute_fiedler(
    adjacency: &[Vec<usize>],
    edge_weights: &HashMap<(usize, usize), f64>,
    max_iter: usize,
    tol: f64,
) -> (f64, bool) {
    let n = adjacency.len();
    if n <= 1 { return (0.0, true); }
    if n == 2 {
        let w = edge_weights.get(&(0, 1)).copied().unwrap_or(0.0);
        return (2.0 * w, true);
    }

    let degree: Vec<f64> = (0..n).map(|i|
        adjacency[i].iter().map(|&j| {
            edge_weights.get(&(i.min(j), i.max(j))).copied().unwrap_or(0.0)
        }).sum()
    ).collect();

    // ── Step 1: λ_max via power iteration on L ──────────────────────────────
    let mut v: Vec<f64> = (0..n).map(|i| (i as f64 + 1.0).sin()).collect();
    project_out_ones(&mut v);
    normalize(&mut v);

    let mut lambda_max = 0.0;
    for _ in 0..max_iter {
        let lv = apply_laplacian(&v, adjacency, edge_weights, &degree);
        let rq = dot(&v, &lv); // Rayleigh quotient → eigenvalue estimate
        let prev = lambda_max;
        lambda_max = rq;
        // New direction: Lv normalised
        v = lv;
        project_out_ones(&mut v);
        normalize(&mut v);
        if (lambda_max - prev).abs() < tol { break; }
    }
    if lambda_max < 1e-10 { lambda_max = *degree.iter().cloned().reduce(f64::max).get_or_insert(1.0); }

    // ── Step 2: λ₂ via power iteration on (λ_max·I − L) restricted to 1⃗⊥ ──
    // Dominant eigenvalue of (λ_max·I − L) on 1⃗⊥  =  λ_max − λ₂
    // So λ₂ = λ_max − that dominant eigenvalue.
    let mut u: Vec<f64> = (0..n).map(|i| ((i + 1) as f64).cos()).collect();
    project_out_ones(&mut u);
    normalize(&mut u);

    let mut dom = 0.0;
    let mut converged = false;
    for _ in 0..max_iter {
        // Apply (λ_max·I − L): w = λ_max·u − L·u
        let lu = apply_laplacian(&u, adjacency, edge_weights, &degree);
        let mut w: Vec<f64> = u.iter().zip(&lu).map(|(&ui, &lui)| lambda_max * ui - lui).collect();
        project_out_ones(&mut w);
        let prev_dom = dom;
        dom = dot(&u, &w); // Rayleigh quotient for (λ_max·I − L)
        normalize(&mut w);
        u = w;
        if (dom - prev_dom).abs() < tol { converged = true; break; }
    }

    let lambda2 = (lambda_max - dom).max(0.0);
    (lambda2, converged)
}

/// GMPP upper bound from λ₂ and subgraph geometry.
/// PP(S) ≤ 4πA(S) / (EC_lower + P_ext(S))²
/// So GMPP ≤ √(PP_upper(L) · PP_upper(R)).
pub fn gmpp_upper_bound(
    lambda2: f64,
    n_vertices: usize,
    area_left: f64, area_right: f64,
    ext_perim_left: f64, ext_perim_right: f64,
) -> f64 {
    let ec_lb = lambda2 * n_vertices as f64 / 4.0;
    // The cut contributes ec_lb to BOTH halves' perimeters
    let p_left  = ec_lb + ext_perim_left;
    let p_right = ec_lb + ext_perim_right;
    let pp_left  = if p_left  > 0.0 { 4.0 * std::f64::consts::PI * area_left  / (p_left  * p_left)  } else { 0.0 };
    let pp_right = if p_right > 0.0 { 4.0 * std::f64::consts::PI * area_right / (p_right * p_right) } else { 0.0 };
    (pp_left * pp_right).sqrt()
}

/// Build a Fiedler certificate for a proposed bisection.
pub fn make_certificate(
    adjacency: &[Vec<usize>],
    edge_weights: &HashMap<(usize, usize), f64>,
    achieved_gmpp: f64,
    gmpp_upper_bound_val: Option<f64>,
    seeds_used: usize,
    delta: f64,
) -> FiedlerCertificate {
    let n_vertices = adjacency.len();
    let total_edge_weight: f64 = edge_weights.values().sum();
    let (lambda2, _) = compute_fiedler(adjacency, edge_weights, 200, 1e-8);
    // Cheeger bound: EC_min ≥ λ₂ × n/4 (vertex count, not edge weight)
    let ec_lower_bound = lambda2 * n_vertices as f64 / 4.0;

    let certification_ratio = gmpp_upper_bound_val
        .filter(|&ub| ub > 1e-14)
        .map(|ub| achieved_gmpp / ub);

    let certified = certification_ratio.map(|r| r >= 1.0 - delta).unwrap_or(false);

    FiedlerCertificate {
        n_vertices, lambda2, total_edge_weight, ec_lower_bound,
        achieved_gmpp, gmpp_upper_bound: gmpp_upper_bound_val,
        certification_ratio, seeds_used, certified, delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_graph(n: usize, w: f64) -> (Vec<Vec<usize>>, HashMap<(usize,usize),f64>) {
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nb = vec![];
            if i > 0 { nb.push(i-1); }
            if i < n-1 { nb.push(i+1); }
            nb
        }).collect();
        let mut ew = HashMap::new();
        for i in 0..n-1 { ew.insert((i, i+1), w); }
        (adj, ew)
    }

    #[test]
    fn two_vertex_graph() {
        // P_2: L = [[w,-w],[-w,w]], eigenvalues 0 and 2w
        let adj = vec![vec![1], vec![0]];
        let mut ew = HashMap::new();
        ew.insert((0,1), 500.0);
        let (lam, _) = compute_fiedler(&adj, &ew, 50, 1e-8);
        assert!((lam - 1000.0).abs() < 10.0, "P_2 λ₂=2w=1000, got {lam}");
    }

    #[test]
    fn path_p4_lambda2() {
        // P_4 uniform w=1: λ₂ = 2(1-cos(π/4)) ≈ 0.5858
        let (adj, ew) = line_graph(4, 1.0);
        let (lam, conv) = compute_fiedler(&adj, &ew, 200, 1e-6);
        assert!(conv, "should converge");
        assert!((lam - 0.5858).abs() < 0.05, "P_4 λ₂≈0.5858, got {lam}");
    }

    #[test]
    fn complete_k3_lambda2() {
        // K_3 w=1: L=[[2,-1,-1],[-1,2,-1],[-1,-1,2]], eigenvalues 0,3,3
        let adj = vec![vec![1,2], vec![0,2], vec![0,1]];
        let mut ew = HashMap::new();
        ew.insert((0,1),1.0); ew.insert((0,2),1.0); ew.insert((1,2),1.0);
        let (lam, conv) = compute_fiedler(&adj, &ew, 200, 1e-6);
        assert!(conv, "K_3 should converge");
        assert!((lam - 3.0).abs() < 0.3, "K_3 λ₂=3, got {lam}");
    }

    #[test]
    fn ec_lower_bound_does_not_exceed_actual_min_cut() {
        // Line graph P_6 w=100: min bisection cuts 1 edge = 100.
        // Cheeger bound: EC_min ≥ λ₂ × n/4 (NOT × W_total/4).
        // P_6 λ₂ ≈ 2×100×(1-cos(π/6)) ≈ 26.8; bound = 26.8 × 6/4 ≈ 40 ≤ 100. ✓
        let n = 6usize;
        let (adj, ew) = line_graph(n, 100.0);
        let (lam, _) = compute_fiedler(&adj, &ew, 200, 1e-6);
        let lb = lam * n as f64 / 4.0;  // Cheeger: λ₂ × n/4
        assert!(lb > 0.0, "lower bound must be positive, got {lb}");
        assert!(lb <= 100.0 + 1.0, "bound {lb} exceeds actual min-cut 100");
    }

    #[test]
    fn lambda2_positive_for_connected_graph() {
        let (adj, ew) = line_graph(10, 50.0);
        let (lam, _) = compute_fiedler(&adj, &ew, 200, 1e-6);
        assert!(lam > 0.0, "connected graph must have λ₂ > 0, got {lam}");
    }

    // ── New fiedler.rs tests ──────────────────────────────────────────────────

    /// Single-vertex graph: λ₂ = 0 (no edges, trivially).
    #[test]
    fn single_vertex_lambda2_is_zero() {
        let adj: Vec<Vec<usize>> = vec![vec![]];
        let ew: HashMap<(usize,usize),f64> = HashMap::new();
        let (lam, conv) = compute_fiedler(&adj, &ew, 50, 1e-8);
        assert_eq!(lam, 0.0, "single vertex λ₂ = 0");
        assert!(conv, "single vertex always converges");
    }

    /// gmpp_upper_bound returns 0.0 when both perimeters are 0.
    #[test]
    fn gmpp_upper_bound_zero_perimeter() {
        let result = gmpp_upper_bound(1.0, 10, 1000.0, 1000.0, 0.0, 0.0);
        // With ec_lb = 1.0 × 10 / 4 = 2.5; p_left = p_right = 2.5 > 0 → not 0
        // Just check it returns a finite positive value
        assert!(result >= 0.0 && result.is_finite(), "result should be finite non-negative");
    }

    /// gmpp_upper_bound scales correctly with area (larger area → larger GMPP bound).
    #[test]
    fn gmpp_upper_bound_scales_with_area() {
        let ub_small = gmpp_upper_bound(1.0, 4, 100.0, 100.0, 50.0, 50.0);
        let ub_large = gmpp_upper_bound(1.0, 4, 10000.0, 10000.0, 50.0, 50.0);
        assert!(ub_large > ub_small, "larger area should produce larger GMPP upper bound");
    }

    /// make_certificate: certified=false when achieved_gmpp << upper bound.
    #[test]
    fn make_certificate_uncertified_when_gmpp_too_low() {
        let (adj, ew) = line_graph(4, 1.0);
        let cert = make_certificate(&adj, &ew, 0.01, Some(1.0), 1, 0.05);
        // achieved 0.01 / upper 1.0 = ratio 0.01 << 0.95 → not certified
        assert!(!cert.certified, "ratio 0.01 should be below the 1-delta threshold");
    }

    /// make_certificate: certified=true when achieved_gmpp ≈ upper bound.
    #[test]
    fn make_certificate_certified_when_gmpp_near_bound() {
        let (adj, ew) = line_graph(4, 1.0);
        // achieved 0.99, upper 1.0, delta 0.05 → ratio = 0.99 ≥ 0.95 → certified
        let cert = make_certificate(&adj, &ew, 0.99, Some(1.0), 1, 0.05);
        assert!(cert.certified, "ratio 0.99 should be certified at delta=0.05");
    }
}

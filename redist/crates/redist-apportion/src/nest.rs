//! NestSection compatible factorization spine algorithm.
//!
//! Finds a compatible bisection-tree structure across congressional, senate, and
//! house seat counts so that state senate districts are (ideally) unions of
//! congressional districts, and house districts are finer subdivisions.
//!
//! **Algorithm**:
//!
//! For seat counts C (congressional), S (senate), H (house):
//! 1. Compute `g = gcd(C, S, H)` — the number of "trunk segments" that all three
//!    chambers agree on.
//! 2. Factor `g` into primes (smallest-first) for the **shared trunk** of all three
//!    spines.
//! 3. Append the remaining tail splits: C/g, S/g, H/g for each chamber.
//! 4. The three spines share the trunk prefix and diverge only in the tail.
//!
//! A spine is a `Vec<u32>` of split counts at each level of the bisection tree.
//! The product of a spine equals the seat count for that chamber.
//!
//! **Compatibility score**: 0 = perfectly compatible (g == min(C,S,H)), 100 = fully
//! incompatible (g == 1, no shared structure at all).

use crate::prime::prime_factor_sequence;

// ---------------------------------------------------------------------------
// Arithmetic helpers
// ---------------------------------------------------------------------------

fn gcd2(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd2(b, a % b) }
}

fn gcd3(a: u32, b: u32, c: u32) -> u32 {
    gcd2(gcd2(a, b), c)
}

// ---------------------------------------------------------------------------
// Core spine functions
// ---------------------------------------------------------------------------

/// Returns `(congressional_spine, senate_spine, house_spine)` where each spine
/// is a `Vec<u32>` of split counts at each level of the compatible bisection tree.
///
/// All three spines share a **common trunk** (the factorisation of `gcd(C,S,H)`)
/// followed by chamber-specific tails for the remaining quotients C/g, S/g, H/g.
///
/// For a tail quotient of 1, no extra levels are appended (the chamber ends at
/// the trunk).
///
/// # Panics
/// Panics if any seat count is 0.
pub fn compatible_spines(
    congressional: u32,
    senate: u32,
    house: u32,
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    assert!(congressional >= 1, "congressional must be >= 1");
    assert!(senate >= 1, "senate must be >= 1");
    assert!(house >= 1, "house must be >= 1");

    let g = gcd3(congressional, senate, house);

    // Shared trunk: prime factors of g (smallest first)
    let trunk = prime_factor_sequence(g);

    // Tail for each chamber: prime factors of the remaining quotient
    let c_tail = prime_factor_sequence(congressional / g);
    let s_tail = prime_factor_sequence(senate / g);
    let h_tail = prime_factor_sequence(house / g);

    let c_spine: Vec<u32> = trunk.iter().chain(c_tail.iter()).cloned().collect();
    let s_spine: Vec<u32> = trunk.iter().chain(s_tail.iter()).cloned().collect();
    let h_spine: Vec<u32> = trunk.iter().chain(h_tail.iter()).cloned().collect();

    // A spine of []: the single "root" node already is the leaf, product = 1.
    // Re-add trunk directly (still correct: product of trunk = g, tail brings it
    // to the full seat count).
    (c_spine, s_spine, h_spine)
}

/// Returns a compatibility score in [0, 100]:
/// * **0** = perfectly compatible — `gcd(C,S,H) == min(C,S,H)`, meaning the
///   smallest chamber is entirely contained in the shared trunk.
/// * **100** = fully incompatible — `gcd(C,S,H) == 1`, no shared structure.
///
/// Formula: `(1 − gcd(C,S,H) / min(C,S,H)) × 100`
pub fn spine_compatibility_score(congressional: u32, senate: u32, house: u32) -> f64 {
    assert!(congressional >= 1, "congressional must be >= 1");
    assert!(senate >= 1, "senate must be >= 1");
    assert!(house >= 1, "house must be >= 1");

    let g = gcd3(congressional, senate, house) as f64;
    let m = congressional.min(senate).min(house) as f64;
    (1.0 - g / m) * 100.0
}

// ---------------------------------------------------------------------------
// US state compatibility table
// ---------------------------------------------------------------------------

/// Compatibility record for a single US state.
#[derive(Debug, Clone)]
pub struct StateCompatibility {
    /// Two-letter postal code (e.g. `"CA"`).
    pub state_code: &'static str,
    /// Congressional seat count (2020 apportionment).
    pub congressional: u32,
    /// State senate seat count.
    pub senate: u32,
    /// State house/assembly seat count.
    pub house: u32,
    /// `gcd(congressional, senate, house)`.
    pub gcd: u32,
    /// Compatibility score ∈ [0, 100]; lower is better.
    pub score: f64,
    /// `true` iff all three chambers are exact multiples of `gcd`, **and**
    /// `gcd == min(congressional, senate, house)` (i.e. score == 0.0).
    pub strictly_compatible: bool,
}

/// For all 50 US states (2020 apportionment), compute compatibility scores.
///
/// Returns `Vec<StateCompatibility>` sorted by score ascending (best first).
pub fn us_state_compatibility_table() -> Vec<StateCompatibility> {
    // (state_code, congressional, senate, house)
    // Sources: 2020 apportionment + NCSL chamber sizes
    #[rustfmt::skip]
    const DATA: &[(&str, u32, u32, u32)] = &[
        ("AL",  7,  35, 105),
        ("AK",  1,  20,  40),
        ("AZ",  9,  30,  60),
        ("AR",  4,  35, 100),
        ("CA", 52,  40,  80),
        ("CO",  8,  35,  65),
        ("CT",  5,  36, 151),
        ("DE",  1,  21,  41),
        ("FL", 28,  40, 120),
        ("GA", 14,  56, 180),
        ("HI",  2,  25,  51),
        ("ID",  2,  35,  70),
        ("IL", 17,  59, 118),
        ("IN",  9,  50, 100),
        ("IA",  4,  50, 100),
        ("KS",  4,  40, 125),
        ("KY",  6,  38, 100),
        ("LA",  6,  39, 105),
        ("ME",  2,  35, 151),
        ("MD",  8,  47, 141),
        ("MA",  9,  40, 160),
        ("MI", 13,  38, 110),
        ("MN",  8,  67, 134),
        ("MS",  4,  52, 122),
        ("MO",  8,  34, 163),
        ("MT",  2,  50, 100),
        ("NE",  3,  49,  49),
        ("NV",  4,  21,  42),
        ("NH",  2,  24, 400),
        ("NJ", 12,  40,  80),
        ("NM",  3,  42,  70),
        ("NY", 26,  63, 150),
        ("NC", 14,  50, 120),
        ("ND",  1,  47,  94),
        ("OH", 15,  33,  99),
        ("OK",  5,  48, 101),
        ("OR",  6,  30,  60),
        ("PA", 17,  50, 203),
        ("RI",  2,  38,  75),
        ("SC",  7,  46, 124),
        ("SD",  1,  35,  70),
        ("TN",  9,  33,  99),
        ("TX", 38,  31, 150),
        ("UT",  4,  29,  75),
        ("VT",  1,  30, 150),
        ("VA", 11,  40, 100),
        ("WA", 10,  49,  98),
        ("WV",  2,  34, 100),
        ("WI",  8,  33,  99),
        ("WY",  1,  30,  60),
    ];

    let mut rows: Vec<StateCompatibility> = DATA
        .iter()
        .map(|&(code, c, s, h)| {
            let g = gcd3(c, s, h);
            let score = spine_compatibility_score(c, s, h);
            let strictly_compatible = score == 0.0;
            StateCompatibility {
                state_code: code,
                congressional: c,
                senate: s,
                house: h,
                gcd: g,
                score,
                strictly_compatible,
            }
        })
        .collect();

    // Sort by score ascending (most compatible first), then by state code for
    // deterministic output within tied scores.
    rows.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.state_code.cmp(b.state_code))
    });

    rows
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // gcd helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_gcd2_basic() {
        assert_eq!(gcd2(12, 8), 4);
        assert_eq!(gcd2(7, 3), 1);
        assert_eq!(gcd2(6, 6), 6);
        assert_eq!(gcd2(0, 5), 5);
    }

    #[test]
    fn test_gcd3_basic() {
        assert_eq!(gcd3(12, 8, 4), 4);
        assert_eq!(gcd3(52, 40, 80), 4);
        assert_eq!(gcd3(38, 31, 150), 1);
        assert_eq!(gcd3(6, 30, 60), 6);
        assert_eq!(gcd3(14, 50, 120), 2);
    }

    // -----------------------------------------------------------------------
    // compatible_spines correctness
    // -----------------------------------------------------------------------

    /// Product of a spine must equal the seat count.
    fn spine_product(spine: &[u32]) -> u32 {
        spine.iter().product::<u32>().max(1)
    }

    #[test]
    fn test_ca_spine_product() {
        // CA: C=52, S=40, H=80 — gcd=4, trunk=[2,2]
        let (c, s, h) = compatible_spines(52, 40, 80);
        assert_eq!(spine_product(&c), 52, "CA congressional spine product");
        assert_eq!(spine_product(&s), 40, "CA senate spine product");
        assert_eq!(spine_product(&h), 80, "CA house spine product");
    }

    #[test]
    fn test_ca_trunk() {
        // CA: gcd=4=[2,2] — all three spines must start with [2,2]
        let (c, s, h) = compatible_spines(52, 40, 80);
        assert!(c.starts_with(&[2, 2]), "CA congressional spine starts with trunk [2,2]");
        assert!(s.starts_with(&[2, 2]), "CA senate spine starts with trunk [2,2]");
        assert!(h.starts_with(&[2, 2]), "CA house spine starts with trunk [2,2]");
    }

    #[test]
    fn test_ca_score() {
        // gcd(52,40,80)=4, min=40 → score = (1-4/40)*100 = 90.0
        let score = spine_compatibility_score(52, 40, 80);
        assert!(
            (score - 90.0).abs() < 1e-9,
            "CA score should be 90.0, got {score}"
        );
    }

    #[test]
    fn test_tx_fully_incompatible() {
        // TX: C=38, S=31, H=150 — gcd=1
        // min(38,31,150) = 31 (senate), score = (1 - 1/31)*100 ≈ 96.77%
        // gcd=1 means no shared trunk — fully structurally incompatible.
        let g = gcd3(38, 31, 150);
        assert_eq!(g, 1, "TX gcd should be 1");

        let score = spine_compatibility_score(38, 31, 150);
        let expected = (1.0 - 1.0_f64 / 31.0) * 100.0; // ≈ 96.774%
        assert!(
            (score - expected).abs() < 1e-9,
            "TX score should be ~{expected:.4}, got {score:.4}"
        );

        // Each spine must still have the correct product
        let (c, s, h) = compatible_spines(38, 31, 150);
        assert_eq!(spine_product(&c), 38);
        assert_eq!(spine_product(&s), 31);
        assert_eq!(spine_product(&h), 150);
    }

    #[test]
    fn test_or_perfectly_compatible() {
        // OR: C=6, S=30, H=60 — gcd=6 → score=0%
        let g = gcd3(6, 30, 60);
        assert_eq!(g, 6, "OR gcd should be 6");

        let score = spine_compatibility_score(6, 30, 60);
        assert!(
            score.abs() < 1e-9,
            "OR score should be 0.0, got {score}"
        );

        // All three spines must be identical (same trunk, no tails)
        let (c, s, h) = compatible_spines(6, 30, 60);
        // trunk = [2,3] (factors of 6)
        assert!(c.starts_with(&[2, 3]), "OR congressional spine starts with [2,3]");
        assert!(s.starts_with(&[2, 3]), "OR senate spine starts with [2,3]");
        assert!(h.starts_with(&[2, 3]), "OR house spine starts with [2,3]");

        assert_eq!(spine_product(&c), 6);
        assert_eq!(spine_product(&s), 30);
        assert_eq!(spine_product(&h), 60);
    }

    #[test]
    fn test_nc_partial_compatibility() {
        // NC: C=14, S=50, H=120 — gcd=2 → score=(1-2/14)*100=85.71…
        let g = gcd3(14, 50, 120);
        assert_eq!(g, 2, "NC gcd should be 2");

        let score = spine_compatibility_score(14, 50, 120);
        let expected = (1.0 - 2.0 / 14.0) * 100.0; // ≈ 85.714…
        assert!(
            (score - expected).abs() < 1e-9,
            "NC score should be ~{expected:.3}, got {score:.3}"
        );

        let (c, s, h) = compatible_spines(14, 50, 120);
        // trunk = [2] (factors of gcd=2)
        assert!(c.starts_with(&[2]), "NC congressional spine starts with trunk [2]");
        assert!(s.starts_with(&[2]), "NC senate spine starts with trunk [2]");
        assert!(h.starts_with(&[2]), "NC house spine starts with trunk [2]");

        assert_eq!(spine_product(&c), 14);
        assert_eq!(spine_product(&s), 50);
        assert_eq!(spine_product(&h), 120);
    }

    // -----------------------------------------------------------------------
    // Single-seat states
    // -----------------------------------------------------------------------

    #[test]
    fn test_single_congressional_seat() {
        // AK: C=1 — gcd(1,20,40)=1, trunk is empty
        let (c, s, h) = compatible_spines(1, 20, 40);
        assert_eq!(spine_product(&c), 1);
        assert_eq!(spine_product(&s), 20);
        assert_eq!(spine_product(&h), 40);
    }

    // -----------------------------------------------------------------------
    // Spine product invariant for all 50 states
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_50_states_spine_products() {
        let table = us_state_compatibility_table();
        assert_eq!(table.len(), 50, "should have exactly 50 states");

        for row in &table {
            let (c, s, h) = compatible_spines(row.congressional, row.senate, row.house);
            assert_eq!(
                spine_product(&c), row.congressional,
                "{} congressional spine product mismatch", row.state_code
            );
            assert_eq!(
                spine_product(&s), row.senate,
                "{} senate spine product mismatch", row.state_code
            );
            assert_eq!(
                spine_product(&h), row.house,
                "{} house spine product mismatch", row.state_code
            );
        }
    }

    // -----------------------------------------------------------------------
    // Score invariants
    // -----------------------------------------------------------------------

    #[test]
    fn test_score_range_all_states() {
        let table = us_state_compatibility_table();
        for row in &table {
            assert!(
                row.score >= 0.0 && row.score <= 100.0,
                "{} score out of [0,100]: {}", row.state_code, row.score
            );
        }
    }

    #[test]
    fn test_score_sorted_ascending() {
        let table = us_state_compatibility_table();
        for w in table.windows(2) {
            assert!(
                w[0].score <= w[1].score,
                "table not sorted: {} ({}) > {} ({})",
                w[0].state_code, w[0].score,
                w[1].state_code, w[1].score
            );
        }
    }

    #[test]
    fn test_or_strictly_compatible() {
        let table = us_state_compatibility_table();
        let or_row = table.iter().find(|r| r.state_code == "OR").unwrap();
        assert!(or_row.strictly_compatible, "OR should be strictly compatible");
        assert_eq!(or_row.gcd, 6);
    }

    #[test]
    fn test_tx_not_strictly_compatible() {
        let table = us_state_compatibility_table();
        let tx_row = table.iter().find(|r| r.state_code == "TX").unwrap();
        assert!(!tx_row.strictly_compatible, "TX should not be strictly compatible");
        assert_eq!(tx_row.gcd, 1);
    }

    // -----------------------------------------------------------------------
    // Shared trunk prefix property
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_spines_share_trunk() {
        // For every state, the first len(trunk) elements of all three spines
        // must equal the factorisation of gcd.
        let table = us_state_compatibility_table();
        for row in &table {
            let trunk = prime_factor_sequence(row.gcd);
            let (c, s, h) = compatible_spines(row.congressional, row.senate, row.house);
            assert!(
                c.starts_with(&trunk),
                "{} congressional spine does not start with trunk {:?}: {:?}",
                row.state_code, trunk, c
            );
            assert!(
                s.starts_with(&trunk),
                "{} senate spine does not start with trunk {:?}: {:?}",
                row.state_code, trunk, s
            );
            assert!(
                h.starts_with(&trunk),
                "{} house spine does not start with trunk {:?}: {:?}",
                row.state_code, trunk, h
            );
        }
    }

    // -----------------------------------------------------------------------
    // Specific known GCD values
    // -----------------------------------------------------------------------

    #[test]
    fn test_known_gcd_values() {
        let cases: &[(&str, u32)] = &[
            ("AL",  7), // gcd(7,35,105)=7
            ("AZ",  3), // gcd(9,30,60)=3
            ("CA",  4), // gcd(52,40,80)=4
            ("FL",  4), // gcd(28,40,120)=4
            ("GA",  2), // gcd(14,56,180)=2
            ("ID",  1), // gcd(2,35,70): gcd(2,35)=1 → 1
            ("MT",  2), // gcd(2,50,100)=2
            ("NJ",  4), // gcd(12,40,80)=4
            ("OR",  6), // gcd(6,30,60)=6
            ("TX",  1), // gcd(38,31,150)=1
        ];

        let table = us_state_compatibility_table();
        for &(code, expected_gcd) in cases {
            let row = table.iter().find(|r| r.state_code == code).unwrap();
            assert_eq!(
                row.gcd, expected_gcd,
                "{} gcd: expected {expected_gcd}, got {}", code, row.gcd
            );
        }
    }

    // -----------------------------------------------------------------------
    // AL: gcd(7,35,105)=7 — all senate/house are multiples of congressional
    // -----------------------------------------------------------------------

    #[test]
    fn test_al_all_chambers_multiples() {
        // gcd(7,35,105) = 7 — congressional divides both senate and house
        let g = gcd3(7, 35, 105);
        assert_eq!(g, 7);
        let score = spine_compatibility_score(7, 35, 105);
        // min=7, gcd=7 → score=0
        assert!(score.abs() < 1e-9, "AL score should be 0.0, got {score}");

        let (c, s, h) = compatible_spines(7, 35, 105);
        let trunk = prime_factor_sequence(7); // [7]
        assert!(c.starts_with(&trunk));
        assert!(s.starts_with(&trunk));
        assert!(h.starts_with(&trunk));
        assert_eq!(spine_product(&c), 7);
        assert_eq!(spine_product(&s), 35);
        assert_eq!(spine_product(&h), 105);
    }
}

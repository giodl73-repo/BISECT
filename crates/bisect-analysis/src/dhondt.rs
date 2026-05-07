/// D'Hondt proportional seat allocation for multi-member constituencies.
///
/// Used for party-list proportional representation systems (Germany, Netherlands,
/// party-list versions of Ireland/Malta). Given vote shares and seat count per
/// constituency, allocates seats to parties.
///
/// For STV (Ireland), seat allocation is voter-preference based (different algorithm).
/// This helper covers party-list PR variants.

/// Allocate `seats` seats among parties using D'Hondt method.
/// `votes`: HashMap<party_id, vote_count>
/// Returns: HashMap<party_id, seats_allocated>
pub fn dhondt_allocate(
    votes: &std::collections::HashMap<String, f64>,
    seats: usize,
) -> std::collections::HashMap<String, usize> {
    if seats == 0 || votes.is_empty() {
        return std::collections::HashMap::new();
    }

    let mut allocated: std::collections::HashMap<String, usize> =
        votes.keys().map(|p| (p.clone(), 0)).collect();

    for _ in 0..seats {
        // Find party with highest quotient: votes / (seats_so_far + 1)
        let winner = votes
            .iter()
            .max_by(|(pa, va), (pb, vb)| {
                let qa = *va / (allocated[*pa] + 1) as f64;
                let qb = *vb / (allocated[*pb] + 1) as f64;
                qa.partial_cmp(&qb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(p, _)| p.clone());

        if let Some(party) = winner {
            *allocated.get_mut(&party).unwrap() += 1;
        }
    }
    allocated
}

/// Gallagher index of disproportionality.
/// Measures how proportional the seat allocation is relative to vote shares.
/// 0.0 = perfectly proportional, higher = more disproportional.
pub fn gallagher_index(
    votes: &std::collections::HashMap<String, f64>,
    seats: &std::collections::HashMap<String, usize>,
    total_seats: usize,
) -> f64 {
    let total_votes: f64 = votes.values().sum();
    if total_votes == 0.0 || total_seats == 0 {
        return 0.0;
    }

    let sum_sq: f64 = votes
        .keys()
        .map(|party| {
            let vote_pct = votes.get(party).unwrap_or(&0.0) / total_votes * 100.0;
            let seat_pct =
                (*seats.get(party).unwrap_or(&0) as f64) / total_seats as f64 * 100.0;
            (vote_pct - seat_pct).powi(2)
        })
        .sum();

    (sum_sq / 2.0).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dhondt_basic_two_party() {
        // 60% vs 40% vote share, 5 seats -> 3+2
        let mut votes = std::collections::HashMap::new();
        votes.insert("A".to_string(), 600.0);
        votes.insert("B".to_string(), 400.0);
        let result = dhondt_allocate(&votes, 5);
        assert_eq!(result["A"], 3);
        assert_eq!(result["B"], 2);
    }

    #[test]
    fn test_dhondt_three_party_ireland_style() {
        // 3-seat constituency: 40%/35%/25%
        let mut votes = std::collections::HashMap::new();
        votes.insert("FF".to_string(), 400.0);
        votes.insert("FG".to_string(), 350.0);
        votes.insert("SF".to_string(), 250.0);
        let result = dhondt_allocate(&votes, 3);
        let total: usize = result.values().sum();
        assert_eq!(total, 3, "must allocate exactly 3 seats");
        assert!(
            result.values().all(|&v| v >= 1),
            "each major party should get at least 1 seat: {:?}",
            result
        );
    }

    #[test]
    fn test_dhondt_no_seats() {
        let mut votes = std::collections::HashMap::new();
        votes.insert("A".to_string(), 100.0);
        let result = dhondt_allocate(&votes, 0);
        assert!(result.is_empty() || result.values().all(|&v| v == 0));
    }

    #[test]
    fn test_dhondt_empty_votes() {
        let votes = std::collections::HashMap::new();
        let result = dhondt_allocate(&votes, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_gallagher_perfect_proportionality() {
        // 50%/50% votes, 2/2 seats (4 total) -> Gallagher near 0
        let mut votes = std::collections::HashMap::new();
        votes.insert("A".to_string(), 50.0);
        votes.insert("B".to_string(), 50.0);
        let mut seats = std::collections::HashMap::new();
        seats.insert("A".to_string(), 2usize);
        seats.insert("B".to_string(), 2);
        let g = gallagher_index(&votes, &seats, 4);
        assert!(
            g < 1.0,
            "perfect proportionality should give low Gallagher, got {g}"
        );
    }

    #[test]
    fn test_gallagher_zero_votes() {
        let votes = std::collections::HashMap::new();
        let seats = std::collections::HashMap::new();
        let g = gallagher_index(&votes, &seats, 5);
        assert_eq!(g, 0.0);
    }

    #[test]
    fn test_gallagher_zero_total_seats() {
        let mut votes = std::collections::HashMap::new();
        votes.insert("A".to_string(), 100.0);
        let seats = std::collections::HashMap::new();
        let g = gallagher_index(&votes, &seats, 0);
        assert_eq!(g, 0.0);
    }

    #[test]
    fn test_seats_per_district_coverage_malta_style() {
        // Malta: 5-seat constituencies -- all 5 seats must be allocated
        let mut votes = std::collections::HashMap::new();
        for (party, v) in [("PN", 450.0), ("PL", 480.0), ("AD", 70.0)] {
            votes.insert(party.to_string(), v);
        }
        let result = dhondt_allocate(&votes, 5);
        assert_eq!(result.values().sum::<usize>(), 5);
    }

    #[test]
    fn test_dhondt_single_party_wins_all() {
        let mut votes = std::collections::HashMap::new();
        votes.insert("A".to_string(), 1000.0);
        votes.insert("B".to_string(), 0.0);
        let result = dhondt_allocate(&votes, 3);
        assert_eq!(result.values().sum::<usize>(), 3);
        assert_eq!(result["A"], 3);
    }

    #[test]
    fn test_dhondt_allocates_exact_seat_count() {
        let mut votes = std::collections::HashMap::new();
        votes.insert("X".to_string(), 300.0);
        votes.insert("Y".to_string(), 200.0);
        votes.insert("Z".to_string(), 100.0);
        for seats in [1, 3, 5, 7, 10] {
            let result = dhondt_allocate(&votes, seats);
            assert_eq!(
                result.values().sum::<usize>(),
                seats,
                "must allocate exactly {seats} seats"
            );
        }
    }
}

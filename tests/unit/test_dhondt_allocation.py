"""
Unit tests for D'Hondt proportional seat allocation.

Tests cover:
- Basic two-party allocation (US cases: PA, TX, NC)
- Multi-party allocation (3+ parties)
- Threshold filtering
- Edge cases (ties, single seat, zero votes)
- Historical validation (known European PR results)
"""

import pytest
from apportionment.proportional.dhondt import (
    allocate_seats_dhondt,
    compute_dhondt_quotients,
    compute_proportionality_deviation,
    suggest_threshold
)


class TestBasicAllocation:
    """Test basic two-party D'Hondt allocation."""

    def test_pennsylvania_17_seats(self):
        """Pennsylvania: 17 seats, 51% D, 49% R → 9 D, 8 R."""
        result = allocate_seats_dhondt(
            vote_shares={"Democratic": 0.51, "Republican": 0.49},
            num_seats=17,
            threshold=0.0
        )
        assert result["Democratic"] == 9
        assert result["Republican"] == 8
        assert sum(result.values()) == 17

    def test_texas_38_seats(self):
        """Texas: 38 seats, 46% D, 52% R → 18 D, 20 R."""
        result = allocate_seats_dhondt(
            vote_shares={"Democratic": 0.46, "Republican": 0.52},
            num_seats=38,
            threshold=0.0
        )
        assert result["Democratic"] == 18
        assert result["Republican"] == 20
        assert sum(result.values()) == 38

    def test_north_carolina_14_seats(self):
        """North Carolina: 14 seats, 48% D, 51% R → 7 D, 7 R."""
        result = allocate_seats_dhondt(
            vote_shares={"Democratic": 0.48, "Republican": 0.51},
            num_seats=14,
            threshold=0.0
        )
        # With minor rounding, could be 6-8 or 7-7
        assert 6 <= result["Democratic"] <= 7
        assert 7 <= result["Republican"] <= 8
        assert sum(result.values()) == 14

    def test_california_52_seats(self):
        """California: 52 seats, 63% D, 35% R → ~33 D, ~18 R."""
        result = allocate_seats_dhondt(
            vote_shares={"Democratic": 0.63, "Republican": 0.35},
            num_seats=52,
            threshold=0.0
        )
        # D should get roughly 63% of 52 = ~33 seats
        assert 32 <= result["Democratic"] <= 34
        assert 18 <= result["Republican"] <= 20
        assert sum(result.values()) == 52


class TestMultiPartyAllocation:
    """Test allocation with 3+ parties."""

    def test_three_party_allocation(self):
        """10 seats: 50% A, 30% B, 20% C → 5 A, 3 B, 2 C."""
        result = allocate_seats_dhondt(
            vote_shares={"Party A": 0.50, "Party B": 0.30, "Party C": 0.20},
            num_seats=10,
            threshold=0.0
        )
        assert result["Party A"] == 5
        assert result["Party B"] == 3
        assert result["Party C"] == 2

    def test_four_party_allocation(self):
        """20 seats: 40% A, 30% B, 20% C, 10% D."""
        result = allocate_seats_dhondt(
            vote_shares={
                "Party A": 0.40,
                "Party B": 0.30,
                "Party C": 0.20,
                "Party D": 0.10
            },
            num_seats=20,
            threshold=0.0
        )
        # A should get ~8, B ~6, C ~4, D ~2
        assert 7 <= result["Party A"] <= 9
        assert 5 <= result["Party B"] <= 7
        assert 3 <= result["Party C"] <= 5
        assert 1 <= result["Party D"] <= 3
        assert sum(result.values()) == 20


class TestThresholds:
    """Test threshold filtering."""

    def test_threshold_filters_small_parties(self):
        """Parties below 5% threshold get 0 seats."""
        result = allocate_seats_dhondt(
            vote_shares={
                "Democratic": 0.50,
                "Republican": 0.45,
                "Libertarian": 0.03,
                "Green": 0.02
            },
            num_seats=20,
            threshold=0.05  # 5%
        )
        assert result["Democratic"] > 0
        assert result["Republican"] > 0
        assert result["Libertarian"] == 0
        assert result["Green"] == 0
        assert sum(result.values()) == 20

    def test_natural_threshold(self):
        """Natural threshold 1/(N+1) for 10-seat state."""
        # 10 seats → threshold = 1/11 ≈ 9.1%
        result = allocate_seats_dhondt(
            vote_shares={
                "Party A": 0.50,
                "Party B": 0.35,
                "Party C": 0.08,  # Below 9.1%
                "Party D": 0.07
            },
            num_seats=10,
            threshold=1/11
        )
        assert result["Party A"] > 0
        assert result["Party B"] > 0
        assert result["Party C"] == 0
        assert result["Party D"] == 0

    def test_threshold_exactly_at_boundary(self):
        """Party exactly at threshold should qualify."""
        result = allocate_seats_dhondt(
            vote_shares={"Party A": 0.60, "Party B": 0.40},
            num_seats=10,
            threshold=0.40  # Party B exactly at threshold
        )
        assert result["Party A"] > 0
        assert result["Party B"] > 0


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_single_seat_state(self):
        """Wyoming: 1 seat, majority party wins."""
        result = allocate_seats_dhondt(
            vote_shares={"Republican": 0.70, "Democratic": 0.30},
            num_seats=1,
            threshold=0.0
        )
        assert result["Republican"] == 1
        assert result["Democratic"] == 0

    def test_exact_tie(self):
        """Perfect 50-50 tie."""
        result = allocate_seats_dhondt(
            vote_shares={"Party A": 0.50, "Party B": 0.50},
            num_seats=10,
            threshold=0.0
        )
        # Should split evenly
        assert result["Party A"] == 5
        assert result["Party B"] == 5

    def test_extreme_disparity(self):
        """95% vs 5% split."""
        result = allocate_seats_dhondt(
            vote_shares={"Dominant": 0.95, "Minor": 0.05},
            num_seats=20,
            threshold=0.0
        )
        assert result["Dominant"] == 19
        assert result["Minor"] == 1

    def test_many_small_parties(self):
        """10 parties each with 10% → all get 1 seat in 10-seat state."""
        vote_shares = {f"Party {i}": 0.10 for i in range(10)}
        result = allocate_seats_dhondt(
            vote_shares=vote_shares,
            num_seats=10,
            threshold=0.0
        )
        # Each party should get exactly 1 seat
        for party in vote_shares:
            assert result[party] == 1

    def test_zero_seats_raises_error(self):
        """num_seats=0 should raise ValueError."""
        with pytest.raises(ValueError, match="num_seats must be >= 1"):
            allocate_seats_dhondt(
                vote_shares={"Party A": 1.0},
                num_seats=0
            )

    def test_vote_shares_sum_normalization(self):
        """Vote shares that don't sum to 1.0 should be normalized."""
        # Shares sum to 0.95 (5% missing)
        result = allocate_seats_dhondt(
            vote_shares={"Party A": 0.50, "Party B": 0.45},
            num_seats=10,
            threshold=0.0
        )
        # Should still work (normalized internally)
        assert sum(result.values()) == 10


class TestHistoricalValidation:
    """Validate against known historical PR results."""

    def test_netherlands_2021(self):
        """
        Netherlands 2021 election (simplified):
        150 seats, major parties.
        """
        # Simplified vote shares (actual result more complex)
        result = allocate_seats_dhondt(
            vote_shares={
                "VVD": 0.217,      # 34 seats actual
                "D66": 0.150,      # 24 seats actual
                "PVV": 0.106,      # 17 seats actual
                "CDA": 0.094,      # 15 seats actual
                "SP": 0.059,       # 9 seats actual
                "Other": 0.374     # Remaining parties
            },
            num_seats=150,
            threshold=0.0067  # 1/150 = 0.67% (actual threshold)
        )
        # Check major parties are in reasonable range
        assert 32 <= result["VVD"] <= 36
        assert 22 <= result["D66"] <= 26
        assert 15 <= result["PVV"] <= 19


class TestQuotientComputation:
    """Test quotient computation helper function."""

    def test_quotients_ordered_correctly(self):
        """Quotients should be in descending order."""
        quotients = compute_dhondt_quotients(
            vote_shares={"A": 0.6, "B": 0.4},
            num_seats=5
        )
        # First quotient should be 0.6/1 = 0.6
        assert quotients[0] == (0.6, "A", 1)
        # Second quotient should be 0.4/1 = 0.4
        assert quotients[1] == (0.4, "B", 1)
        # Third quotient should be 0.6/2 = 0.3
        assert quotients[2] == (0.3, "A", 2)

    def test_quotients_count(self):
        """Should generate num_seats * num_parties quotients."""
        quotients = compute_dhondt_quotients(
            vote_shares={"A": 0.5, "B": 0.3, "C": 0.2},
            num_seats=10
        )
        # 3 parties * 10 seats = 30 quotients
        assert len(quotients) == 30


class TestProportionalityMetrics:
    """Test proportionality deviation measurement."""

    def test_perfect_proportionality(self):
        """Perfect proportional allocation has deviation 0."""
        vote_shares = {"A": 0.5, "B": 0.5}
        seat_allocation = {"A": 5, "B": 5}
        deviation = compute_proportionality_deviation(vote_shares, seat_allocation)
        assert deviation == 0.0

    def test_imperfect_proportionality(self):
        """Imperfect allocation has positive deviation."""
        vote_shares = {"A": 0.51, "B": 0.49}
        seat_allocation = {"A": 9, "B": 8}  # 53% vs 47%
        deviation = compute_proportionality_deviation(vote_shares, seat_allocation)
        # Deviation = |0.53 - 0.51| + |0.47 - 0.49| = 0.02 + 0.02 = 0.04
        assert abs(deviation - 0.04) < 0.01

    def test_maximum_deviation(self):
        """Winner-take-all has maximum deviation."""
        vote_shares = {"A": 0.51, "B": 0.49}
        seat_allocation = {"A": 10, "B": 0}  # Winner-take-all
        deviation = compute_proportionality_deviation(vote_shares, seat_allocation)
        # Deviation = |1.0 - 0.51| + |0.0 - 0.49| = 0.49 + 0.49 = 0.98
        assert abs(deviation - 0.98) < 0.01


class TestThresholdSuggestions:
    """Test threshold suggestion helper."""

    def test_threshold_for_common_states(self):
        """Test natural thresholds for typical state sizes."""
        assert abs(suggest_threshold(1) - 0.50) < 0.01    # Wyoming: 50%
        assert abs(suggest_threshold(10) - 0.091) < 0.01  # 9.1%
        assert abs(suggest_threshold(17) - 0.056) < 0.01  # PA: 5.6%
        assert abs(suggest_threshold(52) - 0.019) < 0.01  # CA: 1.9%

    def test_threshold_decreases_with_seats(self):
        """Threshold should decrease as seats increase."""
        t10 = suggest_threshold(10)
        t20 = suggest_threshold(20)
        t50 = suggest_threshold(50)
        assert t10 > t20 > t50


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

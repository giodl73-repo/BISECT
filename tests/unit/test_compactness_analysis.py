"""
Unit tests for compactness analysis functionality.

Tests Polsby-Popper and Reock compactness metrics, geometric calculations,
and district shape analysis.
"""

import pytest
import pandas as pd
import numpy as np
from pathlib import Path
import sys
from shapely.geometry import Polygon, Point
import math

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestCompactnessDataGeneration:
    """Test compactness analysis data generation."""

    def test_generate_compactness_analysis(self, mock_tracts_medium, mock_districts_medium):
        """Test compactness analysis generation for districts."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # Check structure
        assert len(compactness) == 7, "Should have 7 districts"
        assert 'district' in compactness.columns
        assert 'polsby_popper' in compactness.columns
        assert 'reock' in compactness.columns
        assert 'area_sq_km' in compactness.columns
        assert 'perimeter_km' in compactness.columns

    def test_compactness_metrics_range(self, mock_tracts_medium, mock_districts_medium):
        """Test that compactness metrics are in valid range [0, 1]."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # Polsby-Popper should be in [0, 1]
        assert (compactness['polsby_popper'] >= 0.0).all()
        assert (compactness['polsby_popper'] <= 1.0).all()

        # Reock should be in [0, 1]
        assert (compactness['reock'] >= 0.0).all()
        assert (compactness['reock'] <= 1.0).all()

    def test_area_and_perimeter_positive(self, mock_tracts_medium, mock_districts_medium):
        """Test that area and perimeter are positive."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        assert (compactness['area_sq_km'] > 0).all()
        assert (compactness['perimeter_km'] > 0).all()


class TestPolsbyPopperMetric:
    """Test Polsby-Popper compactness metric."""

    def test_polsby_popper_circle(self):
        """Test PP score for perfect circle (should be ~1.0)."""
        # Create circle
        center = Point(0, 0)
        radius = 1.0
        circle = center.buffer(radius)

        area = circle.area
        perimeter = circle.length

        # PP = 4π * A / P^2
        pp = (4 * math.pi * area) / (perimeter ** 2)

        # Circle should have PP ≈ 1.0
        assert abs(pp - 1.0) < 0.01, f"Circle PP should be ~1.0, got {pp:.3f}"

    def test_polsby_popper_square(self):
        """Test PP score for square."""
        # Create square (side length 2)
        square = Polygon([(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)])

        area = square.area  # 4
        perimeter = square.length  # 8

        # PP = 4π * A / P^2 = 4π * 4 / 64 = π/4 ≈ 0.785
        pp = (4 * math.pi * area) / (perimeter ** 2)

        expected_pp = math.pi / 4
        assert abs(pp - expected_pp) < 0.01, f"Square PP should be ~{expected_pp:.3f}, got {pp:.3f}"

    def test_polsby_popper_elongated(self):
        """Test PP score for elongated rectangle (should be low)."""
        # Create elongated rectangle (10 x 1)
        rectangle = Polygon([(0, 0), (10, 0), (10, 1), (0, 1), (0, 0)])

        area = rectangle.area  # 10
        perimeter = rectangle.length  # 22

        # PP = 4π * A / P^2
        pp = (4 * math.pi * area) / (perimeter ** 2)

        # Elongated shape should have low PP (< 0.3)
        assert pp < 0.3, f"Elongated rectangle should have low PP, got {pp:.3f}"


class TestCompactnessComparisons:
    """Test compactness comparisons between districts."""

    def test_compare_district_compactness(self, mock_tracts_medium, mock_districts_medium):
        """Test ranking districts by compactness."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # Rank by Polsby-Popper
        compactness_sorted = compactness.sort_values('polsby_popper', ascending=False)

        # Most compact district
        most_compact = compactness_sorted.iloc[0]
        assert most_compact['polsby_popper'] >= compactness['polsby_popper'].min()

        # Least compact district
        least_compact = compactness_sorted.iloc[-1]
        assert least_compact['polsby_popper'] <= compactness['polsby_popper'].max()


class TestCompactnessImprovements:
    """Test measurement of compactness improvements."""

    def test_measure_compactness_improvement(self):
        """Test measuring improvement from baseline to new districts."""
        # Baseline (current congressional districts)
        baseline = pd.DataFrame({
            'district': [1, 2, 3],
            'polsby_popper': [0.20, 0.18, 0.22],
            'reock': [0.45, 0.42, 0.48],
        })

        # New algorithmic districts
        new_districts = pd.DataFrame({
            'district': [1, 2, 3],
            'polsby_popper': [0.32, 0.28, 0.35],
            'reock': [0.58, 0.52, 0.62],
        })

        # Calculate improvements
        pp_improvement = (new_districts['polsby_popper'].mean() -
                         baseline['polsby_popper'].mean()) / baseline['polsby_popper'].mean()

        reock_improvement = (new_districts['reock'].mean() -
                           baseline['reock'].mean()) / baseline['reock'].mean()

        # Should show improvement
        assert pp_improvement > 0, "PP should improve"
        assert reock_improvement > 0, "Reock should improve"

        # Check percentage improvement
        assert pp_improvement > 0.30, f"PP improvement {pp_improvement:.1%} should be > 30%"


class TestIsoperimetricQuotient:
    """Test isoperimetric quotient calculations."""

    def test_isoperimetric_quotient_relationship(self):
        """Test relationship between area, perimeter, and compactness."""
        # For any shape: 4π * A / P^2 <= 1
        # (Isoperimetric inequality)

        shapes = [
            Polygon([(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]),  # Square
            Polygon([(0, 0), (2, 0), (2, 0.5), (0, 0.5), (0, 0)]),  # Rectangle
            Point(0, 0).buffer(1),  # Circle
        ]

        for shape in shapes:
            area = shape.area
            perimeter = shape.length

            iq = (4 * math.pi * area) / (perimeter ** 2)

            assert 0 <= iq <= 1.01, \
                f"Isoperimetric quotient {iq:.3f} must be in [0, 1]"


class TestPerimeterToAreaRatio:
    """Test perimeter-to-area ratio calculations."""

    def test_perimeter_area_ratio(self, mock_tracts_medium, mock_districts_medium):
        """Test perimeter-to-area ratio calculation."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # Calculate P/A ratio
        compactness['pa_ratio'] = compactness['perimeter_km'] / compactness['area_sq_km']

        # Ratio should be positive
        assert (compactness['pa_ratio'] > 0).all()

        # More compact districts should have lower P/A ratio
        # (less perimeter for given area)
        correlation = compactness[['polsby_popper', 'pa_ratio']].corr().iloc[0, 1]

        # Should have negative correlation
        assert correlation < 0, "PP and P/A ratio should be negatively correlated"


class TestCompactnessStatistics:
    """Test statistical summaries of compactness."""

    def test_compactness_summary_statistics(self, mock_tracts_medium, mock_districts_medium):
        """Test calculation of summary statistics."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # Mean
        mean_pp = compactness['polsby_popper'].mean()
        assert 0 <= mean_pp <= 1

        # Median
        median_pp = compactness['polsby_popper'].median()
        assert 0 <= median_pp <= 1

        # Standard deviation
        std_pp = compactness['polsby_popper'].std()
        assert std_pp >= 0

        # Min/Max
        min_pp = compactness['polsby_popper'].min()
        max_pp = compactness['polsby_popper'].max()
        assert min_pp <= mean_pp <= max_pp


class TestCompactnessValidation:
    """Test validation of compactness data."""

    def test_validate_compactness_columns(self, mock_tracts_medium, mock_districts_medium):
        """Test that all required columns are present."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        required_cols = ['district', 'polsby_popper', 'reock', 'area_sq_km', 'perimeter_km']
        for col in required_cols:
            assert col in compactness.columns, f"Missing required column: {col}"

    def test_validate_compactness_consistency(self, mock_tracts_medium, mock_districts_medium):
        """Test consistency between area, perimeter, and PP score."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        compactness = generate_mock_compactness_analysis(mock_districts_medium)

        # PP = 4π * A / P^2
        # So P^2 = 4π * A / PP
        for _, row in compactness.iterrows():
            area = row['area_sq_km']
            perimeter = row['perimeter_km']
            pp = row['polsby_popper']

            # Calculate expected perimeter from area and PP
            expected_perimeter_sq = (4 * math.pi * area) / pp
            expected_perimeter = math.sqrt(expected_perimeter_sq)

            # Should match (within tolerance for mock data)
            relative_error = abs(perimeter - expected_perimeter) / expected_perimeter
            assert relative_error < 0.1, \
                f"District {row['district']}: Perimeter inconsistent with area and PP"


class TestGeometricProperties:
    """Test geometric property calculations."""

    def test_centroid_calculation(self):
        """Test centroid calculation for districts."""
        # Square with known centroid
        square = Polygon([(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)])
        centroid = square.centroid

        # Centroid should be at (1, 1)
        assert abs(centroid.x - 1.0) < 0.01
        assert abs(centroid.y - 1.0) < 0.01

    def test_bounding_box_calculation(self):
        """Test bounding box calculation."""
        # Create polygon
        polygon = Polygon([(0, 0), (3, 0), (3, 2), (0, 2), (0, 0)])
        bounds = polygon.bounds  # (minx, miny, maxx, maxy)

        assert bounds == (0.0, 0.0, 3.0, 2.0)


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.compactness,
]

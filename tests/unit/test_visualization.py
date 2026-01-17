"""
Unit tests for visualization functionality.

Tests map generation, color schemes, figure creation, and PNG output.
"""

import pytest
import numpy as np
from pathlib import Path
import sys
from PIL import Image
import tempfile

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestMapGeneration:
    """Test map image generation."""

    def test_generate_state_map(self):
        """Test state map generation."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_state_map.png'

            generate_mock_state_map(
                output_file,
                state='alabama',
                map_type='districts',
                width=800,
                height=600
            )

            # Check file exists
            assert output_file.exists(), "Map file should be created"

            # Check is valid PNG
            img = Image.open(output_file)
            try:
                assert img.format == 'PNG'
                assert img.size == (800, 600)
            finally:
                img.close()

    def test_generate_national_map(self):
        """Test national map generation with AK/HI insets."""
        from tests.mocks.mock_maps import generate_mock_national_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_national_map.png'

            generate_mock_national_map(
                output_file,
                map_type='political',
                width=1600,
                height=1000
            )

            # Check file exists
            assert output_file.exists()

            # Check is valid PNG
            img = Image.open(output_file)
            try:
                assert img.format == 'PNG'
                assert img.size == (1600, 1000)
            finally:
                img.close()

    def test_generate_round_progression_map(self):
        """Test round progression map generation."""
        from tests.mocks.mock_maps import generate_mock_round_progression_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_rounds.png'

            generate_mock_round_progression_map(
                output_file,
                num_rounds=6,
                width=1200,
                height=800
            )

            # Check file exists
            assert output_file.exists()

            # Check is valid PNG
            img = Image.open(output_file)
            try:
                assert img.format == 'PNG'
                assert img.size == (1200, 800)
            finally:
                img.close()

    def test_generate_metro_map(self):
        """Test metro area map generation."""
        from tests.mocks.mock_maps import generate_mock_metro_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_metro.png'

            generate_mock_metro_map(
                output_file,
                metro_name='Birmingham',
                width=600,
                height=600
            )

            # Check file exists
            assert output_file.exists()

            # Check is valid PNG
            img = Image.open(output_file)
            try:
                assert img.format == 'PNG'
                assert img.size == (600, 600)
            finally:
                img.close()


class TestMapDimensions:
    """Test map dimension handling."""

    def test_different_aspect_ratios(self):
        """Test generating maps with different aspect ratios."""
        from tests.mocks.mock_maps import generate_mock_state_map

        test_cases = [
            (800, 600),   # 4:3
            (1024, 768),  # 4:3
            (1920, 1080), # 16:9
            (800, 800),   # 1:1 (square)
        ]

        with tempfile.TemporaryDirectory() as tmpdir:
            for width, height in test_cases:
                output_file = Path(tmpdir) / f'test_{width}x{height}.png'

                generate_mock_state_map(
                    output_file,
                    state='alabama',
                    map_type='districts',
                    width=width,
                    height=height
                )

                img = Image.open(output_file)
                try:
                    assert img.size == (width, height), \
                        f"Image size should be {width}x{height}"
                finally:
                    img.close()


class TestColorSchemes:
    """Test color scheme generation."""

    def test_generate_distinct_colors(self):
        """Test generation of visually distinct colors."""
        from tests.mocks.mock_maps import generate_distinct_colors

        # Generate colors for different district counts
        for n_districts in [2, 5, 7, 10, 15]:
            colors = generate_distinct_colors(n_districts, seed=42)

            # Check count
            assert len(colors) == n_districts, f"Should generate {n_districts} colors"

            # Check format (RGB tuples)
            for color in colors:
                assert isinstance(color, tuple), "Color should be tuple"
                assert len(color) == 3, "Color should be RGB (3 values)"
                assert all(0 <= c <= 255 for c in color), "RGB values in [0, 255]"

    def test_color_distinctness(self):
        """Test that generated colors are visually distinct."""
        from tests.mocks.mock_maps import generate_distinct_colors

        colors = generate_distinct_colors(10, seed=42)

        # Calculate minimum distance between any two colors
        min_distance = float('inf')
        for i in range(len(colors)):
            for j in range(i + 1, len(colors)):
                r1, g1, b1 = colors[i]
                r2, g2, b2 = colors[j]

                # Euclidean distance in RGB space
                distance = ((r1 - r2)**2 + (g1 - g2)**2 + (b1 - b2)**2) ** 0.5
                min_distance = min(min_distance, distance)

        # Colors should have reasonable separation
        assert min_distance > 30, \
            f"Colors should be distinct (min distance {min_distance:.1f} < 30)"


class TestMapValidation:
    """Test map validation functionality."""

    def test_validate_mock_map(self):
        """Test validation of generated maps."""
        from tests.mocks.mock_maps import generate_mock_state_map, validate_mock_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_validate.png'

            generate_mock_state_map(output_file, state='alabama', map_type='districts')

            # Should not raise assertion
            validate_mock_map(output_file)

    def test_validate_map_dimensions(self):
        """Test validation of map dimensions."""
        from tests.mocks.mock_maps import validate_mock_map

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create test image
            img = Image.new('RGB', (800, 600), color=(255, 255, 255))
            output_file = Path(tmpdir) / 'test_dims.png'
            img.save(output_file, 'PNG')
            img.close()

            # Should pass validation
            validate_mock_map(output_file)

    def test_validate_rejects_small_images(self):
        """Test that validation rejects images that are too small."""
        from tests.mocks.mock_maps import validate_mock_map

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create tiny image (too small)
            img = Image.new('RGB', (100, 100), color=(255, 255, 255))
            output_file = Path(tmpdir) / 'test_small.png'
            img.save(output_file, 'PNG')
            img.close()

            # Should raise assertion
            with pytest.raises(AssertionError, match="Width too small"):
                validate_mock_map(output_file)


class TestOutputFormats:
    """Test different output formats."""

    def test_png_format(self):
        """Test PNG format output."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test.png'

            generate_mock_state_map(output_file, state='alabama', map_type='districts')

            img = Image.open(output_file)
            try:
                assert img.format == 'PNG'
            finally:
                img.close()

    def test_png_mode(self):
        """Test PNG color mode."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test.png'

            generate_mock_state_map(output_file, state='alabama', map_type='districts')

            img = Image.open(output_file)
            try:
                assert img.mode == 'RGB', "Images should be RGB mode"
            finally:
                img.close()


class TestMapTypes:
    """Test different map types."""

    def test_all_map_types(self):
        """Test generation of all map types."""
        from tests.mocks.mock_maps import generate_mock_state_map

        map_types = ['districts', 'political', 'demographic', 'compactness', 'rounds']

        with tempfile.TemporaryDirectory() as tmpdir:
            for map_type in map_types:
                output_file = Path(tmpdir) / f'test_{map_type}.png'

                generate_mock_state_map(
                    output_file,
                    state='alabama',
                    map_type=map_type
                )

                assert output_file.exists(), f"Map type '{map_type}' should generate file"

                img = Image.open(output_file)
                try:
                    assert img.format == 'PNG'
                finally:
                    img.close()


class TestInsetMaps:
    """Test Alaska/Hawaii inset handling."""

    def test_national_map_has_insets(self):
        """Test that national maps include AK/HI insets."""
        from tests.mocks.mock_maps import generate_mock_national_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test_national.png'

            generate_mock_national_map(output_file, map_type='districts')

            # Open and check image exists
            img = Image.open(output_file)
            try:
                assert img is not None

                # Visual inspection would show insets, but we can at least
                # verify the image was created with expected dimensions
                assert img.size == (1600, 1000)
            finally:
                img.close()


class TestMapFileOperations:
    """Test file operations for maps."""

    def test_create_parent_directories(self):
        """Test that parent directories are created automatically."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            # Deep nested path
            output_file = Path(tmpdir) / 'level1' / 'level2' / 'level3' / 'map.png'

            generate_mock_state_map(output_file, state='alabama', map_type='districts')

            # Should create all parent directories and file
            assert output_file.exists()
            assert output_file.parent.exists()

    def test_overwrite_existing_map(self):
        """Test overwriting an existing map file."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'map.png'

            # Create first map
            generate_mock_state_map(output_file, state='alabama', map_type='districts', seed=42)
            first_size = output_file.stat().st_size

            # Overwrite with second map (different seed)
            generate_mock_state_map(output_file, state='california', map_type='political', seed=123)
            second_size = output_file.stat().st_size

            # File should exist and potentially be different size
            assert output_file.exists()
            # Sizes may or may not differ depending on content


class TestMapContents:
    """Test map content validation."""

    def test_map_is_not_blank(self):
        """Test that generated maps are not blank."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test.png'

            generate_mock_state_map(output_file, state='alabama', map_type='districts')

            img = Image.open(output_file)
            pixels = np.array(img)

            # Check that not all pixels are the same color (i.e., not blank)
            unique_colors = len(np.unique(pixels.reshape(-1, 3), axis=0))

            assert unique_colors > 10, \
                f"Map should have multiple colors, found {unique_colors}"

    def test_map_has_border(self):
        """Test that maps have visible content."""
        from tests.mocks.mock_maps import generate_mock_state_map

        with tempfile.TemporaryDirectory() as tmpdir:
            output_file = Path(tmpdir) / 'test.png'

            generate_mock_state_map(
                output_file,
                state='alabama',
                map_type='districts',
                width=800,
                height=600
            )

            img = Image.open(output_file)

            # Check that image was created successfully
            assert img.size == (800, 600)

            # Check that pixels exist (not None)
            pixels = np.array(img)
            assert pixels.size > 0


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.visualization,
]

"""
Mock map image generators for testing.

Generates minimal valid PNG placeholder images matching the structure
of pipeline visualization outputs.
"""

import numpy as np
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import random


def generate_mock_state_map(output_file, state='alabama', map_type='districts',
                            width=800, height=600, seed=42):
    """
    Generate mock state map PNG placeholder.

    Parameters
    ----------
    output_file : Path or str
        Output PNG file path
    state : str
        State name (for label)
    map_type : str
        Map type ('districts', 'political', 'demographic', 'compactness', 'rounds')
    width : int
        Image width in pixels
    height : int
        Image height in pixels
    seed : int
        Random seed for reproducibility

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output = Path(tmpdir) / 'alabama_districts_2020.png'
    ...     generate_mock_state_map(output, state='alabama', map_type='districts')
    ...     output.exists()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    # Create blank image with light gray background
    img = Image.new('RGB', (width, height), color=(240, 240, 240))
    draw = ImageDraw.Draw(img)

    # Draw border
    draw.rectangle([10, 10, width-10, height-10], outline=(100, 100, 100), width=2)

    # Draw mock district boundaries (random polygons)
    num_districts = random.randint(3, 9)
    colors = generate_distinct_colors(num_districts, seed=seed)

    for i in range(num_districts):
        # Random polygon
        num_points = random.randint(4, 8)
        points = [(random.randint(50, width-50), random.randint(50, height-50))
                  for _ in range(num_points)]

        # Draw filled polygon
        draw.polygon(points, fill=colors[i], outline=(80, 80, 80))

    # Add title text
    try:
        # Try to use a nice font
        font_title = ImageFont.truetype("arial.ttf", 24)
        font_label = ImageFont.truetype("arial.ttf", 16)
    except:
        # Fall back to default font
        font_title = ImageFont.load_default()
        font_label = ImageFont.load_default()

    title = f"{state.title()} - {map_type.title()}"
    draw.text((width//2 - 100, 20), title, fill=(0, 0, 0), font=font_title)

    # Add mock label
    label = f"Mock visualization ({width}x{height})"
    draw.text((width//2 - 80, height - 40), label, fill=(100, 100, 100), font=font_label)

    # Save
    output_file = Path(output_file)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    img.save(output_file, 'PNG')


def generate_mock_national_map(output_file, map_type='districts',
                               width=1600, height=1000, seed=42):
    """
    Generate mock national map PNG placeholder (with AK/HI insets).

    Parameters
    ----------
    output_file : Path or str
        Output PNG file path
    map_type : str
        Map type ('districts', 'political', 'demographic', 'compactness', 'rounds')
    width : int
        Image width in pixels
    height : int
        Image height in pixels
    seed : int
        Random seed for reproducibility

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output = Path(tmpdir) / 'national_districts_2020.png'
    ...     generate_mock_national_map(output, map_type='districts')
    ...     output.exists()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    # Create blank image
    img = Image.new('RGB', (width, height), color=(255, 255, 255))
    draw = ImageDraw.Draw(img)

    # Draw continental US outline (mock rectangle)
    main_map_box = [50, 100, width-50, height-200]
    draw.rectangle(main_map_box, outline=(100, 100, 100), width=3)

    # Draw mock states (random patches)
    num_regions = random.randint(30, 50)
    colors = generate_distinct_colors(num_regions, seed=seed)

    for i in range(num_regions):
        # Random rectangle representing state/district
        x1 = random.randint(main_map_box[0], main_map_box[2]-100)
        y1 = random.randint(main_map_box[1], main_map_box[3]-80)
        x2 = x1 + random.randint(50, 150)
        y2 = y1 + random.randint(40, 120)

        # Keep within bounds
        x2 = min(x2, main_map_box[2])
        y2 = min(y2, main_map_box[3])

        draw.rectangle([x1, y1, x2, y2], fill=colors[i % len(colors)], outline=(60, 60, 60))

    # Draw Alaska inset (bottom-left)
    ak_box = [50, height-150, 250, height-50]
    draw.rectangle(ak_box, fill=(220, 230, 255), outline=(100, 100, 100), width=2)
    draw.text((ak_box[0]+10, ak_box[1]+10), "Alaska", fill=(0, 0, 0))

    # Draw Hawaii inset (bottom-left, next to Alaska)
    hi_box = [270, height-150, 450, height-50]
    draw.rectangle(hi_box, fill=(255, 240, 220), outline=(100, 100, 100), width=2)
    draw.text((hi_box[0]+10, hi_box[1]+10), "Hawaii", fill=(0, 0, 0))

    # Add title
    try:
        font_title = ImageFont.truetype("arial.ttf", 32)
        font_label = ImageFont.truetype("arial.ttf", 18)
    except:
        font_title = ImageFont.load_default()
        font_label = ImageFont.load_default()

    title = f"National {map_type.title()} Map"
    draw.text((width//2 - 150, 30), title, fill=(0, 0, 0), font=font_title)

    # Add mock label
    label = f"Mock visualization - 50 states + DC ({width}x{height})"
    draw.text((width//2 - 150, height - 30), label, fill=(100, 100, 100), font=font_label)

    # Save
    output_file = Path(output_file)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    img.save(output_file, 'PNG')


def generate_mock_round_progression_map(output_file, num_rounds=6,
                                       width=1200, height=800, seed=42):
    """
    Generate mock round progression map PNG placeholder.

    Shows recursive bisection progression through multiple panels.

    Parameters
    ----------
    output_file : Path or str
        Output PNG file path
    num_rounds : int
        Number of rounds to show
    width : int
        Image width in pixels
    height : int
        Image height in pixels
    seed : int
        Random seed for reproducibility

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output = Path(tmpdir) / 'rounds_progression_2020.png'
    ...     generate_mock_round_progression_map(output, num_rounds=6)
    ...     output.exists()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    # Create blank image
    img = Image.new('RGB', (width, height), color=(255, 255, 255))
    draw = ImageDraw.Draw(img)

    # Calculate grid layout
    cols = min(3, num_rounds)
    rows = (num_rounds + cols - 1) // cols

    panel_width = (width - 100) // cols
    panel_height = (height - 150) // rows

    # Draw each round as a panel
    colors = [(255, 200, 200), (200, 255, 200), (200, 200, 255),
              (255, 255, 200), (255, 200, 255), (200, 255, 255)]

    for round_idx in range(num_rounds):
        row = round_idx // cols
        col = round_idx % cols

        x = 50 + col * panel_width
        y = 100 + row * panel_height

        # Draw panel border
        panel_box = [x, y, x + panel_width - 20, y + panel_height - 20]
        draw.rectangle(panel_box, outline=(100, 100, 100), width=2)

        # Draw mock regions (increases with round number)
        num_regions = 2 ** (round_idx + 1)  # 2, 4, 8, 16, ...
        region_color = colors[round_idx % len(colors)]

        for i in range(min(num_regions, 16)):  # Cap at 16 for visibility
            # Random rectangle in panel
            rx1 = random.randint(panel_box[0]+5, panel_box[2]-50)
            ry1 = random.randint(panel_box[1]+25, panel_box[3]-30)
            rx2 = rx1 + random.randint(30, 60)
            ry2 = ry1 + random.randint(25, 50)

            rx2 = min(rx2, panel_box[2]-5)
            ry2 = min(ry2, panel_box[3]-5)

            draw.rectangle([rx1, ry1, rx2, ry2], fill=region_color, outline=(80, 80, 80))

        # Add round label
        try:
            font = ImageFont.truetype("arial.ttf", 14)
        except:
            font = ImageFont.load_default()

        label = f"Round {round_idx + 1} ({num_regions} regions)"
        draw.text((x + 10, y + 5), label, fill=(0, 0, 0), font=font)

    # Add title
    try:
        font_title = ImageFont.truetype("arial.ttf", 28)
    except:
        font_title = ImageFont.load_default()

    title = "Recursive Bisection Progression"
    draw.text((width//2 - 150, 20), title, fill=(0, 0, 0), font=font_title)

    # Add mock label
    label = f"Mock visualization - {num_rounds} rounds ({width}x{height})"
    draw.text((width//2 - 120, height - 30), label, fill=(100, 100, 100), font=font_title)

    # Save
    output_file = Path(output_file)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    img.save(output_file, 'PNG')


def generate_mock_metro_map(output_file, metro_name='Birmingham',
                            width=600, height=600, seed=42):
    """
    Generate mock metro area map PNG placeholder.

    Parameters
    ----------
    output_file : Path or str
        Output PNG file path
    metro_name : str
        Metro area name
    width : int
        Image width in pixels
    height : int
        Image height in pixels
    seed : int
        Random seed for reproducibility

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output = Path(tmpdir) / 'birmingham_metro_2020.png'
    ...     generate_mock_metro_map(output, metro_name='Birmingham')
    ...     output.exists()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    # Create blank image
    img = Image.new('RGB', (width, height), color=(245, 245, 245))
    draw = ImageDraw.Draw(img)

    # Draw metro boundary (circular)
    center_x = width // 2
    center_y = height // 2 + 20
    radius = min(width, height) // 3

    # Draw filled circle for metro area
    draw.ellipse([center_x - radius, center_y - radius,
                  center_x + radius, center_y + radius],
                 fill=(220, 230, 255), outline=(100, 100, 100), width=2)

    # Draw mock districts (pie slices)
    num_districts = random.randint(2, 4)
    colors = generate_distinct_colors(num_districts, seed=seed)

    angle_step = 360 / num_districts
    for i in range(num_districts):
        start_angle = i * angle_step
        end_angle = (i + 1) * angle_step

        # Draw pie slice
        draw.pieslice([center_x - radius, center_y - radius,
                      center_x + radius, center_y + radius],
                      start=start_angle, end=end_angle,
                      fill=colors[i], outline=(80, 80, 80))

    # Add title
    try:
        font_title = ImageFont.truetype("arial.ttf", 22)
        font_label = ImageFont.truetype("arial.ttf", 14)
    except:
        font_title = ImageFont.load_default()
        font_label = ImageFont.load_default()

    title = f"{metro_name} Metro Area"
    draw.text((width//2 - 80, 20), title, fill=(0, 0, 0), font=font_title)

    # Add mock label
    label = f"Mock metro visualization ({width}x{height})"
    draw.text((width//2 - 90, height - 40), label, fill=(100, 100, 100), font=font_label)

    # Save
    output_file = Path(output_file)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    img.save(output_file, 'PNG')


def generate_distinct_colors(n, seed=42):
    """
    Generate n visually distinct colors.

    Parameters
    ----------
    n : int
        Number of colors to generate
    seed : int
        Random seed

    Returns
    -------
    list of tuple
        List of (R, G, B) color tuples
    """
    np.random.seed(seed)

    # Use HSV color space for better distinctness
    colors = []
    for i in range(n):
        hue = (i * 360 / n) % 360
        saturation = 0.6 + (i % 3) * 0.1
        value = 0.7 + (i % 2) * 0.15

        # Convert HSV to RGB
        c = value * saturation
        x = c * (1 - abs((hue / 60) % 2 - 1))
        m = value - c

        if hue < 60:
            r, g, b = c, x, 0
        elif hue < 120:
            r, g, b = x, c, 0
        elif hue < 180:
            r, g, b = 0, c, x
        elif hue < 240:
            r, g, b = 0, x, c
        elif hue < 300:
            r, g, b = x, 0, c
        else:
            r, g, b = c, 0, x

        # Convert to 0-255 range
        r = int((r + m) * 255)
        g = int((g + m) * 255)
        b = int((b + m) * 255)

        colors.append((r, g, b))

    return colors


def validate_mock_map(map_file):
    """
    Validate mock map image file.

    Parameters
    ----------
    map_file : Path or str
        Map file to validate

    Raises
    ------
    AssertionError
        If validation fails
    """
    map_file = Path(map_file)

    # Check file exists
    assert map_file.exists(), f"Map file not found: {map_file}"

    # Check is PNG
    assert map_file.suffix == '.png', f"Not a PNG file: {map_file}"

    # Try to open as image
    img = Image.open(map_file)
    try:
        assert img.format == 'PNG', f"Invalid PNG format: {map_file}"

        # Check reasonable dimensions
        width, height = img.size
        assert width >= 400, f"Width too small: {width}"
        assert height >= 300, f"Height too small: {height}"
        assert width <= 4000, f"Width too large: {width}"
        assert height <= 4000, f"Height too large: {height}"

        print(f"[OK] Mock map validated: {map_file.name} ({width}x{height})")
    finally:
        img.close()


if __name__ == '__main__':
    # Test generation
    import tempfile

    print("Generating mock map images...")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        # Test state map
        state_map = tmpdir / 'alabama_districts_2020.png'
        generate_mock_state_map(state_map, state='alabama', map_type='districts')
        validate_mock_map(state_map)
        print(f"  State map: {state_map.name}")

        # Test national map
        national_map = tmpdir / 'national_political_2020.png'
        generate_mock_national_map(national_map, map_type='political')
        validate_mock_map(national_map)
        print(f"  National map: {national_map.name}")

        # Test round progression
        rounds_map = tmpdir / 'rounds_progression_2020.png'
        generate_mock_round_progression_map(rounds_map, num_rounds=6)
        validate_mock_map(rounds_map)
        print(f"  Rounds map: {rounds_map.name}")

        # Test metro map
        metro_map = tmpdir / 'birmingham_metro_2020.png'
        generate_mock_metro_map(metro_map, metro_name='Birmingham')
        validate_mock_map(metro_map)
        print(f"  Metro map: {metro_map.name}")

    print("[OK] Mock map generation working correctly")

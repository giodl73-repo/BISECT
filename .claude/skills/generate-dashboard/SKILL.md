---
name: generate-dashboard
description: Generate static HTML dashboard with all redistricting visualizations and data. Bakes district data, maps, and CSVs into single interactive HTML file that opens in browser. Use after completing redistricting pipeline.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Generate Dashboard

## Overview

Create a comprehensive, interactive static HTML dashboard that presents all redistricting results in an accessible web interface. The dashboard bakes all data, maps, and links into a single HTML file that can be opened locally or hosted on a web server.

## Prerequisites

Before generating dashboard:
1. **Redistricting completed** for all states (or subset)
2. **Analysis data exists** (districts.csv, compactness.csv, etc.)
3. **Maps generated** (state maps, national maps)
4. **Output directory structure** in place (`outputs/us_{year}_{version}/`)

## When to Use This Skill

- User says: "Generate the dashboard" or "Create web dashboard"
- User says: "Open the results in a browser"
- After completing full 50-state pipeline run
- After regenerating maps or analysis
- User wants interactive exploration of results
- User wants to share results with others

## What the Dashboard Provides

### Interactive Features

**State Navigation**:
- Dropdown menu with all 50 states + DC
- Quick jump to any state's results
- State-by-state statistics

**Tabbed Interface**:
- **Districts**: Basic district assignment maps
- **Political**: Partisan lean analysis (2020 only)
- **Demographics**: Racial/ethnic composition
- **Compactness**: Polsby-Popper and Reock scores
- **Metro Areas**: Urban area focus maps (if available)
- **National**: US-wide visualizations
- **Rounds**: Algorithm progression maps
- **Data**: CSV downloads and statistics

**Data Display**:
- Inline map viewing (no external image downloads needed)
- Direct links to CSV files
- Summary statistics tables
- Sortable/filterable data tables (future enhancement)

### Content Included

**Per-State Content** (51 states):
- District assignment map
- Political lean map (if 2020)
- Demographic composition maps (3 types)
- Compactness score maps (2 metrics)
- Data CSVs (districts, analysis results)
- Summary statistics

**National Content**:
- All 435 districts map
- National political lean map
- National demographic maps
- National compactness maps
- Round progression series (9 maps)

**Metro Area Content** (if available):
- Top 20 metropolitan areas
- Focused district views
- Organized by state

## Workflow

### Step 1: Verify Prerequisites

Check that output directory exists and is complete:
```bash
# Check output directory
ls outputs/us_2020_v1/

# Check state data completeness
ls outputs/us_2020_v1/states/*/data/districts.csv | wc -l
# Should be 51 (50 states + DC)

# Check maps exist
ls outputs/us_2020_v1/states/*/maps/*.png | wc -l
# Should be 250-400 depending on analysis types
```

If data incomplete:
- Run missing states: `/run-redistricting --states "missing_states"`
- Regenerate maps: `/run-analysis-only`

### Step 2: Run Dashboard Generation Script

Execute the dashboard generator:

**Basic generation**:
```bash
python scripts/web/generate_dashboard.py \
  --year 2020 \
  --version v1
```

**With custom options**:
```bash
python scripts/web/generate_dashboard.py \
  --year 2020 \
  --version v1 \
  --template web/dashboard.html \
  --output outputs/us_2020_v1/index.html \
  --open
```

**Or use batch file wrapper**:
```bash
deploy_web.bat --year 2020 --version v1
```

### Step 3: Monitor Generation Process

Dashboard generation involves:
```
[1/5] Reading template: web/dashboard.html
[2/5] Scanning state data: 51 states found
[3/5] Cataloging maps: 324 maps found
[4/5] Baking data into HTML: district data, statistics
[5/5] Writing output: outputs/us_2020_v1/index.html

Dashboard generated successfully!
Opening in browser...
```

**Typical runtime**: ~5-10 seconds

### Step 4: Verify Dashboard Opens

Browser should automatically open to:
```
file:///C:/src/apportionment/outputs/us_2020_v1/index.html
```

**Manual open** (if auto-open fails):
```bash
# Windows
start outputs/us_2020_v1/index.html

# Or double-click file in explorer
```

### Step 5: Test Dashboard Functionality

**Checklist**:
- [ ] Page loads without errors
- [ ] All 51 states appear in dropdown
- [ ] Tab navigation works (Districts, Political, etc.)
- [ ] State selection updates displayed content
- [ ] Maps load and display correctly
- [ ] Links to CSV files work
- [ ] National maps tab shows all US-wide maps
- [ ] Statistics display accurately

## Dashboard Template Structure

### Template File: `web/dashboard.html`

The template is a single HTML file with embedded CSS and JavaScript:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>US Congressional Redistricting Dashboard</title>
  <style>
    /* Embedded CSS for styling */
    /* No external CSS dependencies */
  </style>
</head>
<body>
  <header>
    <h1>Congressional Redistricting Results ({{YEAR}})</h1>
    <select id="stateSelector">
      <!-- States populated by generator -->
    </select>
  </header>

  <nav id="tabs">
    <button class="tab-btn" data-tab="districts">Districts</button>
    <button class="tab-btn" data-tab="political">Political</button>
    <button class="tab-btn" data-tab="demographics">Demographics</button>
    <button class="tab-btn" data-tab="compactness">Compactness</button>
    <button class="tab-btn" data-tab="national">National</button>
    <button class="tab-btn" data-tab="data">Data</button>
  </nav>

  <main id="content">
    <!-- Content populated by generator -->
  </main>

  <script>
    /* Embedded JavaScript for interactivity */
    /* Data baked in by generator */
    const stateData = {{STATE_DATA_JSON}};
    const nationalMaps = {{NATIONAL_MAPS_JSON}};

    // Tab switching logic
    // State selection logic
    // Map display logic
  </script>
</body>
</html>
```

### Data Baking Process

Generator replaces template placeholders with actual data:

**State data JSON**:
```javascript
const stateData = {
  "california": {
    "name": "California",
    "districts": 52,
    "maps": {
      "districts": "states/california/maps/districts.png",
      "political": "states/california/maps/political_lean.png",
      "compactness_pp": "states/california/maps/compactness_polsby_popper.png",
      "compactness_reock": "states/california/maps/compactness_reock.png",
      "demographic_white": "states/california/maps/demographic_white_percentage.png",
      "demographic_minority": "states/california/maps/demographic_minority_percentage.png"
    },
    "data": {
      "districts_csv": "states/california/data/districts.csv",
      "summary_csv": "states/california/data/district_summary.csv",
      "political_csv": "states/california/data/political_lean.csv",
      "demographic_csv": "states/california/data/demographic_composition.csv",
      "compactness_csv": "states/california/data/compactness.csv"
    },
    "stats": {
      "total_population": 39538223,
      "mean_compactness_pp": 0.42,
      "d_seats": 42,
      "r_seats": 10
    }
  },
  // ... 50 more states
};
```

**National maps JSON**:
```javascript
const nationalMaps = {
  "all_districts": "maps/us_all_districts.png",
  "political_lean": "maps/us_political_lean.png",
  "compactness_pp": "maps/us_compactness_polsby_popper.png",
  "demographic_white": "maps/us_demographic_white_percentage.png",
  "rounds": [
    "maps/rounds/us_round_1.png",
    "maps/rounds/us_round_2.png",
    // ... rounds 3-9
  ]
};
```

## Dashboard Script Reference

### Main Generation Script

**Location**: `scripts/web/generate_dashboard.py`

**Key functions**:
```python
def read_template(template_path):
    """Load dashboard HTML template"""
    pass

def scan_state_data(output_dir, year, version):
    """Scan output directory for all state data and maps"""
    # Returns dict with all state information
    pass

def scan_national_maps(output_dir):
    """Find all national-level maps"""
    pass

def compute_statistics(state_data_csvs):
    """Calculate summary statistics from CSVs"""
    pass

def bake_data_into_template(template, state_data, national_data):
    """Replace template placeholders with actual data"""
    # Returns complete HTML with embedded data
    pass

def write_dashboard(html_content, output_path):
    """Write final HTML file"""
    pass

def open_in_browser(html_path):
    """Open dashboard in default browser"""
    pass
```

### Command-Line Interface

```bash
python scripts/web/generate_dashboard.py \
  --year YEAR \              # Census year (2000/2010/2020)
  --version VERSION \        # Output version tag
  --template TEMPLATE \      # Template file (default: web/dashboard.html)
  --output OUTPUT \          # Output path (default: outputs/us_{year}_{version}/index.html)
  --open \                   # Automatically open in browser
  --no-stats \               # Skip statistics computation (faster)
  --states-filter STATES     # Only include specific states (comma-separated)
```

## Output Files

Dashboard and related files:
```
outputs/us_{year}_{version}/
├── index.html                    # Main dashboard (self-contained)
├── maps/                         # National maps (referenced by dashboard)
│   ├── us_all_districts.png
│   ├── us_political_lean.png
│   └── rounds/
└── states/                       # State-specific content (referenced)
    └── {state}/
        ├── maps/
        │   └── *.png
        └── data/
            └── *.csv
```

**File size**:
- **Dashboard HTML**: ~100-500 KB (with embedded data)
- **Total directory**: ~50-200 MB (depending on map DPI and analysis types)

## Customization

### Modify Template

Edit `web/dashboard.html` to customize:

**Change colors**:
```css
/* In <style> section */
:root {
  --primary-color: #1a73e8;  /* Change theme color */
  --bg-color: #f8f9fa;
  --text-color: #202124;
}
```

**Add custom tabs**:
```html
<!-- In <nav> section -->
<button class="tab-btn" data-tab="custom">Custom Analysis</button>

<!-- In <main> section -->
<div id="tab-custom" class="tab-content">
  <!-- Custom content here -->
</div>
```

**Modify layout**:
```css
/* Change from sidebar to top nav */
.dashboard-container {
  flex-direction: column;  /* Was: row */
}
```

### Add Custom Data

Inject additional data during generation:

```python
# In generate_dashboard.py
custom_data = {
    "algorithm_version": "edge-weighted",
    "generation_date": "2026-01-15",
    "computation_time": "3.5 hours",
}

# Bake into template
html = html.replace("{{CUSTOM_DATA}}", json.dumps(custom_data))
```

### Filter States

Generate dashboard for subset of states:
```bash
python scripts/web/generate_dashboard.py \
  --year 2020 \
  --version v1 \
  --states-filter "california,texas,florida,new_york"
```

## Troubleshooting

**Common Issues**:

**Dashboard doesn't open automatically**:
```
Issue: Browser doesn't open after generation
Solution: Manually open outputs/us_2020_v1/index.html
          Or use: start outputs/us_2020_v1/index.html
```

**Maps not displaying**:
```
Issue: Broken image links, maps show as broken
Cause: Relative paths incorrect
Solution: Ensure dashboard HTML is in outputs/us_{year}_{version}/
          Verify map files exist at referenced paths
```

**Missing state data**:
```
Issue: Some states don't appear in dropdown
Cause: districts.csv missing for those states
Solution: Run redistricting for missing states
          Or regenerate with --allow-partial
```

**Tabs not working**:
```
Issue: Clicking tabs does nothing
Cause: JavaScript error (check browser console: F12)
Solution: Verify template JavaScript section intact
          Check for syntax errors in baked JSON data
```

**Statistics wrong or missing**:
```
Issue: Summary statistics show NaN or incorrect values
Cause: CSV parsing error or missing data
Solution: Validate CSV files exist and are correctly formatted
          Run with --no-stats to skip statistics (debug)
```

**File too large**:
```
Issue: Dashboard HTML is megabytes in size
Cause: Too much data baked into HTML
Solution: Reduce embedded data, reference external files instead
          Or split into multiple dashboard pages
```

## Advanced Usage

### Comparison Dashboard

Create side-by-side comparison of multiple versions:
```bash
# Generate dashboards for different versions
python scripts/web/generate_dashboard.py --year 2020 --version v1
python scripts/web/generate_dashboard.py --year 2020 --version v2

# Create comparison HTML (custom script)
python scripts/web/create_comparison_dashboard.py \
  --versions "v1,v2" \
  --year 2020
```

### Multi-Year Dashboard

Compare across census years:
```bash
# Generate for each year
python scripts/web/generate_dashboard.py --year 2000 --version v1
python scripts/web/generate_dashboard.py --year 2010 --version v1
python scripts/web/generate_dashboard.py --year 2020 --version v1

# Create multi-year dashboard
python scripts/web/create_multiyear_dashboard.py \
  --years "2000,2010,2020"
```

### Export to Static Site

Deploy dashboard as static website:
```bash
# Copy to web hosting directory
cp -r outputs/us_2020_v1/ /var/www/html/redistricting/

# Or use GitHub Pages
cd outputs/us_2020_v1/
git init
git add .
git commit -m "Add redistricting dashboard"
git push origin gh-pages
```

### Embed in Documentation

Include dashboard in larger documentation site:
```html
<!-- In main docs site -->
<iframe src="outputs/us_2020_v1/index.html" width="100%" height="800px">
</iframe>
```

## Integration with Pipeline

Dashboard generation is the final stage of the pipeline:

**Automatic generation**:
```bash
# Dashboard generated automatically at end
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1
# Creates: outputs/us_2020_v1/index.html
# Opens in browser automatically
```

**Manual regeneration** (after updating maps/data):
```bash
# Regenerate dashboard only
python scripts/web/generate_dashboard.py \
  --year 2020 \
  --version v1 \
  --open
```

**Batch file wrapper**:
```bash
# Windows convenience script
deploy_web.bat --year 2020 --version v1
```

## Sharing Results

### Local Sharing

Share complete output directory:
```bash
# Zip for sharing
zip -r redistricting_2020_v1.zip outputs/us_2020_v1/

# Recipient extracts and opens index.html
```

### Web Hosting

Host on static file server:
```bash
# Upload to web server
rsync -avz outputs/us_2020_v1/ user@server:/var/www/redistricting/

# Access at: https://server.com/redistricting/
```

### GitHub Pages

Host on GitHub:
```bash
cd outputs/us_2020_v1/
git init
git add .
git commit -m "Add redistricting results"
git remote add origin https://github.com/user/redistricting.git
git push -u origin main

# Enable GitHub Pages in repo settings
# Access at: https://user.github.io/redistricting/
```

## Performance Notes

**Generation time**:
| States | Maps | Time |
|--------|------|------|
| 5 states | ~30 maps | ~2 sec |
| 10 states | ~60 maps | ~3 sec |
| 51 states | ~400 maps | ~5-10 sec |

**Bottlenecks**:
- Scanning directory structure for maps/CSVs
- Computing statistics from CSVs
- Writing large HTML file

**Optimization**:
- Use `--no-stats` to skip statistics computation
- Cache directory scans for repeated regeneration
- Minimize embedded data (reference external files)

## Browser Compatibility

**Tested browsers**:
- Chrome/Edge 90+ ✓
- Firefox 88+ ✓
- Safari 14+ ✓
- Internet Explorer 11 ⚠️ (limited support)

**Required features**:
- ES6 JavaScript (const, arrow functions)
- CSS Grid
- Flexbox
- Fetch API (for future AJAX features)

**Fallbacks**:
- No external dependencies (no CDN requirements)
- Works offline (all data embedded or local)
- Degrades gracefully in older browsers

## Related Skills

- `/run-redistricting` - Complete pipeline that generates dashboard
- `/create-state-map` - Generate individual state maps for dashboard
- `/create-national-map` - Generate national maps for dashboard
- `/run-analysis-only` - Regenerate analysis data for dashboard

## Best Practices

1. **Regenerate after changes**: Always regenerate dashboard after updating maps/data
2. **Test locally first**: Open dashboard locally before deploying
3. **Version control template**: Track changes to `web/dashboard.html`
4. **Validate data**: Ensure all referenced files exist before generation
5. **Optimize images**: Use appropriate DPI for web (150 is good balance)
6. **Document customizations**: Note any template modifications
7. **Backup outputs**: Archive dashboard + data before regeneration

## What You'll Get

After successful generation:
- **Single HTML file** with all data embedded
- **Interactive interface** for exploring all 51 states
- **Tabbed navigation** for different analysis types
- **Direct links** to all maps and CSV files
- **Summary statistics** for quick insights
- **Automatic browser opening** to view results
- **Self-contained package** for easy sharing
- **No dependencies** (works offline)

## Next Steps

- Share dashboard with collaborators
- Customize template for specific presentation needs
- Create comparison dashboards for different versions/years
- Deploy to web server for public access
- Generate exports/reports from dashboard data

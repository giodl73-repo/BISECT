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

Create comprehensive interactive static HTML dashboard presenting all redistricting results. Bakes all data/maps/links into single HTML file (opens locally or hosted on web server).

## Prerequisites
Redistricting completed (all states or subset), analysis data exists (`districts.csv`, `compactness.csv`), maps generated (state/national), output directory structure (`outputs/us_{year}_{version}/`)

## When to Use
User says "Generate dashboard/Create web dashboard/Open results in browser", after full 50-state pipeline, after regenerating maps/analysis, wants interactive exploration/sharing results

## Dashboard Provides

### Interactive Features
**State Navigation**: Dropdown menu (50 states + DC), quick jump to any state, state-by-state stats
**Tabbed Interface**: Districts (basic assignments), Political (partisan lean, 2020 only), Demographics (racial/ethnic composition), Compactness (PP/Reock scores), Metro Areas (urban focus if available), National (US-wide), Rounds (algorithm progression), Data (CSV downloads/statistics)
**Data Display**: Inline map viewing (no external downloads), direct CSV links, summary statistics tables, sortable/filterable tables (future)

### Content Included
**Per-State** (51): District assignment map, political lean map (2020), demographic composition maps (3 types), compactness score maps (2 metrics), data CSVs (districts/analysis), summary stats
**National**: All 435 districts map, national political/demographic/compactness maps, round progression series (9 maps)
**Metro Areas** (if available): Top 20 metro areas, focused district views, organized by state

## Workflow

### Step 1: Verify Prerequisites
```bash
ls outputs/us_2020_v1/                                     # Check output dir
ls outputs/us_2020_v1/states/*/data/districts.csv | wc -l # Should be 51
ls outputs/us_2020_v1/states/*/maps/*.png | wc -l         # Should be 250-400
```
If incomplete → `/run-redistricting --states "missing"` or `/run-analysis-only`

### Step 2: Run Dashboard Generation
**Basic**: `python scripts/web/generate_dashboard.py --year 2020 --version v1`
**Custom options**: `--template web/dashboard.html --output outputs/us_2020_v1/index.html --open`
**Batch wrapper**: `deploy_web.bat --year 2020 --version v1`

### Step 3: Monitor Generation
```
[1/5] Reading template: web/dashboard.html
[2/5] Scanning state data: 51 states found
[3/5] Cataloging maps: 324 maps found
[4/5] Baking data into HTML: district data, statistics
[5/5] Writing output: outputs/us_2020_v1/index.html
Dashboard generated successfully! Opening in browser...
```
**Runtime**: ~5-10s

### Step 4: Verify Dashboard Opens
Browser auto-opens to `file:///apportionment/outputs/us_2020_v1/index.html`
Check: State dropdown works, tabs display content, maps load, CSVs link correctly, national maps visible

### Step 5: Test Functionality
**Navigation**: Select states from dropdown, switch tabs, verify all maps load
**Links**: Click CSV links (should download/open), verify paths correct
**Data**: Check statistics tables populated, numbers reasonable
**Responsiveness**: Resize browser, check mobile layout (if responsive design)

## Parameters

**Required**: `--year` (2000/2010/2020), `--version` (output version tag)
**Optional**: `--template` (custom HTML template path, default `web/dashboard.html`), `--output` (custom output path, default `outputs/us_{year}_{version}/index.html`), `--open` (auto-open in browser, default true), `--title` (custom dashboard title), `--description` (custom description text)

## Dashboard Structure

### HTML Template
**Base template**: `web/dashboard.html` (Jinja2-style with placeholders)
**Placeholders**: `{{YEAR}}`, `{{VERSION}}`, `{{STATES}}`, `{{MAPS}}`, `{{DATA}}`, `{{STATISTICS}}`
**JavaScript**: Embedded for interactivity (state selection, tab switching, data tables)
**CSS**: Embedded styling (responsive layout, clean design)

### Data Baking
**Maps**: Relative paths embedded (`states/california/maps/districts.png`)
**CSVs**: Direct file links (`states/california/data/districts.csv`)
**Statistics**: JSON embedded in HTML (`<script>var stats = {...}</script>`)
**State list**: Dropdown populated from directory scan

### Output File
**Single HTML**: `outputs/us_2020_v1/index.html` (~500KB-2MB depending on embedded data)
**Self-contained**: All paths relative, works offline
**No dependencies**: Pure HTML/CSS/JS, no external libraries required

## Customization

### Custom Template
Create new template based on `web/dashboard.html`, modify structure/styling, use same placeholders, generate with `--template custom_dashboard.html`

### Custom Styling
Edit CSS in template: Colors, fonts, layout, responsive breakpoints
**Example**: `.state-dropdown { background: #f0f0f0; }`

### Custom Data Display
Modify JavaScript in template for: Custom sorting/filtering, Additional statistics, Different chart types, Custom interactions

### Multiple Dashboards
Generate comparison dashboards: `--output comparison_2010_v1.html`, `--output comparison_2020_v1.html`, link between dashboards for easy comparison

## Troubleshooting

**Missing maps**: `[WARNING] Map not found: states/california/maps/political_lean.png` → Regenerate with `/run-analysis-only`
**Empty state dropdown**: No states found in output dir → Check path, run redistricting
**Broken CSV links**: Files not found → Verify analysis completed, check file permissions
**Dashboard doesn't open**: Browser not found → Manually open `outputs/us_2020_v1/index.html`
**Maps don't load**: Relative paths broken → Check output directory structure, regenerate dashboard
**Large file size**: Dashboard >5MB → Reduce embedded data, use external links for large datasets

## Deployment

### Local Use
Dashboard works immediately after generation, open directly in browser, share via file system/cloud storage

### Web Hosting
**Static hosting**: Copy entire `outputs/us_2020_v1/` directory to web server, dashboard works with relative paths
**Platforms**: GitHub Pages, Netlify, Vercel, AWS S3, any static file hosting
**Commands**: `deploy_web.bat --year 2020 --version v1 --deploy` (copies to web directory)

### Access Control
For private dashboards: Use .htaccess (Apache), nginx auth_basic, cloud platform access controls
**No server-side processing needed** (static HTML)

## Integration with Pipeline

Dashboard auto-generated during post-processing:
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1
# Automatically runs generate_dashboard.py at end
```

**Manual regeneration** (after data changes):
```bash
python scripts/web/generate_dashboard.py --year 2020 --version v1 --force
```

## Best Practices

1. **Regenerate after changes**: Always regenerate after updating maps/data
2. **Test locally first**: Open locally before deploying
3. **Version control template**: Track changes to `web/dashboard.html`
4. **Validate data**: Ensure all files exist before generation
5. **Optimize images**: Use DPI 150 for web (good balance)
6. **Document customizations**: Note template modifications
7. **Backup outputs**: Archive dashboard + data before regeneration

## What You'll Get

Single HTML file (all data embedded), interactive interface (explore 51 states), tabbed navigation (different analysis types), direct links (maps/CSVs), summary statistics (quick insights), automatic browser opening (view results), self-contained package (easy sharing), no dependencies (works offline)

## Related Skills
`/run-redistricting` (auto-generates dashboard), `/run-analysis-only` (regenerate before dashboard), `/create-state-map` (regenerate state maps), `/create-national-map` (regenerate national maps)

## Next Steps
Share with collaborators, customize template for presentation needs, create comparison dashboards (different versions/years), deploy to web server for public access, generate exports/reports from dashboard data

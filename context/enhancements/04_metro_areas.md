# Enhancement 4: Create Urban Metro Area District Maps (MSA/MCSA)

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026

### Current State
- Individual district maps exist for each district
- State-level overview maps exist
- No focused views of major metropolitan areas

**Completion Date:** January 12, 2026
**Implementation:** Created `download_metro_boundaries.py` to download Census CBSA boundaries and `create_metro_area_maps.py` to generate focused maps for the top 20 MSAs. Metro maps are organized by state (e.g., `metro_los_angeles.png` in California's maps directory) for easy integration with the dashboard. All 20 metro maps generated successfully showing districts within metro boundaries.

### Goal
- Create focused maps for major metro areas showing:
  - All districts within the MSA/MCSA boundary
  - Surrounding context (faded neighboring districts)
  - Major city labels
  - Highway networks (optional)

### Implementation Plan

#### New Script
**`scripts/visualization/create_metro_area_maps.py`**

##### Metro Areas to Cover
Using Census Bureau MSA/MCSA definitions, focus on largest metros:

**Top 20 Metropolitan Areas** (by population):
1. New York-Newark-Jersey City, NY-NJ-PA
2. Los Angeles-Long Beach-Anaheim, CA
3. Chicago-Naperville-Elgin, IL-IN-WI
4. Dallas-Fort Worth-Arlington, TX
5. Houston-The Woodlands-Sugar Land, TX
6. Washington-Arlington-Alexandria, DC-VA-MD-WV
7. Philadelphia-Camden-Wilmington, PA-NJ-DE-MD
8. Miami-Fort Lauderdale-Pompano Beach, FL
9. Atlanta-Sandy Springs-Alpharetta, GA
10. Boston-Cambridge-Newton, MA-NH
11. Phoenix-Mesa-Chandler, AZ
12. San Francisco-Oakland-Berkeley, CA
13. Riverside-San Bernardino-Ontario, CA
14. Detroit-Warren-Dearborn, MI
15. Seattle-Tacoma-Bellevue, WA
16. Minneapolis-St. Paul-Bloomington, MN-WI
17. San Diego-Chula Vista-Carlsbad, CA
18. Tampa-St. Petersburg-Clearwater, FL
19. Denver-Aurora-Lakewood, CO
20. St. Louis, MO-IL

##### Inputs
- Census MSA/MCSA boundary shapefiles (year-specific definitions)
- District shapefiles (from tract unions)
- City place boundaries
- District summary data

**IMPORTANT**: MSA/MCSA definitions change by decade:
- 2020 census → 2020 MSA definitions
- 2010 census → 2010 MSA definitions
- Year parameter must be threaded through download, build, and run scripts

##### Processing Steps
1. **Load MSA boundaries**
   - Download from Census TIGER/Line
   - Filter to top 20 by population

2. **For each MSA:**
   - Identify districts that intersect MSA boundary
   - Load district geometries for focal state(s)
   - Load neighboring districts for context
   - Spatial join cities within MSA

3. **Create focused map**
   - Set extent to MSA boundary + 10% margin
   - Plot focal districts (bright colors)
   - Plot neighboring districts (faded gray)
   - Add city labels (largest 10 cities in metro)
   - Add district numbers at centroids
   - Title: "{Metro Area Name} - Congressional Districts"

##### Outputs
- `outputs/us_YEAR_VERSION/metro_areas/new_york.png`
- `outputs/us_YEAR_VERSION/metro_areas/los_angeles.png`
- ... (one per metro area)

#### Visual Specifications
- **Size**: 14x10 inches (landscape)
- **DPI**: 150
- **Colors**: Qualitative colormap (each district distinct)
- **Context**: Gray neighboring districts at 30% opacity
- **Labels**: City names (8-12pt), district numbers (14pt bold)

#### Integration
Add as optional step in `run_complete_redistricting.py`:
```python
# Optional: CREATE METRO AREA MAPS
if args.create_metro_maps:
    run_subscript(
        'scripts/visualization/create_metro_area_maps.py',
        args=['--year', year, '--version', version, '--output-dir', output_dir],
        description='Creating metro area focused maps'
    )
```

# Enhancement 64: District Visualization

**Status**: PLANNED
**Wave**: Wave 9 (API-MIGRATION)
**Priority**: High
**Estimated Complexity**: Medium
**Estimated Hours**: 16-20 hours
**Created**: 2026-01-25

---

## Description

Implement interactive district visualization with Leaflet maps and sortable district tables. Following the Senior Designer's recommendation, this enhancement focuses on basic map functionality first, deferring Alaska/Hawaii insets to a future enhancement. Includes color-by-metric selection, district tooltips, and basic PM2 deployment.

---

## Tasks

### Phase 1: Leaflet Map Component (4-5 hours)

- [ ] Install Leaflet and react-leaflet
  ```bash
  npm install leaflet react-leaflet
  npm install -D @types/leaflet
  ```
- [ ] Create `frontend/src/features/districts/DistrictMap.tsx`
  ```typescript
  interface DistrictMapProps {
    districts: District[];
    colorBy: 'default' | 'compactness' | 'partisan' | 'demographic';
    selectedId?: number;
    onDistrictClick?: (district: District) => void;
  }

  function DistrictMap({ districts, colorBy, selectedId, onDistrictClick }: DistrictMapProps) {
    const mapRef = useRef<L.Map>(null);

    // Convert districts to GeoJSON FeatureCollection
    const geojsonData = useMemo(() => ({
      type: 'FeatureCollection',
      features: districts.map(d => ({
        type: 'Feature',
        id: d.id,
        geometry: d.geometry,
        properties: d,
      })),
    }), [districts]);

    return (
      <MapContainer
        ref={mapRef}
        center={[39.8283, -98.5795]}  // Center of US
        zoom={4}
        className="h-[600px] w-full rounded-lg shadow"
      >
        <TileLayer
          attribution='&copy; OpenStreetMap'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <GeoJSON
          data={geojsonData}
          style={(feature) => getDistrictStyle(feature, colorBy, selectedId)}
          onEachFeature={(feature, layer) =>
            setupDistrictInteraction(feature, layer, onDistrictClick)
          }
        />
      </MapContainer>
    );
  }
  ```
- [ ] Add Leaflet CSS to index.html
- [ ] Configure map bounds to fit continental US

### Phase 2: GeoJSON District Rendering (3-4 hours)

- [ ] Create `frontend/src/api/districts.ts` for loading district geometries
  ```typescript
  export const districtApi = {
    // Load districts for a specific run
    getByRun: (runId: number) =>
      apiClient.get<DistrictListResponse>(`/api/v1/runs/${runId}/districts`),

    // Load geometry for a specific state (lazy loading)
    getGeometry: (runId: number, state: string, year: string) =>
      apiClient.get<GeoJSONFeatureCollection>(
        `/api/v1/files/data/${runId}/${year}/states/${state}/geometry.geojson`
      ),
  };
  ```
- [ ] Implement lazy loading for geometries
  - Load state list first (no geometry)
  - Load geometry on demand when state visible
  - Cache loaded geometries
- [ ] Handle geometry simplification for performance
  - Server-side: Pre-simplify GeoJSON files
  - Client-side: Use Leaflet's smoothFactor

### Phase 3: Color-by-Metric Selection (3-4 hours)

- [ ] Create color scale utilities in `frontend/src/features/districts/utils/colors.ts`
  ```typescript
  export function getDistrictColor(
    district: District,
    colorBy: ColorByMetric
  ): string {
    switch (colorBy) {
      case 'default':
        return DISTRICT_COLORS[district.district_num % DISTRICT_COLORS.length];

      case 'compactness':
        // Blue (compact) to Red (non-compact)
        return interpolateRdYlBu(district.polsby_popper);

      case 'partisan':
        // Blue (Democratic) to Red (Republican)
        return interpolateRdBu(0.5 + district.partisan_lean / 2);

      case 'demographic':
        // Color by majority demographic
        return getDemographicColor(district);
    }
  }
  ```
- [ ] Create color scale legend component
  ```typescript
  function ColorLegend({ colorBy }: { colorBy: ColorByMetric }) {
    return (
      <div className="absolute bottom-4 right-4 bg-white p-3 rounded shadow">
        <h4 className="font-semibold text-sm mb-2">{LEGEND_TITLES[colorBy]}</h4>
        <div className="flex flex-col space-y-1">
          {getLegendItems(colorBy).map(item => (
            <div key={item.label} className="flex items-center">
              <div
                className="w-4 h-4 mr-2 rounded"
                style={{ backgroundColor: item.color }}
              />
              <span className="text-xs">{item.label}</span>
            </div>
          ))}
        </div>
      </div>
    );
  }
  ```
- [ ] Add color-by dropdown in map controls
- [ ] Update map when color-by selection changes

### Phase 4: District Tooltips (2-3 hours)

- [ ] Implement hover tooltips with district stats
  ```typescript
  function setupDistrictInteraction(
    feature: GeoJSON.Feature,
    layer: L.Layer,
    onDistrictClick?: (district: District) => void
  ) {
    const district = feature.properties as District;

    // Tooltip on hover
    layer.bindPopup(() => {
      return `
        <div class="district-popup">
          <h3 class="font-bold">${district.state} District ${district.district_num}</h3>
          <dl class="text-sm">
            <dt>Population</dt>
            <dd>${formatNumber(district.population)}</dd>
            <dt>Compactness</dt>
            <dd>${(district.polsby_popper * 100).toFixed(1)}%</dd>
            ${district.partisan_lean != null ? `
              <dt>Partisan Lean</dt>
              <dd>${formatPartisanLean(district.partisan_lean)}</dd>
            ` : ''}
          </dl>
        </div>
      `;
    });

    // Highlight on hover
    layer.on({
      mouseover: (e) => {
        const layer = e.target;
        layer.setStyle({ weight: 3, fillOpacity: 0.9 });
      },
      mouseout: (e) => {
        const layer = e.target;
        layer.setStyle({ weight: 1, fillOpacity: 0.7 });
      },
      click: () => onDistrictClick?.(district),
    });
  }
  ```
- [ ] Style tooltip popup with Tailwind-compatible CSS
- [ ] Add click-to-select functionality
- [ ] Highlight selected district on map

### Phase 5: District Table (3-4 hours)

- [ ] Create `frontend/src/features/districts/DistrictTable.tsx`
  ```typescript
  interface DistrictTableProps {
    districts: District[];
    selectedId?: number;
    onRowClick?: (district: District) => void;
    sortBy: SortField;
    sortOrder: 'asc' | 'desc';
    onSortChange: (field: SortField, order: 'asc' | 'desc') => void;
  }

  function DistrictTable({
    districts,
    selectedId,
    onRowClick,
    sortBy,
    sortOrder,
    onSortChange
  }: DistrictTableProps) {
    return (
      <Table
        data={districts}
        columns={[
          { header: 'State', accessor: 'state', sortable: true },
          { header: 'District', accessor: 'district_num', sortable: true },
          { header: 'Population', accessor: 'population', sortable: true,
            cell: (d) => formatNumber(d.population) },
          { header: 'Compactness', accessor: 'polsby_popper', sortable: true,
            cell: (d) => `${(d.polsby_popper * 100).toFixed(1)}%` },
          { header: 'Partisan Lean', accessor: 'partisan_lean', sortable: true,
            cell: (d) => formatPartisanLean(d.partisan_lean) },
        ]}
        onRowClick={onRowClick}
        selectedRowId={selectedId}
        sortBy={sortBy}
        sortOrder={sortOrder}
        onSortChange={onSortChange}
      />
    );
  }
  ```
- [ ] Add sortable column headers
- [ ] Implement client-side sorting
- [ ] Highlight row when district selected on map
- [ ] Add filtering by state

### Phase 6: Districts Page Integration (2-3 hours)

- [ ] Create `frontend/src/features/districts/DistrictsPage.tsx`
  ```typescript
  function DistrictsPage() {
    const { runId } = useParams<{ runId: string }>();
    const { data: run } = useRun(Number(runId));
    const { data: districts, isLoading } = useDistricts(Number(runId));

    const [colorBy, setColorBy] = useState<ColorByMetric>('default');
    const [selectedDistrict, setSelectedDistrict] = useState<District | null>(null);
    const [sortBy, setSortBy] = useState<SortField>('state');
    const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

    if (isLoading) return <Spinner size="lg" />;

    return (
      <div className="space-y-6">
        <div className="flex justify-between items-center">
          <h1 className="text-2xl font-bold">
            Districts - {run?.version} ({run?.config.years.join(', ')})
          </h1>
          <ColorBySelector value={colorBy} onChange={setColorBy} />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card title="Map">
            <DistrictMap
              districts={districts}
              colorBy={colorBy}
              selectedId={selectedDistrict?.id}
              onDistrictClick={setSelectedDistrict}
            />
          </Card>

          <Card title="Districts">
            <DistrictTable
              districts={districts}
              selectedId={selectedDistrict?.id}
              onRowClick={setSelectedDistrict}
              sortBy={sortBy}
              sortOrder={sortOrder}
              onSortChange={handleSortChange}
            />
          </Card>
        </div>

        {selectedDistrict && (
          <DistrictDetail district={selectedDistrict} />
        )}
      </div>
    );
  }
  ```
- [ ] Add route: `/runs/:runId/districts`
- [ ] Link from run detail page when completed
- [ ] Sync selection between map and table

### Phase 7: Basic PM2 Deployment (2-3 hours)

- [ ] Update `ecosystem.config.js` for production
  ```javascript
  module.exports = {
    apps: [
      {
        name: 'apportionment-api',
        script: 'uvicorn',
        args: 'main:app --host 0.0.0.0 --port 8002',
        cwd: './api',
        interpreter: 'python',
        env: {
          DATABASE_URL: process.env.DATABASE_URL,
          DEBUG: 'false',
        },
      },
      {
        name: 'apportionment-frontend',
        script: 'npm',
        args: 'run preview -- --port 3002 --host 0.0.0.0',
        cwd: './frontend',
        env: {
          VITE_API_URL: 'http://localhost:8002',
        },
      },
    ],
  };
  ```
- [ ] Create production build script
  ```bash
  # build.sh
  cd frontend && npm run build
  cd ../api && pip install -r requirements.txt
  ```
- [ ] Create start/stop scripts
  ```bash
  # start.sh
  pm2 start ecosystem.config.js

  # stop.sh
  pm2 stop all
  ```
- [ ] Add PM2 logs viewing instructions
- [ ] Document deployment process in README

### Phase 8: E2E Testing (2-3 hours)

- [ ] Create E2E tests for district visualization
  ```typescript
  test.describe('District Visualization', () => {
    test('displays districts on map', async ({ page }) => {
      await page.goto('/runs/1/districts');

      // Wait for map to load
      await expect(page.locator('.leaflet-container')).toBeVisible();

      // Check districts are rendered
      await expect(page.locator('.leaflet-interactive')).toHaveCount(435);
    });

    test('color by compactness works', async ({ page }) => {
      await page.goto('/runs/1/districts');

      // Select compactness color scheme
      await page.selectOption('[data-testid=color-by-select]', 'compactness');

      // Verify legend shows compactness scale
      await expect(page.locator('[data-testid=color-legend]'))
        .toContainText('Compactness');
    });

    test('clicking district selects in table', async ({ page }) => {
      await page.goto('/runs/1/districts');

      // Click a district on map
      await page.locator('.leaflet-interactive').first().click();

      // Verify table row is highlighted
      await expect(page.locator('tr.selected')).toBeVisible();
    });
  });
  ```
- [ ] Test map loading performance
- [ ] Test sync between map and table selection

---

## Architecture Changes

**New Files**:
```
frontend/src/
├── features/
│   └── districts/
│       ├── DistrictsPage.tsx
│       ├── DistrictMap.tsx
│       ├── DistrictTable.tsx
│       ├── DistrictDetail.tsx
│       ├── components/
│       │   ├── ColorBySelector.tsx
│       │   ├── ColorLegend.tsx
│       │   └── StateFilter.tsx
│       ├── utils/
│       │   ├── colors.ts
│       │   └── geometry.ts
│       └── hooks.ts
├── api/
│   └── districts.ts
└── types/
    └── district.ts
```

**Modified Files**:
```
frontend/src/App.tsx                 # Add districts route
frontend/src/features/runs/RunDetail.tsx  # Link to districts
backend/app/api/routes/files.py                 # Add geometry file serving
ecosystem.config.js                  # Update for production
```

**Related DESIGN_PATTERNS.md Sections**:
- Frontend Patterns: Map Visualization Pattern

---

## Testing Strategy

**Test Coverage Target**: 80% minimum

### Unit Tests (10-12 tests)
- Color scale utilities
- Color scale edge cases (missing values, null handling)
- Format functions
- District sorting
- Geometry utility functions

### Integration Tests (8-10 tests)
- Map renders with districts
- Color-by selection changes map
- Table sorting works
- Map/table selection sync
- Map interaction tests (zoom, pan, tooltip)
- Large geometry handling (50k+ tracts)

### E2E Tests (5-7 tests)
- Full districts page load
- Color scheme switching
- District selection flow
- Map zoom and pan
- Performance regression tests

### Performance Tests (2-3 tests)
- Map initial load benchmark
- Map interaction FPS
- Memory usage with large geometries

### Manual Testing
1. Load districts for completed run
2. Switch between color schemes
3. Click districts on map, verify table selection
4. Click table rows, verify map highlight
5. Sort table by different columns
6. Test on different screen sizes
7. Test with colorblind simulation

### Test Effort Estimate
- Unit tests: 10-12 component tests
- Integration tests: 5-6 integration tests
- E2E/Performance tests: 2-3 tests
- Total estimated effort: 15% of wave testing time

---

## Testing Assessment (from Senior Tester)

| Attribute | Value |
|-----------|-------|
| **Risk Rating** | MEDIUM |
| **Original Assessment** | ADEQUATE with performance testing gaps |
| **Testing Priority** | 4 |
| **Recommended Effort** | 15% of total testing effort |

### Gap Analysis

| Test Type | Originally Proposed | Recommended | Gap |
|-----------|---------------------|-------------|-----|
| Unit | 8-10 | 10-12 | Add color scale edge cases |
| Integration | 5-8 | 8-10 | Add map interaction tests |
| E2E | 3-5 | 5-7 | Add performance regression tests |

### Performance Benchmarks (from Senior Tester)

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Map initial load | <3s | Time to first paint with all districts |
| Map interaction | 60fps / <100ms | No jank on pan/zoom |
| Color-by switch | <100ms | Time to rerender with new colors |
| Table sort | <50ms | Time to resort 435 rows |
| Memory usage | <200MB | Browser memory with full geometries |

### Testing Gaps Identified (from Senior Tester)

**1. Missing: Map Rendering Performance Test**
```typescript
describe('Map Performance', () => {
  it('should render 435 districts without jank', async ({ page }) => {
    // Measure Long Tasks during render
    const longTasks: number[] = [];

    await page.evaluate(() => {
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.duration > 50) {
            window.longTasks.push(entry.duration);
          }
        }
      });
      window.longTasks = [];
      observer.observe({ entryTypes: ['longtask'] });
    });

    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-interactive');

    const tasks = await page.evaluate(() => window.longTasks);

    // No individual task should block for more than 100ms
    expect(Math.max(...tasks, 0)).toBeLessThan(100);
  });

  it('should maintain smooth interaction FPS', async ({ page }) => {
    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-container');

    // Measure FPS during pan
    const fps = await page.evaluate(async () => {
      const map = document.querySelector('.leaflet-container');
      let frames = 0;
      let start: number;

      return new Promise<number>((resolve) => {
        const countFrame = () => {
          if (!start) start = performance.now();
          frames++;

          if (performance.now() - start < 1000) {
            requestAnimationFrame(countFrame);
          } else {
            resolve(frames);
          }
        };

        // Simulate pan
        map.dispatchEvent(new MouseEvent('mousedown', { clientX: 200, clientY: 200 }));
        requestAnimationFrame(countFrame);
      });
    });

    expect(fps).toBeGreaterThanOrEqual(55);  // Allow slight drop from 60
  });
});
```

**2. Missing: Geometry Lazy Loading Test**
```typescript
it('should lazy load state geometries', async ({ page }) => {
  const geometryRequests: string[] = [];

  await page.route('**/geometry/**', (route) => {
    geometryRequests.push(route.request().url());
    route.continue();
  });

  await page.goto('/runs/1/districts');

  // Initially only visible states should be loaded
  await page.waitForSelector('.leaflet-interactive');
  const initialCount = geometryRequests.length;
  expect(initialCount).toBeLessThan(50);  // Not all 50 states

  // Zoom to a specific region
  await page.evaluate(() => {
    const map = (window as any).map;
    map.setView([34.0, -118.0], 8);  // California
  });

  // Should load additional geometries
  await page.waitForTimeout(1000);
  expect(geometryRequests.length).toBeGreaterThan(initialCount);
});
```

**3. Missing: Color Scale Consistency Test**
```typescript
describe('Color Scales', () => {
  it('should use consistent colors across views', async ({ page }) => {
    await page.goto('/runs/1/districts?colorBy=compactness');

    // Get colors from map
    const mapColors = await page.evaluate(() => {
      const features = document.querySelectorAll('.leaflet-interactive');
      return Array.from(features).slice(0, 10).map(f =>
        getComputedStyle(f).fill
      );
    });

    // Get colors from table (if colored)
    const tableColors = await page.evaluate(() => {
      const cells = document.querySelectorAll('[data-color]');
      return Array.from(cells).slice(0, 10).map(c =>
        c.getAttribute('data-color')
      );
    });

    // Colors should match for same districts
    expect(mapColors).toEqual(tableColors);
  });

  it('should handle missing compactness values', async ({ page }) => {
    // Mock district with null compactness
    await page.route('**/districts', (route) => {
      route.fulfill({
        json: {
          districts: [
            { id: 1, state: 'test', polsby_popper: null },
          ],
        },
      });
    });

    await page.goto('/runs/1/districts?colorBy=compactness');

    // Should render with default/fallback color, not crash
    await expect(page.locator('.leaflet-interactive')).toBeVisible();
  });
});
```

**4. Missing: Large Geometry Handling Test**
```typescript
it('should handle 50k+ tract geometries', async ({ page }) => {
  // Monitor memory usage
  const initialMemory = await page.evaluate(() => {
    return (performance as any).memory?.usedJSHeapSize || 0;
  });

  await page.goto('/runs/1/districts');
  await page.waitForSelector('.leaflet-interactive');

  const finalMemory = await page.evaluate(() => {
    return (performance as any).memory?.usedJSHeapSize || 0;
  });

  // Memory increase should be reasonable (<200MB)
  const memoryIncrease = (finalMemory - initialMemory) / (1024 * 1024);
  expect(memoryIncrease).toBeLessThan(200);
});
```

### Quality Gates for This Enhancement

- [ ] Map displays all 435 districts correctly
- [ ] Initial map load completes in <3 seconds
- [ ] Map interaction maintains 55+ FPS
- [ ] Color-by-metric changes apply in <100ms
- [ ] Memory usage stays below 200MB
- [ ] Missing/null values don't crash color scales
- [ ] Geometry lazy loading reduces initial load time
- [ ] Table and map selection stay synchronized
- [ ] Test coverage >= 80%

---

## Dependencies

**Prerequisites**:
- Enhancement 60 (Project Setup) - REQUIRED
- Enhancement 61 (Run Management API) - REQUIRED
- Enhancement 63 (React Dashboard Core) - REQUIRED
- Completed redistricting run with output files

**Node Packages** (add to package.json):
- leaflet
- react-leaflet
- @types/leaflet
- d3-scale-chromatic (for color scales)

---

## Success Criteria

- [ ] Map displays all 435 districts
- [ ] Color-by-metric works for all options
- [ ] Tooltips show district statistics
- [ ] Table is sortable by all columns
- [ ] Map and table selection are synchronized
- [ ] PM2 deployment scripts work
- [ ] All tests pass (unit + integration + E2E)
- [ ] Performance acceptable (< 3 second load time)

---

## Design Notes (from Senior Designer)

### Deferred Alaska/Hawaii Insets
Following Senior Designer recommendation:
- **MVP**: Basic map without insets (AK/HI at actual locations)
- **Future**: Add inset maps in Enhancement 64b
- **Rationale**: Inset positioning is complex, not required for MVP

### Map Performance
Rendering 435 districts with Leaflet can be slow. Optimizations:
- **Geometry simplification**: Reduce point count by 90%
- **Progressive loading**: Load visible states first
- **Canvas rendering**: Use Leaflet.Canvas for better performance
- **Lazy loading**: Load geometry on demand

### Geometry File Strategy
Don't store geometries in database:
- **Rationale**: Too large (MB per state), already exist as files
- **Approach**: Serve GeoJSON files via API endpoint
- **Caching**: Browser caches geometry files

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Large geometry files slow loading | Simplify geometries, lazy load |
| Map performance with 435 districts | Use canvas rendering, simplify |
| Color scales hard to distinguish | Test with colorblind-safe palettes |
| Mobile map interaction poor | Test touch gestures, add mobile-specific controls |

---

## Future Enhancements (Post-MVP)

The following are explicitly deferred per Senior Designer recommendation:
- Alaska/Hawaii insets (Enhancement 64b)
- Zoom to state functionality
- Print/export map as image
- Compare two runs side-by-side
- Time-lapse of district changes

---

## Related Documentation

- [Wave 9 Plan](../waves/WAVE09-api-migration.md)
- [Design Patterns](../DESIGN_PATTERNS.md) - Map Visualization Pattern
- [Senior Designer Review](../waves/wave09/01_senior_designer_review.md) - Map recommendations
- [Senior Engineer Review](../waves/wave09/02_senior_engineer_review.md) - Performance patterns
- [Senior Tester Review](../waves/wave09/03_senior_tester_review.md) - Performance testing recommendations
- [TESTING_PATTERNS.md](../TESTING_PATTERNS.md) - Performance testing patterns, Playwright E2E

---

## Engineering Notes (from Senior Engineer)

### Risk Assessment: MEDIUM

Performance-sensitive enhancement with medium risk around large geometry rendering.

### Leaflet Canvas Renderer Recommendation

Per engineer recommendation, use Canvas renderer instead of SVG for better performance with 435 districts:

```typescript
// Use Canvas renderer instead of SVG for better performance
import { MapContainer, TileLayer, GeoJSON } from 'react-leaflet';
import L from 'leaflet';

// Prefer canvas for many features (435 districts)
const renderer = L.canvas({ padding: 0.5 });

function DistrictMap({ districts, colorBy, onDistrictClick }: DistrictMapProps) {
  return (
    <MapContainer center={[39.8, -98.6]} zoom={4}>
      <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" />
      <GeoJSON
        data={geojsonData}
        renderer={renderer}  // Use canvas instead of SVG
        style={getStyle}
        onEachFeature={setupInteraction}
      />
    </MapContainer>
  );
}
```

**Why Canvas over SVG**:
- SVG creates DOM elements for each polygon (435+ elements)
- Canvas draws directly to a single canvas element
- Canvas performs significantly better with many features
- Target: 60fps pan/zoom interactions

### Memoization Requirements

Per engineer recommendation, memoize expensive computations:

```typescript
// Memoize GeoJSON conversion (only recompute when districts change)
const geojsonData = useMemo(() => ({
  type: 'FeatureCollection',
  features: districts.map(d => ({
    type: 'Feature',
    id: d.id,
    geometry: d.geometry,
    properties: d
  }))
}), [districts]);  // Only recompute when districts change

// Memoize style function (only recompute when colorBy changes)
const styleFunction = useMemo(() => {
  return (feature: GeoJSON.Feature) => ({
    fillColor: getDistrictColor(feature.properties, colorBy),
    fillOpacity: 0.7,
    color: '#333',
    weight: 1,
  });
}, [colorBy]);

// Memoize sorted districts for table
const sortedDistricts = useMemo(() => {
  return [...districts].sort((a, b) => {
    const aVal = a[sortBy];
    const bVal = b[sortBy];
    const multiplier = sortOrder === 'asc' ? 1 : -1;
    return (aVal - bVal) * multiplier;
  });
}, [districts, sortBy, sortOrder]);
```

### Performance Considerations for Large Geometries

**Memory Usage**:
- 435 districts x ~1000 points each = ~4M coordinates
- At 8 bytes per coordinate = ~32MB
- Browser can handle this with proper GeoJSON handling

**Optimizations**:

1. **Server-side simplification** (reduce 90% of points):
```python
# api/services/geometry_service.py
import geopandas as gpd

def simplify_geometry(gdf: gpd.GeoDataFrame, tolerance: float = 0.01):
    """Simplify geometries for web display."""
    return gdf.geometry.simplify(tolerance, preserve_topology=True)
```

2. **Client-side simplification if needed**:
```typescript
import simplify from '@turf/simplify';

const simplifiedGeojson = simplify(geojson, { tolerance: 0.01 });
```

3. **Lazy loading per state** (don't load all 50 at once):
```typescript
// Load geometry on demand when state becomes visible
export function useStateGeometry(runId: number, state: string, year: string) {
  return useQuery({
    queryKey: ['geometry', runId, year, state],
    queryFn: () => districtApi.getGeometry(runId, state, year),
    // Only fetch when enabled
    enabled: isStateVisible(state),
    // Cache for session
    staleTime: Infinity,
  });
}
```

### Caching Strategy

Per engineer recommendation, use HTTP caching for geometry files:

```python
# backend/app/api/routes/files.py
from fastapi.responses import FileResponse
import hashlib

@router.get("/geometry/{version}/{year}/{state}")
async def get_geometry(version: str, year: str, state: str):
    file_path = outputs_dir / version / year / "states" / state / "geometry.geojson"

    if not file_path.exists():
        raise HTTPException(404, "Geometry not found")

    # Generate ETag from file modification time
    mtime = file_path.stat().st_mtime
    etag = hashlib.md5(f"{file_path}:{mtime}".encode()).hexdigest()

    return FileResponse(
        file_path,
        media_type="application/geo+json",
        headers={
            "ETag": etag,
            "Cache-Control": "public, max-age=3600"  # Cache 1 hour
        }
    )
```

**Browser caching**:
- Geometries are immutable once generated
- ETag ensures browser only re-downloads if changed
- 1 hour cache reduces server load

### Performance Benchmarks (Target Metrics)

Per engineer recommendation:

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Map initial load | <3s | Time to first paint with all districts |
| Map interaction | 60fps | No jank on pan/zoom |
| Color-by switch | <100ms | Time to rerender with new colors |
| Table sort | <50ms | Time to resort 435 rows |

### Testing Map Performance

```typescript
test.describe('District Map Performance', () => {
  test('loads 435 districts in under 3 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-interactive', { state: 'visible' });

    const loadTime = Date.now() - startTime;
    expect(loadTime).toBeLessThan(3000);
  });

  test('maintains 60fps during pan', async ({ page }) => {
    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-container');

    // Measure frame rate during interaction
    const fps = await page.evaluate(() => {
      return new Promise((resolve) => {
        let frameCount = 0;
        const startTime = performance.now();

        function countFrame() {
          frameCount++;
          if (performance.now() - startTime < 1000) {
            requestAnimationFrame(countFrame);
          } else {
            resolve(frameCount);
          }
        }

        requestAnimationFrame(countFrame);
      });
    });

    expect(fps).toBeGreaterThan(55);
  });
});
```

---

**Enhancement 64 Summary**: Implement interactive district visualization with Leaflet maps, color-by-metric selection, district tooltips, sortable table, and basic PM2 deployment.

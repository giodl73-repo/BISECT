# Enhancement 29: Artifacts Dashboard Tab

**Status**: 🔄 IN PROGRESS
**Priority**: Medium
**Estimated Complexity**: Medium (3-5 hours)
**Created**: January 16, 2026
**Started**: January 16, 2026

## Current State

The dashboard (`web/dashboard.html`) displays redistricting outputs organized by dimensions:
- Overview, Rounds, Districts, Political, Demographics, Compactness, Cities, Urban Areas
- Content is state-specific (select a state, then view dimensions)
- No way to view compiled artifacts (PDFs, figures) generated from `artifacts/` directory
- Users must navigate file system to find compiled guides, presentations, and papers
- Shared figures stored in `outputs/figures/` are not accessible via dashboard

After reorganization (completed Jan 16, 2026):
- `artifacts/` contains source files for papers, presentations, guides, figures
- `outputs/artifacts/` contains compiled PDFs
- `outputs/figures/` contains shared figures used across documents
- No interface to browse or view these artifacts

## Goal

Add an "Artifacts" tab to the dashboard that provides browser-based access to:

1. **Compiled PDFs** from `outputs/artifacts/`:
   - Guides (e.g., Layman's Guide to Edge-Weighted Bisection)
   - Presentations (e.g., Conference Slides)
   - Papers (e.g., Research Papers)

2. **Shared Figures** from `outputs/figures/`:
   - Schematic diagrams
   - Real tracts examples
   - Round progression maps

**Display Requirements**:
- PDFs embedded using `<embed>` or `<iframe>` with appropriate dimensions
- Figures displayed as images with appropriate dimensions
- Different dimensions for each artifact type:
  - **Guides**: Letter size (8.5" × 11") → ~850px × 1100px display
  - **Presentations**: Widescreen (16:9) → ~960px × 540px display
  - **Papers**: Letter size (8.5" × 11") → ~850px × 1100px display
  - **Figures**: Auto-sized with max width

## Implementation Plan

### Phase 1: Dashboard Structure Enhancement

- [ ] Add "Artifacts" dimension tab to dashboard.html
  - Icon: 📄 (document emoji)
  - Position: After "Urban Areas" tab
  - Label: "📄 Artifacts"

- [ ] Create artifacts content layout section
  - Grid layout for artifact categories
  - Category cards: Guides, Presentations, Papers, Figures
  - Each card shows available artifacts with thumbnails/icons

### Phase 2: PDF Display Implementation

- [ ] Implement PDF viewer functionality
  - Use `<embed type="application/pdf">` for inline PDF display
  - Fallback to download link if browser doesn't support inline PDFs
  - Add dimension classes for different PDF types

- [ ] Create artifact navigation
  - List available guides with titles and descriptions
  - List available presentations with titles
  - List available papers with titles
  - Click to open in full-width viewer

- [ ] Add dimension-specific CSS
  ```css
  .pdf-viewer-guide {
      width: 850px;
      height: 1100px;
      max-width: 100%;
  }

  .pdf-viewer-presentation {
      width: 960px;
      height: 540px;
      max-width: 100%;
  }

  .pdf-viewer-paper {
      width: 850px;
      height: 1100px;
      max-width: 100%;
  }
  ```

### Phase 3: Figures Gallery Implementation

- [ ] Create figures gallery layout
  - Grid of figure categories (schematic, real tracts, round progression)
  - Thumbnail view with click to enlarge
  - Lightbox/modal for full-size viewing

- [ ] Scan `outputs/figures/` directory structure
  - Auto-discover figures from subdirectories
  - Group by category
  - Display with titles and descriptions

- [ ] Add figure viewer
  - Modal popup for full-size display
  - Image zoom controls
  - Previous/next navigation

### Phase 4: Static Artifact Baking (CRITICAL for CORS)

**IMPORTANT**: Dashboard runs on `file://` protocol, so we CANNOT dynamically load content at runtime (CORS restrictions). Must statically bake artifact list into HTML during generation.

- [ ] Modify `scripts/web/generate_dashboard.py`:
  - Scan `outputs/artifacts/` directory at generation time
  - Scan `outputs/figures/` directory at generation time
  - Build artifact registry with metadata
  - Inject artifact data directly into HTML as JavaScript object

- [ ] Artifact scanning logic in generate_dashboard.py:
  ```python
  def scan_artifacts(output_dir):
      """Scan outputs/artifacts/ and outputs/figures/ directories."""
      artifacts = {
          'guides': [],
          'presentations': [],
          'papers': [],
          'figures': {'schematic': [], 'real_tracts': [], 'round_progression': []}
      }

      # Scan PDFs
      artifacts_dir = output_dir / 'artifacts'
      if artifacts_dir.exists():
          # Scan guides
          for pdf in (artifacts_dir / 'guides').rglob('*.pdf'):
              artifacts['guides'].append({
                  'title': pdf.stem.replace('_', ' ').title(),
                  'path': f'artifacts/{pdf.relative_to(artifacts_dir)}',
                  'description': '...'
              })
          # Similar for presentations and papers

      # Scan figures
      figures_dir = output_dir.parent / 'figures'
      if figures_dir.exists():
          for img in (figures_dir / 'schematic').glob('*.png'):
              artifacts['figures']['schematic'].append({
                  'name': img.stem,
                  'path': f'../figures/schematic/{img.name}'
              })
          # Similar for other figure categories

      return artifacts
  ```

- [ ] Inject artifacts into HTML template:
  ```html
  <script>
      // Artifacts data baked in at generation time
      const ARTIFACTS = {{{artifacts_json}}};  // Template variable
  </script>
  ```

- [ ] JavaScript functions to use baked data:
  - `loadArtifactsTab()` - Uses pre-baked ARTIFACTS object
  - `displayPDF(path, type)` - PDF viewer
  - `displayFigure(path, title)` - Figure viewer
  - No fetch/XHR needed - all paths already in HTML

### Phase 5: Dashboard Integration

- [ ] Integrate artifacts tab into dashboard navigation
  - Add tab click handler
  - Update `loadContent()` function to handle "artifacts" dimension
  - Make artifacts tab global (not state-specific like other tabs)

- [ ] Handle special "usa" state selection
  - Artifacts tab should be always accessible
  - Independent of state selection
  - Can be viewed alongside national maps

- [ ] Add "Artifacts" quick link in header
  - Optional: Add direct access button in header
  - Allows jumping to artifacts from any view

### Phase 6: Testing & Validation

- [ ] Test PDF display in multiple browsers
  - Chrome/Edge (Chromium)
  - Firefox
  - Safari (if available)

- [ ] Test figure gallery
  - Image loading performance
  - Modal functionality
  - Responsive layout

- [ ] Test with missing artifacts
  - Handle gracefully if PDFs not compiled yet
  - Show helpful message: "Compile artifacts using: artifacts/compile.bat"

## Files to Modify/Create

### Modify

- `web/dashboard.html` - Add artifacts tab, PDF viewers, figures gallery
  - Add dimension tab in navigation
  - Add CSS for artifact viewers (PDF dimensions, gallery layout)
  - Add JavaScript for artifact loading and display
  - Add artifact metadata structure

### Create

No new files needed (all changes in existing dashboard.html)

## Testing Plan

1. **Manual browser testing**:
   ```bash
   # Compile artifacts first
   cd artifacts
   compile.bat --skip-images

   # Generate dashboard
   cd ..
   python scripts/web/generate_dashboard.py --year 2020 --version v1

   # Open dashboard
   start outputs/us_2020_v1/index.html
   ```

2. **Test artifacts tab**:
   - Click "📄 Artifacts" tab
   - Verify artifact categories display
   - Click on guide → verify PDF loads with correct dimensions (850×1100)
   - Click on presentation → verify PDF loads with correct dimensions (960×540)

3. **Test figures gallery**:
   - Navigate to Figures section
   - Verify thumbnails display
   - Click thumbnail → verify modal opens with full-size image
   - Test navigation between figures

4. **Browser compatibility**:
   - Test in Chrome/Edge (primary browser)
   - Test in Firefox (if available)
   - Verify PDF fallback links work

5. **Responsive testing**:
   - Test on smaller screen sizes
   - Verify PDF viewers scale down appropriately
   - Verify gallery layout adapts

## Success Criteria

- [ ] "📄 Artifacts" tab appears in dashboard navigation
- [ ] Clicking tab shows artifact categories (Guides, Presentations, Papers, Figures)
- [ ] PDFs display inline with correct dimensions for each type
- [ ] Figures gallery shows all available figures organized by category
- [ ] Modal viewer works for full-size figure display
- [ ] Artifacts tab works in both state-specific and USA (national) views
- [ ] Graceful handling when artifacts not yet compiled
- [ ] Cross-browser compatibility (Chrome/Edge/Firefox)

## Benefits

1. **Centralized access**: All artifacts accessible from single dashboard interface
2. **No file navigation**: Users don't need to browse file system
3. **Context preservation**: Can view artifacts while exploring redistricting data
4. **Better discoverability**: Users can easily find guides, presentations, figures
5. **Professional presentation**: Embedded PDFs look polished vs. file downloads
6. **Figure browsing**: Easy to explore all generated figures in one place

## Dependencies

- Artifacts reorganization (completed Jan 16, 2026)
- Compiled artifacts in `outputs/artifacts/` directory
- Generated figures in `outputs/figures/` directory
- Dashboard generation script: `scripts/web/generate_dashboard.py`

## Risks & Mitigations

- **Risk 1**: PDF embedding may not work in all browsers
  - *Mitigation*: Provide download link fallback for unsupported browsers
  - *Mitigation*: Test in major browsers (Chrome, Firefox, Edge)

- **Risk 2**: Large PDFs may load slowly
  - *Mitigation*: Show loading indicator while PDF loads
  - *Mitigation*: Consider lazy loading (only load when clicked)

- **Risk 3**: Artifacts directory structure may change
  - *Mitigation*: Use configuration object for artifact paths
  - *Mitigation*: Make paths relative to dashboard location

- **Risk 4**: Missing artifacts if not compiled
  - *Mitigation*: Show friendly message: "Artifacts not yet compiled. Run: artifacts/compile.bat"
  - *Mitigation*: List expected artifacts with "not available" status

## Implementation Notes

### Key Decisions

*To be filled during implementation*

### PDF Display Approach

Two options considered:
1. **`<embed>` tag**: Native browser PDF viewer, better integration
2. **`<iframe>` tag**: More control, better fallback handling

**Decision**: Use `<embed>` with `<iframe>` fallback for maximum compatibility.

### Artifact Discovery Approach

Two options:
1. **Static manifest**: List artifacts explicitly in dashboard code
2. **Dynamic discovery**: Scan directories at runtime (requires server-side)

**Decision**: Use static manifest (hardcoded list) since dashboard is static HTML. Easy to maintain and update when new artifacts added.

### Figures Display Approach

Options:
1. **Inline gallery**: Display all figures on page
2. **Thumbnail + modal**: Thumbnails with click to enlarge

**Decision**: Use thumbnail + modal approach to avoid page bloat and improve loading performance.

## Completion Summary

*To be filled after completion*

## Related Documentation

- [Enhancement 13: Directory Unification](../completed/13_directory_unification.md) - Context for artifacts/ organization
- [ARCHITECTURE.md](../../ARCHITECTURE.md#dashboard-generation) - Dashboard architecture
- [CLAUDE.md](../../CLAUDE.md#web-dashboard) - Quick reference for dashboard

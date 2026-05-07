# Assets Prepared - Phase 2 Chapters ✅

**Date**: 2026-02-08
**Status**: Alabama figures copied and ready for Chapters 1-3

## Assets Copied (5 files)

### Chapter 1: From Tracts to Graphs
✅ **alabama_tracts.png** (522 KB)
- Source: `outputs/V1/2020/states/alabama/maps/all_districts.png`
- Shows: Alabama's 1,181 census tracts as colored regions
- Used in: "Imagine a State as a Giant Puzzle" section

⚠️ **adjacency_process.png** - NOT YET CREATED
- Need to create a diagram showing: Geographic map → Node detection → Edge list
- Will add in Phase 3 or use placeholder

### Chapter 2: Splitting in Two
✅ **alabama_round_1_2_regions.png** (544 KB)
- Source: `outputs/V1/2020/states/alabama/maps/rounds/round_01.png`
- Shows: Alabama split into 2 regions (blue/orange)
- Region A: ~2.15M people (42.8%) → 3 districts
- Region B: ~2.88M people (57.2%) → 4 districts

### Chapter 3: The Recursive Magic
✅ **alabama_round_1_2_regions.png** (544 KB)
- Same as Chapter 2 (reused)
- Shows: Round 1 of recursive bisection

✅ **alabama_round_2_4_regions.png** (544 KB)
- Source: `outputs/V1/2020/states/alabama/maps/rounds/round_02.png`
- Shows: Round 2 with 4 regions (some final districts, some need more splitting)

✅ **alabama_round_3_7_districts.png** (597 KB)
- Source: `outputs/V1/2020/states/alabama/maps/rounds/round_03.png`
- Shows: Final 7 districts, all balanced

## File Sizes

Total size: **~2.75 MB** for 5 images (very reasonable!)

```
522 KB - chapter1/alabama_tracts.png
544 KB - chapter2/alabama_round_1_2_regions.png
544 KB - chapter3/alabama_round_1_2_regions.png
544 KB - chapter3/alabama_round_2_4_regions.png
597 KB - chapter3/alabama_round_3_7_districts.png
```

## Manifest Generated ✅

`public/figures/manifest.json` contains paths for dynamic loading:
- Chapter 1: 1 figure
- Chapter 2: 1 figure
- Chapter 3: 3 figures
- Chapters 4-6: Empty (Phase 3)

## What's Still Missing (Expected)

### For Chapter 1:
- `adjacency_process.png` - Need to create or find this diagram

### For Chapters 4-6 (Phase 3):
- Chapter 4: national_comparison_bar.png, state_scatter.png
- Chapter 5: figure1_success_rates.png, figure1_50state_threshold.png
- Chapter 6: figure2_compactness_tradeoff.png

### Research Papers:
- All 10 PDFs (will need to compile from LaTeX or copy from artifacts/)

## How to View

1. **Start dev server** (if not running):
   ```bash
   cd web/docs
   npm run dev
   ```

2. **Visit**: http://localhost:5173

3. **Navigate**:
   - Home → Chapter 1 → See Alabama tracts map
   - Chapter 1 → Chapter 2 → See first split
   - Chapter 2 → Chapter 3 → See full recursive progression

## Image Paths in Code

All images use absolute paths from `/figures/`:
```vue
<FigureCard
  src="/figures/chapter1/alabama_tracts.png"
  alt="Census tracts in Alabama"
/>
```

Vite serves these from `public/figures/` automatically during dev and bundles them for production.

## Verification Steps

### Quick Check:
```bash
# Count figures
ls web/docs/public/figures/chapter*/*.png | wc -l
# Should return: 5

# Check file sizes
du -sh web/docs/public/figures/chapter*
# Should show: reasonable sizes (500-600KB each)
```

### Browser Check:
1. Open dev server (http://localhost:5173)
2. Open browser DevTools (F12)
3. Go to Network tab
4. Navigate to Chapter 1
5. Look for `alabama_tracts.png` - should load with 200 OK
6. Navigate to Chapter 3
7. All 3 round figures should load

## Next Steps

### Immediate (Test Phase 2):
- [ ] Verify all images load in browser
- [ ] Check image quality/resolution
- [ ] Verify captions match content
- [ ] Test on mobile (responsive images)

### Phase 3 (Chapters 4-6):
- [ ] Find/create remaining figures
- [ ] Update asset preparation script
- [ ] Copy Chapter 4-6 figures
- [ ] Compile or locate PDFs

### Phase 4 (Polish):
- [ ] Optimize images (convert to WebP)
- [ ] Add lazy loading
- [ ] Create low-res placeholders
- [ ] Implement progressive image loading

## Troubleshooting

### Images not showing?
1. Check browser console for 404 errors
2. Verify file exists: `ls web/docs/public/figures/chapter1/alabama_tracts.png`
3. Restart dev server: `npm run dev`
4. Clear browser cache: Ctrl+Shift+R

### Wrong image displayed?
1. Check figure path in .vue file
2. Verify manifest.json has correct path
3. Check filename matches exactly (case-sensitive!)

### Image quality poor?
- Current images are ~500-600KB PNG files
- Should display crisp on desktop and mobile
- If needed, can regenerate at higher DPI from source data

---

**Status**: ✅ Phase 2 assets complete! Alabama figures ready for Chapters 1-3.

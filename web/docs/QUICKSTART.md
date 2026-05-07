# Educational Docs Site - Quick Start

Get the educational redistricting website running in 3 steps!

## Prerequisites

- **Node.js 18+** and npm
- **Python 3.13+** (for asset preparation)

## Step 1: Install Dependencies

```bash
cd web/docs
npm install
```

This installs Vue 3, Vite, Tailwind, GSAP, D3.js, and other dependencies.

**Expected output**: `added 155 packages in ~40s`

## Step 2: Prepare Assets (Optional for Development)

```bash
# From project root
python scripts/web/generate_docs_site.py

# Or dry-run to see what would be copied
python scripts/web/generate_docs_site.py --dry-run
```

This copies figures from `artifacts/` and `outputs/` to `public/figures/`.

**Note**: You can skip this step for initial development. Placeholder images will show missing asset messages.

## Step 3: Start Development Server

```bash
cd web/docs
npm run dev

# Or use the batch file (Windows)
dev.bat
```

Visit **http://localhost:5173** in your browser!

## What You'll See

1. **Landing page** with animated hero and chapter cards
2. **Chapter 1** fully templated with scroll animations
3. **Chapters 2-6** as placeholders (ready for content)
4. **Research page** with 10 paper cards
5. **Navigation** that works on mobile and desktop

## Development Workflow

### Adding Content to a Chapter

1. Open `src/pages/Chapter2_Splitting.vue` (or any chapter)
2. Replace the placeholder `<ScrollSection>` with real content
3. Follow the pattern from `Chapter1_TractsToGraphs.vue`:
   - Story hook section
   - Concept explanation with `ConceptCard`
   - Figures with `FigureCard`
   - Key takeaway section
   - Next chapter CTA

### Adding Figures

1. Place figure in `public/figures/chapter{N}/`
2. Reference in component: `<FigureCard src="/figures/chapter2/my_figure.png" .../>`
3. Update `scripts/web/generate_docs_site.py` to automate copying

### Adding Interactivity

1. Create new component in `src/components/`
2. Use D3.js for data visualizations
3. Use GSAP for complex animations
4. Import and use in chapter pages

### Editing Styles

- **Colors**: Edit `tailwind.config.js` (Schoolhouse Rock palette)
- **Animations**: Edit `src/styles/schoolhouse.css` (keyframes, utilities)
- **Component-specific**: Use `<style scoped>` in .vue files

## Building for Production

```bash
# Full build process
cd web/docs
python ../../scripts/web/generate_docs_site.py  # Prepare assets
npm run build                                    # Build static site
npm run preview                                  # Test production build

# Or use batch file (Windows)
deploy.bat
```

Output will be in `dist/` directory - ready to deploy!

## Deployment Options

### Option 1: GitHub Pages (Free, Recommended)

```bash
npm run build
# Push dist/ to gh-pages branch
# Enable GitHub Pages in repo settings
```

### Option 2: Netlify

1. Run `npm run build`
2. Drag `dist/` folder to netlify.com
3. Done! (or use Netlify CLI for automation)

### Option 3: Vercel

```bash
npm install -g vercel
vercel --prod
```

### Option 4: Static Hosting

Copy `dist/` contents to any web server (Apache, Nginx, S3, etc.)

## Project Structure

```
web/docs/
├── src/
│   ├── components/     # Reusable Vue components
│   ├── pages/          # Page components (Home, Chapters, Research)
│   ├── styles/         # Custom CSS + Tailwind
│   ├── router.js       # Route definitions
│   ├── App.vue         # Root component
│   └── main.js         # App entry point
├── public/
│   ├── figures/        # Images organized by chapter
│   ├── papers/         # Research PDFs
│   └── assets/         # Icons, logos, etc.
├── index.html          # Entry point
├── package.json        # Dependencies
├── vite.config.js      # Build config
└── tailwind.config.js  # Theme config
```

## Common Tasks

### Add a new page

1. Create `src/pages/NewPage.vue`
2. Add route in `src/router.js`
3. Add navigation link in `src/components/Navigation.vue`

### Change color scheme

Edit `tailwind.config.js`:
```js
'schoolhouse': {
  'blue': '#2563eb',     // Change these!
  'orange': '#f97316',
  // ...
}
```

### Add a new component

1. Create `src/components/MyComponent.vue`
2. Import in page: `import MyComponent from '@/components/MyComponent.vue'`
3. Use: `<MyComponent />`

### Debug build issues

```bash
# Clear cache and rebuild
rm -rf dist/ node_modules/
npm install
npm run build
```

## Troubleshooting

### Dev server won't start
- Check Node.js version: `node --version` (need 18+)
- Delete `node_modules/` and run `npm install` again
- Check port 5173 isn't already in use

### Images not showing
- Run asset preparation: `python scripts/web/generate_docs_site.py`
- Check image paths: `/figures/chapter1/...` (leading slash!)
- Verify files exist in `public/figures/`

### Animations not working
- Check browser console for errors
- Verify GSAP is imported: `import gsap from 'gsap'`
- Use Intersection Observer for scroll-triggered animations

### Build fails
- Check for TypeScript/syntax errors in .vue files
- Verify all imports resolve correctly
- Run `npm run build` and check error messages

## Next Steps

1. **Explore the code** - Start with `src/pages/Home.vue` and `Chapter1_TractsToGraphs.vue`
2. **Add Chapter 2 content** - Follow the Chapter 1 pattern
3. **Customize colors** - Make it your own in `tailwind.config.js`
4. **Add interactivity** - Build D3.js visualizations in Phase 4

## Resources

- **Vue 3 Docs**: https://vuejs.org/guide/introduction.html
- **Vite Docs**: https://vitejs.dev/guide/
- **Tailwind Docs**: https://tailwindcss.com/docs
- **GSAP Docs**: https://greensock.com/docs/
- **D3.js Docs**: https://d3js.org/getting-started

## Support

For issues or questions:
1. Check `README.md` for detailed documentation
2. Review `PHASE1_COMPLETE.md` for current status
3. See main project `CLAUDE.md` for broader context

---

**Ready to build!** 🚀 Run `npm run dev` and visit http://localhost:5173

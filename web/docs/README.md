# Algorithmic Redistricting Educational Website

**"Schoolhouse Rock" Style Educational Experience**

An interactive, engaging website that makes complex redistricting algorithms accessible to general audiences through storytelling, animations, and progressive disclosure.

## Overview

This educational site walks users through 6 chapters explaining how algorithmic redistricting works:

1. **Tracts to Graphs** - Census tracts become connected networks
2. **Splitting in Two** - METIS algorithm divides regions into balanced parts
3. **Recursive Magic** - Creating any number of districts through recursion
4. **Making it Compact** - Edge-weighting for geographic sensibility (56% improvement)
5. **Voting Rights Act** - Ensuring minority representation (42% threshold)
6. **Edge-Factor Solution** - Balancing compactness with representation (137 MM districts)

## Technology Stack

- **Vue 3** - Progressive JavaScript framework
- **Vite** - Fast build tool and dev server
- **Tailwind CSS** - Utility-first CSS framework with custom Schoolhouse Rock theme
- **GSAP** - Animation library for scroll-triggered reveals
- **D3.js** - Interactive data visualizations
- **Vue Router** - Client-side routing

## Development Setup

### Prerequisites

- Node.js 18+ and npm
- Python 3.13+ (for asset preparation script)

### Installation

```bash
cd web/docs
npm install
```

### Development Server

```bash
npm run dev
```

Visit http://localhost:5173 to view the site.

### Build for Production

```bash
# Prepare assets (copy figures from artifacts/ and outputs/)
npm run prepare-assets

# Build static site
npm run build

# Preview production build
npm run preview
```

Production files will be in `dist/` directory.

## Project Structure

```
web/docs/
├── index.html              # Entry point
├── package.json            # Dependencies
├── vite.config.js          # Vite configuration
├── tailwind.config.js      # Tailwind + Schoolhouse Rock theme
├── postcss.config.js       # PostCSS configuration
├── src/
│   ├── main.js             # App entry point
│   ├── App.vue             # Root component
│   ├── router.js           # Vue Router config
│   ├── styles/
│   │   └── schoolhouse.css # Custom styles + animations
│   ├── components/
│   │   ├── Navigation.vue  # Sticky nav with progress
│   │   ├── Hero.vue        # Animated hero sections
│   │   ├── ScrollSection.vue  # Scroll-triggered sections
│   │   ├── FigureCard.vue  # Figure display
│   │   └── ConceptCard.vue # Concept cards
│   └── pages/
│       ├── Home.vue        # Landing page
│       ├── Chapter1_TractsToGraphs.vue
│       ├── Chapter2_Splitting.vue
│       ├── Chapter3_Recursion.vue
│       ├── Chapter4_Compactness.vue
│       ├── Chapter5_VRA.vue
│       ├── Chapter6_EdgeFactor.vue
│       └── Research.vue    # Papers library
└── public/
    ├── figures/            # Copied from artifacts/outputs
    ├── papers/             # PDFs
    └── assets/             # Icons, backgrounds
```

## Design System

### Colors (Schoolhouse Rock Palette)

- **Blue** (#2563eb) - Primary, trustworthy (Chapter 1)
- **Orange** (#f97316) - Energy, action (Chapter 2)
- **Green** (#10b981) - Growth, positive (Chapter 3)
- **Purple** (#8b5cf6) - Creativity, algorithms (Chapter 4)
- **Red** (#ef4444) - Important, critical (Chapter 5)
- **Yellow** (#fbbf24) - Attention, caution (Chapter 6)

### Typography

- **Headings**: Nunito (rounded, playful)
- **Body**: Inter (clean, readable)
- **Code**: JetBrains Mono

### Animations

- Scroll-triggered reveals (fade-in, slide-up)
- Figure zoom-in effects
- Interactive hover states
- Smooth page transitions

## Content Guidelines

### Writing Style

- **Simple language** - Avoid jargon, explain technical terms
- **Story-driven** - Use metaphors and real-world examples
- **Progressive disclosure** - Start simple, add complexity gradually
- **Engaging tone** - Playful but informative (Schoolhouse Rock vibe)

### Chapter Structure

Each chapter follows this pattern:

1. **Hero** - Big title + subtitle
2. **Story Hook** - "Imagine..." opening with relatable metaphor
3. **Concept Explanation** - Core idea with visuals
4. **Interactive Demo** - Hands-on exploration
5. **Real Example** - Minnesota/Alabama case studies
6. **Key Takeaway** - One-sentence summary in bold
7. **Research Links** - "Learn more" links to papers

## Deployment

### GitHub Pages (Recommended)

```bash
# Build production site
npm run build

# Deploy to gh-pages branch
# (script coming in Phase 6)
```

### Static Hosting

The built site (`dist/` directory) is fully static and can be hosted anywhere:

- Netlify
- Vercel
- GitHub Pages
- Any web server
- Zipped and shared (works offline!)

## Implementation Phases

- **Phase 1** ✅ - Foundation (scaffold, design system, navigation)
- **Phase 2** - Chapters 1-3 (content + basic interactivity)
- **Phase 3** - Chapters 4-6 (advanced concepts)
- **Phase 4** - Interactive features (Graph Builder, Split Simulator, etc.)
- **Phase 5** - Research integration (paper links, figure galleries)
- **Phase 6** - Polish + deploy (optimization, accessibility, SEO)

## Assets

### Figures to Include

High-priority figures from research papers and outputs:

- Census tract maps (Vermont, Minnesota, Alabama)
- Progressive bisection sequences (round-by-round)
- National compactness comparisons
- VRA threshold analysis charts
- Edge-factor tradeoff graphs
- 50 state final maps

### Asset Preparation Script

```bash
python scripts/web/generate_docs_site.py
```

This script:
- Copies figures from `artifacts/` and `outputs/`
- Organizes by chapter
- Generates JSON manifest for dynamic loading
- Updates paths in components

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (responsive design)

## Accessibility

- ARIA labels on interactive elements
- Keyboard navigation support
- Skip-to-content link
- Alt text on all images
- High contrast color palette
- Focus indicators

## Performance Targets

- Initial load: <3s
- Page navigation: <1s
- Lighthouse score: 90+
- Mobile-friendly

## License

Educational use only. Part of the Algorithmic Redistricting Research Project.

## Contributing

See main project CONTRIBUTING.md for guidelines.

---

**Created with Vue 3 + Vite + Tailwind**
Making redistricting algorithms fun since 2026! 🎨📊🗳️

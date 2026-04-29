# Documentation Directory

**Last Updated**: January 17, 2026

This directory contains **human-friendly** documentation for users, researchers, and developers. If you're new to the project, start here!

## For Different Audiences

### I'm New to This Project

**Start here**:
1. [GETTING_STARTED.md](GETTING_STARTED.md) - Complete setup and first-run tutorial (30 minutes)
2. [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) - How to read the maps and dashboards (15 minutes)
3. [DATA_DICTIONARY.md](DATA_DICTIONARY.md) - Understanding the output data (reference)

**If you run into problems**:
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Solutions to common errors

### I'm a Researcher or Data Analyst

**Understanding the algorithm**:
- [RECURSIVE_BISECTION.md](RECURSIVE_BISECTION.md) - How redistricting works
- [DATA_DICTIONARY.md](DATA_DICTIONARY.md) - Field definitions for statistical analysis
- [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) - Interpreting compactness and political metrics

**Data sources**:
- [CENSUS_DATA_ANALYSIS.md](CENSUS_DATA_ANALYSIS.md) - Census data details

### I'm a Developer

**Getting started**:
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development workflow, git practices
- [DEPENDENCIES.md](DEPENDENCIES.md) - Software requirements and installation

**For development work, also see**:
- `../context/ARCHITECTURE.md` - System design (AI-optimized, compact)
- `../context/CODING_PATTERNS.md` - Code conventions (AI-optimized)
- `../context/SKILLS.md` - Claude Code skills for automation
- `../CLAUDE.md` - AI assistant guide

### I'm Using AI Assistants (Claude Code, etc.)

The `context/` directory contains AI-optimized documentation (compact, token-efficient). However, when helping users, reference these human-friendly docs in `docs/` instead.

**Key principle**: Two documentation systems for two audiences:
- **`docs/`** - Human-friendly (detailed, examples, explanations)
- **`context/`** - AI-optimized (compact, symbolic, pattern-first)

## Documentation Index

### User Guides (Detailed, Examples, Step-by-Step)

| File | Purpose | Read Time |
|------|---------|-----------|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Complete setup and first run | 30 min |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Common errors and solutions | 20 min (reference) |
| [DATA_DICTIONARY.md](DATA_DICTIONARY.md) | Field-by-field output explanation | 30 min (reference) |
| [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) | Reading maps and dashboards | 25 min |

### Algorithm & Technical

| File | Purpose | Read Time |
|------|---------|-----------|
| [RECURSIVE_BISECTION.md](RECURSIVE_BISECTION.md) | Algorithm explanation | 15 min |
| [REDIST_CLI.md](REDIST_CLI.md) | Rust `redist` CLI reference | 10 min |
| [DEPENDENCIES.md](DEPENDENCIES.md) | Software requirements | 10 min |
| [CENSUS_DATA_ANALYSIS.md](CENSUS_DATA_ANALYSIS.md) | Census data details | 15 min |

### Project Information

| File | Purpose | Read Time |
|------|---------|-----------|
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development workflow | 15 min |
| [CHANGELOG.md](CHANGELOG.md) | Version history | 5 min (reference) |
| [ENHANCEMENTS_2026.md](ENHANCEMENTS_2026.md) | 2026 improvements | 5 min |

## Quick Links

**Having trouble?**
→ [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

**Want to understand the output?**
→ [DATA_DICTIONARY.md](DATA_DICTIONARY.md)

**Don't know where to start?**
→ [GETTING_STARTED.md](GETTING_STARTED.md)

**Want to contribute?**
→ [CONTRIBUTING.md](CONTRIBUTING.md)

## Documentation Philosophy

### Why Two Documentation Systems?

**Human Docs (`docs/`)**:
- Optimized for clarity and accessibility
- Detailed explanations with examples
- Step-by-step walkthroughs
- Assumes reader is new to project

**AI Context (`context/`)**:
- Optimized for AI token efficiency
- Compact notation (symbols, abbreviations)
- Pattern-first, minimal prose
- Inline documentation

**Example Comparison**:

**Human version** (docs/DATA_DICTIONARY.md):
```markdown
### polsby_popper
**Type**: Float (0.0 to 1.0)

**Description**: Polsby-Popper compactness score measuring how close
a district's shape is to a perfect circle.

**Formula**: 4π × Area / Perimeter²

**Example Values**:
- 1.0 = Perfect circle (theoretical maximum)
- 0.67 = Fairly compact (e.g., Wyoming at-large)
- 0.34 = Moderately compact (typical urban district)
- 0.14 = Highly irregular (potential gerrymander)

**Interpretation**:
[Table showing score ranges and compactness levels]

**Why it matters**: Lower scores may indicate gerrymandering...
```

**AI version** (context/DATA_FORMATS.md):
```markdown
### district_summary.csv
`district,population,area_sq_km,perimeter_km,polsby_popper,reock,...`

**polsby_popper**: Float [0,1], 4π×A/P², circle=1, gerrymander<0.15
```

**Key Differences**:
- Human: 200 words, detailed interpretation
- AI: 20 words, formula + key thresholds
- Both accurate, different purposes

## Finding Information

**Installation issue?**
→ [TROUBLESHOOTING.md](TROUBLESHOOTING.md) → "Installation Issues" section

**Don't understand a CSV column?**
→ [DATA_DICTIONARY.md](DATA_DICTIONARY.md) → Table of Contents → Field name

**Want to change the algorithm?**
→ [CONTRIBUTING.md](CONTRIBUTING.md) → Development workflow
→ `../context/CODING_PATTERNS.md` → Code conventions

**Need to explain compactness to a non-technical audience?**
→ [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) → "Interpreting Compactness" section

## Contributing to Documentation

**Found an error?**
- Open an issue or submit a pull request
- See [CONTRIBUTING.md](CONTRIBUTING.md)

**Want to add a new guide?**
- Follow existing structure (Last Updated, Table of Contents, Examples)
- Optimize for human readers (clarity over brevity)
- Add to this README index

**AI documentation?**
- AI-optimized docs go in `context/`, not `docs/`
- See `context/SKILL_COMPACTION_GUIDE.md` for AI doc patterns

## Updates

This documentation is actively maintained. Recent additions:

**January 17, 2026**:
- ✅ Created [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Comprehensive error solutions
- ✅ Created [DATA_DICTIONARY.md](DATA_DICTIONARY.md) - Field-by-field explanations
- ✅ Created [GETTING_STARTED.md](GETTING_STARTED.md) - Complete setup guide
- ✅ Created [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) - Map interpretation guide
- ✅ Updated README.md to prominently feature user-friendly docs

**Previously**:
- RECURSIVE_BISECTION.md - Algorithm explanation
- DEPENDENCIES.md - Installation requirements
- CONTRIBUTING.md - Development workflow
- CHANGELOG.md - Version history

See [CHANGELOG.md](CHANGELOG.md) for full project history.

# Contributing to Congressional Redistricting System

## Quick Start: Using Claude Code Skills

**This project includes 31 Claude Code skills** that automate common development tasks. Simply describe what you want to do naturally, and Claude will offer the appropriate skill.

**Example workflows**:
- "I want to plan a new feature" → Claude suggests `/enhancement-plan`
- "Run redistricting for 2020" → Claude suggests `/run-redistricting`
- "Consolidate these duplicate scripts" → Claude suggests `/consolidate-scripts`
- "Compile my LaTeX paper" → Claude suggests `/compile-latex`

**See [SKILLS.md](../context/SKILLS.md) for complete documentation** of all 31 available skills across:
- Enhancement workflow
- Data management
- Pipeline execution
- Visualization
- Documentation
- Research & analysis
- Code organization

---

## Git Workflow

This project uses a **feature branch + pull request** workflow for all changes.

### Initial Setup

1. **Clone the repository** (if working from another machine):
   ```bash
   git clone <repository-url>
   cd apportionment
   ```

2. **Configure your identity**:
   ```bash
   git config user.name "Your Name"
   git config user.email "your.email@example.com"
   ```

3. **Download data files** (not in git):
   ```bash
   # Recommended: use the Rust CLI to download all census data
   redist fetch --year 2020           # TIGER + redistricting data
   redist fetch --year 2020 --release # Also pull pre-built adjacency files from GitHub Releases

   # Convert adjacency pkl files to fast native format (after --release download)
   python scripts/data/generate_adj_bin.py --year 2020

   # Alternative: Python download orchestrator (slower)
   python scripts/data/download_orchestrator.py --stages redistricting --year 2020
   ```

---

## Making Changes

### 1. Create a Feature Branch

Always create a new branch for your work:

```bash
# Update main branch
git checkout main
git pull origin main

# Create and switch to feature branch
git checkout -b feature/your-feature-name

# Examples:
git checkout -b feature/add-2000-census-support
git checkout -b fix/unicode-encoding-issue
git checkout -b enhance/compactness-metrics
```

**Branch naming conventions**:
- `feature/` - New features
- `fix/` - Bug fixes
- `enhance/` - Improvements to existing features
- `docs/` - Documentation updates
- `refactor/` - Code refactoring

### 2. Make Your Changes

Work on your feature branch:

```bash
# Check status frequently
git status

# Stage specific files
git add scripts/some_script.py
git add config_2020.py

# Or stage all changes
git add -A

# Commit with descriptive message
git commit -m "Add 2000 census configuration

- Created config_2000.py with state apportionments
- Updated download scripts to handle 2000 API format
- Added column mapping for GEOID00, ALAND00, etc."
```

### 3. Push Your Branch

```bash
# First time pushing a new branch
git push -u origin feature/your-feature-name

# Subsequent pushes
git push
```

### 4. Create a Pull Request

1. Go to your repository on GitHub/GitLab/Bitbucket
2. Click "New Pull Request" or "Create Pull Request"
3. Select your feature branch as the source
4. Select `main` as the target
5. Fill in the PR template:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Enhancement
- [ ] Documentation
- [ ] Refactoring

## Changes Made
- List specific changes
- Use bullet points
- Be detailed

## Testing Done
- [ ] Tested with --print-only mode
- [ ] Ran full pipeline for test state (e.g., DE)
- [ ] Verified output files are correct
- [ ] No errors in logs

## Screenshots (if applicable)
Attach screenshots of maps, outputs, etc.

## Notes
Any additional context or concerns
```

### 5. Review and Merge

- Wait for code review
- Address any feedback
- Once approved, merge the PR
- Delete the feature branch after merging

---

## Best Practices

### Commit Messages

**Good commit messages**:
```bash
git commit -m "Fix Unicode encoding errors on Windows console

- Replaced box-drawing characters with ASCII hyphens
- Affects all print-only mode outputs
- Resolves UnicodeEncodeError in Windows cmd.exe"
```

**Bad commit messages**:
```bash
git commit -m "fix bug"
git commit -m "updates"
git commit -m "wip"
```

### Commit Frequency

- Commit logical units of work
- Commit frequently (every 30-60 minutes of work)
- Don't commit broken code to main
- Use feature branches for experimental work

### What to Commit

**DO commit**:
- ✅ Source code (`scripts/`, `src/`)
- ✅ Configuration files (`config_2020.py`, `config_2010.py`)
- ✅ Documentation (`.md` files)
- ✅ Requirements (`requirements.txt`)
- ✅ Tests
- ✅ Build scripts

**DON'T commit**:
- ❌ Data files (`data/raw/*.parquet`, `data/adjacency/*.pkl`)
- ❌ Output files (`outputs/`)
- ❌ Generated maps (`.png`, `.jpg`)
- ❌ Cache files (`__pycache__/`, `*.pyc`)
- ❌ IDE files (`.vscode/`, `.idea/`)
- ❌ Environment files (`.env`)
- ❌ Personal notes (unless useful to team)

---

## Common Workflows

### Add Support for New Census Year

**Using Claude Code Skills**: Ask Claude "I want to add 2000 census support" and it will guide you through using:
- `/enhancement-plan` - Create enhancement specification
- `/enhancement-implement` - Execute the implementation
- `/census-download` - Download 2000 census data
- `/adjacency-build` - Build adjacency graphs
- `/enhancement-document` - Complete documentation

**Manual workflow**:
```bash
# Create feature branch
git checkout -b feature/add-2000-census

# Create config file
cp config_2020.py config_2000.py
# Edit config_2000.py with 2000 apportionment

# Update scripts to support year 2000
# ... make changes ...

# Test
python scripts/run_complete_redistricting.py --year 2000 --print-only

# Commit and push
git add config_2000.py
git add scripts/*.py
git commit -m "Add 2000 census support"
git push -u origin feature/add-2000-census

# Create PR on GitHub
```

### Fix a Bug

```bash
# Create fix branch
git checkout -b fix/unicode-encoding

# Fix the bug
# ... make changes ...

# Test thoroughly
python scripts/run_all_states.py --year 2020 --print-only CA

# Commit with clear description
git add scripts/run_all_states.py
git commit -m "Fix Unicode encoding error in print-only mode

Windows console can't render box-drawing characters.
Replaced with ASCII hyphens throughout."

# Push and create PR
git push -u origin fix/unicode-encoding
```

### Update Documentation

**Using Claude Code Skills**: Ask Claude "Update the documentation" and it will use:
- `/update-docs` - Systematically review and update all docs
- `/create-architecture-diagram` - Create/update Mermaid diagrams
- `/create-session-archive` - Archive session notes after major work

**Manual workflow**:
```bash
git checkout -b docs/update-readme

# Update documentation
vim README.md

# Commit
git add README.md
git commit -m "Update README with 2010 census instructions"
git push -u origin docs/update-readme
```

### Plan and Implement an Enhancement

**Using Claude Code Skills** (recommended):
```
1. Ask: "I want to add [feature name]"
2. Claude suggests: /enhancement-plan
3. Review and approve the plan
4. Claude suggests: /enhancement-implement
5. Claude implements with todo tracking
6. Claude suggests: /enhancement-document
7. Complete with updated documentation
```

**Enhancement Priorities**:

When creating enhancements, assign a priority to help with planning:

- **Critical**: Blocks publication or breaks core functionality → work on immediately
- **High**: Answers key research questions, significant impact → next 1-2 sprints
- **Medium**: Improves quality/capabilities but not urgent → next quarter
- **Low**: Quality-of-life improvements, polish → when time permits
- **Research**: Experimental/exploratory work, uncertain value → no timeline

View enhancements by priority: [INDEX.md - By Priority](../context/enhancements/INDEX.md#by-priority)

See [SKILLS.md](../context/SKILLS.md) for detailed enhancement workflow documentation.

---

## Working with Data Files

Data files are **excluded from git** because they're large (GBs).

### Sharing Data Files

If you need to share data files with collaborators:

1. **Use cloud storage**: Upload to S3, Google Drive, etc.
2. **Document download instructions** in README.md
3. **Provide download scripts**: Use existing `download_*.py` scripts

### Regenerating Data

Anyone can regenerate data files:

```bash
# Download from Census Bureau (Rust CLI — recommended)
redist fetch --year 2020
redist fetch --year 2020 --release          # includes adjacency files
python scripts/data/generate_adj_bin.py --year 2020  # convert to fast format

# Alternative: Python download orchestrator
python scripts/data/download_orchestrator.py --stages redistricting --year 2020
python scripts/data/download_orchestrator.py --stages adjacency --year 2020
```

---

## Code Review Guidelines

### For Authors

- Keep PRs focused (one feature/fix per PR)
- Write clear PR descriptions
- Test thoroughly before requesting review
- Respond to feedback promptly
- Don't take feedback personally

### For Reviewers

- Review within 24-48 hours
- Be constructive and specific
- Suggest improvements, don't just criticize
- Approve when ready, request changes if needed
- Test the changes if possible

---

## Release Process

### Versioning

Use semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (e.g., 1.0.0 → 2.0.0)
- **MINOR**: New features (e.g., 1.0.0 → 1.1.0)
- **PATCH**: Bug fixes (e.g., 1.0.0 → 1.0.1)

### Creating a Release

```bash
# Update version in relevant files
# Commit version bump
git add -A
git commit -m "Bump version to 1.1.0"

# Tag the release
git tag -a v1.1.0 -m "Release 1.1.0 - 2010 Census Support

- Added 2010 census configuration
- Multi-year pipeline support
- Non-scrolling progress bars"

# Push tags
git push origin v1.1.0
git push --tags

# Create release on GitHub with release notes
```

---

## Questions?

- Check existing PRs for examples
- Ask in issues/discussions
- Review git documentation: https://git-scm.com/doc

---

**Remember**: Commit early, commit often, use feature branches, and write good commit messages!

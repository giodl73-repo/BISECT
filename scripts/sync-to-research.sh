#!/bin/bash
#
# Sync research files from apportionment repo to research monorepo (gerry module)
#
# Usage:
#   ./scripts/sync-to-research.sh              # Normal sync (local only)
#   ./scripts/sync-to-research.sh --push       # Sync and push to remote
#   ./scripts/sync-to-research.sh --dry-run    # Preview what would happen
#   ./scripts/sync-to-research.sh --message "Custom commit message"
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$SCRIPT_DIR")"
RESEARCH_REPO="https://github.com/giodl_microsoft/research.git"
RESEARCH_DIR="${SOURCE_DIR}/../research"
GERRY_DIR="${RESEARCH_DIR}/gerry"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
PUSH_CHANGES=false
DRY_RUN=false
CUSTOM_MESSAGE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH_CHANGES=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --message|-m)
            CUSTOM_MESSAGE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--push] [--dry-run] [--message \"msg\"]"
            exit 1
            ;;
    esac
done

echo ""
echo -e "${BLUE}=== Gerry Research Sync ===${NC}"
echo "Source: $SOURCE_DIR/research"
echo "Target: $GERRY_DIR"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No changes will be made${NC}"
    echo ""
fi

# Verify source research directory exists
if [ ! -d "$SOURCE_DIR/research" ]; then
    echo -e "${RED}Error: research/ directory not found in $SOURCE_DIR${NC}"
    exit 1
fi

# Clone or update research monorepo
if [ -d "$RESEARCH_DIR/.git" ]; then
    echo -e "${GREEN}Found existing research repo${NC}"
    if [ "$DRY_RUN" = false ]; then
        cd "$RESEARCH_DIR"
        git fetch origin 2>/dev/null || true
        git checkout main 2>/dev/null || git checkout master 2>/dev/null || git checkout -b main
        git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || true
    fi
else
    echo -e "${YELLOW}Cloning research repo...${NC}"
    if [ "$DRY_RUN" = false ]; then
        git clone "$RESEARCH_REPO" "$RESEARCH_DIR" 2>/dev/null || {
            echo -e "${YELLOW}Repo doesn't exist yet, creating new repo...${NC}"
            mkdir -p "$RESEARCH_DIR"
            cd "$RESEARCH_DIR"
            git init
            git checkout -b main
            git remote add origin "$RESEARCH_REPO"
        }
    else
        echo "  Would clone: $RESEARCH_REPO"
    fi
fi

# Ensure gerry directory exists
if [ "$DRY_RUN" = false ]; then
    mkdir -p "$GERRY_DIR"
fi

# Files to sync
echo ""
echo "Syncing research/ contents to gerry/:"
echo "  - panel-* (paper directories)"
echo "  - docs/ (PDFs + markdown)"
echo "  - Makefile"
echo "  - RESEARCH.md, REVIEW_PANEL.md, REVIEWERS.md"

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${YELLOW}DRY RUN - Would sync the above files${NC}"
    echo ""
    exit 0
fi

# Clean existing gerry files
echo ""
echo "Cleaning existing gerry/ files..."
if [ -d "$GERRY_DIR" ]; then
    find "$GERRY_DIR" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
fi

# Copy research files to gerry/
echo "Copying research files..."
cd "$SOURCE_DIR"

# Paper directories
for dir in research/panel-*/; do
    if [ -d "$dir" ]; then
        cp -r "$dir" "$GERRY_DIR/"
        echo "  ✓ $(basename "$dir")/"
    fi
done

# Docs (PDFs + markdown)
if [ -d "research/docs" ]; then
    cp -r research/docs "$GERRY_DIR/"
    echo "  ✓ docs/"
fi

# Makefile
if [ -f "research/Makefile" ]; then
    cp research/Makefile "$GERRY_DIR/"
    echo "  ✓ Makefile"
fi

# Markdown files (module-specific only; global files live at repo root)
for md in RESEARCH.md REVIEW_PANEL.md REVIEWERS.md; do
    if [ -f "research/$md" ]; then
        cp "research/$md" "$GERRY_DIR/"
        echo "  ✓ $md"
    fi
done

# Show what changed (from research repo root)
cd "$RESEARCH_DIR"
echo ""
echo "Changes:"
git status --short

# Build commit message
SOURCE_COMMIT="$(cd "$SOURCE_DIR" && git rev-parse --short HEAD)"
SYNC_DATE="$(date -u +"%Y-%m-%d %H:%M:%S UTC")"

if [ -n "$CUSTOM_MESSAGE" ]; then
    COMMIT_MSG="[gerry] $CUSTOM_MESSAGE

Source commit: $SOURCE_COMMIT
Synced: $SYNC_DATE"
else
    COMMIT_MSG="[gerry] Sync from apportionment repo

Source commit: $SOURCE_COMMIT
Synced: $SYNC_DATE"
fi

# Commit changes
echo ""
git add -A
if git diff --cached --quiet; then
    echo -e "${YELLOW}No changes to commit - already in sync${NC}"
else
    git commit -m "$COMMIT_MSG"
    echo -e "${GREEN}✓ Changes committed${NC}"

    if [ "$PUSH_CHANGES" = true ]; then
        echo ""
        echo "Pushing to remote..."
        git push -u origin main 2>/dev/null || git push -u origin master 2>/dev/null || {
            echo -e "${RED}Push failed. You may need to set up the remote:${NC}"
            echo "  cd $RESEARCH_DIR"
            echo "  git remote add origin $RESEARCH_REPO"
            echo "  git push -u origin main"
            exit 1
        }
        echo -e "${GREEN}✓ Pushed to remote${NC}"
    else
        echo ""
        echo -e "${YELLOW}Committed locally. Run with --push to push to remote.${NC}"
    fi
fi

echo ""
echo -e "${GREEN}=== Sync complete ===${NC}"
echo "Research directory: $GERRY_DIR"
echo ""

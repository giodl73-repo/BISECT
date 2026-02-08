#!/usr/bin/env python3
"""
Research Paper Status Checker

Automatically determines the status of all research papers in the portfolio.

Usage:
    python research/check_paper_status.py                    # Show all papers
    python research/check_paper_status.py --paper NAME       # Show specific paper
    python research/check_paper_status.py --phase review     # Filter by phase
    python research/check_paper_status.py --summary          # Brief summary only
"""

import os
import sys
from pathlib import Path
from typing import Dict, List, Optional
import argparse


class PaperStatus:
    """Represents the status of a research paper."""

    PHASES = {
        'design': '[DESIGN]',
        'analysis': '[ANALYSIS]',
        'writing': '[WRITING]',
        'review': '[REVIEW]',
        'complete': '[COMPLETE]',
        'accepted': '[ACCEPTED]'
    }

    def __init__(self, directory: Path):
        self.directory = directory
        self.name = directory.name
        self.path = directory

        # Scan for indicators
        self.has_pdf = (directory / "main.pdf").exists()
        self.has_reviews = (directory / "reviews").exists()
        self.review_count = self._count_reviews()
        self.has_results = self._check_results()
        self.has_figures = self._check_figures()
        self.status_docs = self._count_status_docs()

        # PDF page counts
        self.main_pages = self._get_pdf_pages("main.pdf")
        self.has_supplement = self._check_supplement()
        self.supplement_pages = self._get_supplement_pages()

        # Determine phase and progress
        self.phase = self._determine_phase()
        self.progress = self._estimate_progress()
        self.status_text = self._get_status_text()

    def _count_reviews(self) -> int:
        """Count review files."""
        reviews_dir = self.directory / "reviews"
        if not reviews_dir.exists():
            return 0
        return len(list(reviews_dir.glob("*.md")))

    def _check_results(self) -> bool:
        """Check if results directory exists and has data."""
        results_dir = self.directory / "results"
        if not results_dir.exists():
            return False
        csv_files = list(results_dir.glob("*.csv"))
        return len(csv_files) > 0

    def _check_figures(self) -> bool:
        """Check if figures directory exists."""
        figures_dir = self.directory / "figures"
        if not figures_dir.exists():
            return False
        png_files = list(figures_dir.glob("*.png"))
        return len(png_files) > 0

    def _count_status_docs(self) -> int:
        """Count status documentation files."""
        patterns = ["*COMPLETE*.md", "*STATUS*.md", "*REVISION*.md", "*PLAN*.md"]
        count = 0
        for pattern in patterns:
            count += len(list(self.directory.glob(pattern)))
        return count

    def _get_pdf_pages(self, filename: str) -> int:
        """Get page count from PDF file."""
        pdf_path = self.directory / filename
        if not pdf_path.exists():
            return 0

        # Try pypdf (new PyPDF2)
        try:
            from pypdf import PdfReader
            with open(pdf_path, 'rb') as f:
                reader = PdfReader(f)
                return len(reader.pages)
        except:
            pass

        # Try PyPDF2 (old)
        try:
            import PyPDF2
            with open(pdf_path, 'rb') as f:
                reader = PyPDF2.PdfReader(f)
                return len(reader.pages)
        except:
            pass

        # Try pdfinfo command-line tool
        try:
            import subprocess
            result = subprocess.run(['pdfinfo', str(pdf_path)],
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                for line in result.stdout.split('\n'):
                    if line.startswith('Pages:'):
                        return int(line.split(':')[1].strip())
        except:
            pass

        # If all methods fail, return 0
        return 0

    def _check_supplement(self) -> bool:
        """Check if supplementary document exists."""
        supplement_names = ["main_supplement.pdf", "supplement.pdf", "supplementary.pdf",
                           "main_suppl.pdf", "appendix.pdf"]
        for name in supplement_names:
            if (self.directory / name).exists():
                return True
        return False

    def _get_supplement_pages(self) -> int:
        """Get page count from supplementary document."""
        if not self.has_supplement:
            return 0
        supplement_names = ["main_supplement.pdf", "supplement.pdf", "supplementary.pdf",
                           "main_suppl.pdf", "appendix.pdf"]
        for name in supplement_names:
            pages = self._get_pdf_pages(name)
            if pages > 0:
                return pages
        return 0

    def _determine_phase(self) -> str:
        """Determine current phase based on indicators."""
        # Check _panel.yaml first (authoritative source)
        panel_yaml = self.directory / "_panel.yaml"
        if panel_yaml.exists():
            try:
                with open(panel_yaml, 'r', encoding='utf-8') as f:
                    content = f.read()
                    # Look for stage: accepted or stage: ready
                    if 'stage: accepted' in content:
                        return 'accepted'
                    elif 'stage: ready' in content:
                        return 'complete'
            except:
                pass

        # Check for completion markers
        if (self.directory / "PAPER_COMPLETE.md").exists():
            return 'complete'

        if (self.directory / "ACCEPTED.md").exists():
            return 'accepted'

        # Check for review phase
        if self.has_reviews and self.review_count >= 5:
            return 'review'

        # Check for writing phase
        if self.has_pdf and self.has_results:
            if self.has_reviews:
                return 'review'
            return 'writing'

        # Check for analysis phase
        if self.has_results:
            return 'analysis'

        # Check for design phase
        if self.status_docs > 0 or (self.directory / "README.md").exists():
            return 'design'

        return 'design'

    def _estimate_progress(self) -> int:
        """Estimate completion percentage."""
        score = 0

        # Base indicators (0-40)
        if (self.directory / "README.md").exists(): score += 5
        if (self.directory / "main.tex").exists(): score += 10
        if self.has_results: score += 15
        if self.has_figures: score += 10

        # PDF and compilation (40-60)
        if self.has_pdf: score += 20

        # Review process (60-90)
        if self.has_reviews:
            score += 10
            if self.review_count >= 5: score += 10
            if self.review_count >= 10: score += 10

        # Completion markers (90-100)
        if (self.directory / "PAPER_COMPLETE.md").exists(): score += 10
        if (self.directory / "REVISION_PLAN.md").exists(): score += 5

        return min(100, score)

    def _get_status_text(self) -> str:
        """Get human-readable status."""
        if self.phase == 'accepted':
            # Try to get venue from _panel.yaml
            venue = self._get_accepted_venue()
            if venue:
                return f"Accepted at {venue}"
            return "Accepted for publication"
        elif self.phase == 'complete':
            return "Ready for submission"
        elif self.phase == 'review':
            rounds = self._detect_review_round()
            return f"Round {rounds} review"
        elif self.phase == 'writing':
            return "Paper writing in progress"
        elif self.phase == 'analysis':
            return "Data analysis phase"
        else:
            return "Experimental design"

    def _get_accepted_venue(self) -> Optional[str]:
        """Get accepted venue from _panel.yaml."""
        panel_yaml = self.directory / "_panel.yaml"
        if not panel_yaml.exists():
            return None
        try:
            with open(panel_yaml, 'r', encoding='utf-8') as f:
                for line in f:
                    if 'accepted_venue:' in line:
                        venue = line.split('accepted_venue:')[1].strip().strip('"').strip("'")
                        return venue
        except:
            pass
        return None

    def _detect_review_round(self) -> int:
        """Detect which review round."""
        reviews_dir = self.directory / "reviews"
        if not reviews_dir.exists():
            return 0

        # Check for round markers
        if list(reviews_dir.glob("ROUND3*.md")):
            return 3
        elif list(reviews_dir.glob("ROUND2*.md")):
            return 2
        elif list(reviews_dir.glob("ROUND1*.md")) or self.review_count > 0:
            return 1
        return 0

    def get_emoji_status(self) -> str:
        """Get emoji for current phase."""
        return self.PHASES.get(self.phase, '❓')

    def get_progress_bar(self, width: int = 20) -> str:
        """Generate progress bar."""
        filled = int(width * self.progress / 100)
        bar = '#' * filled + '-' * (width - filled)
        return f"[{bar}] {self.progress}%"

    def __str__(self) -> str:
        """String representation."""
        return f"{self.name}: {self.get_emoji_status()} {self.status_text} ({self.progress}%)"


def scan_papers(research_dir: Path) -> List[PaperStatus]:
    """Scan all paper directories."""
    papers = []
    for item in sorted(research_dir.iterdir()):
        if item.is_dir() and item.name.startswith("gerry-"):
            papers.append(PaperStatus(item))
    return papers


def print_summary(papers: List[PaperStatus]):
    """Print brief summary."""
    print("\n" + "="*70)
    print(" Research Paper Portfolio Status")
    print("="*70)

    # Count by phase
    by_phase = {}
    for paper in papers:
        phase = paper.phase
        by_phase[phase] = by_phase.get(phase, 0) + 1

    print(f"\nTotal Papers: {len(papers)}")
    print("\nBy Phase:")
    for phase in ['accepted', 'complete', 'review', 'writing', 'analysis', 'design']:
        count = by_phase.get(phase, 0)
        emoji = PaperStatus.PHASES[phase]
        print(f"  {emoji}: {count} papers")

    # Average progress
    avg_progress = sum(p.progress for p in papers) / len(papers) if papers else 0
    print(f"\nAverage Progress: {avg_progress:.1f}%")

    print("\n" + "="*70)


def print_detailed(papers: List[PaperStatus]):
    """Print detailed status for all papers."""
    print("\n" + "="*70)
    print(" Research Paper Portfolio - Detailed Status")
    print("="*70 + "\n")

    # Group by phase
    by_phase = {}
    for paper in papers:
        phase = paper.phase
        if phase not in by_phase:
            by_phase[phase] = []
        by_phase[phase].append(paper)

    # Print each phase
    for phase in ['complete', 'review', 'writing', 'analysis', 'design', 'accepted']:
        if phase not in by_phase:
            continue

        papers_in_phase = by_phase[phase]
        emoji = PaperStatus.PHASES[phase]
        print(f"\n{emoji} {phase.upper()} ({len(papers_in_phase)} papers)")
        print("-" * 70)

        for paper in sorted(papers_in_phase, key=lambda p: -p.progress):
            print(f"\n  {paper.name.replace('gerry-', '').replace('-', ' ').title()}")
            print(f"  Status: {paper.status_text}")
            print(f"  Progress: {paper.get_progress_bar()}")

            # Key indicators
            indicators = []
            if paper.has_pdf:
                pdf_info = "[OK] PDF"
                if paper.main_pages > 0:
                    pdf_info += f" ({paper.main_pages}p)"
                    if paper.has_supplement and paper.supplement_pages > 0:
                        pdf_info += f" + Suppl ({paper.supplement_pages}p)"
                indicators.append(pdf_info)
            if paper.has_results:
                indicators.append("[OK] Data")
            if paper.has_figures:
                indicators.append("[OK] Figures")
            if paper.has_reviews:
                indicators.append(f"[OK] Reviews ({paper.review_count})")

            if indicators:
                print(f"  Indicators: {', '.join(indicators)}")

    print("\n" + "="*70)


def print_paper_detail(paper: PaperStatus):
    """Print detailed info for single paper."""
    print("\n" + "="*70)
    print(f" {paper.name.replace('gerry-', '').replace('-', ' ').title()}")
    print("="*70 + "\n")

    print(f"Phase: {paper.get_emoji_status()} {paper.phase.upper()}")
    print(f"Status: {paper.status_text}")
    print(f"Progress: {paper.get_progress_bar(40)}")
    print(f"Directory: {paper.path}")

    print("\nIndicators:")
    pdf_info = f"PDF (main.pdf)"
    if paper.main_pages > 0:
        pdf_info += f" - {paper.main_pages} pages"
    print(f"  {'[OK]' if paper.has_pdf else '[ ]'} {pdf_info}")
    if paper.has_supplement:
        supp_info = f"Supplement"
        if paper.supplement_pages > 0:
            supp_info += f" - {paper.supplement_pages} pages"
        print(f"  [OK] {supp_info}")
    print(f"  {'[OK]' if paper.has_results else '[ ]'} Results data")
    print(f"  {'[OK]' if paper.has_figures else '[ ]'} Figures")
    print(f"  {'[OK]' if paper.has_reviews else '[ ]'} Reviews ({paper.review_count} files)")
    print(f"  {paper.status_docs} status documentation files")

    # Check for specific files
    print("\nKey Files:")
    key_files = [
        "main.tex", "main.pdf", "references.bib",
        "PAPER_COMPLETE.md", "REVISION_PLAN.md", "README.md"
    ]
    for filename in key_files:
        exists = (paper.path / filename).exists()
        print(f"  {'[OK]' if exists else '[ ]'} {filename}")

    print("\n" + "="*70)


def main():
    parser = argparse.ArgumentParser(description="Check research paper status")
    parser.add_argument("--paper", help="Show details for specific paper")
    parser.add_argument("--phase", choices=['design', 'analysis', 'writing', 'review', 'complete'],
                       help="Filter papers by phase")
    parser.add_argument("--summary", action="store_true", help="Show brief summary only")

    args = parser.parse_args()

    # Find research directory
    if Path("research").exists():
        research_dir = Path("research")
    elif Path("../research").exists():
        research_dir = Path("../research")
    else:
        print("Error: Could not find research directory")
        sys.exit(1)

    # Scan papers
    papers = scan_papers(research_dir)

    if not papers:
        print("No research papers found in research/")
        sys.exit(1)

    # Show specific paper
    if args.paper:
        paper = next((p for p in papers if args.paper in p.name), None)
        if not paper:
            print(f"Paper '{args.paper}' not found")
            print(f"Available papers: {', '.join(p.name for p in papers)}")
            sys.exit(1)
        print_paper_detail(paper)
        return

    # Filter by phase
    if args.phase:
        papers = [p for p in papers if p.phase == args.phase]
        if not papers:
            print(f"No papers in phase '{args.phase}'")
            sys.exit(1)

    # Show summary or detailed
    if args.summary:
        print_summary(papers)
    else:
        print_detailed(papers)
        print_summary(papers)


if __name__ == "__main__":
    main()

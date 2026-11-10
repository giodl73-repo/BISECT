#!/usr/bin/env python3
"""
Build the academic paper by compiling LaTeX.

Prerequisites:
- All statistics generated (run generate_all_statistics.py first)
- LaTeX installed (pdflatex, bibtex)
"""

import subprocess
import sys
from pathlib import Path

def check_prerequisites():
    """Check that all required files exist."""

    print("="*70)
    print("CHECKING PREREQUISITES")
    print("="*70)

    required_data = [
        'paper/data/population_stats.csv',
        'paper/data/compactness_stats.csv',
        'paper/data/political_stats.csv',
        'paper/data/example_state_selection.csv',
    ]

    required_figures = [
        'paper/figures/population_deviation_hist.png',
        'paper/figures/compactness_distribution.png',
        'paper/figures/political_lean_pie.png',
        'paper/figures/example_state_round_0.png',
        'paper/figures/example_state_round_1.png',
        'paper/figures/example_state_round_2.png',
    ]

    all_exist = True

    print("\nRequired data files:")
    for filepath in required_data:
        path = Path(filepath)
        if path.exists():
            print(f"  ✓ {filepath}")
        else:
            print(f"  ✗ {filepath} - MISSING")
            all_exist = False

    print("\nRequired figure files:")
    for filepath in required_figures:
        path = Path(filepath)
        if path.exists():
            print(f"  ✓ {filepath}")
        else:
            print(f"  ✗ {filepath} - MISSING")
            all_exist = False

    return all_exist

def check_latex():
    """Check if LaTeX is installed."""

    try:
        result = subprocess.run(['pdflatex', '--version'],
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            print("\n✓ LaTeX (pdflatex) is installed")
            return True
        else:
            print("\n✗ LaTeX (pdflatex) not working")
            return False
    except (FileNotFoundError, subprocess.TimeoutExpired):
        print("\n✗ LaTeX (pdflatex) not found in PATH")
        print("  Install LaTeX to compile the paper:")
        print("  - Windows: MiKTeX (https://miktex.org/)")
        print("  - macOS: MacTeX (https://www.tug.org/mactex/)")
        print("  - Linux: sudo apt-get install texlive-full")
        return False

def compile_latex():
    """Compile the LaTeX paper."""

    paper_tex = Path('paper/paper.tex')

    if not paper_tex.exists():
        print(f"\n✗ ERROR: {paper_tex} not found")
        print("  Create the LaTeX source file first")
        return False

    print("\n" + "="*70)
    print("COMPILING LATEX")
    print("="*70)

    # Change to paper directory for compilation
    import os
    original_dir = os.getcwd()
    os.chdir('paper')

    try:
        # Run pdflatex twice (for references)
        for i in range(2):
            print(f"\nRunning pdflatex (pass {i+1}/2)...")
            result = subprocess.run(['pdflatex', '-interaction=nonstopmode', 'paper.tex'],
                                  capture_output=True, text=True, timeout=60)

            if result.returncode != 0:
                print(f"✗ pdflatex failed on pass {i+1}")
                print("Error output:")
                print(result.stdout[-1000:] if result.stdout else "No output")
                return False

        # Run bibtex if references exist
        bib_file = Path('references.bib')
        if bib_file.exists():
            print("\nRunning bibtex...")
            result = subprocess.run(['bibtex', 'paper'],
                                  capture_output=True, text=True, timeout=30)

            if result.returncode == 0:
                # Run pdflatex one more time after bibtex
                print("\nRunning pdflatex (final pass)...")
                result = subprocess.run(['pdflatex', '-interaction=nonstopmode', 'paper.tex'],
                                      capture_output=True, text=True, timeout=60)

        # Check if PDF was created
        pdf_file = Path('paper.pdf')
        if pdf_file.exists():
            file_size = pdf_file.stat().st_size / 1024  # KB
            print(f"\n✓ Successfully created: paper.pdf ({file_size:.1f} KB)")

            # Count pages (rough estimate from log)
            log_file = Path('paper.log')
            if log_file.exists():
                log_content = log_file.read_text()
                import re
                match = re.search(r'Output written on paper\.pdf \((\d+) pages?', log_content)
                if match:
                    num_pages = int(match.group(1))
                    print(f"  Pages: {num_pages}")

                    if num_pages > 8:
                        print(f"  ⚠ Warning: Paper is {num_pages} pages (target: 8)")
                    elif num_pages < 8:
                        print(f"  ⓘ Info: Paper is {num_pages} pages (target: 8)")

            return True
        else:
            print("\n✗ PDF file was not created")
            return False

    except subprocess.TimeoutExpired:
        print("\n✗ LaTeX compilation timed out")
        return False
    except Exception as e:
        print(f"\n✗ Compilation error: {e}")
        return False
    finally:
        os.chdir(original_dir)

def main():
    """Main function."""

    print("="*70)
    print("PAPER BUILD SYSTEM")
    print("="*70)

    # Check prerequisites
    if not check_prerequisites():
        print("\n✗ Missing required files")
        print("\nRun data generation first:")
        print("  python paper/analysis/generate_all_statistics.py")
        return 1

    # Check LaTeX
    latex_available = check_latex()

    if not latex_available:
        print("\n✗ Cannot compile paper without LaTeX")
        print("  Install LaTeX and try again")
        return 1

    # Compile paper
    if not compile_latex():
        print("\n✗ Paper compilation failed")
        return 1

    print("\n" + "="*70)
    print("✓ PAPER BUILD COMPLETE")
    print("="*70)
    print("\nView the paper:")
    print("  paper/paper.pdf")

    return 0

if __name__ == '__main__':
    sys.exit(main())

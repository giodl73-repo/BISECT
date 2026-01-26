"""
Wave Manager Configuration

Centralized configuration for the Wave Manager application.
"""

from pathlib import Path

# Base directories
BASE_DIR = Path(__file__).parent.parent.parent
CONTEXT_DIR = BASE_DIR / 'context'
WAVES_DIR = CONTEXT_DIR / 'waves'
ENHANCEMENTS_DIR = CONTEXT_DIR / 'enhancements'

# Server configuration
PORT = 5104
HOST = '0.0.0.0'
DEBUG = True

# Project configuration
PROJECT_NAME = "Apportionment"
PROJECT_COLOR = "#2563eb"  # Blue

# GitHub configuration
GITHUB_REPO = "https://github.com/yourusername/apportionment"  # Base repository URL

# Ensure directories exist
WAVES_DIR.mkdir(parents=True, exist_ok=True)
ENHANCEMENTS_DIR.mkdir(parents=True, exist_ok=True)

"""
Enhancement Markdown Parser

Parses enhancement markdown files and extracts metadata.
"""

import re
from pathlib import Path


def parse_enhancement(file_path):
    """
    Parse an enhancement markdown file and extract key information

    Args:
        file_path: Path to enhancement .md file

    Returns:
        Dict with enhancement metadata or None if parsing fails
    """
    try:
        content = file_path.read_text(encoding='utf-8')
        metadata = extract_metadata(content)

        # Extract ID from filename (XX_name.md)
        match = re.match(r'(\d+)_', file_path.name)
        enhancement_id = int(match.group(1)) if match else None

        if not enhancement_id:
            return None

        # Determine directory (active or completed)
        directory = 'active' if file_path.parent.name == 'active' else 'completed'

        return {
            'id': enhancement_id,
            'title': metadata.get('title', f'Enhancement {enhancement_id}'),
            'status': metadata.get('status', 'Unknown'),
            'priority': metadata.get('priority', 'Unknown'),
            'complexity': metadata.get('complexity', 'Unknown'),
            'created': metadata.get('created', 'Unknown'),
            'completed': metadata.get('completed', ''),
            'started': metadata.get('started', ''),
            'file': f'{directory}/{file_path.name}',
            'summary': metadata.get('summary', '')
        }

    except Exception as e:
        print(f"[ERROR] Failed to parse {file_path}: {e}")
        return None


def extract_metadata(content):
    """
    Extract metadata from enhancement markdown content

    Args:
        content: Full markdown content as string

    Returns:
        Dict with extracted metadata fields
    """
    metadata = {}

    # Extract title (first # heading)
    title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    if title_match:
        metadata['title'] = title_match.group(1).strip()

    # Extract frontmatter fields
    patterns = {
        'status': r'\*\*Status\*\*:\s*(.+)',
        'priority': r'\*\*Priority\*\*:\s*(.+)',
        'complexity': r'\*\*Estimated Complexity\*\*:\s*(.+)',
        'created': r'\*\*Created\*\*:\s*(.+)',
        'completed': r'\*\*Completed\*\*:\s*(.+)',
        'started': r'\*\*Started\*\*:\s*(.+)',
    }

    for key, pattern in patterns.items():
        match = re.search(pattern, content)
        if match:
            metadata[key] = match.group(1).strip()

    # Extract summary (first paragraph after frontmatter)
    # Look for text between "## Current State" and next heading
    summary_match = re.search(
        r'##\s+(?:Current State|Goal)\s*\n\n(.+?)(?:\n\n|\n##)',
        content,
        re.DOTALL
    )
    if summary_match:
        summary = summary_match.group(1).strip()
        # Take first 200 characters
        if len(summary) > 200:
            summary = summary[:200] + '...'
        metadata['summary'] = summary

    return metadata


def validate_enhancement(content):
    """
    Validate enhancement markdown content

    Args:
        content: Markdown content string

    Returns:
        Dict with 'valid' boolean and 'errors' list
    """
    errors = []

    # Check for required fields
    required_patterns = {
        'Title': r'^#\s+Enhancement',
        'Status': r'\*\*Status\*\*:',
        'Priority': r'\*\*Priority\*\*:',
        'Created': r'\*\*Created\*\*:',
    }

    for field, pattern in required_patterns.items():
        if not re.search(pattern, content, re.MULTILINE):
            errors.append(f'Missing required field: {field}')

    # Check status value
    status_match = re.search(r'\*\*Status\*\*:\s*(.+)', content)
    if status_match:
        status = status_match.group(1).strip()
        valid_statuses = ['📋 PLANNED', '🔄 IN PROGRESS', '✅ COMPLETED']
        if status not in valid_statuses:
            errors.append(f'Invalid status: {status}. Must be one of: {", ".join(valid_statuses)}')

    # Check for common sections
    required_sections = ['## Current State', '## Goal', '## Implementation Plan']
    for section in required_sections:
        if section not in content:
            errors.append(f'Missing section: {section}')

    return {
        'valid': len(errors) == 0,
        'errors': errors
    }


def get_enhancement_id_from_filename(filename):
    """
    Extract enhancement ID from filename (XX_name.md)

    Args:
        filename: Enhancement filename

    Returns:
        Enhancement ID as int, or None if not found
    """
    match = re.match(r'(\d+)_', filename)
    return int(match.group(1)) if match else None

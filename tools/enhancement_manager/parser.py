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

        return {
            'id': enhancement_id,
            'title': metadata.get('title', f'Enhancement {enhancement_id}'),
            'status': metadata.get('status', 'Unknown'),
            'priority': metadata.get('priority', 'Unknown'),
            'complexity': metadata.get('complexity', 'Unknown'),
            'created': metadata.get('created', 'Unknown'),
            'completed': metadata.get('completed', ''),
            'started': metadata.get('started', ''),
            'commits': metadata.get('commits', []),
            'github_urls': metadata.get('github_urls', []),
            'size_lines': metadata.get('size_lines', 0),
            'size_files': metadata.get('size_files', 0),
            'size_category': metadata.get('size_category', 'Unknown'),
            'file': file_path.name,
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

    # Extract title (first # or ## heading)
    # Try level 1 heading first (newer format)
    title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    if not title_match:
        # Fall back to level 2 heading (older format)
        title_match = re.search(r'^##\s+(.+)$', content, re.MULTILINE)
    if title_match:
        title = title_match.group(1).strip()
        # Remove emoji status indicators from END of title if present
        title = re.sub(r'\s*[✅🔄📋]\s*(COMPLETED|IN PROGRESS|PLANNED)?\s*$', '', title).strip()
        metadata['title'] = title

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

    # Extract commits field
    commits_match = re.search(r'\*\*Commits\*\*:\s*(.+)', content)
    if commits_match:
        commits_text = commits_match.group(1).strip()
        # Parse markdown links: [short_sha](github_url)
        commit_links = re.findall(r'\[([a-f0-9]+)\]\((https://github\.com/[^\)]+)\)', commits_text)
        if commit_links:
            metadata['commits'] = [sha for sha, url in commit_links]
            metadata['github_urls'] = [url for sha, url in commit_links]
        else:
            metadata['commits'] = []
            metadata['github_urls'] = []
    else:
        metadata['commits'] = []
        metadata['github_urls'] = []

    # Extract size field
    size_match = re.search(r'\*\*Size\*\*:\s*(.+)', content)
    if size_match:
        size_text = size_match.group(1).strip()
        # Parse format: "XS - 1,250 lines changed (15 files)"
        category_match = re.match(r'(XS|S|M|L|XL)', size_text)
        lines_match = re.search(r'([\d,]+)\s+lines?\s+changed', size_text)
        files_match = re.search(r'\((\d+)\s+files?\)', size_text)

        metadata['size_category'] = category_match.group(1) if category_match else 'Unknown'
        metadata['size_lines'] = int(lines_match.group(1).replace(',', '')) if lines_match else 0
        metadata['size_files'] = int(files_match.group(1)) if files_match else 0
    else:
        metadata['size_category'] = 'Unknown'
        metadata['size_lines'] = 0
        metadata['size_files'] = 0

    # Infer status from title if not explicitly set (older format)
    if 'status' not in metadata:
        title = metadata.get('title', '')
        if '✅' in title or 'COMPLETED' in title:
            metadata['status'] = '✅ COMPLETED'
        elif '🔄' in title or 'IN PROGRESS' in title:
            metadata['status'] = '🔄 IN PROGRESS'
        elif '📋' in title or 'PLANNED' in title:
            metadata['status'] = '📋 PLANNED'
        else:
            # Default to unknown
            metadata['status'] = 'Unknown'

    # Extract summary (first paragraph after frontmatter)
    # Look for text between "## Current State" or "### Current State" or "### Goal" and next heading
    summary_match = re.search(
        r'###?\s+(?:Current State|Goal|Problem)\s*\n+(.+?)(?:\n\n|\n###?)',
        content,
        re.DOTALL
    )
    if summary_match:
        summary = summary_match.group(1).strip()
        # Remove markdown formatting and take first 200 characters
        summary = re.sub(r'\*\*(.+?)\*\*', r'\1', summary)  # Remove bold
        summary = re.sub(r'`(.+?)`', r'\1', summary)  # Remove code blocks
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

    # Check for required fields (relaxed for older formats)
    # Only require a title with "Enhancement"
    has_title_level1 = re.search(r'^#\s+Enhancement', content, re.MULTILINE)
    has_title_level2 = re.search(r'^##\s+Enhancement', content, re.MULTILINE)

    if not (has_title_level1 or has_title_level2):
        errors.append('Missing required field: Title (must contain "Enhancement")')

    # Check status value only if present (optional for older format)
    status_match = re.search(r'\*\*Status\*\*:\s*(.+)', content)
    if status_match:
        status = status_match.group(1).strip()
        valid_statuses = ['📋 PLANNED', '🔄 IN PROGRESS', '✅ COMPLETED']
        if status not in valid_statuses:
            errors.append(f'Invalid status: {status}. Must be one of: {", ".join(valid_statuses)}')

    # Don't require specific sections - different formats have different structures

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

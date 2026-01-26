"""
Wave and Phase Markdown Parser

Parses wave markdown files (collections of phases) and phase markdown files (individual enhancements).
Waves are stored in context/waves/ as WAVE##-NAME.md
Phases are stored in context/enhancements/ as ##_name.md
"""

import re
from pathlib import Path
from config import GITHUB_REPO


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

    # Extract commits field (try frontmatter first, then Git Commits section)
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
        # Try extracting from ## Git Commits section
        git_section_match = re.search(r'##\s+Git Commits\s*\n+((?:[-*]\s+`[a-f0-9]+`[^\n]*\n?)+)', content, re.IGNORECASE)
        if git_section_match:
            git_section = git_section_match.group(1)
            # Extract commit SHAs from backticks: - `sha` - message
            commit_shas = re.findall(r'[-*]\s+`([a-f0-9]+)`', git_section)
            metadata['commits'] = commit_shas
            # Construct GitHub URLs from SHAs
            metadata['github_urls'] = [f"{GITHUB_REPO}/commit/{sha}" for sha in commit_shas]
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
    # Support both "Enhancement" and "E#" format
    has_title_level1 = re.search(r'^#\s+(?:Enhancement|E\d+)', content, re.MULTILINE)
    has_title_level2 = re.search(r'^##\s+(?:Enhancement|E\d+)', content, re.MULTILINE)

    if not (has_title_level1 or has_title_level2):
        errors.append('Missing required field: Title (must contain "Enhancement" or "E#")')

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


def parse_wave(file_path):
    """
    Parse a wave markdown file and extract key information

    Args:
        file_path: Path to wave .md file (WAVE##-NAME.md)

    Returns:
        Dict with wave metadata or None if parsing fails
    """
    try:
        content = file_path.read_text(encoding='utf-8')
        metadata = extract_wave_metadata(content)

        # Extract wave ID from filename (WAVE##-name.md)
        match = re.match(r'WAVE(\d+)', file_path.name, re.IGNORECASE)
        wave_id = int(match.group(1)) if match else None

        if not wave_id:
            return None

        # Extract wave name from filename (WAVE##-NAME.md)
        name_match = re.match(r'WAVE\d+-(.+)\.md', file_path.name, re.IGNORECASE)
        wave_name = name_match.group(1).replace('-', ' ').replace('_', ' ').title() if name_match else f'Wave {wave_id}'

        # Extract body content (everything after the frontmatter separator)
        # Skip the title and all **Field**: value lines, then skip the --- separator
        body_content = content
        lines = content.split('\n')
        body_start_idx = 0

        # Skip title line (starts with #)
        in_frontmatter = False
        for idx, line in enumerate(lines):
            stripped = line.strip()

            # Skip title
            if stripped.startswith('#'):
                in_frontmatter = True
                continue

            # If we're in frontmatter and hit the separator, skip it and start body after
            if in_frontmatter and stripped == '---':
                body_start_idx = idx + 1
                break

        if body_start_idx > 0:
            body_content = '\n'.join(lines[body_start_idx:]).strip()

        return {
            'id': wave_id,
            'name': wave_name,
            'title': metadata.get('title', wave_name),
            'status': metadata.get('status', 'Unknown'),
            'goal': metadata.get('goal', ''),
            'success_metrics': metadata.get('success_metrics', []),
            'start_date': metadata.get('start_date', ''),
            'end_date': metadata.get('end_date', ''),
            'phase_ids': metadata.get('phase_ids', []),
            'phase_mappings': metadata.get('phase_mappings', []),
            'file': file_path.name,
            'content': body_content  # Include body content (after frontmatter) for rendering
        }

    except Exception as e:
        print(f"[ERROR] Failed to parse wave {file_path}: {e}")
        return None


def extract_wave_metadata(content):
    """
    Extract metadata from wave markdown content

    Args:
        content: Full markdown content as string

    Returns:
        Dict with extracted metadata fields
    """
    metadata = {}

    # Extract title (first # heading)
    title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    if title_match:
        title = title_match.group(1).strip()
        # Remove emoji status indicators from title if present
        title = re.sub(r'\s*[✅🔄📋]\s*(COMPLETED|IN PROGRESS|PLANNED)?\s*$', '', title).strip()
        metadata['title'] = title

    # Extract frontmatter fields
    patterns = {
        'status': r'\*\*Status\*\*:\s*(.+)',
        'start_date': r'\*\*Start Date\*\*:\s*(.+)',
        'end_date': r'\*\*End Date\*\*:\s*(.+)',
    }

    for key, pattern in patterns.items():
        match = re.search(pattern, content)
        if match:
            metadata[key] = match.group(1).strip()

    # Extract phases mapping (explicit phase definitions)
    # Format: **Phases**:
    #         - Phase 1: Enhancement 1
    #         - Phase 2: Enhancements 3, 4
    #         - Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
    phases_section = re.search(
        r'\*\*Phases\*\*:\s*\n((?:[-*]\s+Phase\s+\d+[A-Z]?:.*\n?)+)',
        content,
        re.MULTILINE
    )
    if phases_section:
        phases_text = phases_section.group(1)
        phase_mappings = []
        # Parse each line: - Phase 1: Enhancement 1 or - Phase 1: E1, 2 [- Name] [(status)]
        for line in phases_text.split('\n'):
            # Match format: Phase {num}: Enhancement(s) {ids} or Phase {num}: E {ids} [- {name}] [(status)]
            phase_match = re.match(
                r'[-*]\s+Phase\s+(\d+[A-Z]?):\s+(?:Enhancement[s]?|E)\s*([\d,\s]+)(?:\s+-\s+([^(]+?))?(?:\s+\([^)]+\))?\s*$',
                line.strip()
            )
            if phase_match:
                phase_num = phase_match.group(1)
                enh_nums = [int(n.strip()) for n in phase_match.group(2).split(',') if n.strip().isdigit()]
                phase_name = phase_match.group(3).strip() if phase_match.group(3) else None
                phase_mappings.append({
                    'phase': phase_num,
                    'name': phase_name,
                    'enhancements': enh_nums
                })
        metadata['phase_mappings'] = phase_mappings

    # Extract goal (can be under ## Goal, ## Goals, ### Goal, etc.)
    goal_match = re.search(
        r'###?\s+Goals?\s*\n+(.+?)(?:\n\n|\n###?)',
        content,
        re.DOTALL
    )
    if goal_match:
        goal = goal_match.group(1).strip()
        metadata['goal'] = goal

    # Extract success metrics (list items under Success Metrics section)
    metrics_match = re.search(
        r'###?\s+Success Metrics\s*\n+((?:[-*]\s+.+\n?)+)',
        content,
        re.MULTILINE
    )
    if metrics_match:
        metrics_text = metrics_match.group(1).strip()
        # Parse list items
        metrics = [line.strip('- *').strip() for line in metrics_text.split('\n') if line.strip()]
        metadata['success_metrics'] = metrics
    else:
        metadata['success_metrics'] = []

    # Extract phase/enhancement references
    # Prefer explicit phase_mappings, fall back to legacy Enhancements field
    phase_ids = []

    if 'phase_mappings' in metadata and metadata['phase_mappings']:
        # Use explicit phase mappings - collect all enhancement IDs
        for mapping in metadata['phase_mappings']:
            phase_ids.extend(mapping['enhancements'])
    else:
        # Fall back to legacy parsing
        # Pattern 1: "**Enhancements**: 1, 2, 3" or "**E**: 1, 2, 3" (in frontmatter with markdown bold)
        phases_match = re.search(r'\*\*(?:Enhancements|Phases|E)\*\*:\s*([\d,\s]+)', content, re.IGNORECASE)
        if phases_match:
            numbers = phases_match.group(1)
            phase_ids.extend([int(n.strip()) for n in numbers.split(',') if n.strip().isdigit()])

        # Pattern 2: List items with links like "- [Enhancement 1](../enhancements/1_name.md)" or "- [E1](../enhancements/1_name.md)"
        # Only match in the beginning of the document (before first ## heading after frontmatter)
        frontmatter_end = re.search(r'\n##\s+', content)
        if frontmatter_end:
            frontmatter_section = content[:frontmatter_end.start()]
            phase_links = re.findall(r'\[(?:Enhancement|Phase)\s+(\d+)\]', frontmatter_section, re.IGNORECASE)
            phase_ids.extend([int(n) for n in phase_links])
            # Also support E# format links
            e_format_links = re.findall(r'\[E(\d+)\]', frontmatter_section)
            phase_ids.extend([int(n) for n in e_format_links])

    # Remove duplicates and sort
    metadata['phase_ids'] = sorted(list(set(phase_ids)))

    # Infer status from title if not explicitly set
    if 'status' not in metadata:
        title = metadata.get('title', '')
        if '✅' in title or 'COMPLETED' in title:
            metadata['status'] = '✅ COMPLETED'
        elif '🔄' in title or 'IN PROGRESS' in title:
            metadata['status'] = '🔄 IN PROGRESS'
        elif '📋' in title or 'PLANNED' in title:
            metadata['status'] = '📋 PLANNED'
        else:
            metadata['status'] = '📋 PLANNED'

    return metadata


def link_phases_to_wave(wave, all_phases):
    """
    Link phase objects to a wave based on phase_ids

    Args:
        wave: Wave dict with phase_ids field
        all_phases: List of all phase dicts

    Returns:
        Wave dict with 'phases' field containing list of matching phase objects
    """
    phase_map = {p['id']: p for p in all_phases}
    wave['phases'] = [phase_map[pid] for pid in wave['phase_ids'] if pid in phase_map]
    return wave

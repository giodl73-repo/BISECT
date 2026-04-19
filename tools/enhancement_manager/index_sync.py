"""
INDEX.md Synchronization

Updates docs/enhancements/INDEX.md when enhancement status changes.
"""

import re
from pathlib import Path
from datetime import datetime


def update_index_status(enhancement_id, new_status, metadata, index_path):
    """
    Update enhancement status in INDEX.md

    Args:
        enhancement_id: Enhancement number
        new_status: New status (📋 PLANNED / 🔄 IN PROGRESS / ✅ COMPLETED)
        metadata: Enhancement metadata dict
        index_path: Path to INDEX.md file

    Returns:
        True if successful, False otherwise
    """
    try:
        # Read INDEX.md
        content = index_path.read_text(encoding='utf-8')

        # Backup original
        backup_path = index_path.parent / 'INDEX.md.backup'
        backup_path.write_text(content, encoding='utf-8')

        # Determine which section to move to/from
        if '✅' in new_status:
            target_section = 'completed'
        elif '🔄' in new_status:
            target_section = 'in_progress'
        else:  # 📋
            target_section = 'planned'

        # Find the enhancement entry
        entry_pattern = rf'\| \[{enhancement_id}\]\([^\)]+\) \| [^\|]+ \|'

        # Check if entry exists
        if not re.search(entry_pattern, content):
            print(f"[WARN] Enhancement {enhancement_id} not found in INDEX.md")
            return False

        # Extract current entry
        entry_match = re.search(
            rf'\| \[{enhancement_id}\]\(([^\)]+)\) \| ([^\|]+) \| ([^\|]+) \| \[View\]\([^\)]+\) \|',
            content
        )

        if not entry_match:
            print(f"[WARN] Could not parse entry for enhancement {enhancement_id}")
            return False

        file_path = entry_match.group(1)
        title = entry_match.group(2).strip()
        current_detail = entry_match.group(3).strip()  # Date or status detail

        # Create new entry based on target section
        if target_section == 'completed':
            # Completed format: | # | Title | Completion Date | Files |
            completion_date = metadata.get('completed', datetime.now().strftime('%b %d, %Y'))
            new_entry = f'| [{enhancement_id}]({file_path}) | {title} | {completion_date} | [View]({file_path}) |'
        elif target_section == 'in_progress':
            # In Progress format: | # | Title | Status | Files |
            status_detail = metadata.get('started', 'In Progress')
            new_entry = f'| [{enhancement_id}]({file_path}) | {title} | {status_detail} | [View]({file_path}) |'
        else:  # planned
            # Planned format: | # | Title | Priority | Files |
            priority = metadata.get('priority', 'Medium')
            new_entry = f'| [{enhancement_id}]({file_path}) | {title} | {priority} | [View]({file_path}) |'

        # Remove old entry from current section
        old_entry_line = entry_match.group(0)
        content = content.replace(old_entry_line + '\n', '')

        # Find target section and add entry
        if target_section == 'completed':
            # Add to completed section (after the table header)
            completed_header = r'### ✅ Completed \((\d+) enhancements\)\n\n\| # \| Title \| Completion Date \| Files \|\n\|---|-------|----------------|-------\|'
            match = re.search(completed_header, content)
            if match:
                # Update count
                current_count = int(match.group(1))
                new_count = current_count + 1
                content = re.sub(
                    r'### ✅ Completed \(\d+ enhancements\)',
                    f'### ✅ Completed ({new_count} enhancements)',
                    content
                )

                # Insert entry (newest first - after header)
                insert_pos = match.end()
                content = content[:insert_pos] + '\n' + new_entry + content[insert_pos:]

        elif target_section == 'in_progress':
            # Add to in progress section
            in_progress_header = r'### 🔄 In Progress \((\d+) enhancement[s]?\)\n\n\| # \| Title \| Status \| Files \|\n\|---|-------|--------|-------\|'
            match = re.search(in_progress_header, content)
            if match:
                # Update count
                current_count = int(match.group(1))
                new_count = current_count + 1
                content = re.sub(
                    r'### 🔄 In Progress \(\d+ enhancement[s]?\)',
                    f'### 🔄 In Progress ({new_count} enhancement{"s" if new_count != 1 else ""})',
                    content
                )

                # Insert entry
                insert_pos = match.end()
                content = content[:insert_pos] + '\n' + new_entry + content[insert_pos:]

        else:  # planned
            # Add to planned section
            planned_header = r'### 📋 Planned \((\d+) enhancements\)\n\n\| # \| Title \| Priority \| Files \|\n\|---|-------|----------|-------\|'
            match = re.search(planned_header, content)
            if match:
                # Update count
                current_count = int(match.group(1))
                new_count = current_count + 1
                content = re.sub(
                    r'### 📋 Planned \(\d+ enhancements\)',
                    f'### 📋 Planned ({new_count} enhancements)',
                    content
                )

                # Insert entry (after header, maintaining order by ID)
                insert_pos = match.end()
                content = content[:insert_pos] + '\n' + new_entry + content[insert_pos:]

        # Write updated INDEX.md
        index_path.write_text(content, encoding='utf-8')

        print(f"[OK] Updated INDEX.md: Enhancement {enhancement_id} -> {target_section}")
        return True

    except Exception as e:
        print(f"[ERROR] Failed to update INDEX.md: {e}")
        # Restore from backup if it exists
        if backup_path.exists():
            index_path.write_text(backup_path.read_text(encoding='utf-8'), encoding='utf-8')
            print("[OK] Restored INDEX.md from backup")
        return False

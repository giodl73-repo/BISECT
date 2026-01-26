"""
Wave Manager Web App - Flask Backend

A web application to view, filter, search, and edit waves and phases.
Waves are collections of related phases (enhancements).
- Waves: WAVE##-NAME.md in context/waves/
- Phases: ##_name.md in context/enhancements/

Reads/writes directly to markdown files.
"""

from flask import Flask, jsonify, request, send_from_directory
from pathlib import Path
import re
import json
from datetime import datetime

# Import our parser and sync modules
from parser import parse_enhancement, extract_metadata, validate_enhancement, parse_wave, link_phases_to_wave
try:
    from index_sync import update_index_status
except ImportError:
    # index_sync may not exist yet
    update_index_status = None

# Import configuration
from config import WAVES_DIR, ENHANCEMENTS_DIR, PORT, HOST, DEBUG, PROJECT_NAME, PROJECT_COLOR

app = Flask(__name__, static_folder='static', static_url_path='')

# Base paths
BASE_PATH = ENHANCEMENTS_DIR
INDEX_PATH = BASE_PATH / 'INDEX.md'


@app.route('/')
def index():
    """Serve the main web interface"""
    return send_from_directory('static', 'index.html')


@app.route('/api/config', methods=['GET'])
def get_config():
    """Get project configuration for branding"""
    return jsonify({
        'projectName': PROJECT_NAME,
        'projectColor': PROJECT_COLOR
    })


@app.route('/api/waves', methods=['GET'])
def get_waves():
    """
    Get list of all waves with metadata

    Returns:
        JSON list of waves with id, name, status, goals, phases, etc.
    """
    waves = []

    # Scan waves directory
    if WAVES_DIR.exists():
        for file_path in WAVES_DIR.glob('WAVE*.md'):
            try:
                wave = parse_wave(file_path)
                if wave:
                    waves.append(wave)
            except Exception as e:
                print(f"[WARN] Failed to parse {file_path.name}: {e}")

    # Sort by ID (ascending - Wave 1, Wave 2, etc.)
    waves.sort(key=lambda x: x['id'])

    return jsonify({'waves': waves})


@app.route('/api/waves/<int:wave_id>', methods=['GET'])
def get_wave(wave_id):
    """
    Get full details of a single wave including its phases

    Args:
        wave_id: Wave number

    Returns:
        JSON with wave metadata and list of phases
    """
    # Find the wave file
    wave_file = find_wave_file(wave_id)

    if not wave_file:
        return jsonify({'error': 'Wave not found'}), 404

    try:
        wave = parse_wave(wave_file)
        if not wave:
            return jsonify({'error': 'Failed to parse wave'}), 500

        # Get all phases and link them to this wave
        phases_response = get_enhancements()
        all_phases = phases_response.get_json()['enhancements']

        # Link phases to wave
        wave = link_phases_to_wave(wave, all_phases)

        # Read full content
        content = wave_file.read_text(encoding='utf-8')

        return jsonify({
            'id': wave_id,
            'name': wave['name'],
            'title': wave['title'],
            'status': wave['status'],
            'goal': wave['goal'],
            'success_metrics': wave['success_metrics'],
            'start_date': wave['start_date'],
            'end_date': wave['end_date'],
            'phases': wave['phases'],
            'phase_mappings': wave.get('phase_mappings', []),
            'content': content,
            'file_path': str(wave_file.relative_to(WAVES_DIR.parent))
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/waves/<int:wave_id>/phases', methods=['GET'])
def get_wave_phases(wave_id):
    """
    Get all phases belonging to a wave

    Args:
        wave_id: Wave number

    Returns:
        JSON list of phases in the wave
    """
    # Find the wave file
    wave_file = find_wave_file(wave_id)

    if not wave_file:
        return jsonify({'error': 'Wave not found'}), 404

    try:
        wave = parse_wave(wave_file)
        if not wave:
            return jsonify({'error': 'Failed to parse wave'}), 500

        # Get all phases and link them to this wave
        phases_response = get_enhancements()
        all_phases = phases_response.get_json()['enhancements']

        # Link phases to wave
        wave = link_phases_to_wave(wave, all_phases)

        return jsonify({'phases': wave['phases']})
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/shutdown', methods=['POST'])
def shutdown():
    """Shutdown the Flask server"""
    try:
        print("[OK] Shutdown requested from web interface")
        # Get the shutdown function from werkzeug
        func = request.environ.get('werkzeug.server.shutdown')
        if func is None:
            # Alternative method for Flask in debug mode
            import os
            import signal
            print("[OK] Terminating server process...")
            os.kill(os.getpid(), signal.SIGTERM)
            return jsonify({'success': True, 'message': 'Server shutting down...'})
        func()
        return jsonify({'success': True, 'message': 'Server shutting down...'})
    except Exception as e:
        print(f"[ERROR] Shutdown failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/enhancements', methods=['GET'])
def get_enhancements():
    """
    Get list of all enhancements with metadata

    Returns:
        JSON list of enhancements with id, title, status, dates, etc.
    """
    enhancements = []

    # Scan enhancements directory (skip INDEX.md and templates)
    for file_path in BASE_PATH.glob('*.md'):
        # Skip INDEX.md and other non-enhancement files
        if file_path.name in ['INDEX.md', 'README.md']:
            continue

        try:
            enhancement = parse_enhancement(file_path)
            if enhancement:
                enhancements.append(enhancement)
        except Exception as e:
            print(f"[WARN] Failed to parse {file_path.name}: {e}")

    # Sort by ID (descending - newest first)
    enhancements.sort(key=lambda x: x['id'], reverse=True)

    return jsonify({'enhancements': enhancements})


@app.route('/api/enhancements/<int:enhancement_id>', methods=['GET'])
def get_enhancement(enhancement_id):
    """
    Get full content of a single enhancement

    Args:
        enhancement_id: Enhancement number

    Returns:
        JSON with full markdown content and metadata
    """
    # Find the enhancement file
    file_path = find_enhancement_file(enhancement_id)

    if not file_path:
        return jsonify({'error': 'Enhancement not found'}), 404

    try:
        content = file_path.read_text(encoding='utf-8')
        metadata = extract_metadata(content)

        return jsonify({
            'id': enhancement_id,
            'content': content,
            'metadata': metadata,
            'file_path': str(file_path.relative_to(BASE_PATH))
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/enhancements/<int:enhancement_id>', methods=['PUT'])
def update_enhancement(enhancement_id):
    """
    Update enhancement content

    Args:
        enhancement_id: Enhancement number

    Body:
        JSON with 'content' field containing updated markdown

    Returns:
        Success/error status
    """
    file_path = find_enhancement_file(enhancement_id)

    if not file_path:
        return jsonify({'error': 'Enhancement not found'}), 404

    try:
        data = request.get_json()
        new_content = data.get('content')

        if not new_content:
            return jsonify({'error': 'No content provided'}), 400

        # Validate the new content
        validation = validate_enhancement(new_content)
        if not validation['valid']:
            return jsonify({
                'error': 'Invalid enhancement content',
                'details': validation['errors']
            }), 400

        # Write the updated content
        file_path.write_text(new_content, encoding='utf-8')

        return jsonify({'success': True, 'message': 'Enhancement updated'})

    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/enhancements/<int:enhancement_id>/status', methods=['POST'])
def update_status(enhancement_id):
    """
    Update enhancement status (and auto-update INDEX.md)

    Args:
        enhancement_id: Enhancement number

    Body:
        JSON with 'status' (PLANNED/IN PROGRESS/COMPLETED)
        and optional 'completed_date'

    Returns:
        Success/error status
    """
    file_path = find_enhancement_file(enhancement_id)

    if not file_path:
        return jsonify({'error': 'Enhancement not found'}), 404

    try:
        data = request.get_json()
        new_status = data.get('status')
        completed_date = data.get('completed_date')

        if not new_status:
            return jsonify({'error': 'No status provided'}), 400

        # Validate status value
        valid_statuses = ['📋 PLANNED', '🔄 IN PROGRESS', '✅ COMPLETED']
        if new_status not in valid_statuses:
            return jsonify({'error': f'Invalid status. Must be one of: {valid_statuses}'}), 400

        # Read current content
        content = file_path.read_text(encoding='utf-8')

        # Update status line
        content = re.sub(
            r'\*\*Status\*\*: [^\n]+',
            f'**Status**: {new_status}',
            content
        )

        # Add completed date if status is COMPLETED
        if new_status == '✅ COMPLETED' and completed_date:
            if '**Completed**:' not in content:
                # Add completed date after Created line
                content = re.sub(
                    r'(\*\*Created\*\*: [^\n]+)',
                    f'\\1\n**Completed**: {completed_date}',
                    content
                )
            else:
                # Update existing completed date
                content = re.sub(
                    r'\*\*Completed\*\*: [^\n]+',
                    f'**Completed**: {completed_date}',
                    content
                )

        # Write updated content
        file_path.write_text(content, encoding='utf-8')

        # Update INDEX.md
        if update_index_status:
            try:
                metadata = extract_metadata(content)
                update_index_status(
                    enhancement_id=enhancement_id,
                    new_status=new_status,
                    metadata=metadata,
                    index_path=INDEX_PATH
                )
            except Exception as e:
                print(f"[WARN] Failed to update INDEX.md: {e}")
                # Don't fail the request if INDEX update fails

        return jsonify({'success': True, 'message': 'Status updated'})

    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/stats', methods=['GET'])
def get_stats():
    """
    Get enhancement statistics

    Returns:
        JSON with totals, completion rate, complexity distribution, etc.
    """
    try:
        # Get all enhancements
        response = get_enhancements()
        enhancements = response.get_json()['enhancements']

        # Calculate statistics
        total = len(enhancements)
        completed = len([e for e in enhancements if '✅' in e['status']])
        in_progress = len([e for e in enhancements if '🔄' in e['status']])
        planned = len([e for e in enhancements if '📋' in e['status']])

        # Completion rate
        completion_rate = (completed / total * 100) if total > 0 else 0

        # Priority distribution
        priority_dist = {}
        for e in enhancements:
            priority = e.get('priority', 'Unknown')
            # Extract first word (Critical/High/Medium/Low/Research)
            match = re.match(r'(Critical|High|Medium|Low|Research)', priority)
            if match:
                level = match.group(1)
                priority_dist[level] = priority_dist.get(level, 0) + 1
            else:
                priority_dist['Unknown'] = priority_dist.get('Unknown', 0) + 1

        # Complexity distribution
        complexity_dist = {}
        for e in enhancements:
            complexity = e.get('complexity', 'Unknown')
            # Extract just the level (Low/Medium/High)
            match = re.match(r'(Low|Medium|Medium-High|High|Very High)', complexity)
            if match:
                level = match.group(1)
                complexity_dist[level] = complexity_dist.get(level, 0) + 1

        # Size distribution (XS/S/M/L/XL)
        size_dist = {}
        total_lines = 0
        total_files = 0
        for e in enhancements:
            category = e.get('size_category', 'Unknown')
            if category != 'Unknown':
                size_dist[category] = size_dist.get(category, 0) + 1
                total_lines += e.get('size_lines', 0)
                total_files += e.get('size_files', 0)

        # Average size by priority
        size_by_priority = {}
        for priority_level in ['Critical', 'High', 'Medium', 'Low', 'Research']:
            priority_enhancements = [
                e for e in enhancements
                if priority_level in e.get('priority', '') and e.get('size_lines', 0) > 0
            ]
            if priority_enhancements:
                avg_lines = sum(e.get('size_lines', 0) for e in priority_enhancements) / len(priority_enhancements)
                size_by_priority[priority_level] = round(avg_lines)

        # Recent completions (last 30 days)
        recent_completions = []
        for e in enhancements:
            if '✅' in e['status'] and e.get('completed'):
                try:
                    # Parse date
                    date_str = e['completed']
                    # Handle various date formats
                    for fmt in ['%B %d, %Y', '%b %d, %Y', '%Y-%m-%d']:
                        try:
                            completed_date = datetime.strptime(date_str.replace('Jan ', 'January '), fmt)
                            days_ago = (datetime.now() - completed_date).days
                            if days_ago <= 30:
                                recent_completions.append({
                                    'id': e['id'],
                                    'title': e['title'],
                                    'date': date_str,
                                    'days_ago': days_ago
                                })
                            break
                        except ValueError:
                            continue
                except Exception:
                    pass

        # Sort by date (most recent first)
        recent_completions.sort(key=lambda x: x['days_ago'])

        return jsonify({
            'total': total,
            'completed': completed,
            'in_progress': in_progress,
            'planned': planned,
            'completion_rate': round(completion_rate, 1),
            'priority_distribution': priority_dist,
            'complexity_distribution': complexity_dist,
            'size_distribution': size_dist,
            'size_by_priority': size_by_priority,
            'total_lines_changed': total_lines,
            'total_files_modified': total_files,
            'recent_completions': recent_completions[:10]  # Top 10
        })

    except Exception as e:
        return jsonify({'error': str(e)}), 500


def find_enhancement_file(enhancement_id):
    """
    Find enhancement file by ID in enhancements directory

    Args:
        enhancement_id: Enhancement number

    Returns:
        Path object or None if not found
    """
    # Search in enhancements directory
    # Try both zero-padded (01_) and non-padded (1_) formats
    patterns = [
        f'{enhancement_id}_*.md',           # Non-padded: 1_file.md, 10_file.md
        f'{enhancement_id:02d}_*.md',       # Zero-padded: 01_file.md, 10_file.md
    ]

    for pattern in patterns:
        for file_path in BASE_PATH.glob(pattern):
            return file_path

    return None


def find_wave_file(wave_id):
    """
    Find wave file by ID in waves directory

    Args:
        wave_id: Wave number

    Returns:
        Path object or None if not found
    """
    # Search in waves directory
    if WAVES_DIR.exists():
        # Pad single digit with zero for matching (WAVE01, WAVE02, etc.)
        padded_id = f'{wave_id:02d}'

        # Try with zero-padded ID first (WAVE01-name.md)
        for file_path in WAVES_DIR.glob(f'WAVE{padded_id}-*.md'):
            return file_path
        for file_path in WAVES_DIR.glob(f'WAVE{padded_id}*.md'):
            return file_path

        # Try without zero padding (WAVE1-name.md)
        for file_path in WAVES_DIR.glob(f'WAVE{wave_id}-*.md'):
            return file_path
        for file_path in WAVES_DIR.glob(f'WAVE{wave_id}*.md'):
            return file_path

    return None


if __name__ == '__main__':
    print("[OK] Starting Wave Manager...")
    print(f"[OK] Waves path: {WAVES_DIR}")
    print(f"[OK] Phases path: {ENHANCEMENTS_DIR}")
    print(f"[OK] Server running at http://{HOST}:{PORT}")
    print("[OK] Press Ctrl+C to stop")

    app.run(debug=DEBUG, port=PORT, host=HOST)

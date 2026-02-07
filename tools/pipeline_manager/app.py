"""
Pipeline Manager Web Application - Flask Backend

A minimal web application to manage pipeline runs, view progress, and manage versions.
"""

from flask import Flask, jsonify, request, send_from_directory
from pathlib import Path
import argparse
import sys
import os
import webbrowser
import threading
import time
import signal
import logging

# Verbosity control (set via environment variable DEBUG_PIPELINE_MANAGER)
DEBUG = os.environ.get('DEBUG_PIPELINE_MANAGER', '0') == '1'

if DEBUG: print("[DEBUG] Loading app.py module - version 2026-01-19-21:30", flush=True)

# Import our manager modules
from run_manager import RunManager
from version_manager import list_versions, delete_version, get_error_log

app = Flask(__name__, static_folder='static', static_url_path='')

# Disable Flask/Werkzeug request logging
log = logging.getLogger('werkzeug')
log.setLevel(logging.ERROR)

# Initialize run manager
PROGRESS_FILE = Path(__file__).parent.parent.parent / 'outputs' / 'runs_progress.json'
run_manager = RunManager(PROGRESS_FILE)


@app.route('/')
def index():
    """Serve the main web interface"""
    response = send_from_directory('static', 'index.html')
    response.headers['Cache-Control'] = 'no-cache, no-store, must-revalidate'
    response.headers['Pragma'] = 'no-cache'
    response.headers['Expires'] = '0'
    return response


@app.route('/dashboard')
def dashboard():
    """Serve the master dashboard"""
    web_dir = project_root / 'web'
    response = send_from_directory(web_dir, 'master_dashboard.html')
    response.headers['Cache-Control'] = 'no-cache, no-store, must-revalidate'
    response.headers['Pragma'] = 'no-cache'
    response.headers['Expires'] = '0'
    return response


@app.route('/api/ping')
def ping():
    """Simple ping endpoint"""
    return {'pong': True, 'version': '2026-01-19-09:25'}


@app.route('/api/versions', methods=['GET'])
def get_versions():
    """
    Get list of all versions with metadata.

    Returns:
        JSON list of versions
    """
    try:
        versions = list_versions()
        return jsonify({'success': True, 'versions': versions})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/versions/<version_name>', methods=['DELETE'])
def delete_version_endpoint(version_name):
    """
    Delete a version directory.

    Args:
        version_name: Name of version to delete

    Returns:
        JSON response with success status
    """
    try:
        success, message = delete_version(version_name)
        return jsonify({'success': success, 'message': message}), 200 if success else 400
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/runs/start', methods=['POST'])
def start_run():
    """
    Start a new pipeline run.

    Request body:
        {
            "version": "v2",
            "years": ["2020", "2010"],
            "states": ["CA", "TX", "NY"],  // optional
            "workers": 4,
            "dpi": 150,
            "partition_mode": "edge-weighted"
        }

    Returns:
        JSON response with run_id or error
    """
    try:
        if DEBUG: print(f"[DEBUG] /api/runs/start called", flush=True)
        if DEBUG: print(f"[DEBUG] request.method: {request.method}", flush=True)
        if DEBUG: print(f"[DEBUG] request.content_type: {request.content_type}", flush=True)
        if DEBUG: print(f"[DEBUG] request.data: {request.data}", flush=True)

        config = request.json
        if DEBUG: print(f"[DEBUG] parsed config: {config}", flush=True)
        if DEBUG: print(f"[DEBUG] config['years'] type: {type(config.get('years'))}, value: {config.get('years')}", flush=True)

        if not config:
            print(f"[DEBUG] No config provided", flush=True)
            return jsonify({'success': False, 'error': 'No configuration provided'}), 400

        # Validate required fields
        required = ['version', 'years']
        for field in required:
            if field not in config:
                print(f"[DEBUG] Missing field: {field}", flush=True)
                return jsonify({'success': False, 'error': f'Missing required field: {field}'}), 400

        # Set defaults
        config.setdefault('workers', 4)
        config.setdefault('dpi', 150)
        config.setdefault('partition_mode', 'edge-weighted')

        if DEBUG: print(f"[DEBUG] Final config: {config}", flush=True)

        # Start run
        if DEBUG: print(f"[DEBUG] Calling run_manager.start_run", flush=True)
        success, result = run_manager.start_run(config)
        if DEBUG: print(f"[DEBUG] start_run returned: success={success}, result={result}", flush=True)

        if success:
            return jsonify({'success': True, 'run_id': result})
        else:
            return jsonify({'success': False, 'error': result}), 400

    except Exception as e:
        if DEBUG: print(f"[DEBUG] Exception in start_run: {e}", flush=True)
        import traceback
        if DEBUG: print(f"[DEBUG] Traceback: {traceback.format_exc()}", flush=True)
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/runs/active', methods=['GET'])
def get_active_run():
    """
    Get active run progress.

    Returns:
        JSON with active run data (or null if no active run)
    """
    try:
        active = run_manager.get_active_run()

        # Debug: log worker years
        if active and 'workers' in active:
            worker_years = {}
            for wkey, worker in active['workers'].items():
                year = worker.get('year', 'unknown')
                if year not in worker_years:
                    worker_years[year] = []
                worker_years[year].append(f"{wkey} ({worker.get('state', '?')})")

            if DEBUG: print(f"[API] Workers by year: {worker_years}", flush=True)

        return jsonify({'success': True, 'active_run': active})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/runs/history', methods=['GET'])
def get_run_history():
    """
    Get run history.

    Returns:
        JSON list of past runs
    """
    try:
        history = run_manager.get_history()
        return jsonify({'success': True, 'history': history})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/suggest-version', methods=['GET'])
def suggest_version():
    """
    Suggest next available version name.

    Returns:
        JSON with suggested version name
    """
    try:
        suggested = run_manager.suggest_version_name()
        return jsonify({'success': True, 'version': suggested})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/runs/<run_id>/pause', methods=['POST'])
def pause_run(run_id):
    """
    Pause an active run (preserves files for resume).

    Args:
        run_id: Run ID to pause

    Returns:
        JSON response with success status
    """
    try:
        success, message = run_manager.pause_run(run_id)
        return jsonify({'success': success, 'message': message}), 200 if success else 400
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/runs/<run_id>', methods=['DELETE'])
def cancel_run(run_id):
    """
    Cancel an active run and delete output files.

    Args:
        run_id: Run ID to cancel

    Returns:
        JSON response with success status
    """
    try:
        success, message = run_manager.cancel_run(run_id)
        return jsonify({'success': success, 'message': message}), 200 if success else 400
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/logs/<version>/<year>', methods=['GET'])
def get_logs(version, year):
    """
    Get error logs for a version/year.

    Args:
        version: Version name
        year: Year (2020, 2010, 2000)

    Returns:
        JSON with log content
    """
    try:
        success, content = get_error_log(version, year)
        if success:
            return jsonify({'success': True, 'log': content})
        else:
            return jsonify({'success': False, 'error': content}), 404
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500


@app.route('/api/test', methods=['GET'])
def test_endpoint():
    """Test endpoint to verify code updates"""
    return jsonify({'success': True, 'message': 'Test endpoint works!', 'timestamp': '2026-01-19-09:15'})


@app.route('/api/config', methods=['GET'])
def get_config():
    """Get configuration options"""
    return jsonify({
        'success': True,
        'config': {
            'years': ['2020', '2010', '2000'],
            'states': ['ak', 'al', 'ar', 'az', 'ca', 'co', 'ct', 'de', 'fl', 'ga',
                       'hi', 'ia', 'id', 'il', 'in', 'ks', 'ky', 'la', 'ma', 'md',
                       'me', 'mi', 'mn', 'mo', 'ms', 'mt', 'nc', 'nd', 'ne', 'nh',
                       'nj', 'nm', 'nv', 'ny', 'oh', 'ok', 'or', 'pa', 'ri', 'sc',
                       'sd', 'tn', 'tx', 'ut', 'va', 'vt', 'wa', 'wi', 'wv', 'wy'],
            'partition_modes': ['edge-weighted', 'unweighted'],
            'default_workers': 4,
            'default_dpi': 150
        }
    })


print(f"[DEBUG] Routes registered: {[rule.rule for rule in app.url_map.iter_rules()]}", flush=True)


@app.route('/api/shutdown', methods=['POST'])
def shutdown():
    """Shutdown the Flask server"""
    try:
        print("[OK] Shutdown requested from web interface")
        import os
        import signal
        print("[OK] Terminating server process...")
        os.kill(os.getpid(), signal.SIGTERM)
        return jsonify({'success': True, 'message': 'Server shutting down...'})
    except Exception as e:
        print(f"[ERROR] Shutdown failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='Pipeline Manager Web Application')
    parser.add_argument('--port', type=int, default=5100,
                       help='Port to run on (default: 5100)')
    parser.add_argument('--host', type=str, default='127.0.0.1',
                       help='Host to bind to (default: 127.0.0.1)')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode')
    args = parser.parse_args()

    print("=" * 70)
    print("Pipeline Manager Web Application")
    print("=" * 70)
    print(f"Starting server on http://{args.host}:{args.port}")
    print(f"Progress file: {PROGRESS_FILE}")
    print("=" * 70)
    print()

    # Ensure outputs directory exists
    PROGRESS_FILE.parent.mkdir(parents=True, exist_ok=True)

    # Open browser automatically after short delay
    def open_browser():
        time.sleep(1.5)  # Wait for server to start
        url = f"http://{args.host}:{args.port}"
        print(f"Opening browser to {url}...")
        webbrowser.open(url)

    browser_thread = threading.Thread(target=open_browser)
    browser_thread.daemon = True
    browser_thread.start()

    # Handle Ctrl+C gracefully
    def signal_handler(sig, frame):
        print("\n\n[INFO] Shutting down server...")
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)

    try:
        app.run(host=args.host, port=args.port, debug=args.debug, threaded=True, use_reloader=False)
    except OSError as e:
        if 'Address already in use' in str(e):
            print(f"\n[ERROR] Port {args.port} is already in use!")
            print(f"Try running with a different port: python app.py --port {args.port + 1}")
        else:
            raise


if __name__ == '__main__':
    main()

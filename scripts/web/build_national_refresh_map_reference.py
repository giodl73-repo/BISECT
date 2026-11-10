"""Build an owned native sequence reference with current source identities."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(data).hexdigest(), byte_length=len(data))


def main():
    prior = json.loads((ROOT / 'target/national-matrix/map-engine-1792829799214781500/evidence.json').read_text(encoding='utf-8'))
    names = {r['path'] for r in prior['source_files']}
    names |= {'crates/bisect-wasm/src/partition_timing.rs', 'scripts/web/build_national_refresh_map_reference.py'}
    before = [pin(ROOT / name) for name in sorted(names)]
    output = ROOT / 'target/national-engine-refresh' / ('map-engine-' + str(time.time_ns()))
    output.mkdir(parents=True)
    command = ['cargo', 'build', '-p', 'bisect-wasm', '--release', '--example', 'execute_tool_sequence', '--locked', '--offline', '-j', '1']
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    log = output / 'build.log'
    log.write_text(result.stdout + result.stderr, encoding='utf-8')
    stable = [pin(ROOT / name) for name in sorted(names)] == before
    record = dict(schema_version='bisect-national-current-map-reference-v1', status='failed', command=command, exit_code=result.returncode, source_files=before, source_unchanged=stable, build_log=pin(log), scope='Current native sequence adapter for exact national map/stream output comparison, not a CLI directory-discovery or complete parity claim.')
    if result.returncode == 0 and stable:
        reference = output / 'execute_tool_sequence.exe'
        shutil.copy2(ROOT / 'target/release/examples/execute_tool_sequence.exe', reference)
        record.update(status='passed', native_sequence=pin(reference))
    (output / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(output / 'evidence.json', flush=True)
    assert record['status'] == 'passed'


if __name__ == '__main__':
    main()

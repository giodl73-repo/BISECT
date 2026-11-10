"""Actual native ignored-name branch with an owned manifest/source cache."""
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(p):
    raw = p.read_bytes()
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(raw).hexdigest(), byte_length=len(raw))


def main():
    output = ROOT / 'target/state-name-workflow' / ('native-ignored-' + str(time.time_ns()))
    cache = output / 'source-cache'
    source = cache / 'V3/data/2020/adjacency'
    source.mkdir(parents=True)
    fixture = json.loads((ROOT / 'scripts/web/fixtures/native-state-source.json').read_text(encoding='utf-8'))['nativeAdjacencyInputs']
    for key, suffix in [('adjacency', '.adj.bin'), ('geoids', '_geoids.json'), ('centroids', '_centroids.json')]:
        (source / ('ri_adjacency_2020' + suffix)).write_bytes(base64.b64decode(fixture[key]['base64'], validate=True))
    # The native path selector checks pkl presence, then the reader prefers the
    # actual companion bin. This marker is not claimed as a graph transcription.
    (source / 'ri_adjacency_2020.pkl').write_bytes(b'')
    manifest = json.loads((ROOT / 'data/manifest.json').read_text(encoding='utf-8'))
    manifest['local_outputs_dir'] = str(cache)
    manifest_file = output / 'input-manifest.json'
    manifest_file.write_text(json.dumps(manifest), encoding='utf-8')
    cli = output / 'bisect-reference.exe'
    shutil.copy2(ROOT / 'target/release/bisect.exe', cli)
    target = output / 'outputs'
    args = ['state', '--state', 'RI', '--year', '2020', '--state-name', '../Ignored', '--districts', '2', '--metis-engine', 'metis-core', '--structure', 'nway', '--search', 'single', '--seed', '18446744073709551615', '--balance-tolerance', '5', '--manifest', '--output-dir', str(target)]
    result = subprocess.run([str(cli), *args], cwd=output, env={**os.environ, 'BISECT_MANIFEST': str(manifest_file)}, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=120)
    (output / 'stdout.log').write_text(result.stdout, encoding='utf-8')
    (output / 'stderr.log').write_text(result.stderr, encoding='utf-8')
    plan_file = target / '2020/states/rhode_island/plan.rplan'
    status = 'failed'
    if result.returncode == 0 and plan_file.is_file():
        plan = json.loads(plan_file.read_text(encoding='utf-8'))
        assert plan['metadata']['label'] == 'rhode_island_congressional_2020'
        status = 'passed'
    record = dict(status=status, native=pin(cli), arguments=args, exit_code=result.returncode, input_manifest=pin(manifest_file), files=[pin(p) for p in sorted(output.rglob('*')) if p.is_file()], relative_target='2020/states/rhode_island', pkl_presence_marker_only=True, scope='Actual no-adjacency native branch ignores even an unsafe requested state-name. The owned manifest selects a presence-marker pkl and real preferred binary/companions. No pkl graph equivalence or complete native provenance-hash claim.')
    (output / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(output, flush=True)
    assert status == 'passed'


if __name__ == '__main__':
    main()

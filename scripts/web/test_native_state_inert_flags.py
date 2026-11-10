"""Actual CLI diagnostic for four currently parsed-but-unused State flags.

This is a native integration test, not full field/browser/release acceptance.
Every native effect is confined to a new owned temporary output tree.
"""
import base64
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
output = ROOT / 'target/state-inert-flags' / str(time.time_ns())
output.mkdir(parents=True, exist_ok=False)
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def pin(p):
    return {'path': p.relative_to(ROOT).as_posix(), 'sha256': digest(p), 'bytes': p.stat().st_size}


executable = 'bisect.exe' if os.name == 'nt' else 'bisect'
cli = output / executable
shutil.copy2(ROOT / 'target/release' / executable, cli)
fixture_file = ROOT / 'scripts/web/fixtures/native-state-source.json'
fixture = json.loads(fixture_file.read_text(encoding='utf-8'))['nativeAdjacencyInputs']
source = output / 'sources'
source.mkdir()
for key, name in [('adjacency', 'fixture.adj.bin'), ('geoids', 'fixture_geoids.json'), ('centroids', 'fixture_centroids.json')]:
    if fixture.get(key):
        (source / name).write_bytes(base64.b64decode(fixture[key]['base64'], validate=True))
receipt = {
    'status': 'running', 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'scope': 'Actual copied native CLI, four-unit custom adjacency, nway/single/seed42: assignment/context and selected manifest effects for four parsed-unused State flags; not browser or full flag acceptance',
    'native': pin(cli), 'fixture': pin(fixture_file), 'test': pin(Path(__file__)), 'outcomes': [],
    'source_identities_observed': [pin(ROOT / name) for name in ['crates/bisect-runner/src/state_args.rs', 'crates/bisect-cli/src/main.rs']],
    'complete_build_provenance': False,
}
selected_fields = ['label', 'state_code', 'year', 'chamber', 'num_districts', 'population_source', 'partition_mode', 'seed', 'adjacency_sha256']
cases = [
    ('baseline', []), ('dpi', ['--dpi', '300']), ('print-only', ['--print-only']),
    ('target-mm-districts', ['--target-mm-districts', '1']), ('objtype', ['--objtype', 'vol']),
    ('combined', ['--dpi', '300', '--print-only', '--target-mm-districts', '1', '--objtype', 'vol']),
    ('invalid-dpi', ['--dpi', '600']),
]
baseline = None
try:
    for name, extra in cases:
        target = output / name
        arguments = ['state', '--state', 'RI', '--year', '2020', '--adjacency', str(source / 'fixture.adj.bin'), '--districts', '2', '--metis-engine', 'metis-core', '--structure', 'nway', '--search', 'single', '--seed', '42', '--balance-tolerance', '5', '--output-dir', str(target), '--manifest', *extra]
        reply = subprocess.run([str(cli), *arguments], cwd=output, capture_output=True, text=True, encoding='utf-8', errors='strict', timeout=120)
        (output / f'{name}.stdout.log').write_text(reply.stdout, encoding='utf-8')
        (output / f'{name}.stderr.log').write_text(reply.stderr, encoding='utf-8')
        files = [pin(p) for p in sorted(target.rglob('*')) if p.is_file()]
        outcome = {'case': name, 'arguments': arguments, 'exit_code': reply.returncode, 'files': files}
        receipt['outcomes'].append(outcome)
        if name == 'invalid-dpi':
            assert reply.returncode != 0, reply.stdout
            assert not files, 'Parser refusal must not generate output'
            continue
        assert reply.returncode == 0, reply.stderr
        assignments = list(target.rglob('final_assignments.json'))
        contexts = list(target.rglob('context.rctx'))
        manifests = list(target.rglob('manifest.json'))
        assert len(assignments) == len(contexts) == len(manifests) == 1
        manifest = json.loads(manifests[0].read_text(encoding='utf-8'))
        effects = {'assignments': digest(assignments[0]), 'context': digest(contexts[0]), 'manifest_fields': {key: manifest[key] for key in selected_fields}}
        if baseline is None:
            baseline = effects
        else:
            assert effects == baseline, name
        outcome['effects'] = effects
    receipt['status'] = 'passed'
except Exception as e:
    receipt['status'] = 'failed'
    receipt['error'] = repr(e)
    raise
finally:
    receipt['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    (output / 'evidence.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': receipt['status'], 'output': str(output), 'cases': len(receipt['outcomes'])}), flush=True)

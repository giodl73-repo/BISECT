"""Retain actual native State output namespaces for the naming workflow."""
import base64
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(path):
    raw = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(raw).hexdigest(), byte_length=len(raw))


def main():
    output = ROOT / 'target/state-identity' / ('native-' + str(time.time_ns()))
    output.mkdir(parents=True)
    cli = output / 'bisect-reference.exe'
    shutil.copy2(ROOT / 'target/release/bisect.exe', cli)
    fixture_path = ROOT / 'scripts/web/fixtures/native-state-source.json'
    fixture = json.loads(fixture_path.read_text(encoding='utf-8'))['nativeAdjacencyInputs']
    source = output / 'sources'
    source.mkdir()
    for key, name in [('adjacency', 'fixture.adj.bin'), ('geoids', 'fixture_geoids.json'), ('centroids', 'fixture_centroids.json')]:
        if fixture.get(key):
            (source / name).write_bytes(base64.b64decode(fixture[key]['base64'], validate=True))
    outcomes = []
    for name, alias, label in [('registry-default', None, None), ('raw-portable', 'Research_Area', None), ('raw-spaces', 'Research Area', None), ('explicit-label', 'Research_Area', 'chosen_plan')]:
        target = output / name
        args = ['state', '--state', 'RI', '--year', '2020', '--adjacency', str(source / 'fixture.adj.bin'), '--districts', '2', '--metis-engine', 'metis-core', '--structure', 'nway', '--search', 'single', '--seed', '18446744073709551615', '--balance-tolerance', '5', '--output-dir', str(target), '--manifest']
        if alias is not None:
            args += ['--state-name', alias]
        if label is not None:
            args += ['--label', label]
        result = subprocess.run([str(cli), *args], cwd=output, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=120)
        (output / (name + '.stdout.log')).write_text(result.stdout, encoding='utf-8')
        (output / (name + '.stderr.log')).write_text(result.stderr, encoding='utf-8')
        files = [pin(p) for p in sorted(target.rglob('*')) if p.is_file()]
        outcomes.append(dict(name=name, arguments=args, exit_code=result.returncode, files=files))
    record = dict(status='passed' if all(r['exit_code'] == 0 for r in outcomes) else 'failed', native=pin(cli), fixture=pin(fixture_path), outcomes=outcomes, scope='Actual retained native CLI output-tree reference for State identity. Four complete two-district fixed-seed runs. Browser/source/target acceptance remains separate.')
    (output / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(output, flush=True)
    assert record['status'] == 'passed'


if __name__ == '__main__':
    main()

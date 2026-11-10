"""Exercise the current native CLI against frozen synthetic map controls.

No partition is regenerated. All filesystem inputs are reconstructed from the
hash-bound corpus in a fresh retained directory; no fixture paths are followed.
"""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--native-cli', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
binary = args.native_cli.resolve()
assert binary.is_file()
parent = args.output.resolve()
parent.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='native-', dir=parent))
fixture = Path(__file__).resolve().parent/'fixtures/fine-unit-map'
raw = (fixture/'corpus.json').read_bytes()
sha = lambda b: hashlib.sha256(b).hexdigest()
assert sha(raw) == json.loads((fixture/'manifest.json').read_text(encoding='utf-8'))['corpus_sha256']
corpus = json.loads(raw)
assert corpus['schema_version'] == 'bisect-fine-unit-map-regression-v1'
record = dict(status='running', native_sha256=sha(binary.read_bytes()),
              corpus_sha256=sha(raw), cases=[], scope=corpus['scope'])
cwd = root/'cwd'

def source(tiger):
    stem = 'tl_2020_01_'+('bg' if tiger['kind'] == 'block-groups' else 'tract')
    folder = cwd/'data/2020/tiger/tracts'/stem
    folder.mkdir(parents=True, exist_ok=True)
    for part in ['shp', 'dbf', 'shx']:
        value = base64.b64decode(tiger[part]['base64'], validate=True)
        file = folder/(stem+'.'+part)
        if file.exists():
            assert file.read_bytes() == value
        else:
            file.write_bytes(value)

def run(name, request, resolution, *, rounds=None, missing_manifest=False,
        legacy=False, expected_error=None):
    output = root/name
    plan = output/('v1/2020/states/alabama' if legacy else 'v1/2020/plans/MapCase')
    (plan/'data').mkdir(parents=True)
    (plan/'data/final_assignments.json').write_bytes(base64.b64decode(request['assignments_base64'], validate=True))
    if not missing_manifest:
        manifest = dict(corpus['native_manifest'])
        if resolution == '__absent__':
            manifest.pop('plan_resolution', None)
        else:
            manifest['plan_resolution'] = resolution
        (plan/'manifest.json').write_text(json.dumps(manifest), encoding='utf-8')
    if rounds is not None:
        folder = plan/'intermediate/depth_00'
        folder.mkdir(parents=True)
        (folder/'assignments.json').write_bytes(base64.b64decode(rounds['assignments_base64'], validate=True))
    command = [str(binary), 'map', '--state', 'AL', '--year', '2020',
               '--version', 'v1', '--output-base', str(output), '--types',
               'rounds' if rounds is not None else request['map_type'], '--dpi', '72', '--force']
    if not legacy:
        command += ['--label', 'MapCase']
    result = subprocess.run(command, cwd=cwd, capture_output=True)
    (output/'stdout').write_bytes(result.stdout)
    (output/'stderr').write_bytes(result.stderr)
    if expected_error:
        assert result.returncode != 0 and expected_error.encode() in result.stderr, (name, result.stderr)
        record['cases'].append(dict(name=name, refused=True))
        return None
    assert result.returncode == 0, (name, result.stderr)
    png = plan/('maps/rounds/round_00.png' if rounds is not None else 'maps/'+request['map_type']+'.png')
    digest = sha(png.read_bytes())
    record['cases'].append(dict(name=name, png_sha256=digest))
    return digest

try:
    for case in corpus['cases']:
        request = case['request']['input']
        source(request['tiger'])
        if 'unit_index_base64' in request:
            folder = cwd/'outputs/v1/data/2020/adjacency'
            folder.mkdir(parents=True, exist_ok=True)
            (folder/'al_bg_adjacency_2020_geoids.json').write_bytes(base64.b64decode(request['unit_index_base64'], validate=True))
        # The actual native map command reads analysis beside the plan.
        if 'analysis_base64' in request:
            folder = root/case['name']/'v1/2020/plans/MapCase/analysis'
            folder.mkdir(parents=True)
            (folder/(request['map_type']+'.json')).write_bytes(base64.b64decode(request['analysis_base64'], validate=True))
        resolution = 'bg' if case['kind'] == 'block-groups' else 'tract'
        assert run(case['name'], request, resolution) == case['native_png_sha256']
    bg = next(c for c in corpus['cases'] if c['name'] == 'bg-full-districts')
    tract = next(c for c in corpus['cases'] if c['name'] == 'tract-full-districts')
    for alias in ['bg', 'block_group', 'block-group']:
        assert run('alias-'+alias, bg['request']['input'], alias) == bg['native_png_sha256']
    for name in ['missing-field', 'legacy-missing-file']:
        assert run(name, tract['request']['input'], '__absent__',
                   missing_manifest=name == 'legacy-missing-file',
                   legacy=name == 'legacy-missing-file') == tract['native_png_sha256']
    for name, value, error in [('unknown', 'blocks', 'Recorded final map resolution is not supported'),
                               ('numeric', 12, 'invalid manifest.json'),
                               ('null', None, 'invalid manifest.json')]:
        run(name, bg['request']['input'], value, expected_error=error)
    run('label-missing-manifest', bg['request']['input'], 'bg', missing_manifest=True,
        expected_error='has no manifest.json')
    seeds = tract['request']['input']
    assert run('bg-final-tract-round', bg['request']['input'], 'bg', rounds=seeds) == run('tract-final-tract-round', bg['request']['input'], 'tract', rounds=seeds)
    record['status'] = 'passed-maintained-native-fine-maps'
except BaseException as error:
    record.update(status='failed-maintained-native-fine-maps', error=repr(error))
    raise
finally:
    (root/'evidence.json').write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
print(json.dumps(dict(root=str(root), status=record['status'], cases=len(record['cases']))))

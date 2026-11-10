"""Check retained full-chamber browser compactness against the native analysis kernel."""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile


STATES = {'NC': (2672, 14), 'CA': (9129, 52)}
CAMPAIGN = 'batch-full-chamber-map-report-v1'


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def checked_workflow(profile, matrix_hash, cases):
    receipt = read(profile / 'evidence.json')
    assert receipt['manifest_sha256'] == matrix_hash and receipt['followup_id'] == CAMPAIGN
    assert not receipt['full_parity']
    assert len(receipt['outcomes']) == len(cases)
    assert {row['state'] for row in receipt['outcomes']} == set(cases)
    for row in receipt['outcomes']:
        state = row['state']
        assert not row['partition_execution_performed']
        assert all(row[k] is True for k in ('exact_fixed_clock_request', 'exact_native_wasm_result',
                                           'open_inert', 'archive_roundtrip_exact', 'zip_download'))
        source = profile / state
        request = read(source / 'actual-gui-request.json')
        assert request == read(source / 'gui-clock-reference-request.json')
        assert request['operation'] == 'run-accepted-state-workflow'
        accepted = request['input']
        geometry = accepted['options']['analysis']['compactness']
        assert geometry['dissolve_method'] == 'batch'
        assert geometry['coordinate_basis'] == 'nad83-degrees' and geometry['crs_label'] == 'EPSG:4269'
        text = geometry['unit_geometry_json']
        assert sha(text.encode()) == cases[state]['geometry']['decoded_sha256']
        collection = json.loads(text)
        assert collection['geometry_profile'] == 'display-topology'
        plan = json.loads(accepted['document_json'])['plan']
        units, districts = STATES[state]
        ids = plan['units']['unit_ids']
        assert len(ids) == units and plan['k'] == districts
        assert len(collection['features']) == units
        assert {f['properties'].get('GEOID', f['properties'].get('geoid')) for f in collection['features']} == set(ids)
        result = read(source / 'gui-clock-native-stdout.json')
        assert result['partition_execution_performed'] is False
        workflow = result['workflow']
        name = 'analysis/compactness.json'
        report = base64.b64decode(workflow['files_base64'][name], validate=True)
        assert sha(report) == workflow['file_sha256'][name]
        with zipfile.ZipFile(source / 'outputs.zip') as archive:
            assert archive.namelist().count(name) == 1
            assert archive.read(name) == report
            assert archive.read('plan.rplan') == accepted['document_json'].encode()
            assert archive.read('context.rctx') == accepted['context_json'].encode()
        yield state, {'document_json': accepted['document_json'], **geometry}, report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('core', 'c-enabled', 'declaration', 'native-kernel', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    matrix_bytes = args.declaration.read_bytes()
    matrix = json.loads(matrix_bytes)
    campaign, = (f for c in matrix['followup_campaigns'] for f in c['map_report_followups'] if f['id'] == CAMPAIGN)
    cases = {r['state']: r for r in campaign['rows']}
    assert set(cases) == set(STATES)
    kernel = args.native_kernel.resolve(strict=True)
    args.output.mkdir()
    core = {state: (request, report) for state, request, report in checked_workflow(args.core, sha(matrix_bytes), cases)}
    other = {state: (request, report) for state, request, report in checked_workflow(args.c_enabled, sha(matrix_bytes), cases)}
    assert core == other
    outcomes = {}
    for state, (request, report) in core.items():
        request_path = args.output / f'{state}-compactness-request.json'
        native_path = args.output / f'{state}-compactness-native.json'
        request_path.write_text(json.dumps(request, separators=(',', ':')), encoding='utf8')
        completed = subprocess.run([str(kernel), str(request_path), str(native_path)],
                                   capture_output=True, timeout=300, check=False)
        (args.output / f'{state}-kernel.stderr').write_bytes(completed.stderr)
        assert completed.returncode == 0, completed.stderr.decode(errors='replace')
        assert native_path.read_bytes() == report, f'{state} compactness report differs from the native kernel'
        district_rows = json.loads(report)['districts']
        assert len(district_rows) == STATES[state][1] and all(r['status'] == 'measured' for r in district_rows)
        outcomes[state] = dict(units=STATES[state][0], districts=STATES[state][1],
                               report_sha256=sha(report), request_sha256=sha(request_path.read_bytes()))
    receipt = dict(status='passed-full-chamber-browser-compactness-native-kernel',
                   profile='explicit-batch-retained-display-topology', manifest_sha256=sha(matrix_bytes),
                   browser_profiles=['core', 'c-enabled'], native_kernel_sha256=sha(kernel.read_bytes()),
                   outcomes=outcomes, full_parity=False)
    (args.output / 'evidence.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf8')
    print(receipt['status'])


if __name__ == '__main__':
    main()

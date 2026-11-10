"""Read back C State report artifacts independently of the browser adapters."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def native_artifact(row, name):
    entry = next(ref for ref in row['files'] if ref['path'].endswith('/' + name))
    data = Path(entry['path']).read_bytes()
    assert sha(data) == entry['sha256'] and len(data) == entry['byte_length']
    return data


def plan_assignments(document):
    plan = document['plan']
    return dict(zip(plan['units']['unit_ids'], (d + 1 for d in plan['assignment']), strict=True))


def main(run_root, output):
    receipt = read(run_root / 'evidence.json')
    assert receipt['status'] == 'passed-C-report-workflow' and receipt['browser_checks']
    assert receipt['actual_c_dispatch'] and not receipt['actual_os_picker_permission']
    assert all(stage['status'] == 'passed' and stage['exit_code'] == 0 for stage in receipt['stages'])
    native = read(Path(receipt['native_reference']['path']))
    native_rows = {r['name']: r for r in native['outcomes']}
    assert native['requested_metis_engine'] == 'c-ffi' and len(native_rows) == 11
    baseline = plan_assignments(json.loads(native_artifact(native_rows['total'], 'plan.rplan')))
    runtime = read(Path(receipt['kernel_browser']['path']))['loaded_artifacts']
    assert runtime['metis_sidecar']['runtime'] == 'msvcrt-v1'
    results = {}
    for mode in ('portable', 'folder'):
        browser_file = Path(receipt[mode + '_browser']['path'])
        browser = read(browser_file)
        assert browser['actual_c_dispatch'] and browser['wasm']['sha256'] == runtime['engine']['sha256']
        expected_names = set(native_rows) - ({'cvap-directory'} if mode == 'portable' else set())
        assert set(c['name'] for c in browser['cases']) == expected_names
        checked = []
        for name in sorted(expected_names):
            row = native_rows[name]
            path = browser_file.parent / (name + '-completed.bisect')
            packet = read(path)
            run = packet['result']
            report = run['native_report_sources']
            prepared = run['prepared']
            assert prepared['result']['assignments'] == baseline
            assert prepared['result']['backend'] == 'c-metis-msvcrt-v1'
            assert prepared['runtime_artifacts'] == runtime
            assert report['requested_population_source'] == row['requested']
            assert report['cvap_missing_warning'] == row['missing_cvap_warning']
            assert report['balance_population_source'] == 'selected-graph-populations'
            if row['expected_native_failure']:
                stderr = native_artifact(row, 'stderr.log').decode(errors='replace')
                assert row['exit_code'] != 0 and 'VAP demographics load failed' in stderr
                assert run['status'] == 'partial-output' and prepared['export'] is None and report['status'] == 'failed'
                if name == 'cvap-empty-file':
                    assert 'empty VAP demographics file' in stderr and 'empty VAP demographics file' in prepared['export_error']
                else:
                    assert row['presence'] == 'directory' and 'directory' in prepared['export_error']
            else:
                assert row['exit_code'] == 0 and report['status'] == 'bound'
                plan = json.loads(prepared['export']['document_json'])
                original = json.loads(native_artifact(row, 'plan.rplan'))
                assert plan_assignments(plan) == plan_assignments(original) == baseline
                assert plan['plan']['units']['unit_ids'] == original['plan']['units']['unit_ids']
                source = json.loads(native_artifact(row, 'context.rctx'))
                context = json.loads(prepared['export']['context_json'])
                for field in ('populations', 'demographics'):
                    assert context.get(field) == source.get(field)
                manifest = json.loads(native_artifact(row, 'manifest.json'))
                assert manifest['native_replay_recipe']['recursive_metis_backend'] == 'c-ffi'
                assert manifest['native_replay_recipe']['state']['algorithm']['metis']['seed'] == '18446744073709551615'
            checked.append({'name': name, 'native_export_failed': row['expected_native_failure'], 'project_sha256': sha(path.read_bytes())})
        if mode == 'portable':
            path = browser_file.parent / 'cvap-empty-file-retained.zip'
            with zipfile.ZipFile(path) as z:
                assert z.testzip() is None
                assert json.loads(z.read('final_assignments.json')) == baseline
                assert not any(p.endswith(('plan.rplan', 'context.rctx')) for p in z.namelist())
            assert read(browser_file.parent / 'explicit-replay.bisect')['result']['prepared']['result']['assignments'] == baseline
        else:
            assert browser['actual_opfs_handles'] and not browser['actual_os_picker_permission']
            assert len(browser['faults']) == 9 and browser['explicit_recovery']
            recovered = read(browser_file.parent / 'recovered-run.bisect')['result']
            assert recovered['native_report_sources']['status'] == 'bound'
            assert recovered['prepared']['result']['assignments'] == baseline
        results[mode] = checked
    with output.open('x', encoding='utf8') as f:
        json.dump({'status': 'passed-independent-c-report-readback', 'runtime': runtime, 'cases': results,
                   'actual_os_picker_permission': False, 'full_parity': False}, f, indent=2)


if __name__ == '__main__':
    main(*map(Path, sys.argv[1:]))

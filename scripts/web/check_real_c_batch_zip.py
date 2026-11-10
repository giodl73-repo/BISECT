"""Independently check exported browser C batches against their native plans."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import zipfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(root, c_dir):
    native = json.loads((root / 'native-evidence.json').read_bytes())
    c = json.loads((root / c_dir / 'evidence.json').read_bytes())
    core = json.loads((root / 'browser-core/evidence.json').read_bytes())
    for evidence in (c, core):
        assert evidence['status'] == 'passed-packaged-real-c-batch' and evidence['open_inert']
        assert evidence['native_sha256'] == native['native_exe_sha256']
        assert len(evidence['cold']['failures']) == 2 and evidence['cold']['slots'] == 2
    assert core['online'] is None and core['offline'] is None
    assert c['mixed_runtime_refused_before_state_dispatch']
    offline = (root / c_dir / 'offline.zip').read_bytes()
    assert offline == (root / c_dir / 'online.zip').read_bytes()
    assert sha(offline) == c['online']['zip_sha256'] == c['offline']['zip_sha256']
    results = {}
    with zipfile.ZipFile(io.BytesIO(offline)) as batch:
        assert batch.testzip() is None
        assert set(batch.namelist()) == {'RI/prepared-run.zip', 'NH/prepared-run.zip', 'batch-receipt.json'}
        receipt = json.loads(batch.read('batch-receipt.json'))
        assert receipt['workers']['effective'] == 2 and receipt['summary']['succeeded'] == 2
        for row in receipt['states']:
            state = row['state_code']
            reference = native['states'][state]
            assert row['status'] == 'ok'
            with zipfile.ZipFile(io.BytesIO(batch.read(f'{state}/prepared-run.zip'))) as output:
                assert output.testzip() is None
                request = output.read('request.json')
                assert sha(request) == c['offline']['states'][state]['request_sha256']
                options = json.loads(request)['options']
                assert (options['seed'], options['metis_engine'], options['metis_runtime']) == ('42', 'c-ffi', 'msvcrt-v1')
                assert json.loads(request)['graph'] == json.loads((root/state/'graph.json').read_bytes())
                assignments = json.loads(output.read('final_assignments.json'))
                assert assignments == reference['assignments']
                plan = json.loads(output.read('plan.rplan'))['plan']
                assert dict(zip(plan['units']['unit_ids'], (d + 1 for d in plan['assignment']), strict=True)) == assignments
                graph = json.loads(request)['graph']
                populations = [sum(p for g, p in zip(graph['geoids'], graph['population'], strict=True)
                                   if assignments[g] == d) for d in (1, 2)]
                assert populations == reference['district_populations']
                state_receipt = json.loads(output.read('prepared-run-receipt.json'))
                assert state_receipt == row['result']
                assert state_receipt['prepared']['runtime_artifacts'] == c['offline']['states'][state]['runtime']
                results[state] = {'tracts': len(assignments), 'district_populations': populations,
                                  'request_sha256': sha(request), 'plan_sha256': sha(output.read('plan.rplan'))}
    assert sum(item['tracts'] for item in results.values()) == 600
    return {'status': 'passed-independent-real-c-batch-zip', 'states': results, 'zip_sha256': sha(offline),
            'both_workers_explicit_c': True, 'mixed_runtime_refused': True, 'full_parity': False}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('root', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--c-dir', type=Path, default=Path('browser-c'))
    args = p.parse_args()
    result = check(args.root, args.c_dir)
    with args.output.open('x', encoding='utf8') as f:
        json.dump(result, f, indent=2)

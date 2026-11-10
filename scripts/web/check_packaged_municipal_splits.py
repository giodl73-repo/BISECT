"""Independently audit the native/packaged-browser custom municipal witnesses."""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_bytes())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(runtime_path, browser_dir):
    runtime_bytes = runtime_path.read_bytes()
    runtime = json.loads(runtime_bytes)
    browser = read(browser_dir / 'evidence.json')
    assert runtime['status'] == 'passed-explicit-municipal-cli-wasm-and-package'
    assert browser['status'] == 'passed-packaged-offline-municipal-splits'
    assert browser['runtime_receipt_sha256'] == digest(runtime_bytes)
    assert browser['wasm_sha256'] == runtime['wasm']['sha256'] and browser['source_open_inert']
    checks = {row['name']: row for row in browser['checks']}
    assert len(checks) == len(runtime['rows']) == 9
    results = {}
    county_baseline = None
    for row in runtime['rows']:
        name = row['name']
        native = Path(row['native_stdout']['path']).read_bytes()
        assert digest(native) == row['native_stdout']['sha256']
        if 'wasm_result' not in row:
            assert row['native_status'] != 0 and checks[name]['outcome'] == 'matching-refusal'
            assert not (browser_dir / f'{name}-splits.json').exists()
            results[name] = {'outcome': 'matching-refusal'}
            continue
        assert row['native_status'] == 0
        served = (browser_dir / f'{name}-splits.json').read_bytes()
        assert served + b'\n' == native.replace(b'\r\n', b'\n')
        assert digest(served) == checks[name]['report_sha256']
        assert row['context']['sha256'] == checks[name]['context_sha256']
        report = json.loads(served)
        request = read(Path(row['request']['path']))
        assert digest(request['context_json'].encode()) == row['context']['sha256']
        ctx = json.loads(request['context_json'])
        plan = json.loads(request['document_json'])['plan']
        units = plan['units']['unit_ids']
        assert units == ctx['units']['unit_ids'] and len(set(units)) == len(units)
        districts = [d + 1 for d in plan['assignment']]
        ids = ctx['subdivisions'].get('municipal_ids')
        if ids is None:
            assert not report['municipalities']['available'] and 'municipal_identity_sha256' not in report
            county_baseline = report['counties']
            results[name] = {'outcome': 'legacy-unchanged'}
            continue
        assert report['counties'] == county_baseline
        mapping = dict(zip(units, ids, strict=True))
        text = json.dumps(mapping, ensure_ascii=False, separators=(',', ':'), sort_keys=True).encode()
        assert digest(b'bisect-explicit-municipal-identity-v1\0' + text) == report['municipal_identity_sha256']
        groups = defaultdict(list)
        for i, district in zip(ids, districts, strict=True):
            if i is not None:
                groups[i].append(district)
        unknown = ids.count(None)
        muni = report['municipalities']
        assert muni['known_units'] == len(ids) - unknown and muni['missing_units'] == unknown
        if unknown:
            assert not muni['available'] and muni['total'] is None and muni['split'] is None and muni['split_list'] == []
        else:
            split = [{'municipality_id': key, 'municipality_name': None,
                      'districts_containing': sorted(set(members)), 'unit_count': len(members),
                      'split_severity': len(set(members))} for key, members in sorted(groups.items()) if len(set(members)) > 1]
            assert muni['available'] and muni['total'] == len(groups) and muni['split'] == len(split) and muni['split_list'] == split
        results[name] = {'outcome': 'native-exact', 'unknown': unknown, 'split': muni['split'], 'report_sha256': digest(served)}
    return {'status': 'passed-independent-packaged-municipal-splits', 'results': results,
            'wasm_sha256': browser['wasm_sha256'], 'browser_receipt_sha256': digest((browser_dir/'evidence.json').read_bytes())}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('runtime', type=Path)
    p.add_argument('browser', type=Path)
    p.add_argument('output', type=Path)
    args = p.parse_args()
    verified = verify(args.runtime, args.browser)
    with args.output.open('x', encoding='utf-8') as stream:
        json.dump(verified, stream, indent=2)

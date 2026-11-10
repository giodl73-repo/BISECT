"""Read desktop Export outputs, both WASM receipts and ZIP/project bytes independently."""

import argparse
import base64
import csv
import hashlib
import io
import json
from pathlib import Path
import zipfile


MAX_U64 = 2**64 - 1
MAX_JS_INTEGER = 2**53 - 1


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(p):
    return json.loads(p.read_bytes())


def source_bytes(value):
    return base64.b64decode(value, validate=True)


def native_tree(case):
    out = case/'output'
    return {p.relative_to(out).as_posix():p.read_bytes() for p in out.rglob('*') if p.is_file()}


def check_zip(p, expected):
    with zipfile.ZipFile(p) as z:
        names = z.namelist()
        assert len(names) == len(set(names)) and set(names) == set(expected)
        assert {n:z.read(n) for n in names} == expected


def validate_source_case(case, record, response, native):
    if record['refused']:
        assert record['native_exit'] == 1 and not native and 'expected u64' in response['error']
        return
    result = response['result']
    assert record['native_exit'] == 0 and result['status'] == 'completed'
    assert result['selected_source'] == record['selected']
    assert {n:s.encode() for n,s in result['files'].items()} == native
    assert result['file_sha256'] == {n:sha(s) for n,s in native.items()}
    source = read(case/'requests.json')['folder']['input']['files']
    assert result['source_sha256'] == {n:sha(source_bytes(s)) for n,s in source.items()}
    document = json.loads(native['ExportWidth_reproducibility/ExportWidth.rplan'])
    assert result['document_json'].encode() == native['ExportWidth_reproducibility/ExportWidth.rplan']
    count = document['metadata']['num_districts']
    assert type(count) is int and result['declared_districts'] == (count if count <= MAX_JS_INTEGER else str(count))
    assert json.loads(native['ExportWidth_gerrychain.json'])['num_districts'] == count
    assert json.loads(native['ExportWidth_gerrychain.json'])['assignment'] == document['assignments']
    assert {unit:int(district) for unit,district in list(csv.reader(io.StringIO(native['ExportWidth_tracts.csv'].decode())))[1:]} == document['assignments']
    if record['id'] == 'legacy-both':
        assert count == document['assignments']['44001000100'] == MAX_U64
        assert [f['properties']['district_id'] for f in json.loads(native['ExportWidth.geojson'])['features']] == [2,MAX_U64]
    if record['id'] == 'original-ri-block':
        assert len(document['assignments']) == result['units'] == 25649
        assert document['assignments'] == json.loads(source_bytes(source['data/final_assignments.json']))


def audit(direct, browser, *, suite, original_ri=False):
    e,b = read(direct/'evidence.json'),read(browser/'evidence.json')
    assert e['status'].startswith('passed-native-') and b['status'] == 'passed-export-width-in-staged-browser'
    assert e['wasm_sha256'] == b['wasm_sha256'] and sha(Path(e['native']['path']).read_bytes()) == e['native']['sha256']
    assert len(e['records']) == (25 if original_ri else 24 if suite=='source' else 18)
    for record in e['records']:
        case = direct/record['id']; native = native_tree(case); result = read(case/'result.json')
        if suite == 'source':
            validate_source_case(case, record, result['folder'], native)
        else:
            folder = result['folder']['result']
            assert {n:s.encode() for n,s in folder['files'].items()} == native
            if record['wasm_rejected']:
                assert record['native_exit'] == 1 and 'expected u64' in result['direct']['error']
                assert folder['status'] == 'partial' and set(native) == {'WidthExport_tracts.csv'}
            else:
                prefix = 'WidthExport_reproducibility/'
                assert record['native_exit'] == 0 and folder['status'] == 'completed'
                assert {n:s.encode() for n,s in result['direct']['result']['files'].items()} == {n[len(prefix):]:s for n,s in native.items() if n.startswith(prefix)}
                req = read(case/'requests.json')['direct']
                assert native[prefix+'manifest.json'] == req['manifest_json'].encode()
                if record['id'] == 'wide-num_districts':
                    assert json.loads(native[prefix+'manifest.json'])['num_districts'] == MAX_U64
                    assert b'--districts '+str(MAX_U64).encode() in native[prefix+'verify.sh']
    assert b['actual_worker_runs'] == len(b['outcomes']) == (1 if original_ri else 5 if suite=='source' else 6)
    for row in b['outcomes']:
        case = direct/row['id']; browser_case = browser/row['id']; mode = row['mode']
        expected = read(case/'result.json')[mode]
        request = read(case/'requests.json')[mode]
        saved = read(browser_case/(mode+'-saved.bisect'))
        assert row['source_retained'] and row['inert_open']
        if mode == 'folder':
            assert saved['files']['exportFolderInputs'] == request['input']
        else:
            assert source_bytes(saved['files']['auditPlanText']['base64']) == request['document_json'].encode()
            assert source_bytes(saved['files']['reproducibilityInputs']['manifest']['base64']) == request['manifest_json'].encode()
        if 'result' in expected:
            assert saved['result'] == expected['result']
            check_zip(browser_case/(mode+'.zip'),{n:s.encode() for n,s in expected['result']['files'].items()})
        else:
            refusal = saved['files']['workerFailureReceipt']
            assert refusal['request_sha256'] == sha(refusal['request_json'].encode())
            assert json.loads(refusal['request_json']) == request and 'expected u64' in refusal['error']
            assert not (browser_case/(mode+'.zip')).exists()
    return dict(wasm_sha256=e['wasm_sha256'],native=e['native']['sha256'],desktop_cases=len(e['records']),
                browser_cases=b['actual_worker_runs'],original_ri_block=original_ri)


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--release',type=Path,required=True)
    p.add_argument('--ri',type=Path)
    p.add_argument('--profile',choices=('core','c'),action='append')
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    results={}
    for profile in a.profile or ('core','c'):
        d=a.release/profile
        records=dict(source=audit(d/'export-source',d/'export-source-browser',suite='source'),
                     reproducibility=audit(d/'reproducibility',d/'reproducibility-browser',suite='reproducibility'))
        if a.ri:
            records['ri']=audit(a.ri/profile/'direct',a.ri/profile/'browser',suite='source',original_ri=True)
        assert len({(r['wasm_sha256'],r['native']) for r in records.values()}) == 1
        results[profile]=records
    with a.output.open('x',encoding='utf8',newline='\n') as f:
        json.dump(dict(status='passed-exact-native-export-width-independent-readback',profiles=results,full_parity=False),f,indent=2)
        f.write('\n')
    print('Verified exact desktop Export, browser ZIPs and inert saved project sources.')

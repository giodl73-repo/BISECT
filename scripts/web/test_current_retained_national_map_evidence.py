"""Independent readback of retained 50-state maps and current native assignments."""

import argparse
import base64
import gzip
import hashlib
import io
import json
from pathlib import Path
import struct


MAX_PROJECT = 256 * 1024 * 1024
BAD = {'__proto__', 'prototype', 'constructor'}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def pinned(record):
    path = Path(record['path'])
    data = path.read_bytes()
    assert len(data) == record['byte_length'] and sha(data) == record['sha256']
    return data


def inflate(data, limit):
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as reader:
        decoded = reader.read(limit + 1)
    assert len(decoded) <= limit
    return decoded


def project(file):
    data = file.read_bytes()
    assert len(data) <= MAX_PROJECT
    value = json.loads(inflate(data, MAX_PROJECT) if data.startswith(b'\x1f\x8b') else data)
    if value['schema_version'] == 'bisect-workbench-archive-v2':
        strings = value.pop('shared_strings')
        refs = value.pop('shared_string_refs')
        assert 0 < len(strings) <= 2048 and all(type(s) is str for s in strings)
        assert 0 < len(refs) <= 10000
        keys, used = set(), set()
        for ref in refs:
            assert set(ref) == {'path', 'string_index'}
            parts, index = ref['path'], ref['string_index']
            assert type(parts) is list and 0 < len(parts) <= 64
            assert all(type(p) in (int, str) and p not in BAD for p in parts)
            assert type(index) is int and 0 <= index < len(strings) and tuple(parts) not in keys
            keys.add(tuple(parts))
            used.add(index)
            parent = value
            for component in parts[:-1]:
                parent = parent[component]
            assert parent[parts[-1]] is None
            parent[parts[-1]] = strings[index]
        assert used == set(range(len(strings)))
    else:
        assert value['schema_version'] == 'bisect-workbench-archive-v1'
    return value


def read(file):
    return json.loads(file.read_bytes())


def audit(reference, stage, browsers, partition):
    rows = read(reference / 'acceptance-contract.json')['rows']
    pack = read(reference / 'national-map-sources.json')['states']
    native_stage = read(partition / 'status.json')
    assert native_stage['status'] == 'passed-current-native-both-wasm-and-national-browser'
    partitions = json.loads(pinned(native_stage['native']))['outcomes']
    graph_root = Path(native_stage['profiles']['core']['wasm']['path']).parent
    actual_native = {r['state']: r for r in partitions}
    assert len(rows) == len(pack) == len(actual_native) == 50
    native_bytes = (stage / 'native-stdout.jsonl').read_bytes()
    assert native_bytes == (reference / 'native-stdout.jsonl').read_bytes()
    native = [json.loads(line) for line in native_bytes.splitlines()]
    assert len(native) == 52 and all(r['ok'] is True for r in native)
    result = native[-1]['result']
    rendered = {r['state']: r for r in result['provided_states']}
    assert len(rendered) == 50 and result['execution_performed'] is False
    states = []
    for row in rows:
        state = row['state']
        selected = pack[state]
        plan_bytes = base64.b64decode(selected['plan']['base64'], validate=True)
        assert sha(plan_bytes) == row['plan_sha256'] == rendered[state]['plan_source_sha256']
        plan = json.loads(plan_bytes)['plan']
        unit_ids = plan['units']['unit_ids']
        literal = dict(zip(unit_ids, (i + 1 for i in plan['assignment']), strict=True))
        assert len(literal) == len(unit_ids) == row['units'] and plan['k'] == row['districts']
        actual = actual_native[state]
        native_assignments = json.loads(pinned(actual['assignments']))
        order = read(graph_root / state / 'request.json')['input']['run']['graph']['geoids']
        assert len(order) == len(set(order)) == len(literal)
        assert native_assignments == {str(i): literal[geoid] for i, geoid in enumerate(order)}
        geometry = row['geometry']
        packed = base64.b64decode(selected['geometry']['base64'], validate=True)
        assert sha(packed) == geometry['sha256'] and len(packed) == geometry['byte_length']
        decoded = inflate(packed, geometry['decoded_byte_length'])
        assert sha(decoded) == geometry['decoded_sha256'] == rendered[state]['geometry_source_sha256']
        shape = json.loads(decoded)
        assert {f['properties']['geoid'] for f in shape['features']} == set(unit_ids)
        assert len(shape['features']) == row['units'] and shape['state'] == state
        assert (shape['geometry_profile'],shape['coordinate_basis'],shape['display_simplification_degrees']) == ('display-topology','nad83-degrees',0.0002)
        states.append(dict(state=state, units=row['units'], districts=row['districts'], native=actual['assignments']['sha256'],
                           plan=row['plan_sha256'], geometry=geometry['decoded_sha256']))
    assert sum(r['units'] for r in states) == result['units'] == 84208
    assert sum(r['districts'] for r in states) == result['districts'] == 435
    png, svg = (reference / 'national.png').read_bytes(), (reference / 'national.svg').read_bytes()
    assert png == base64.b64decode(result['png_base64'], validate=True) and svg == result['svg_text'].encode()
    assert sha(png) == result['png_sha256'] and sha(svg) == result['svg_sha256']
    assert png[:8] == b'\x89PNG\r\n\x1a\n' and struct.unpack('>II',png[16:24]) == (result['width'],result['height'])
    original = project(reference / 'national.bisect-workbench')
    assert original['result'] == result and original['operation'] == 'render-native-national-map-batch'
    profiles = {}
    for profile,browser in browsers.items():
        ui = read(browser / 'evidence.json')
        wasm = read(stage / profile / 'evidence.json')
        assert ui['status'] == wasm['status'] == 'passed' and wasm['native_execution_fresh']
        assert wasm['wasm']['sha256'] == ui['wasm_sha256'] and wasm['exact_native_stream_replies'] == 52
        assert ui['exact_native_result_and_exports'] and ui['archive_roundtrip_exact'] and ui['open_inert'] and ui['partition_operations'] == 0
        assert (browser/'national.png').read_bytes() == png and (browser/'national.svg').read_bytes() == svg
        for record in ui['served_assets']:
            pinned(record)
        saved = project(browser / 'saved.bisect-workbench')
        assert original['files'] == saved['files'] and saved['result'] == result
        assert saved['operation'] == original['operation']
        profiles[profile] = dict(wasm=ui['wasm_sha256'], browser=ui['browser'], saved_exact=True, inert_open=True,
                                 browser_evidence_sha256=sha((browser/'evidence.json').read_bytes()))
    return dict(status='passed-independent-current-retained-national-map', states=states, profiles=profiles,
                png_sha256=sha(png), svg_sha256=sha(svg), partition_operations=0, full_parity=False)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('reference','stage','core-browser','c-browser','partition','output'):
        p.add_argument('--'+name, type=Path, required=True)
    a = p.parse_args()
    r = audit(a.reference,a.stage,{'core':a.core_browser,'c':a.c_browser},a.partition)
    with a.output.open('x',encoding='utf8',newline='\n') as f:
        json.dump(r,f,indent=2)
        f.write('\n')
    print('Verified 50 accepted plans against current native assignments, exact map outputs, and both browser archives.')

"""Read raw TIGER, desktop PNG and browser archive evidence independently."""

import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
import struct


STATES = {'CA': (9129, 3165316, 52), 'TX': (6896, 3241644, 38)}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def archive(path):
    raw = path.read_bytes()
    value = json.loads(gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw)
    if value['schema_version'] == 'bisect-workbench-archive-v2':
        strings, refs = value.pop('shared_strings'), value.pop('shared_string_refs')
        assert 0 < len(strings) <= 2048 and 0 < len(refs) <= 10000
        used, locations = set(), set()
        for ref in refs:
            components, idx = tuple(ref['path']), ref['string_index']
            assert type(idx) is int and 0 <= idx < len(strings)
            assert 0 < len(components) <= 64 and components not in locations
            assert not set(components) & {'__proto__', 'prototype', 'constructor'}
            target = value
            for part in components[:-1]:
                target = target[part]
            assert target[components[-1]] is None and isinstance(strings[idx], str)
            target[components[-1]] = strings[idx]
            used.add(idx)
            locations.add(components)
        assert used == set(range(len(strings)))
        value['schema_version'] = 'bisect-workbench-archive-v1'
    assert value['schema_version'] == 'bisect-workbench-archive-v1'
    return value


def shapefile_counts(shp, shx):
    assert struct.unpack_from('>I', shp)[0] == 9994 and struct.unpack_from('>I', shx)[0] == 9994
    assert struct.unpack_from('>I', shp, 24)[0] * 2 == len(shp)
    assert struct.unpack_from('>I', shx, 24)[0] * 2 == len(shx)
    offset, positions, records = 100, 0, 0
    while offset < len(shp):
        assert 100 + 8 * (records + 1) <= len(shx)
        record, words = struct.unpack_from('>II', shp, offset)
        indexed_offset, indexed_words = struct.unpack_from('>II', shx, 100 + 8 * records)
        assert indexed_offset * 2 == offset and indexed_words == words and record == records + 1
        end = offset + 8 + words * 2
        assert offset + 12 <= end <= len(shp)
        kind, = struct.unpack_from('<I', shp, offset + 8)
        assert kind in (5, 15, 25) and offset + 52 <= end
        points, = struct.unpack_from('<I', shp, offset + 48)
        assert points > 0
        positions += points
        records += 1
        offset = end
    assert offset == len(shp) and len(shx) == 100 + records * 8
    return records, positions


def check(root, profile, state):
    n, b = root / ('native-' + profile) / state, root / ('browser-' + profile) / state
    native, browser, result = read(n / 'evidence.json'), read(b / 'evidence.json'), read(n / 'result.json')
    units, positions, districts = STATES[state]
    assert native['status'] == 'passed-current-original-tiger-native-cli-and-wasm'
    assert browser['status'] == 'passed-current-original-tiger-map-browser'
    assert native['wasm_sha256'] == browser['wasm_sha256']
    assert sha((Path(browser['site']) / 'bisect_wasm.wasm').read_bytes()) == native['wasm_sha256']
    assert sha(Path(native['native_cli']['path']).read_bytes()) == native['native_cli']['sha256']
    assert (result['units'], result['source_positions'], result['districts']) == (units, positions, districts)
    assert result['execution_performed'] is False and result['plan_hash'] is None
    assert result['geometry_materialization'] == 'original-wkb-direct'
    assert browser['archive_open_no_execution'] and browser['real_rust_worker'] and not browser['page_errors']
    cli_png, = (n / 'outputs').rglob('districts.png')
    png = cli_png.read_bytes()
    assert png == (b / 'state.png').read_bytes() == base64.b64decode(result['png_base64'], validate=True)
    assert png[:16] == b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR'
    assert struct.unpack('>II', png[16:24]) == (result['width'], result['height']) and result['width'] == 576
    svg = (b / 'state.svg').read_bytes()
    assert svg == result['svg_text'].encode() and sha(svg) == result['svg_sha256']
    assert sha(png) == result['png_sha256']
    original = archive(n / 'state.bisect-workbench')
    saved = archive(b / 'saved.bisect-workbench')
    assert original['files'] == saved['files'] and original['result'] == saved['result'] == result
    inputs = saved['files']['nativeMapInputs']
    raw = {}
    for part in ('shp', 'shx', 'dbf'):
        raw[part] = base64.b64decode(inputs['tiger'][part]['base64'], validate=True)
        binding = result['tiger_sources'][part]
        assert binding['sha256'] == sha(raw[part]) and binding['byte_length'] == len(raw[part])
    assert shapefile_counts(raw['shp'], raw['shx']) == (units, positions)
    assert struct.unpack_from('<I', raw['dbf'], 4)[0] == units
    assignments = base64.b64decode(inputs['assignments']['base64'], validate=True)
    actual, = (n / 'outputs').rglob('final_assignments.json')
    assert assignments == actual.read_bytes() and sha(assignments) == result['assignment_source_sha256']
    assert len(json.loads(assignments)) == units
    return dict(units=units, source_positions=positions, districts=districts, desktop_png_sha256=sha(png),
                browser_svg_sha256=sha(svg), assignment_sha256=sha(assignments),
                tiger_sha256={kind: sha(data) for kind, data in raw.items()})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--profile', choices=['core', 'c'], action='append')
    args = parser.parse_args()
    profiles = args.profile or ['core', 'c']
    assert len(profiles) == len(set(profiles))
    results = {profile: {state: check(args.root, profile, state) for state in STATES} for profile in profiles}
    if len(profiles) == 2:
        assert results['core'] == results['c']
    status = 'passed-current-original-tiger-map-readback'
    suffix = '-' + profiles[0] if len(profiles) == 1 else ''
    with (args.root / f'independent-readback{suffix}.json').open('x', encoding='utf8', newline='\n') as f:
        json.dump(dict(status=status, profiles=results, full_parity=False), f, indent=2)
        f.write('\n')
    print(status)

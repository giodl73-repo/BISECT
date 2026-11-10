"""Audit 64-bit manifest readers and retain their exact witnesses compactly."""

import argparse
import base64
from collections import Counter
import hashlib
import json
import lzma
from pathlib import Path


OPERATIONS = {'verify-manifest-certificate': 18, 'verify-manifest-prechecks': 22,
              'discover-original-assignments': 14, 'summarize-plan-labels': 18}
MAX_COUNT = 2**64 - 1


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def decoded(source):
    return base64.b64decode(source['base64'], validate=True)


def check_browser(row, case, browser):
    prefix = browser / f'reader-{case}'
    original = read(prefix.with_suffix('.bisect'))
    saved = read(prefix.with_name(prefix.name + '-saved.bisect'))
    assert saved['files'] == original['files'] and saved['result'] == original['result'] == row['actual']
    assert saved['operation'] == original['operation'] == row['operation']
    inputs = row['input']
    if row['operation'] == 'summarize-plan-labels':
        saved = saved['files']['multiSummaryInputs']
        assert saved['labels'] == inputs['labels']
        assert {label: {name: decoded(v).decode() for name, v in files.items()}
                for label, files in saved['plans'].items()} == inputs['plans']
        for extension, key in [('txt', 'table'), ('csv', 'csv')]:
            assert prefix.with_name(prefix.name + '-table.' + extension).read_bytes() == row['actual'][key].encode()
    else:
        verify = row['operation'].startswith('verify-')
        saved = saved['files']['packageVerificationInputs' if verify else 'assignmentDiscoveryInputs']
        assert decoded(saved['manifest']) == inputs['manifest_json'].encode()
        assert {name: value['base64'] for name, value in saved['files'].items()} == inputs['files_base64']
        if verify:
            assert read(prefix.with_name(prefix.name + '-receipt.json')) == row['actual']
        else:
            assert saved['explicitLabel'] == inputs['explicit_label']


def check_profile(native, browser):
    n, b = read(native / 'evidence.json'), read(browser / 'evidence.json')
    assert n['status'] == 'passed-manifest-reader-width' and b['status'] == 'passed-manifest-reader-width-workbench'
    assert b['staged_site'] and b['browser'] and n['wasm']['sha256'] == b['engine_sha256']
    assert sha((Path(b['staged_site']) / 'bisect_wasm.wasm').read_bytes()) == b['engine_sha256']
    assert sha((native / 'evidence.json').read_bytes()) == b['reference']['sha256']
    for item in [*n['sources'], n['cli'], n['native'], n['wasm'], *b['sources']]:
        assert sha(Path(item['path']).read_bytes()) == item['sha256']
    assert sha(Path('scripts/web/test_manifest_reader_width_browser.mjs').read_bytes()) == b['test_sha256']
    assert Counter(r['operation'] for r in n['records']) == OPERATIONS
    assert len(b['outcomes']) == b['explicit_runs'] == len(n['records']) == 72
    wide_fields = set()
    for i, (row, outcome) in enumerate(zip(n['records'], b['outcomes'], strict=True)):
        assert row['name'] == outcome['name'] and row['operation'] == outcome['operation']
        assert all(outcome[k] is True for k in ('explicit_run', 'exact_request', 'exact_result', 'exact_saved_sources'))
        assert read(native / f'{i}.json') == {'operation': row['operation'], 'input': row['input']}
        assert row['native']['status'] == 0
        assert json.loads(row['native']['stdout']) == row['actual']
        if row['name'].startswith('wide-') and row['operation'] == 'verify-manifest-certificate':
            field = row['name'].removeprefix('wide-').removesuffix('-certificate')
            assert json.loads(row['input']['manifest_json'])[field] == MAX_COUNT
            assert row['actual']['status'] == 'verified'
            wide_fields.add(field)
        check_browser(row, i, browser)
    assert len(wide_fields) == 13
    assert len(n['cli_records']) == 18
    expected_cli = {r['name']: r for r in n['records']}
    for index, result in enumerate(n['cli_records']):
        args = result['args']
        assert args[0] == 'verify' and args[-1] == '--dry-run'
        selected = native / f'cli-{index}'
        bound = expected_cli[result['name']]['input']
        assert (selected / 'manifest.json').read_bytes() == bound['manifest_json'].encode()
        assert all((selected / name).read_bytes() == base64.b64decode(raw, validate=True)
                   for name, raw in bound['files_base64'].items())
        if result['name'].startswith('audited-count-') and result['name'] != 'audited-count-1':
            assert result['status'] == 1
            assert 'ILP report count mismatch: manifest=' + result['name'].removeprefix('audited-count-') in result['stderr']
            assert 'certificate verified' not in result['stderr']
        else:
            assert result['status'] == 0 and 'certificate verified' in result['stderr']
            assert '[DRY RUN] Command not executed.' in result['stderr']
    return dict(wasm=n['wasm']['sha256'], native_cli=n['cli']['sha256'], wide_fields=sorted(wide_fields),
                normalized_parser_diagnostics=0, browser_runs=72, native_cli_dry_runs=18)


def retain(native, browser, output):
    rows = []
    for prefix, root in [('native', native), ('browser', browser)]:
        for f in sorted(root.rglob('*')):
            if f.is_file():
                data = f.read_bytes()
                assert len(data) <= 2 * 1024 * 1024
                rows.append(dict(path=prefix + '/' + f.relative_to(root).as_posix(), bytes=len(data),
                                 sha256=sha(data), base64=base64.b64encode(data).decode()))
    original = json.dumps({'files': rows}, separators=(',', ':')).encode()
    assert len(original) < 12 * 1024 * 1024
    encoded = lzma.compress(original, preset=6)
    assert lzma.decompress(encoded) == original
    with output.open('xb') as f:
        f.write(encoded)
    return dict(files=len(rows), bytes=len(encoded), sha256=sha(encoded), decoded_bytes=len(original),
                decoded_sha256=sha(original))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('native-core', 'native-c', 'browser-core', 'browser-c', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir()
    profiles = {}
    for name, n, b in [('core', args.native_core, args.browser_core), ('c', args.native_c, args.browser_c)]:
        profiles[name] = check_profile(n, b)
        profiles[name]['witnesses'] = retain(n, b, args.output / (name + '-witnesses.json.xz'))
    assert profiles['core']['wasm'] != profiles['c']['wasm']
    assert profiles['core']['wide_fields'] == profiles['c']['wide_fields']
    out = dict(status='passed-current-manifest-width-independent-readback', profiles=profiles,
               full_parity=False)
    with (args.output / 'evidence.json').open('x', encoding='utf8', newline='\n') as f:
        json.dump(out, f, indent=2)
        f.write('\n')
    print(out['status'])


if __name__ == '__main__':
    main()

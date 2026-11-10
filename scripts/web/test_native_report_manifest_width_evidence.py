"""Independently read desktop and staged browser Report width artifacts."""

import argparse
import base64
import hashlib
import json
import lzma
from pathlib import Path
import zipfile


MAX_COUNT = 2**64 - 1
LABEL = 'WidthOutput'


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def retain(groups, path):
    entries = []
    for prefix, root in groups:
        for f in sorted(root.rglob('*')):
            if f.is_file():
                data = f.read_bytes()
                assert len(data) < 4 * 1024 * 1024
                entries.append(dict(path=prefix + '/' + f.relative_to(root).as_posix(), bytes=len(data),
                                    sha256=sha(data), base64=base64.b64encode(data).decode()))
    raw = json.dumps(dict(files=entries), separators=(',', ':')).encode()
    assert len(raw) < 20 * 1024 * 1024
    encoded = lzma.compress(raw, preset=6)
    assert lzma.decompress(encoded) == raw
    with path.open('xb') as f:
        f.write(encoded)
    return dict(files=len(entries), bytes=len(encoded), sha256=sha(encoded),
                decoded_bytes=len(raw), decoded_sha256=sha(raw))


def check(root, profile, output):
    base = root / 'candidate-checks-r1'
    native, wasm, browser = root / ('native-report-' + profile), base / ('wasm-' + profile), base / ('browser-' + profile)
    n, w, b = (read(p / 'evidence.json') for p in (native, wasm, browser))
    image = read(root / f'build-r1/{profile}/image.json')
    assert n['status'] == 'passed-actual-native-report-width-reference' and n['native'] == w['native']
    assert w['status'] == 'passed-report-width-native-and-wasm' and b['status'] == 'passed-report-width-staged-browser'
    assert sha(Path(n['native']['path']).read_bytes()) == n['native']['sha256']
    assert w['wasm_sha256'] == b['wasm_sha256'] == image['wasm_sha256'] == sha((Path(image['site']) / 'bisect_wasm.wasm').read_bytes())
    assert b['actual_worker_runs'] == 38 and b['fallback_refusals'] == 2
    assert b['exact_u64_in_sandboxed_preview'] and w['label_report_count_alias_refused']
    assert len(n['records']) == len(w['outcomes']) == 18 and len(b['outcomes']) == 36
    assert {(r['id'], r['mode']) for r in b['outcomes']} == {(r['id'], mode) for r in n['records'] for mode in ('audit', 'report')}
    assert all(all(r[k] for k in ('inert_open', 'exact_worker_result', 'exact_downloads', 'exact_saved_source')) for r in b['outcomes'])
    wide_fields = set()
    for record in n['records']:
        name = record['id']
        a, p, q = native / name, wasm / name, browser / name
        manifest = (a / f'outputs/v1/2020/plans/{LABEL}/manifest.json').read_bytes()
        assert sha(manifest) == record['manifest_sha256']
        for mode, call in record['commands'].items():
            assert call['exit'] == 0 and call['argv'][0] == 'report'
            assert ('--audit-only' in call['argv']) == (mode == 'audit')
            for stream in ('stdout', 'stderr'):
                assert sha((a / (mode + '.' + stream)).read_bytes()) == call[stream + '_sha256']
        invalid = record['invalid']
        audit = (a / 'audit/audit.json').read_bytes()
        assert (a / 'report/audit.json').read_bytes() == audit
        report = (a / 'report' / (LABEL + '_report.json')).read_bytes()
        html = (a / 'report' / (LABEL + '_report.html')).read_bytes()
        assert [sha(x) for x in (audit, report, html)] == [record[k] for k in ('audit_sha256', 'json_sha256', 'html_sha256')]
        if name.startswith('wide-'):
            field = name.removeprefix('wide-')
            assert json.loads(manifest)[field] == MAX_COUNT
            wide_fields.add(field)
        assert json.loads(audit)['label'] == json.loads(report)['label'] == (LABEL if invalid else 'WidthRecorded')
        assert (audit, report, html) == tuple((q / file).read_bytes() for file in ('audit.json', 'report.json', 'report.html'))
        assert (q / 'companion-audit.json').read_bytes() == audit
        result = read(p / 'result.json')
        assert result['audit']['audit_json'].encode() == result['report']['audit_json'].encode() == audit
        assert result['report']['report_json'].encode() == report
        assert result['report']['report_html'].encode() == html
        for mode in ('audit', 'report'):
            project = read(q / (mode + '-saved.bisect'))
            assert project['result'] == result[mode]
            assert result[mode]['manifest_resolution']['fallback_used'] == invalid
            files = project['files']
            source = files['commissionAuditSource'] if mode == 'audit' else files['commissionInputs']['files']['manifest.json']
            assert base64.b64decode(source['base64'], validate=True) == manifest
        if name == 'wide-num_districts':
            assert json.loads(report)['sections']['executive_summary']['num_districts'] == MAX_COUNT
            assert str(MAX_COUNT).encode() in html and b'--districts ' + str(MAX_COUNT).encode() in audit
            with zipfile.ZipFile(q / 'delivery.zip') as z:
                assert len(z.namelist()) == 3
                assert {k: z.read(k) for k in z.namelist()} == {
                    'audit.json': audit, LABEL + '_report.html': html, LABEL + '_report.json': report}
    assert len(wide_fields) == 13
    mismatch = read(wasm / 'label-report-wide-alias.json')['response']
    assert mismatch['states'][0]['status'] == 'failed' and 'district count does not match' in mismatch['states'][0]['error']
    assert not any(p.endswith('_report.json') for p in mismatch['files'])
    return dict(wasm=image['wasm_sha256'], native=n['native']['sha256'], fields=sorted(wide_fields),
                direct_native_cases=18, browser_deliveries=36, mismatched_count_refused=True,
                witnesses=retain([('native', native), ('wasm', wasm), ('browser', browser)], output / (profile + '-witnesses.json.xz')))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.mkdir()
    result = dict(status='passed-current-report-width-independent-readback',
                  profiles={p: check(args.root, p, args.output) for p in ('core', 'c')}, full_parity=False)
    assert result['profiles']['core']['wasm'] != result['profiles']['c']['wasm']
    with (args.output / 'evidence.json').open('x', encoding='utf8', newline='\n') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(result['status'])

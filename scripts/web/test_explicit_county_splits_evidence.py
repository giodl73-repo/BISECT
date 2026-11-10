"""Check original CLI receipts, independent county counts and browser downloads."""

import argparse
import base64
from collections import defaultdict
import hashlib
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(root, profile):
    native = root / ('native-' + profile)
    browser = root / ('browser-' + profile)
    n, b = read(native / 'evidence.json'), read(browser / 'evidence.json')
    assert n['status'] == 'passed-explicit-county-native-and-wasm'
    assert b['status'] == 'passed-explicit-county-browser' and n['wasm_sha256'] == b['wasm_sha256']
    assert sha(Path(n['native_cli']['path']).read_bytes()) == n['native_cli']['sha256']
    assert (n['hosts'], n['cli_runs'], n['rejections'], n['legacy_byte_matches']) == (8, 7, 8, 3)
    assert n['independent_membership_oracle'] and n['numeric_custom_ids_require_explicit_counties']
    positive = {'imported': 'imported', 'precinct': 'precinct', 'imported-numeric': 'imported'}
    negative = {'missing-county': b'missing county identity',
                'mismatched-context': b'ordered unit identity mismatch',
                'extra-analyzers': b'requires --types splits alone',
                'mismatched-assignments': b'Native assignments differ'}
    assert n['desktop_cases'].keys() == positive.keys() | negative.keys()
    for name, case in n['desktop_cases'].items():
        stdout, stderr = (native / (name + '.stdout')).read_bytes(), (native / (name + '.stderr')).read_bytes()
        assert sha(stdout) == case['stdout_sha256'] and sha(stderr) == case['stderr_sha256']
        assert case['arguments'][0] == 'analyze'
        if name in negative:
            assert case['exit'] == 1 and not stdout and negative[name] in stderr
            continue
        assert case['exit'] == 0 and stdout.endswith(b'\n') and not stdout.endswith(b'\n\n')
        report = json.loads(stdout)['counties']
        dest = native / ('cli-' + name)
        plan, = dest.rglob('plan.rplan')
        plan = read(plan)['plan']
        context, = dest.rglob('context.rctx')
        context = read(context)
        ids = plan['units']['unit_ids']
        assert context['units']['unit_ids'] == ids
        assert plan['units']['unit_kind'] == positive[name]
        assert all(len(unit) == 11 and unit.isdecimal() for unit in ids) == (name == 'imported-numeric')
        groups = defaultdict(list)
        for county, district in zip(context['subdivisions']['county_ids'], plan['assignment'], strict=True):
            groups[county].append(district + 1)
        assert set(groups) == {'44001', 'supplied-A'}
        expected = [{'county_id': county, 'county_name': None, 'districts_containing': sorted(set(rows)),
                     'split_severity': len(set(rows)), 'unit_count': len(rows)}
                    for county, rows in sorted(groups.items()) if len(set(rows)) > 1]
        assert report['split_list'] == expected and report['legal_standard'] is None
        assert report['compliance_assessment'] is None
    desktop = (native / 'native-report.json').read_bytes()
    assert desktop == (native / 'imported.stdout').read_bytes()
    assert (browser / 'cli-matched-splits.json').read_bytes() + b'\n' == desktop
    assert sha(desktop) == b['native_stdout_sha256'] and b['exact_desktop_cli_download']
    observed = read(browser / 'cli-matched-request.json')
    sources = read(native / 'project.bisect')['files']
    for file, field in [('auditPlanText', 'document_json'), ('auditContextText', 'context_json')]:
        assert observed[field].encode() == base64.b64decode(sources[file]['base64'], validate=True)
    assert b['safe_text_names'] and b['missing_ids_refused'] and b['repair_explicit']
    assert b['analysis_partition_executions'] == 0 and not b['page_errors'] and b['inert_open_and_edits']
    return dict(wasm=n['wasm_sha256'], native_cli=n['native_cli']['sha256'],
                desktop_stdout_sha256=sha(desktop), positive=len(positive), negative=len(negative))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    args = parser.parse_args()
    profiles = {profile: check(args.root, profile) for profile in ('core', 'c')}
    assert profiles['core']['wasm'] != profiles['c']['wasm']
    assert profiles['core']['desktop_stdout_sha256'] == profiles['c']['desktop_stdout_sha256']
    result = dict(status='passed-current-explicit-county-native-browser-readback', profiles=profiles,
                  full_parity=False)
    with (args.root / 'independent-readback.json').open('x', encoding='utf8', newline='\n') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(result['status'])

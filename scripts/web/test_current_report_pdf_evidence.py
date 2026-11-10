"""Independently bind current Report PDF artifacts to their native HTML sources."""

import argparse
import base64
import hashlib
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_bytes())


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def audit(run):
    evidence = read(run / 'evidence.json')
    inspection = read(run / 'pdf-inspection.json')
    assert evidence['schema_version'] == 'bisect-report-pdf-browser-v1'
    assert evidence['native_result_matches'] == evidence['actual_pdf_cases'] == 6
    assert evidence['accepted_workflow_exact_native_and_browser'] is True
    assert evidence['os_print_dialog_tested'] is evidence['full_parity'] is False
    for tool in ('native_cli', 'native_host'):
        pin = evidence[tool]
        assert digest(Path(pin['path'])) == pin['sha256']
    wasm = Path(evidence['package']) / 'bisect_wasm.wasm'
    assert digest(wasm) == evidence['wasm']['sha256']
    assert all(digest(Path(r['path'])) == r['sha256'] for r in evidence['served_assets'])
    references = {name: read(run / (name + '-native-reply.json')) for name in
                  ('commission', 'label', 'comparison', 'accepted-workflow')}
    native_label, = (text for name, text in references['label']['files'].items() if name.endswith('.html'))
    workflow = references['accepted-workflow']
    assert workflow['partition_execution_performed'] is False
    assert workflow['workflow']['status'] == 'partial'
    workflow_path, = (name for name in workflow['workflow']['files_base64'] if name.endswith('.html'))
    workflow_html = base64.b64decode(workflow['workflow']['files_base64'][workflow_path], validate=True)
    assert hashlib.sha256(workflow_html).hexdigest() == workflow['workflow']['file_sha256'][workflow_path]
    original = {
        'commission-letter': references['commission']['report_html'].encode(),
        'commission-letter-landscape': references['commission']['report_html'].encode(),
        'label-a4-portrait': native_label.encode(),
        'label-a4-landscape': native_label.encode(),
        'comparison-letter': references['comparison']['html'].encode(),
        'accepted-workflow-letter': workflow_html,
    }
    cases = read(run / 'pdf-cases.json')
    independent = {r['id']: r for r in inspection['rows']}
    assert {r['id'] for r in cases} == set(independent) == set(original)
    for row in cases:
        case = row['id']
        assert (run / row['original_html']).read_bytes() == original[case]
        r = independent[case]
        assert r['pdf_sha256'] == digest(run / row['restricted_pdf'])
        assert all(r[k] is True for k in ('all_visible_text_preserved', 'native_restricted_text_equal',
                                         'media_box_matches', 'numbered_bottom_margin_footers', 'no_active_or_external_actions'))
        assert bool(r['native_png_pixel_matches']) == bool(row.get('map_png'))
        printed = read(run / row['record'])
        assert printed['selected_pdf']['sha256'] == r['pdf_sha256']
        assert printed['selected_pdf']['content_match_verified'] is printed['pdf_save_observed'] is False
        assert printed['settings'] == row['settings']
        assert printed['original_html_sha256'] == digest(run / row['original_html'])
    native_cli = read(run / 'native-cli-cases.json')
    assert [(r['status'], r['html_present'], r['pdf_present']) for r in native_cli] == [(0, True, False), (1, False, False)]
    saved = read(run / 'accepted-workflow-letter-saved.bisect')
    assert saved['result'] == workflow
    request = read(run / 'accepted-workflow-native-request.json')['input']
    for field, source in [('document_json', 'auditPlanText'), ('context_json', 'auditContextText')]:
        assert base64.b64decode(saved['files'][source]['base64'], validate=True) == request[field].encode()
    return dict(path=str(run), wasm_sha256=digest(wasm), pdfs=len(cases), pages=sum(r['pages'] for r in independent.values()),
                independent_pdf_inspection_sha256=digest(run / 'pdf-inspection.json'), cases=independent,
                os_print_dialog_tested=False, selected_pdf_content_match=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--core', type=Path, required=True)
    parser.add_argument('--c', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    profiles = {'core': audit(args.core), 'c': audit(args.c)}
    assert profiles['core']['wasm_sha256'] != profiles['c']['wasm_sha256']
    with args.output.open('x', encoding='utf8') as output:
        json.dump(dict(status='passed-current-report-pdf-native-html-readback', profiles=profiles, full_parity=False), output, indent=2)
        output.write('\n')
    print('Verified exact native HTML and PDF records in both profiles.')

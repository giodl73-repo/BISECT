"""Compare extracted shared rendering with the retained unmodified desktop CLI.

The original binary is an explicit integration-tier prerequisite. Small input
fixtures and expected native report bytes live in the source tree.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', default='target/native-label-summary-delivery/baseline/bisect.exe')
    parser.add_argument('--native', default='target/release/bisect.exe')
    args = parser.parse_args()
    baseline = (ROOT / args.baseline).resolve()
    native = (ROOT / args.native).resolve()
    fixtures = ROOT / 'crates/bisect-report/tests/fixtures/label-summary'
    provenance = json.loads((fixtures / 'provenance.json').read_text(encoding='utf-8'))
    assert digest(baseline) == provenance['native_cli']['sha256'], 'Original CLI differs; do not rewrite the baseline'
    root = ROOT / 'target/native-label-summary-delivery' / ('cli-' + str(time.time_ns()))
    root.mkdir()
    declared = dict(cases=['html', 'json', 'missing-index', 'invalid-json', 'unsupported-format', 'no-analyzed-year'], allowances=['Only top-level report JSON created and report-index created UTC time fields; original bytes retained.'], exact=['All other report/index bytes', 'Registry bytes', 'File roster', 'Exit status', 'stdout/stderr'])
    (root / 'comparison-contract.json').write_text(json.dumps(declared, indent=2)+'\n', encoding='utf-8')

    def normalize(name, raw):
        if name.startswith('reports/') and name.endswith('.json') and (name.endswith('_report.json') or name.endswith('/index.json')):
            text = raw.decode('utf-8')
            value = json.loads(text)
            if 'created' in value:
                assert isinstance(value['created'], str) and re.fullmatch(r'\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ', value['created'])
                text, count = re.subn(r'^  "created": "[^"\r\n]*"', '  "created": "<UTC-runtime-observation>"', text, count=1, flags=re.M)
                assert count == 1
            return text.encode('utf-8')
        return raw

    outcomes = []
    for case in declared['cases']:
        replies = []
        for name, cli in [('original', baseline), ('shared', native)]:
            folder = root / case / name
            analysis = folder / 'analysis/summary_fixture/2020'
            analysis.mkdir(parents=True)
            registry = {'summary_fixture': {'built': ['2020'], 'analyzed': [] if case == 'no-analyzed-year' else ['2020'], 'reported': []}}
            (folder / '.bisect').write_text(json.dumps(registry), encoding='utf-8')
            if case != 'missing-index':
                (analysis / 'index.json').write_bytes(b'{' if case == 'invalid-json' else (fixtures / 'analysis-index.json').read_bytes())
            fmt = 'json' if case == 'json' else 'pdf' if case == 'unsupported-format' else 'html'
            command = ['label-report', 'summary_fixture', '--year', '2020', '--format', fmt]
            result = subprocess.run([str(cli), *command], cwd=folder, capture_output=True, text=True)
            record = dict(arguments=command, exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr)
            files = {path.relative_to(folder).as_posix(): path.read_bytes() for path in folder.rglob('*') if path.is_file()}
            (folder / 'outcome.json').write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
            replies.append((record, files))
        expected, actual = replies
        assert actual[0] == expected[0], (case, actual[0], expected[0])
        assert expected[1].keys() == actual[1].keys(), case
        for name, raw in expected[1].items():
            assert normalize(name, actual[1][name]) == normalize(name, raw), (case, name)
        outcomes.append(dict(case=case, exit_code=actual[0]['exit_code'], exact_files=len(actual[1])))
    evidence = dict(original_cli=dict(path=str(baseline), sha256=digest(baseline)), native_cli=dict(path=str(native), sha256=digest(native)), outcomes=outcomes, claimed_scope='Native renderer extraction only; no WASM/UI/package acceptance', full_parity=False)
    (root / 'evidence.json').write_text(json.dumps(evidence, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(dict(root=str(root), cases=len(outcomes))))


if __name__ == '__main__':
    main()

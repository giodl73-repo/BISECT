"""Compare real desktop multi-label table and CSV bytes with saved browser downloads."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(native, browser, root):
    reference, gui = read(native / 'evidence.json'), read(browser / 'evidence.json')
    assert reference['status'] == 'passed-manifest-reader-width'
    assert gui['status'] == 'passed-manifest-reader-width-workbench' and gui['staged_site']
    assert sha((native / 'evidence.json').read_bytes()) == gui['reference']['sha256']
    cli = Path(reference['cli']['path']).resolve(strict=True)
    assert sha(cli.read_bytes()) == reference['cli']['sha256']
    cases = []
    for i, record in enumerate(reference['records']):
        if record['operation'] != 'summarize-plan-labels':
            continue
        result = record['actual']
        inputs = record['input']
        assert inputs['labels'] == ['one', 'two'] and set(inputs['plans']) <= {'one', 'two'}
        assert set(inputs['plans']) == ({'one'} if record['name'].startswith('invalid-') else {'one', 'two'})
        out = root / f'case-{i}'
        outputs = out / 'outputs'
        for label, files in inputs['plans'].items():
            plan = outputs / 'v1/2020/plans' / label
            plan.mkdir(parents=True)
            for filename, text in files.items():
                assert filename in {'manifest.json', 'summary.json', 'compactness.json', 'splits.json', 'contiguity.json'}
                (plan / filename).write_bytes(text.encode())
        commands = {}
        for fmt, extension, key in [('table', 'txt', 'table'), ('csv', 'csv', 'csv')]:
            argv = ['compare', '--plan-a', 'one', '--labels', 'one,two', '--year', '2020', '--version', 'v1',
                    '--output-base', str(outputs.resolve()), '--format', fmt]
            completed = subprocess.run([cli, *argv], cwd=out, capture_output=True, timeout=45, check=False)
            (out / (fmt + '.stdout')).write_bytes(completed.stdout)
            (out / (fmt + '.stderr')).write_bytes(completed.stderr)
            assert completed.returncode == 0, completed.stderr
            download = (browser / f'reader-{i}-table.{extension}').read_bytes()
            assert completed.stdout == download == result[key].encode(), (record['name'], fmt, completed.stdout, download)
            commands[fmt] = dict(args=argv, exit=0, stdout_sha256=sha(completed.stdout), stderr_sha256=sha(completed.stderr))
        cases.append(dict(name=record['name'], commands=commands))
    assert len(cases) == 18
    return dict(native_cli=reference['cli']['sha256'], wasm=gui['engine_sha256'], cases=cases)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('native-core', 'native-c', 'browser-core', 'browser-c', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir()
    profiles = {name: check(native, browser, args.output / name)
                for name, native, browser in [('core', args.native_core, args.browser_core),
                                              ('c', args.native_c, args.browser_c)]}
    status = 'passed-native-compare-manifest-width-browser-downloads'
    (args.output / 'evidence.json').write_text(json.dumps(dict(status=status, profiles=profiles, full_parity=False),
                                                           indent=2) + '\n', encoding='utf8')
    print(status)

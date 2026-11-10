"""Read back the desktop and browser LabelReport installation witnesses."""

import argparse
import hashlib
import json
from pathlib import Path
import re


def read(path):
    return json.loads(path.read_bytes())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def same_files(actual, expected):
    assert set(actual) == set(expected)
    for name, text in actual.items():
        if name == '.bisect':
            assert json.loads(text) == json.loads(expected[name])
        else:
            assert text == expected[name], name


def live_clock(text):
    return re.sub(r'^  "created": "[0-9TZ:.-]+"', '  "created": "<declared-clock>"', text, count=1, flags=re.M)


def audit(native_root, browser_root):
    e, b = read(native_root/'evidence.json'), read(browser_root/'evidence.json')
    assert sha(Path(e['wasm']['path']).read_bytes()) == e['wasm']['sha256'] == b['wasm_sha256']
    assert sha(Path(e['native']['path']).read_bytes()) == e['native']['sha256']
    assert sha(Path(e['fixed_clock_native']['path']).read_bytes()) == e['fixed_clock_native']['sha256'] == b['native_sha256']
    assert len(e['outcomes']) == 9 and len(e['recoveries']) == 31
    assert b['actual_worker_opfs'] and b['baseline_saved_open_inert'] and b['explicit_prefix_continuation']
    assert b['portable_without_directory_api'] and b['source_edit_retires_review'] and not b['os_picker_tested']
    for pin in b['served_assets']:
        path = Path(pin['path'])
        assert path.stat().st_size == pin['byte_length'] and sha(path.read_bytes()) == pin['sha256']
    prefix_cases = []
    for row in e['outcomes']:
        name = row['name']
        tested = read(native_root/(name+'.json'))
        original = read(native_root/(name+'-originals.json'))
        actual, fixed = original['actual'], original['fixed_clock_native']
        assert tested['input'] == original['input'] and tested['args'] == original['args']
        assert tested['actual'] == actual and tested['native']['files'] == fixed['files']
        same_files(actual,fixed['files'])
        live = original['native']
        assert live['status'] == fixed['status'] == row['native_exit'] == (1 if name=='partial' else 0)
        assert set(live['files']) == set(actual)
        if name != 'self-overlap':
            for file,text in actual.items():
                if file != '.bisect':
                    assert live_clock(text) == live_clock(live['files'][file]), (name,file)
        receipt = tested['receipt']
        assert receipt['cursor'] == receipt['total_steps'] == row['steps']
        assert receipt['status'] == row['status'] == ('completed-prefix' if name=='partial' else 'completed')
        prefix_cases.append(dict(id=name,native_exit=live['status'],files=len(actual),completed_steps=receipt['cursor']))
    for name in ('shared','overlap-create','self-overlap'):
        count = next(r['steps'] for r in e['outcomes'] if r['name']==name)
        recovered = [r for r in e['recoveries'] if r['name']==name]
        assert sorted(r['stop'] for r in recovered) == list(range(count+1))
        assert all(max(0,r['stop']-1) <= r['resume_cursor'] <= count for r in recovered)
    native_browser = read(browser_root/'native-outcome.json')
    assert native_browser['status'] == 0
    same_files(read(browser_root/'actual-files.json'),native_browser['files'])
    interrupted = read(browser_root/'interrupted.json')
    assert interrupted['status'] == 'cancelled' and interrupted['effects_may_have_started']
    saved = read(browser_root/'reviewed-baseline.bisect')
    baseline = saved['files']['nativeSummaryApplicationBaseline']
    original = read(browser_root/'original.bisect')
    assert saved['files']['labelReportInputs'] == original['files']['labelReportInputs']
    assert baseline['before']['files']['.bisect']['sha256']
    return dict(wasm=b['wasm_sha256'],native=e['native']['sha256'],fixed_clock=b['native_sha256'],
                desktop_cases=prefix_cases,recovery_boundaries=len(e['recoveries']),browser=b['browser'],
                os_picker_tested=False,source_archive_preserved=True)


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('core-native','c-native','core-browser','c-browser','output'):
        p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args()
    profiles={name:audit(getattr(a,name+'_native'),getattr(a,name+'_browser')) for name in ('core','c')}
    with a.output.open('x',encoding='utf8',newline='\n') as f:
        json.dump(dict(status='passed-current-native-summary-installation-readback',profiles=profiles,full_parity=False),f,indent=2)
        f.write('\n')
    print('Verified native LabelReport files, all declared recovery prefixes and both browser workspaces.')

"""Compare the independent desktop map renderer against the retained national map."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--native', type=Path, required=True)
    p.add_argument('--reference', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    native, ref, out = (q.resolve() for q in (a.native, a.reference, a.output))
    out.mkdir()
    expected = json.loads((ref/'acceptance-contract.json').read_bytes())
    requests = ref/'requests.jsonl'
    assert requests.stat().st_size < 400*1024*1024
    input_file = out/'national-input.json'
    states = {}
    with requests.open(encoding='utf8') as source, input_file.open('x', encoding='utf8', newline='\n') as f:
        begin = json.loads(next(source))
        assert begin['operation'] == 'begin-native-national-map'
        assert begin['input'] == {'dpi': 72, 'map_type': 'districts'}
        f.write('{"dpi":72,"map_type":"districts","states":{')
        for line in source:
            message = json.loads(line)
            if message['operation'] == 'finish-native-national-map':
                assert not source.read(1)
                break
            assert message['operation'] == 'append-native-national-map'
            state, value = message['input']['state'], message['input']['input']
            assert state not in states
            row, = (r for r in expected['rows'] if r['state'] == state)
            plan_hash = hashlib.sha256(value['document_json'].encode()).hexdigest()
            geometry_hash = hashlib.sha256(value['unit_geometry_json'].encode()).hexdigest()
            assert plan_hash == row['plan_sha256'] and geometry_hash == row['geometry']['decoded_sha256']
            if states:
                f.write(',')
            f.write(json.dumps(state)+':')
            json.dump(value, f, separators=(',', ':'), ensure_ascii=False)
            states[state] = {'plan': plan_hash, 'geometry': geometry_hash}
        else:
            raise AssertionError('Missing national map finish request')
        f.write('}}\n')
    assert len(states) == 50
    stem = out/'national-desktop'
    with (out/'stdout.txt').open('xb') as stdout, (out/'stderr.txt').open('xb') as stderr:
        result = subprocess.run([str(native), str(input_file), str(stem)], cwd=out, stdout=stdout,
                                stderr=stderr, timeout=1800, check=False)
    assert result.returncode == 0, (out/'stderr.txt').read_text(encoding='utf8', errors='replace')
    assert (out/'stdout.txt').read_text(encoding='utf8').strip() == '1152 720'
    outputs = {kind: sha(stem.with_suffix('.'+kind)) for kind in ('svg', 'png')}
    assert outputs == {kind: sha(ref/('national.'+kind)) for kind in outputs}
    with (out/'evidence.json').open('x', encoding='utf8') as f:
        json.dump(dict(status='passed-independent-desktop-national-renderer', native={'path':str(native),'sha256':sha(native)},
                       states=states, requests_sha256=sha(requests), input_sha256=sha(input_file),
                       output_sha256=outputs, partition_operations=0, full_parity=False), f, indent=2)
        f.write('\n')
    print('Independent desktop renderer matched the national SVG and PNG byte for byte.')

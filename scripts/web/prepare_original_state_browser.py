"""Stage fixed browser prerequisites from a completed original-state stream."""
import json
from pathlib import Path
import re
import sys
import zipfile

root = Path('target/stream-state-source-batch')
for state in sys.argv[1:]:
    assert re.fullmatch('[A-Z]{2}', state)
    evidence = json.loads((root / f'original-{state.lower()}-topology-evidence.json').read_text(encoding='utf-8'))
    assert evidence['summary']['succeeded'] == 1 and evidence['summary']['failed'] == 0
    folder = evidence['output_folder']
    assert re.fullmatch(r'bisect-run-[A-Za-z0-9-]{1,80}', folder)
    directory = root / folder / state
    output = Path('target/full-state-browser') / state
    output.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(directory / 'state-output.zip') as archive:
        assert archive.testzip() is None
        for name in ['prepared-graph.json', 'prepared-geometry.json']:
            (output / name).write_bytes(archive.read(state + '/' + name))
    (output / 'source.bisect-workbench').write_bytes((directory / 'source-project.bisect-workbench').read_bytes())
    graph = json.loads((output / 'prepared-graph.json').read_text(encoding='utf-8'))
    assert graph['state'] == state and graph['year'] == '2020'
    inputs = {'state': state, 'units': len(graph['geoids']), 'districts': evidence['selection']['states'][0]['districts']}
    (output / 'browser-inputs.json').write_text(json.dumps(inputs, indent=2) + '\n', encoding='utf-8')
    print(f'Original {state} source project and browser graph/geometry references prepared.')
if len(sys.argv) == 1:
    raise SystemExit('Supply one or more original state codes, e.g. TX MD.')

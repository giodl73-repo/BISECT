"""Compare saved WASM preparation with the independently run native SHP reader."""
import json
from pathlib import Path
import zipfile

root = Path('target/stream-state-source-batch')
receipt = json.loads((root / 'original-ca-evidence.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root / receipt['output_folder'] / 'CA/state-output.zip') as output:
    prepared = json.loads(output.read('CA/source-preparation.json'))
native = json.loads(Path('target/original-state-acquisition/CA/native-preparation.json').read_text(encoding='utf-8'))
graph = json.loads(prepared['graph_json'])
for key in ['schema_version', 'unit_kind', 'state', 'year', 'geoids', 'population', 'adjacency', 'areas']:
    assert graph[key] == native['graph'][key], key
assert json.loads(prepared['geometry_json']) == native['geometry']
assert [edge[:2] for edge in graph['edges']] == [edge[:2] for edge in native['graph']['edges']]
assert [(bridge['from'], bridge['to']) for bridge in prepared['bridges']] == [(bridge['from'], bridge['to']) for bridge in native['bridges']]
maximum_delta = 0
def compare(left, right):
    global maximum_delta
    if isinstance(left, list):
        assert len(left) == len(right)
        for a, b in zip(left, right):
            compare(a, b)
    else:
        delta = abs(left - right)
        assert delta <= 1e-5
        maximum_delta = max(maximum_delta, delta)
for key in ['centroids', 'exterior_perimeters']:
    compare(graph[key], native['graph'][key])
compare([edge[2] for edge in graph['edges']], [edge[2] for edge in native['graph']['edges']])
compare([bridge['cost'] for bridge in prepared['bridges']], [bridge['cost'] for bridge in native['bridges']])
assert prepared['population'] == '39538223' and prepared['units'] == 9129
Path('target/original-state-acquisition/CA/preparation-comparison.json').write_text(json.dumps({'units':prepared['units'], 'population':prepared['population'], 'maximum_projected_delta_m':maximum_delta, 'native_source_graph_correspondence':True, 'display_geometry_exact':True}, indent=2), encoding='utf-8')
print(f'Original CA native source preparation: every GEOID/population/edge/area/display coordinate matches; maximum projected delta {maximum_delta} m passed.')

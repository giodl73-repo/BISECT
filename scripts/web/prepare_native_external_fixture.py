from pathlib import Path
import json, hashlib
def hashed(value): return 'sha256:' + hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
ids = ['unit-a', 'unit-b']; population = 2**54 + 1
instance = dict(schema_version='certified-recursive-bisection-split-instance-v1', model_id='certified-standard-bisect-split-v1', node_path='', parent_certificate_id=None, unit_universe_hash=hashed(dict(unit_ids=ids)), unit_ids=ids, populations=[population,0], edges=[dict(left=0,right=1,weight=1)], k_parent=2,k_left=1,k_right=1,orientation_rule='equal-seats-unit-zero-left')
discovery = dict(schema_version='certified-split-discovery-v1', instance_hash=hashed(instance), solver_name='two-unit contract fixture', solver_version=None, method='unique feasible canonical two-unit split', objective=dict(primary=dict(max_population_deviation_scaled=population,total_population_deviation_scaled=2*population,weighted_boundary_cut=1),canonical_assignment=[0,1]))
discovery['discovery_id'] = hashed(discovery)
root=Path('target/native-external-certificate'); root.mkdir(parents=True,exist_ok=True)
(root/'large-instance.json').write_text(json.dumps(instance,indent=2),encoding='utf-8')
(root/'large-discovery.json').write_text(json.dumps(discovery,indent=2),encoding='utf-8')

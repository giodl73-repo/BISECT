"""Independent canonical projection for the supplied native RCTX test fixtures."""
import json,hashlib
from pathlib import Path
out=Path('target/context-engine-input');out.mkdir(parents=True,exist_ok=True)
c=json.loads(json.loads(Path('target/native-zero-boundaries/all-zero-export.json').read_text(encoding='utf-8'))['context_json'])
def rehash(c):
 p={k:c.get(k) for k in ['units','graph','populations','source_hashes']}
 for k in ['subdivisions','demographics','geometry']:
  if c.get(k) is not None:p[k]=c[k]
 return 'sha256:'+hashlib.sha256(json.dumps(p,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
assert rehash(c)==c['context_hash']
mutations={'missing-graph':lambda c:c.update(graph=None),'missing-population':lambda c:c.update(populations=None),'missing-weight':lambda c:c['graph']['adjacency'][0][0].pop('weight'),'reverse-weight':lambda c:c['graph']['adjacency'][0][0].update(weight=1.0),'reverse-kind':lambda c:c['graph']['adjacency'][0][0].update(kind='bridge'),'wrong-unit-hash':lambda c:c['units'].update(unit_universe_hash='sha256:'+'0'*64),'wrong-census-county':lambda c:c['subdivisions']['county_ids'].__setitem__(0,'wrong')}
for name,mutate in mutations.items():
 changed=json.loads(json.dumps(c));mutate(changed);changed['context_hash']=rehash(changed);(out/(name+'.rctx')).write_text(json.dumps(changed,ensure_ascii=False),encoding='utf-8')
for kind in ['boundary','point-touch','bridge','ferry','water','custom']:
 changed=json.loads(json.dumps(c))
 for row in changed['graph']['adjacency']:
  for edge in row:edge['kind']=kind
 changed['context_hash']=rehash(changed);(out/(kind+'.rctx')).write_text(json.dumps(changed,ensure_ascii=False),encoding='utf-8')
custom=json.loads(json.loads(Path('target/native-custom-units/precinct-2020-export.json').read_text(encoding='utf-8'))['context_json']);assert rehash(custom)==custom['context_hash'];custom['subdivisions']['county_ids'][0]=None;custom['context_hash']=rehash(custom);(out/'missing-custom-county.rctx').write_text(json.dumps(custom,ensure_ascii=False),encoding='utf-8')
print('Independent native RCTX hash projection matched; rehashed edge/metadata/refusal fixtures prepared.')

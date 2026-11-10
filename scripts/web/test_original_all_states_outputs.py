"""Independent exact-population/graph-contiguity verification of original-state outputs."""
import argparse, gzip, hashlib, json, math
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--states',default='');args=parser.parse_args()
root=Path('target/original-all-state-wasm');manifest=json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
states=args.states.split(',') if args.states else sorted(s for s in manifest['states'] if s!='DC')
assert len(states)==len(set(states)) and all(s in manifest['states'] and s!='DC' for s in states)
if not args.states: assert len(states)==50 and sum(manifest['states'][s]['districts']['2020'] for s in states)==435
rows=[];districts_checked=0
for state in states:
 folder=root/state;receipt=json.loads((folder/'receipt.json').read_text(encoding='utf-8'));assert receipt['state']==state and receipt['status']=='verified',state
 assert receipt['native_source_graph_parity'] and receipt['native_full_run_workflow_parity'] and receipt['native_assignment_parity']
 graph_bytes=gzip.decompress((folder/'engine-graph.json.gz').read_bytes());geometry_bytes=gzip.decompress((folder/'display-geometry.geojson.gz').read_bytes())
 assert len(graph_bytes)==receipt['graph_bytes'] and hashlib.sha256(graph_bytes).hexdigest()==receipt['graph_sha256']
 assert len(geometry_bytes)==receipt['geometry_bytes'] and hashlib.sha256(geometry_bytes).hexdigest()==receipt['geometry_sha256']
 graph=json.loads(graph_bytes);assignments=json.loads((folder/'assignments.json').read_text(encoding='utf-8'));ids=graph['geoids'];weights=graph['population'];adjacency=graph['adjacency'];k=receipt['districts'];fips=manifest['states'][state]['fips']
 assert graph['state']==state and graph['year']=='2020' and graph['unit_kind']=='tract'
 assert len(ids)==len(set(ids))==len(weights)==len(adjacency)==receipt['units']
 assert all(len(s)==11 and s.isdigit() and s.startswith(fips) for s in ids)
 assert set(assignments)==set(ids) and set(assignments.values())==set(range(1,k+1))
 assert all(type(p) is int and p>=0 for p in weights)
 population=sum(weights);assert str(population)==receipt['population'] and population==receipt['metrics']['population']
 indices={s:i for i,s in enumerate(ids)};populations=[];maximum=0
 for district in range(1,k+1):
  members={indices[s] for s,d in assignments.items() if d==district};assert members
  visited=set();pending=[next(iter(members))]
  while pending:
   vertex=pending.pop()
   if vertex in visited:continue
   visited.add(vertex);pending.extend(n for n in adjacency[vertex] if n in members and n not in visited)
  assert visited==members,(state,district,'disconnected induced district')
  count=sum(weights[i] for i in members);deviation=abs(count*k-population)*100/population;maximum=max(maximum,deviation)
  assert deviation<=0.5+1e-10,(state,district,'0.5-percent population miss',deviation)
  native=receipt['metrics']['district_metrics'][district-1];assert native['district']==district and native['population']==count and native['units']==len(members) and native['components']==1
  populations.append(count);districts_checked+=1
 assert math.isclose(maximum,receipt['metrics']['max_deviation_percent'],abs_tol=1e-10)
 rows.append({'state':state,'units':len(ids),'population':population,'districts':k,'max_deviation_percent':maximum,'compressed_input_bytes':sum((folder/n).stat().st_size for n in ['engine-graph.json.gz','display-geometry.geojson.gz'])})
report={'states':len(rows),'districts_checked':districts_checked,'compressed_input_bytes':sum(r['compressed_input_bytes'] for r in rows),'checks':'Independent Python integer population sums, 0.5-percent targets, complete IDs/nonempty districts, induced graph BFS, native per-district metrics and exact gzip/derived source SHA-256. This checks the supplied native bridge graph, not independent geographic connectivity or source authenticity.','rows':rows}
(root/('independent-'+('-'.join(states) if args.states else 'all-50')+'.json')).write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(f'Independent original-state verification: {len(rows)} full states, {districts_checked} districts, exact populations and connected induced graph districts; {report["compressed_input_bytes"]} compressed input bytes.')

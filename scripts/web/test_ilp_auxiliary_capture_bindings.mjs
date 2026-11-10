import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLaboratoryIlpReports} from '../../web/lab/laboratory-ilp-reports.js';

const graph=JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph;
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const options={structure:'ilp',search:'single',districts:2,seed:42,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,ilp_max_tracts:0,capture_ilp_reports:true};
const raw={c000:0,cns01:0,cns02:0,cns05:0,cns07:0,cns08:0,cns09:0,cns10:0,cns11:0};
let cases=0,refusals=0;
for(const weights of ['vra-aligned','economic-character','housing-character']){
 const request={graph,options:{...options,weights}},config={...request.options};
 if(weights==='vra-aligned'){
  request.demographics={schema_version:'bisect-demographic-fractions-v1',state:graph.state,year:graph.year,basis:'total-population',source_label:'SYNTHETIC ILP minority source',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,.5]))};
  config.demographics={[graph.state]:request.demographics};
 }else{
  const kind=weights.split('-')[0];request.options.character_alpha=.5;config.character_alpha=.5;
  request.character={schema_version:'bisect-character-v1',state:graph.state,year:graph.year,data_year:'2020',source_label:'SYNTHETIC ILP character source',data:{kind,...(kind==='economic'?{raw_counts:Object.fromEntries(graph.geoids.map(id=>[id,{...raw}]))}:{}),characters:Object.fromEntries(graph.geoids.map(id=>[id,kind==='economic'?{commercial_intensity:0,industrial_fraction:0,jobs_per_resident:0}:{pct_single_family:.8,pct_multifamily:.2,pct_owner:.6,housing_vintage:.3}]))}};
  config.characters={[graph.state]:request.character};
 }
 const result=engine.execute(request),state={metrics:{ilp_report_capture:result.ilp_report_capture,recorded_options:result.options}};
 await verifyLaboratoryIlpReports(graph,state,result.assignments,config);
 for(const key of ['partisan','demographics','character','elections']){
  const changed=structuredClone(state),packet=changed.metrics.ilp_report_capture,source=JSON.parse(packet.source_json);
  if(source[key])source[key].source_label='changed source';else source[key]={injected:'unrequested source'};
  packet.source_json=JSON.stringify(source);packet.source_sha256=createHash('sha256').update(packet.source_json).digest('hex');
  await assert.rejects(()=>verifyLaboratoryIlpReports(graph,changed,result.assignments,config),/disagree with the returned run/);refusals++;
 }
 cases++;
}
await fs.mkdir('target/ilp-auxiliary-capture-bindings',{recursive:true});await fs.writeFile('target/ilp-auxiliary-capture-bindings/evidence.json',JSON.stringify({legitimate_vra_and_character_packets:cases,rehashed_changed_or_injected_auxiliary_refusals:refusals,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`ILP capture auxiliary bindings: ${cases} legitimate minority/character packets and ${refusals} rehashed changed/injected auxiliary refusals.`);

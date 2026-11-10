import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyElectionResult} from '../../web/lab/election-input.js';

const graph=JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph;
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const elections={schema_version:'bisect-election-counts-v1',state:graph.state,year:graph.year,election_year:'2016',source_label:'SYNTHETIC root binding votes',counts:Object.fromEntries(graph.geoids.map(id=>[id,{democratic:50,two_party:100}]))};
const base={structure:'proportional-section',search:'multi',districts:2,seed:42,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,proportional_eta:1.5};
const raw={c000:0,cns01:0,cns02:0,cns05:0,cns07:0,cns08:0,cns09:0,cns10:0,cns11:0};let cases=0,refusals=0;
for(const weights of ['geographic','unweighted','county','partisan','vra-aligned','economic-character','housing-character']){
 const request={graph,elections,options:{...base,weights,alpha_county:weights==='county'?2:0},coi:{source_name:'SYNTHETIC COI.json',weights_json:JSON.stringify({[graph.geoids[0]]:9,[graph.geoids[1]]:0,'unknown-parity-unit':2})}};
 const config={...request.options,coi:request.coi};
 if(weights==='partisan'){request.options.dem_threshold=.55;request.options.rep_threshold=.45;config.dem_threshold=.55;config.rep_threshold=.45;request.partisan={schema_version:'bisect-partisan-shares-v1',state:graph.state,year:graph.year,source_label:'SYNTHETIC shares',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%4<2?.8:.2]))};config.partisans={[graph.state]:request.partisan};}
 if(weights==='vra-aligned'){request.demographics={schema_version:'bisect-demographic-fractions-v1',state:graph.state,year:graph.year,basis:'total-population',source_label:'SYNTHETIC minority fractions',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,i<8?.8:.2]))};config.demographics={[graph.state]:request.demographics};}
 if(weights.endsWith('-character')){const kind=weights.split('-')[0];request.options.character_alpha=.5;config.character_alpha=.5;request.character={schema_version:'bisect-character-v1',state:graph.state,year:graph.year,data_year:'2020',source_label:'SYNTHETIC observations',data:{kind,...(kind==='economic'?{raw_counts:Object.fromEntries(graph.geoids.map(id=>[id,{...raw}]))}:{}),characters:Object.fromEntries(graph.geoids.map(id=>[id,kind==='economic'?{commercial_intensity:0,industrial_fraction:0,jobs_per_resident:0}:{pct_single_family:.8,pct_multifamily:.2,pct_owner:.6,housing_vintage:.3}]))}};config.characters={[graph.state]:request.character};}
 const result=engine.execute(request);await verifyElectionResult(graph,config,elections,result);
 const changed=structuredClone(result);changed.metrics.structure_evidence.root_cut++;await assert.rejects(()=>verifyElectionResult(graph,config,elections,changed));refusals++;
 cases++;
}
await fs.mkdir('target/proportional-root-weight-bindings',{recursive:true});await fs.writeFile('target/proportional-root-weight-bindings/evidence.json',JSON.stringify({all_weight_families_with_coi:cases,altered_root_cost_refusals:refusals,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Proportional root bindings: ${cases} native-computed weight/COI families and ${refusals} altered root-cost refusals.`);

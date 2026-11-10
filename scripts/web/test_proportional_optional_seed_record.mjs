import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyElectionResult} from '../../web/lab/election-input.js';

const graph=JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph;
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const elections={schema_version:'bisect-election-counts-v1',state:graph.state,year:graph.year,election_year:'2016',source_label:'SYNTHETIC optional-seed regression',counts:Object.fromEntries(graph.geoids.map(id=>[id,{democratic:50,two_party:100}]))};
const base={structure:'proportional-bisect',weights:'unweighted',search:'single',districts:2,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
let cases=0,refusals=0;
for(const seed of [null,0,'18446744073709551615']){
 const options={...base,seed},result=engine.execute({graph,options,elections});assert.equal(result.metrics.structure_evidence.requested_seed,seed);
 await verifyElectionResult(graph,options,elections,result);
 const changed=structuredClone(result);changed.metrics.structure_evidence.requested_seed=seed===null?0:null;await assert.rejects(()=>verifyElectionResult(graph,options,elections,changed));refusals++;
 const old=structuredClone(result);delete old.metrics.structure_evidence.requested_seed;await verifyElectionResult(graph,options,elections,old);cases++;
}
await fs.mkdir('target/proportional-optional-seed-record',{recursive:true});await fs.writeFile('target/proportional-optional-seed-record/evidence.json',JSON.stringify({exact_optional_seed_cases:cases,changed_optional_seed_refusals:refusals,historical_field_omission_still_imports:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log('Proportional optional seed: None, explicit zero and maximum u64 remain distinct; altered records refused, historical missing-field claims retained.');

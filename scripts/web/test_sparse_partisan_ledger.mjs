import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/config-run-wasm/case-0.json','utf8')).input.graph;
const partisan={schema_version:'bisect-partisan-shares-v1',state:graph.state,year:graph.year,source_label:'Synthetic sparse partisan ledger fixture',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,[.8,.5,.2,.5][i%4]]))};
const options={structure:'standard-bisect',weights:'partisan',search:'multi',districts:4,seed:42,seeds:3,steps:20,percentile:.5,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:100,dem_threshold:.55,rep_threshold:.45};
const result=engine.execute({graph,options,partisan}),config={...options,name:'Sparse partisan ledger',mode:'state',states:[graph.state],year:graph.year,chamber:'congressional',timeout_seconds:30,partisans:{[graph.state]:partisan}},state={code:graph.state,metrics:{...result.metrics,district_count:4,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
verifyLabAssignments(graph,state,result.assignments,config);await verifyDemographicEvidence(graph,state,config);
for(const key of ['dispatch_representation','native_candidate_objective','reported_weighted_boundary']){const bad=structuredClone(state);bad.metrics.weighting_evidence[key]='changed';assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,config),/dispatch evidence/);}
const legacy=structuredClone(state);for(const key of ['dispatch_representation','native_candidate_objective','reported_weighted_boundary'])delete legacy.metrics.weighting_evidence[key];verifyLabAssignments(graph,legacy,result.assignments,config);await verifyDemographicEvidence(graph,legacy,config);
await fs.mkdir('target/sparse-partisan-ledger',{recursive:true});await fs.writeFile('target/sparse-partisan-ledger/evidence.json',JSON.stringify({actual_sparse_output_checked:true,three_dispatch_tag_tampers_refused:true,legacy_format_arithmetic_retained:true,legacy_algorithm_replay_not_claimed:true},null,2));console.log('Partisan sparse ledger: actual output, three dispatch tag refusals and legacy-format arithmetic checks passed.');

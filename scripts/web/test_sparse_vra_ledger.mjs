import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const request=JSON.parse(await fs.readFile('target/config-vra-oracle/case-40.json.tool.json','utf8')),out=engine.execute(request),graph=request.input.graph,input=request.input.demographics;
const config={...out.engine_request.options,name:'Sparse VRA ledger fixture',mode:'state',states:[graph.state],year:graph.year,chamber:'congressional',timeout_seconds:30,demographics:{[graph.state]:input}};
const state={code:graph.state,metrics:{...out.result.metrics,district_count:out.engine_request.options.districts,districts:out.result.metrics.district_metrics,balance_passed:out.result.metrics.within_requested_tolerance}};
verifyLabAssignments(graph,state,out.result.assignments,config);await verifyDemographicEvidence(graph,state,config,out.result.assignments);
const tampered=structuredClone(state);tampered.metrics.weighting_evidence.dispatch_representation='dense-unit-expansion';assert.throws(()=>verifyLabAssignments(graph,tampered,out.result.assignments,config),/dispatch evidence/);
// A recorded legacy-format arithmetic fixture; this does not claim legacy execution replay.
const legacy=structuredClone(state);for(const key of ['dispatch_representation','native_candidate_objective','reported_weighted_boundary'])delete legacy.metrics.weighting_evidence[key];
const high=Object.values(input.minority_fractions).filter(f=>f>=.4).length,alpha=Math.max(3,10*(1-.7*high/graph.geoids.length)),leftSeats=legacy.metrics.root_split.left_districts;
let cut=0;for(const [u,v]of [...graph.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1]))if((out.result.assignments[graph.geoids[u]]<=leftSeats)!==(out.result.assignments[graph.geoids[v]]<=leftSeats))cut+=input.minority_fractions[graph.geoids[u]]>=.4&&input.minority_fractions[graph.geoids[v]]>=.4?alpha:1;
legacy.metrics.root_split.weighted_boundary=cut;const normalised=cut/Math.sqrt(Math.min(leftSeats,config.districts-leftSeats));legacy.metrics.structure_evidence.normalised_cut=normalised;legacy.metrics.structure_evidence.selection_score=normalised-config.w_vra*legacy.metrics.structure_evidence.alignment*Math.max(normalised,1);
verifyLabAssignments(graph,legacy,out.result.assignments,config);await verifyDemographicEvidence(graph,legacy,config,out.result.assignments);
await fs.mkdir('target/sparse-vra-ledger',{recursive:true});await fs.writeFile('target/sparse-vra-ledger/evidence.json',JSON.stringify({current_native_sparse_ledger:true,dispatch_tag_tamper_refused:true,legacy_dense_format_arithmetic_retained:true,legacy_execution_replay_not_claimed:true},null,2));
console.log('Sparse VRA ledger: current actual output, dispatch tamper refusal and legacy dense-format arithmetic checks passed; no legacy execution replay claimed.');

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const root='target/minority-weights-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const size=6,n=36,adjacency=[],edges=[];
for(let i=0;i<n;i++){const ns=[];if(i%size)ns.push(i-1);if(i%size<size-1)ns.push(i+1);if(i>=size)ns.push(i-size);if(i<n-size)ns.push(i+size);adjacency.push(ns);for(const j of ns)if(i<j)edges.push([i,j,50+(i+j)%7]);}
const graph={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:n},(_,i)=>'19001'+String(i).padStart(6,'0')),adjacency,population:Array(n).fill(100),edges,areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-93+i%size*.01,42+Math.floor(i/size)*.01])};
const options={structure:'standard-bisect',weights:'vra-aligned',search:'single',districts:4,seed:42,seeds:4,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:20};
const native=(exe,path)=>JSON.parse(execFileSync(exe,[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
const records=[];let exportChecked=false;
for(const profile of ['zero','all','stripe','threshold'])for(const structure of ['standard-bisect','ratio-optimal'])for(const search of ['single','multi']){
 const demographics={schema_version:'bisect-demographic-fractions-v1',state:'IA',year:'2020',basis:'total-population',source_label:'Synthetic minority boundary regression',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,profile==='zero'?0:profile==='all'?1:profile==='stripe'?(i%size<3?.8:.1):[.399999,.4,.400001][i%3]]))};
 const request={graph,options:{...options,structure,search},demographics},path=`${root}/case-${records.length}.json`;await fs.writeFile(path,JSON.stringify(request));
 const result=engine.execute(request);assert.deepEqual(result,native('target/release/examples/execute_request.exe',path));
 const weights=native('target/debug/examples/minority_weights_reference.exe',path);
 const yaml_text=`name: minority_reference\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  weights: vra-aligned\n  search: ${search}\n  seeds: ${request.options.seeds}\n  seed: ${request.options.seed}\n  balance_tolerance: ${request.options.balance_tolerance}\n`;
 const oraclePath=path+'.cli.json';await fs.writeFile(oraclePath,JSON.stringify({graph,demographics,yaml_text,options:{districts:request.options.districts,iterations:request.options.iterations}}));
 const oracle=native('target/debug/examples/config_seed_reference.exe',oraclePath);assert.deepEqual(result.assignments,oracle.assignments);
 const cut=e=>e.filter(([u,v])=>result.assignments[graph.geoids[u]]!==result.assignments[graph.geoids[v]]).reduce((sum,edge)=>sum+edge[2],0);
 assert.ok(Math.abs(result.metrics.weighted_boundary-cut(weights))<1e-9);assert.equal(result.metrics.graph_boundary_m,cut(edges));
 const scaled=engine.execute({...request,graph:{...graph,edges:edges.map(([u,v,w])=>[u,v,w*123])}});assert.deepEqual(scaled.assignments,result.assignments);assert.equal(scaled.metrics.weighted_boundary,result.metrics.weighted_boundary);
 const config={...request.options,demographics:{IA:demographics},name:'Minority boundaries',mode:'state',states:['IA'],year:'2020',chamber:'congressional',timeout_seconds:30};validateLabConfig(config);
 const state={code:'IA',metrics:{...result.metrics,district_count:4,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};
 verifyLabAssignments(graph,state,result.assignments,config);await verifyDemographicEvidence(graph,state,config,result.assignments);
 const bad=structuredClone(state);bad.metrics.weighting_evidence.alpha+=.1;assert.throws(()=>verifyLabAssignments(graph,bad,result.assignments,config),/minority weighting/);
 const changed=structuredClone(config);changed.demographics.IA.minority_fractions[graph.geoids[0]]=demographics.minority_fractions[graph.geoids[0]]===1?.999:.001;await assert.rejects(verifyDemographicEvidence(graph,state,changed,result.assignments),/identity mismatch/);
 if(!exportChecked){
  const exportRequest={operation:'export-engine-plan',request,assignments:result.assignments,label:'Minority weighted synthetic plan',chamber:'congressional',created_at:'2026-10-07T00:00:00Z'},exportPath=root+'/export.json';await fs.writeFile(exportPath,JSON.stringify(exportRequest));
  const exported=engine.execute(exportRequest);assert.deepEqual(exported,native('target/release/examples/execute_tool_request.exe',exportPath));
  assert.ok(exported.context.source_hashes['bisect.demographic-input']);assert.equal(exported.demographic_basis,'total-population');exportChecked=true;
 }
 records.push({profile,structure,search,evidence:result.metrics.weighting_evidence,weighted_cut:result.metrics.weighted_boundary});
}
const valid=JSON.parse(await fs.readFile(root+'/case-0.json','utf8'));let rejected=0;
for(const change of [r=>delete r.demographics,r=>r.demographics.basis='voting-age-population',r=>r.demographics.year='2010',r=>delete r.demographics.minority_fractions[graph.geoids[0]],r=>r.demographics.minority_fractions[graph.geoids[0]]=-.1,r=>r.options.alpha_county=1]){
 const request=structuredClone(valid);change(request);const path=`${root}/reject-${rejected}.json`;await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>engine.execute(request));assert.throws(()=>native('target/release/examples/execute_request.exe',path));rejected++;
}
assert.ok(engine.execute(valid).assignments);
await fs.writeFile(root+'/evidence.json',JSON.stringify({records,rejected,exportChecked},null,2));
console.log(`Minority boundary weights: ${records.length} native/WASM and native override-oracle assignment/cut matches, geographic-length independence, threshold boundaries, project evidence/tamper checks, native plan export and ${rejected} rejections passed.`);

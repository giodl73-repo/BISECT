import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {ilpReportAuditProject} from '../../web/lab/ilp-report-handoff.js';

const root='target/ilp-size-skip-reports';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json','utf8')).graph;
const options={structure:'ilp',weights:'geographic',search:'single',districts:2,seed:42,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,ilp_method:'formulation-only',ilp_fallback:'metis',ilp_max_tracts:0,capture_ilp_reports:true};
const request={graph,options}, actual=engine.execute(request),packet=actual.ilp_report_capture;
assert.equal(packet.nodes.length,1);assert.equal(packet.nodes[0].stage,'size-guard-skipped');assert.deepEqual(packet.files,{});
await assert.rejects(()=>ilpReportAuditProject(packet),/no solver artifacts/);
const nativeRun=async(name,reportDir,districts=2,maxTracts=0,tolerance=10)=>{
 const file=root+'/'+name+'.json';await fs.writeFile(file,JSON.stringify({graph,districts,report_dir:reportDir,state_args:['--state',graph.state,'--year',graph.year,'--partition-mode','ilp','--districts',String(districts),'--balance-tolerance',String(tolerance),'--ilp-method','formulation-only','--ilp-fallback','metis','--ilp-max-tracts',String(maxTracts),'--metis-engine','metis-core']}));
 const native=spawnSync('target/release/examples/ilp_state_reference.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.error,undefined);assert.equal(native.status,0,native.stderr);return JSON.parse(native.stdout);
};
const fresh=path.resolve(root,'fresh-'+Date.now());assert.deepEqual(await nativeRun('fresh',fresh),actual.assignments);
await assert.rejects(()=>fs.stat(fresh),e=>e.code==='ENOENT');
const stale=path.resolve(root,'stale-'+Date.now());await fs.mkdir(stale+'/depth_00',{recursive:true});await fs.writeFile(stale+'/depth_00/node_root.json','prior invocation report');await fs.writeFile(stale+'/audit-summary.json','prior invocation summary');
assert.deepEqual(await nativeRun('stale',stale),actual.assignments);assert.equal(await fs.readFile(stale+'/depth_00/node_root.json','utf8'),'prior invocation report');assert.equal(await fs.readFile(stale+'/audit-summary.json','utf8'),'prior invocation summary');
const mixed=engine.execute({graph,options:{...options,districts:4,ilp_max_tracts:8,balance_tolerance:20}}),mixedDir=path.resolve(root,'mixed-'+Date.now());
await fs.mkdir(mixedDir+'/depth_00',{recursive:true});await fs.writeFile(mixedDir+'/depth_00/node_root.json','old skipped root');await fs.writeFile(mixedDir+'/node_old.json','old unrelated report');
assert.deepEqual(await nativeRun('mixed',mixedDir,4,8,20),mixed.assignments);const mp=mixed.ilp_report_capture;
assert.equal(mp.nodes.filter(n=>n.stage==='size-guard-skipped').length,1);assert.equal(mp.nodes.filter(n=>n.stage==='native-solver-attempt').length,2);
const normalize=text=>{const r=JSON.parse(text);r.result.solve_time_secs=0;return r;};
for(const [name,text]of Object.entries(mp.files)){const native=await fs.readFile(mixedDir+'/'+name,'utf8');if(name.endsWith('.lp'))assert.equal(native,text);else assert.deepEqual(normalize(native),normalize(text));}
const scope=JSON.parse(await fs.readFile(mixedDir+'/audit-summary.json','utf8'));assert.deepEqual(scope.report_selection,['depth_01/node_0.json','depth_01/node_1.json']);assert.equal(scope.checked,2);assert.equal(await fs.readFile(mixedDir+'/depth_00/node_root.json','utf8'),'old skipped root');assert.equal(await fs.readFile(mixedDir+'/node_old.json','utf8'),'old unrelated report');
const audit=await ilpReportAuditProject(mp);assert.equal(audit.result,null);assert.equal(Object.keys(audit.files.ilpAuditInputs.reports).length,2);
await fs.writeFile(root+'/evidence.json',JSON.stringify({all_skip_native_missing_directory_matches:true,stale_files_preserved_not_audited:true,empty_packet_audit_refused:true,mixed_skip_two_native_reports_match:true,only_measured_solve_time_excluded:true,inert_mixed_audit_handoff:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log('ILP size skips: native missing/stale report directories preserve fallback plans; empty audit refused; mixed skips retain two exact native report/model pairs and inert audit handoff.');

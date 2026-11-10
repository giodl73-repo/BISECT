import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const repo=path.resolve(process.argv[3]??'.'),{instantiateEngine}=await import(pathToFileURL(path.join(repo,'web/lab/wasm-engine.js'))),{stateBatchRequest}=await import(pathToFileURL(path.join(repo,'web/lab/state-batch-files.js')));
const root=path.resolve(process.argv[2]),out=path.join(root,'wasm-decision-comparison');await fs.mkdir(out,{recursive:true});const engine=await instantiateEngine(await fs.readFile(path.join(root,'owned/engine.wasm'))),status=JSON.parse(await fs.readFile(path.join(root,'status.json'),'utf8')),manifest=await fs.readFile(path.join(root,'inputs/manifest.json'),'utf8');
const source=text=>({name:'retained-native-snapshot.json',base64:Buffer.from(text).toString('base64')}),sha=b=>createHash('sha256').update(b).digest('hex');const rows=[];let operations=0;await fs.copyFile(path.join(root,'status.json'),path.join(out,'native-status-original.json')); // Preserve raw nanosecond integers; never reserialize them through JavaScript numbers.
for(const row of status.cases){
 const paths=Object.keys(row.before),manifests={};for(const name of paths.filter(n=>n.endsWith('/manifest.json')))manifests[name]=await fs.readFile(path.join(root,row.name,'before',name),'utf8');
 const records=[];let shouldExecute=false,refusal=null;const years=row.year==='all'?['2020','2010','2000']:[row.year];
 for(const year of years){
  let scheduled=true;
  if(row.family!=='state'){
   const request=stateBatchRequest({year,states:[row.states],manifest:source(manifest),completion:{source:source(JSON.stringify({paths})),reprocess:row.extra.includes('--reprocess')}}),reply=engine.execute(request);operations++;records.push({request,reply});scheduled=reply.states.length>0;
  }
  if(scheduled){
   const request={operation:row.family==='state'?'inspect-native-run-target':'inspect-native-state-run-target',version:'vBranches',year,...(row.family==='state'?{label:'Workspace_RI'}:{state:'RI'}),input:{source_name:'retained-native-before.json',folder_json:JSON.stringify({version:'vBranches',paths,manifests}),force:row.extra.includes('--force')}};
   let reply=null,error=null;try{reply=engine.execute(request);}catch(failure){error=failure.message;}operations++;records.push({request,reply,error});
   if(error){refusal=error;assert.match(error,row.name.includes('incomplete')?/Incomplete plan/:/already exists/);}else{assert.equal(reply.status,'ready');shouldExecute=true;}
  }
 }
 assert.equal(shouldExecute,row.execution_observed,row.name);assert.equal(refusal!==null,row.exit_code!==0,row.name);
 await fs.writeFile(path.join(out,row.name+'.json'),JSON.stringify({case:row.name,native_case:{name:row.name,family:row.family,arguments:row.arguments,exit_code:row.exit_code,execution_observed:row.execution_observed,before_file_sha256:Object.fromEntries(Object.entries(row.before).map(([name,file])=>[name,file.sha256]))},records,decisions:{execution:shouldExecute,refusal}},null,2));
 rows.push({case:row.name,exact_scheduling_and_guard_decision:true,native_execution:row.execution_observed,wasm_execution_allowed:shouldExecute,native_exit_code:row.exit_code,wasm_refusal:refusal,records_sha256:sha(await fs.readFile(path.join(out,row.name+'.json')))});
}
assert.equal(rows.length,18);await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed',scope:'WASM completion selection and target guards consume exact retained native pre-run path/manifest snapshots for 18 actual CLI scenarios. Decision equivalence only; raw paths/errors/artifact identities preserved, no claim of raw-message, browser root-grant/write/recovery equivalence.',cases:rows,wasm_operations:operations,wasm:engine.artifactIdentity,native_status_sha256:sha(await fs.readFile(path.join(root,'status.json'))),script_sha256:sha(await fs.readFile(new URL(import.meta.url)))},null,2));console.log(JSON.stringify({status:'passed',actual_native_cases:18,wasm_operations:operations}));

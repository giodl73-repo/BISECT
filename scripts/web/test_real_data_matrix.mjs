// Explicit integration tier; never a prerequisite for a portable clean checkout.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import path from 'node:path';
import {spawn,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {digest,realDataMatrixRequest} from './fixtures/real-data-matrix-case.mjs';
const manifestPath='docs/specs/wasm-real-data-matrix.json',manifestBytes=await fs.readFile(manifestPath),manifest=JSON.parse(manifestBytes);
assert.equal(manifest.schema_version,'bisect-real-data-matrix-v1');
const args=process.argv.slice(2);assert(args.every(arg=>['--rows=','--row=','--root=','--wasm=','--host='].some(prefix=>arg.startsWith(prefix))));
const option=name=>args.find(arg=>arg.startsWith('--'+name+'='))?.split('=').slice(1).join('=');
const hashFile=async filename=>{const hash=createHash('sha256');for await(const chunk of createReadStream(filename))hash.update(chunk);return hash.digest('hex');};
async function checkFile(pin){const stat=await fs.stat(pin.path);assert.equal(stat.size,pin.byte_length,pin.path+' byte length');assert.equal(await hashFile(pin.path),pin.sha256,pin.path+' changed');}
for(const pin of manifest.recipe_sources)await checkFile(pin);
const selected=option('rows')?.split(',')||manifest.rows.map(row=>row.state);assert(selected.length&&new Set(selected).size===selected.length&&selected.every(state=>manifest.rows.some(row=>row.state===state)),'Unknown/duplicate state selection');
if(option('row')){
 const row=manifest.rows.find(row=>row.id===option('row'));assert(row&&option('root')&&option('wasm')&&option('host'));
 const root=path.join(option('root'),row.state);await fs.mkdir(root,{recursive:true});
 const outcome={id:row.id,state:row.state,status:'running',manifest_sha256:digest(manifestBytes),attempted_execution:false,browser:'not-run',fresh_raw_derivation:'not-run',independent_cli_orchestration:'not-run',remaining_checks:row.remaining_checks};
 const record=async()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(outcome,null,2)+'\n');await record();
 try{
  console.log(row.state+' verifying immutable source prerequisites');
  try{for(const pin of [...row.sources,row.graph,row.derivation_witness])await checkFile(pin);}catch(error){if(error.code==='ENOENT'){outcome.status='not-run';outcome.reason='Missing declared integration prerequisite: '+error.path;await record();process.exitCode=2;}else throw error;}
  if(outcome.status!=='not-run'){
   const engine=await instantiateEngine(await fs.readFile(option('wasm'))),request=await realDataMatrixRequest(engine,row),requestText=JSON.stringify(request);
   assert.equal(digest(requestText),row.request_sha256,'Predeclared request changed');
   const requestFile=path.join(root,'request.json');await fs.writeFile(requestFile,requestText);outcome.request_sha256=digest(requestText);outcome.attempted_execution=true;
   console.log(row.state+' native adapter execution');const started=performance.now();let native,nativeError;
   try{const text=execFileSync(option('host'),[requestFile],{encoding:'utf8',maxBuffer:256*1024*1024,stdio:['ignore','pipe','pipe'],windowsHide:true});await fs.writeFile(path.join(root,'native-reply.json'),text);native=JSON.parse(text);}
   catch(error){if(error.status!==1)throw error;const stderr=String(error.stderr);await fs.writeFile(path.join(root,'native-stderr.txt'),stderr);nativeError=JSON.parse(stderr.trim().replace(/^Error: /,''));assert.equal(typeof nativeError,'string');}
   outcome.native_seconds=(performance.now()-started)/1000;
   console.log(row.state+' WASM execution');const wasmStarted=performance.now();let actual,actualError;
   try{actual=engine.execute(request);await fs.writeFile(path.join(root,'wasm-reply.json'),JSON.stringify(actual));}catch(error){actualError=error.message;await fs.writeFile(path.join(root,'wasm-error.json'),JSON.stringify({error:actualError}));}
   outcome.wasm_seconds=(performance.now()-wasmStarted)/1000;
   if(row.expected_native.status==='failed'){
    assert.equal(nativeError,row.expected_native.error,'Native outcome differs from fixed expectation');assert.equal(actualError,row.expected_native.error,'WASM failure differs');assert.equal(native,undefined);assert.equal(actual,undefined);outcome.status='matching-native-failure';outcome.error=actualError;outcome.successful_workload_support=false;
   }else{
    assert.equal(nativeError,undefined);assert.equal(actualError,undefined);assert.deepEqual(actual,native,'Complete reply comparison');assert.equal(actual.status,'completed');
    const graph=request.input.run.graph,assignments=actual.configured_run.result.assignments,keys=Object.keys(assignments);assert.deepEqual(keys.sort(),[...graph.geoids].sort());
    assert.equal(String(actual.configured_run.engine_request.options.seed),row.seed);
    const population=Array(row.districts).fill(0n),members=Array.from({length:row.districts},()=>[]);
    for(let index=0;index<graph.geoids.length;index++){const district=assignments[graph.geoids[index]];assert(Number.isInteger(district)&&district>=1&&district<=row.districts);population[district-1]+=BigInt(graph.population[index]);members[district-1].push(index);}
    assert.equal(population.reduce((a,b)=>a+b,0n).toString(),row.population);
    for(const vertices of members){assert(vertices.length);const district=assignments[graph.geoids[vertices[0]]],seen=new Set([vertices[0]]),pending=[vertices[0]];while(pending.length){for(const neighbor of graph.adjacency[pending.pop()])if(assignments[graph.geoids[neighbor]]===district&&!seen.has(neighbor)){seen.add(neighbor);pending.push(neighbor);}}assert.equal(seen.size,vertices.length,'District disconnected');}
    for(const count of population)assert(Math.abs(Number(count)*row.districts/Number(row.population)-1)*100<=row.balance_tolerance_percent);
    outcome.status='passed-native-adapter-tier';outcome.population=population.map(String);outcome.all_units_assigned=true;outcome.independent_population_and_contiguity=true;outcome.full_reply_equal=true;
   }
   outcome.wasm_sha256=engine.artifactIdentity.sha256;outcome.native_sha256=await hashFile(option('host'));outcome.source_identities_checked=true;await record();
  }
 }catch(error){outcome.status='failed';outcome.error=error.stack||error.message;await record();throw error;}
 console.log(row.state+' '+outcome.status);
}else{
 const root=path.resolve('target/wasm-real-data-matrix','run-'+Date.now());await fs.mkdir(path.join(root,'runtime'),{recursive:true});
 const wasm=option('wasm')||'dist/wasm-workspace-continuation-validated/toolkit/bisect_wasm.wasm',host=option('host')||'target/release/examples/execute_tool_request.exe';
 const ownedWasm=path.join(root,'runtime/engine.wasm'),ownedHost=path.join(root,'runtime/native.exe');await fs.copyFile(wasm,ownedWasm);await fs.copyFile(host,ownedHost);await fs.writeFile(path.join(root,'declared-matrix.json'),manifestBytes,{flag:'wx'});
 const summary={schema_version:'bisect-real-data-matrix-evidence-v1',manifest_sha256:digest(manifestBytes),wasm_sha256:await hashFile(ownedWasm),native_sha256:await hashFile(ownedHost),status:'running',rows:[],scope:manifest.scope};
 const write=async()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(summary,null,2)+'\n');await write();console.log('Matrix evidence '+root);
 for(const state of selected){const row=manifest.rows.find(row=>row.state===state);const code=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,[process.argv[1],'--row='+row.id,'--root='+root,'--wasm='+ownedWasm,'--host='+ownedHost],{stdio:'inherit',windowsHide:true});child.on('error',reject);child.on('exit',resolve);});
  let result;try{result=JSON.parse(await fs.readFile(path.join(root,state,'evidence.json'),'utf8'));}catch{result={id:row.id,status:'failed',error:'Child terminated without a readable outcome'};}
  summary.rows.push(result);if(code!==0&&result.status!=='not-run')summary.status='failed';await write();
 }
 summary.counts={attempts:summary.rows.filter(row=>row.attempted_execution).length,successful_executions:summary.rows.filter(row=>row.status==='passed-native-adapter-tier').length,matching_native_failures:summary.rows.filter(row=>row.status==='matching-native-failure').length,failures:summary.rows.filter(row=>row.status==='failed').length,not_run:summary.rows.filter(row=>row.status==='not-run').length,browser_executions:0};
 summary.status=summary.counts.failures?'failed':summary.counts.not_run?'incomplete':'passed-declared-native-adapter-tier';await write();console.log(JSON.stringify({evidence:path.join(root,'evidence.json'),status:summary.status,counts:summary.counts}));if(summary.status==='failed')process.exitCode=1;else if(summary.status==='incomplete')process.exitCode=2;
}

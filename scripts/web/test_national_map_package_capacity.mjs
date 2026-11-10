import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {readNationalMapBatchSource,nationalMapBatchInput,executeNationalMapBatch,MAX_NATIONAL_MAP_PACKAGE_BYTES} from '../../web/lab/national-map-batch.js';
import {parseNativeNationalMapJson} from '../../web/lab/project.js';
import {createStoredWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/national-map-package-capacity';await fs.mkdir(root,{recursive:true});
let read=false;
await assert.rejects(readNationalMapBatchSource({size:MAX_NATIONAL_MAP_PACKAGE_BYTES+1,arrayBuffer(){read=true;}}),/128 MiB/);
assert.equal(read,false,'Oversized packages must refuse before reading');
assert.equal(MAX_NATIONAL_MAP_PACKAGE_BYTES,128*1024*1024);
if(!process.argv.includes('--original-byte-capacity')){
 console.log('National package capacity: 128 MiB pre-read bound passed.');
}else{
 const states={},sha=b=>createHash('sha256').update(b).digest('hex'),source=(name,b)=>({name,base64:Buffer.from(b).toString('base64')});
 for(const state of ['AL','AZ']){
  const project=JSON.parse(await fs.readFile('target/original-state-algorithm/standard-bisect/'+state+'/workflow.bisect','utf8'));
  const proof=JSON.parse(await fs.readFile('target/original-state-algorithm/standard-bisect/'+state+'/evidence.json','utf8'));
  assert(proof.full_native_wasm_workflow_match&&proof.native_yaml_assignment_parity);
  const geometry=Buffer.from(project.files.planGeometry.base64,'base64');assert.equal(sha(geometry),proof.geometry_sha256);
  // Legal trailing JSON whitespace tests byte limits without inventing source
  // coordinates, units, assignments or a successful 50-state generation.
  const padded=Buffer.alloc(40*1024*1024,32);geometry.copy(padded);
  assert.deepEqual(JSON.parse(padded.toString('utf8')),JSON.parse(geometry.toString('utf8')));
  states[state]={plan:source(state+'.rplan',project.result.practitioner_export.document_json),geometry:source(state+'.geojson',padded),geometry_format:'json',analysis:null,dissolve_method:'batch'};
 }
 const batch={dpi:72,map_type:'districts',states},packageBytes=Buffer.from(JSON.stringify(batch));
 await fs.writeFile(root+'/package.json',packageBytes);
 assert(packageBytes.length>64*1024*1024&&packageBytes.length<MAX_NATIONAL_MAP_PACKAGE_BYTES);
 assert.throws(()=>parseNativeNationalMapJson(packageBytes.toString('utf8')),/64 MiB/,'The legacy joint-operation bound stays unchanged');
 const selected=await readNationalMapBatchSource({name:'large-national.json',size:packageBytes.length,arrayBuffer:async()=>packageBytes.buffer.slice(packageBytes.byteOffset,packageBytes.byteOffset+packageBytes.byteLength)});
 const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
 const requests=[],outcomes=[],recorded={execute(request){requests.push(request);try{const result=engine.execute(request);outcomes.push({ok:true,result});return result;}catch(error){outcomes.push({ok:false,error:error.message});throw error;}}};
 const result=await executeNationalMapBatch(recorded,nationalMapBatchInput(selected));assert.equal(result.units,3202);assert.equal(result.districts,16);
 const requestFile=root+'/requests.jsonl';await fs.writeFile(requestFile,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');
 const native=execFileSync('target/debug/examples/execute_tool_sequence.exe',[requestFile],{encoding:'utf8',windowsHide:true,maxBuffer:128*1024*1024}).trim().split('\n').map(r=>JSON.parse(r));assert.deepEqual(native,outcomes);
 const backup=await createStoredWorkbenchArchive({name:'Original AL/AZ byte capacity',files:{nationalMapBatchInputs:{source:selected}},operation:'render-native-national-map-batch',constraints:[],result,lastOperation:'render-native-national-map-batch'});
 await fs.writeFile(root+'/backup.bisect-workbench',backup);await fs.writeFile(root+'/result.json',JSON.stringify(result));
 assert(backup instanceof Uint8Array,'Large nested source package must use bounded compressed backup');
 const reopened=await parseWorkbenchArchiveBytes(backup);assert.deepEqual(reopened.files.nationalMapBatchInputs.source,selected);assert.deepEqual(reopened.result,result);
 const bomb={...batch,states:{AL:{...states.AL,geometry_format:'gzip',geometry:source('oversized.geojson.gz',gzipSync(Buffer.alloc(65*1024*1024,32)))}}};
 await assert.rejects(executeNationalMapBatch(engine,bomb),/exceeds 64 MiB/);
 const fresh=engine.execute({operation:'begin-native-national-map',input:{dpi:72,map_type:'districts'}});engine.execute({operation:'abort-native-national-map',session:fresh.session});
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,package_bytes:packageBytes.length,native_wasm_matches:native.length,compressed_backup_bytes:backup.length,exact_backup_reopen:true,gzip_expansion_refusal_and_cleanup:true,scope:'Two genuine original state plans and geometry with legal trailing JSON whitespace exercise package byte capacity. This does not establish 50-state geometry/position or generation capacity.'},null,2));
 console.log('National package byte capacity: original AL/AZ coordinates/assignments retained, >64 MiB package selected, exact native/WASM render, compressed backup reopen and bounded gzip cleanup passed.');
}

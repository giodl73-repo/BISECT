import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preparedEngineRequest,preparedEngineDownload,readPreparedEngineSource,validatePreparedEngineInputs,PREPARED_ENGINE_DEFAULTS} from '../../web/lab/prepared-engine-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/prepared-engine-guards';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/prepared-engine/geographic-3.bisect','utf8')),inputs=fixture.files.preparedEngineInputs,base=preparedEngineRequest(inputs),raw=JSON.parse(inputs.source.text);
let refusals=0;
for(const mutate of [
 r=>r.input.request_json='globalThis.preparedEnginePwned=1;',
 r=>r.input.request_json='{"operation":"audit-plan","document":{}}',
 r=>r.input.request_json=JSON.stringify({...raw,options:{...raw.options,unknown:true}}),
 r=>r.input.request_json=JSON.stringify({...raw,graph:{...raw.graph,geoids:raw.graph.geoids.map((id,i)=>i?id:'__proto__')}}),
 r=>r.input.request_json=JSON.stringify({...raw,graph:{...raw.graph,population:[1]}}),
 r=>r.input.request_json=JSON.stringify({...raw,options:{...raw.options,seed:'18446744073709551616'}}),
 r=>r.input.request_json=JSON.stringify({...raw,options:{...raw.options,ilp_gap:-1}}),
 r=>r.input.request_json=JSON.stringify({operation:'run-prepared-engine',input:{request_json:'{}'}}),
 r=>r.input.inspect_only='yes',r=>r.input.label='',r=>r.input.created_at='',
 r=>r.input.source={unexpected:true},
 ]){
  const bad=structuredClone(base);mutate(bad);assert.throws(()=>engine.execute(bad));await fs.writeFile(root+'/invalid.json',JSON.stringify(bad));assert.throws(()=>execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(root+'/invalid.json')],{stdio:'pipe'}));refusals++;
 }
const numeric=structuredClone(base);numeric.input.request_json=inputs.source.text.replace('"seed":"18446744073709551615"','"seed":18446744073709551615');assert.notEqual(numeric.input.request_json,base.input.request_json);const numericRun=engine.execute(numeric);await fs.writeFile(root+'/numeric.json',JSON.stringify(numeric));const nativeNumeric=JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(root+'/numeric.json')],{encoding:'utf8',maxBuffer:64*1024*1024}));assert.deepEqual(numericRun,nativeNumeric);assert.deepEqual(numericRun.result,fixture.result.result);assert.equal(numericRun.executed_request.options.seed,'18446744073709551615');
let unitMatches=0;
for(const [kind,year]of [['county','2020'],['tract','2020'],['block-group','2010'],['block','2000'],['precinct','2024'],['imported','2018']]){
 const prior=JSON.parse(await fs.readFile('target/native-character-units/'+kind+'-'+year+'-housing.bisect','utf8')),f=prior.files,request=JSON.parse(f.nativeCharacterEngineInputs.source.text);request.native_character_source={...f.nativeCharacterInputs.settings,source_name:f.nativeCharacterInputs.source.name,source_base64:f.nativeCharacterInputs.source.base64};
 const tool=preparedEngineRequest({source:{name:'declared-unit.json',text:JSON.stringify(request)},settings:f.nativeCharacterEngineInputs.settings}),actual=engine.execute(tool);await fs.writeFile(root+'/unit.json',JSON.stringify(tool));const expected=JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(root+'/unit.json')],{encoding:'utf8',maxBuffer:64*1024*1024}));assert.deepEqual(actual,expected);assert.deepEqual(actual.result,prior.result.result);assert.deepEqual(actual.export,prior.result.export);assert.equal(actual.export.document.plan.units.unit_kind,kind);unitMatches++;
}
assert.equal(globalThis.preparedEnginePwned,undefined);
const opaque='globalThis.preparedEnginePwned=1;',bytes=Buffer.from(opaque),source=await readPreparedEngineSource({name:'opaque.json',size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});assert.equal(source.text,opaque);
const saved=createProject({name:'Opaque source is data',files:{preparedEngineInputs:{source,settings:PREPARED_ENGINE_DEFAULTS}},operation:'run-prepared-engine',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(saved)),saved);
const bom=Buffer.concat([Buffer.from([239,187,191]),Buffer.from(' {\r\n"seed":18446744073709551615\r\n}\r\n')]),retained=await readPreparedEngineSource({name:'BOM.json',size:bom.length,arrayBuffer:async()=>Uint8Array.from(bom).buffer});assert.equal(Buffer.from(retained.text).toString('base64'),bom.toString('base64'));
let read=false;await assert.rejects(readPreparedEngineSource({name:'too-large.json',size:25*1024*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(readPreparedEngineSource({name:'changed.json',size:1,arrayBuffer:async()=>new Uint8Array(2).buffer}));
assert.throws(()=>validatePreparedEngineInputs({...inputs,unknown:true}));assert.throws(()=>preparedEngineRequest({source:null,settings:PREPARED_ENGINE_DEFAULTS}));assert.throws(()=>preparedEngineDownload(fixture.result,'unknown'));assert.throws(()=>preparedEngineDownload({schema_version:'bisect-prepared-engine-request-v1'},'plan'));
await fs.writeFile(root+'/evidence.json',JSON.stringify({matching_native_refusals:refusals,numeric_u64_full_execution_matches_native:true,declared_unit_namespace_matches:unitMatches,successful_native_comparisons:unitMatches+1,raw_opaque_text_not_evaluated:true,source_only_project_open_inert:true,bom_crlf_and_numeric_u64_bytes_preserved:true,oversize_rejected_before_read:true,changed_read_rejected:true,review_cannot_download_plan:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Prepared engine guards: ${refusals} matching Native refusals, inert opaque text/project retention, exact BOM/CRLF/u64 bytes and pre-read size/freshness guards passed.`);

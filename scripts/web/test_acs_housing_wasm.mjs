import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {ACS_HOUSING_DEFAULTS,acsHousingRequest,readAcsHousingSource,acsHousingDownload} from '../../web/lab/acs-housing-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/acs-housing-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const header=['B25024_001E','B25024_002E','B25024_003E','B25024_007E','B25024_008E','B25024_009E','B25003_001E','B25003_002E','B25035_001E','state','county','tract'];
const row=['100','40','10','10','20','10','80','60','1980','44','001','000101'];
const changed=values=>row.map((v,i)=>Object.hasOwn(values,i)?values[i]:v);
const fixtures=[['valid',[header,row,changed({11:'000102',0:100,1:20,8:2000})],'ready'],['defaults',[header,changed({0:null,1:'bad',6:0,8:'-666666666'})],'ready'],['clamps',[header,changed({1:999,2:999,4:999,7:999,8:1700}),changed({11:'000102',8:2200})],'ready'],['duplicate',[header,row,row],'unavailable'],['foreign',[header,changed({9:'37'})],'unavailable'],['nonnumeric-fips',[header,changed({9:'xx'})],'unavailable'],['malformed-fips',[header,changed({9:'4'}),row],'ready'],['short',[['tract',...header.slice(0,-1)],['000101']],'unavailable'],['empty',[],'unavailable'],['header-only',[header],'unavailable']];
for(const [name,body,status]of fixtures){
 const raw=JSON.stringify(body,null,2)+'\n',bytes=new TextEncoder().encode(raw),source=await readAcsHousingSource({name:name+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer}),inputs={source,settings:{...ACS_HOUSING_DEFAULTS}},request=acsHousingRequest(inputs),result=engine.execute(request);
 assert.equal(request.input.body_json,raw);await fs.writeFile(root+'/request.json',JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8'})),result);
 await fs.writeFile(root+'/body.json',raw);const native=spawnSync('target/debug/examples/acs_housing_reference.exe',[root+'/body.json',root+'/native.csv'],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);assert.equal(await fs.readFile(root+'/native.csv','utf8'),result.csv_text);
 assert.equal(result.character_status,status);assert.equal(result.execution_performed,false);assert.equal(result.source_sha256,createHash('sha256').update(bytes).digest('hex'));assert.equal(result.csv_sha256,createHash('sha256').update(result.csv_text).digest('hex'));assert.equal(new TextDecoder().decode(acsHousingDownload(result,'csv').bytes),result.csv_text);
 if(status==='ready')assert.deepEqual(JSON.parse(new TextDecoder().decode(acsHousingDownload(result,'character').bytes)),result.character_input);else assert.throws(()=>acsHousingDownload(result,'character'));
 if(name==='valid')assert.match(result.csv_text,/44001000101,0.500000,0.400000,0.750000,0.500000\n/);if(name==='malformed-fips'){assert.equal(result.skipped_fips_rows,1);assert.match(native.stderr,/malformed FIPS/);}if(name==='short')assert.equal(result.skipped_short_rows,1);
 const project=createProject({name:'ACS '+name,files:{acsHousingInputs:inputs},operation:'prepare-acs-housing-source',constraints:[],result,lastOperation:'prepare-acs-housing-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
for(const body of ['{}','[[]]','[["tract"]]','[','[null]']){const request={operation:'prepare-acs-housing-source',input:{...ACS_HOUSING_DEFAULTS,source_name:'bad.json',body_json:body}};assert.throws(()=>engine.execute(request));await fs.writeFile(root+'/body.json',body);const p=spawnSync('target/debug/examples/acs_housing_reference.exe',[root+'/body.json',root+'/bad.csv'],{encoding:'utf8'});assert.notEqual(p.status,0);assert.doesNotMatch(p.stderr,/panicked/);}
let read=false;await assert.rejects(()=>readAcsHousingSource({name:'huge.json',size:16*1024*1024+1,arrayBuffer(){read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readAcsHousingSource({name:'bad.json',size:1,arrayBuffer:async()=>Uint8Array.of(255).buffer}));
await assert.rejects(()=>readAcsHousingSource({name:'changed.json',size:2,arrayBuffer:async()=>new TextEncoder().encode('[]\n').buffer}));
const validResult=JSON.parse(await fs.readFile(root+'/valid-result.json','utf8'));assert.throws(()=>acsHousingRequest({source:null,settings:ACS_HOUSING_DEFAULTS}));assert.throws(()=>acsHousingDownload({...validResult,execution_performed:true},'csv'));
console.log('ACS housing: 10 native-host/WASM and native CSV byte comparisons, defaults/clamps, malformed/reordered row safety, strict character availability, source/hash/project/download preservation and 5 native error cases passed.');

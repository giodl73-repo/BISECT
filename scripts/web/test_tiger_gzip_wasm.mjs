import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {gzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tigerRequest,readTigerPart} from '../../web/lab/tiger-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/tiger-gzip-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=(name,b)=>({name,base64:Buffer.from(b).toString('base64')}),hash=b=>createHash('sha256').update(b).digest('hex');
for(const name of ['tracts-tracts','groups-block-groups','blocks-2020-blocks','blocks-2010-blocks','blocks-2000-block-centroids']){
 const original=JSON.parse(await fs.readFile('target/tiger-source-wasm/'+name+'.bisect','utf8')).files.tigerInputs,plain=engine.execute(tigerRequest(original)),input=structuredClone(original);
 for(const part of ['shp','dbf','shx'])if(input[part])input[part]=source(input[part].name+'.gz',gzipSync(Buffer.from(input[part].base64,'base64')));
 const result=engine.execute(tigerRequest(input));assert.equal(result.records_json,plain.records_json);assert.equal(result.records_sha256,plain.records_sha256);for(const part of ['shp','dbf','shx'])if(input[part]){assert.equal(result[part].sha256,hash(Buffer.from(input[part].base64,'base64')));assert.equal(result[part+'_decoded'].encoding,'gzip');assert.equal(result[part+'_decoded'].decoded_sha256,plain[part].sha256);}
 await fs.writeFile(root+'/request.json',JSON.stringify(tigerRequest(input)));const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8'}));for(const part of ['shp','dbf','shx'])assert.deepEqual(host[part+'_decoded'],result[part+'_decoded']);if(input.kind==='tracts'||input.kind==='block-groups')assert.deepEqual(host.records,result.records);
 const project=createProject({name:'Gzip '+name,files:{tigerInputs:input},operation:'prepare-tiger-source',constraints:[],result,lastOperation:'prepare-tiger-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));
 const corrupt=Buffer.from(input.shp.base64,'base64');corrupt[corrupt.length-8]^=1;assert.throws(()=>engine.execute(tigerRequest({...input,shp:source('corrupt.gz',corrupt)})),/gzip/);
 assert.throws(()=>engine.execute(tigerRequest({...input,shp:source('truncated.gz',Buffer.from(input.shp.base64,'base64').subarray(0,15))})),/gzip/);
}
const original=JSON.parse(await fs.readFile('target/tiger-source-wasm/tracts-tracts.bisect','utf8')).files.tigerInputs;
assert.throws(()=>engine.execute(tigerRequest({...original,shp:source('bomb.gz',gzipSync(Buffer.alloc(128*1024*1024+1)))})),/128 MiB/);
// Compression changes only the selected transport bytes, including tract graph construction.
const tract=JSON.parse(await fs.readFile('target/tract-preparation-wasm/2020.bisect','utf8')),input=structuredClone(tract.files.tigerInputs);for(const part of ['shp','dbf','shx'])if(input[part])input[part]=source(input[part].name+'.gz',gzipSync(Buffer.from(input[part].base64,'base64')));
const prepared=engine.execute({operation:'prepare-tract-engine-input',input:{state:'RI',tiger:input,pl94:tract.files.pl94Inputs,projection:tract.files.tractPreparationSettings.projection}});assert.equal(prepared.graph_json,tract.result.graph_json);assert.equal(prepared.geometry_json,tract.result.geometry_json);
if(process.argv.includes('--local-ca')){
 const prefix='data/2020/tiger/tracts/tl_2020_06_tract/tl_2020_06_tract',input={kind:'tracts',year:2020,shp:null,dbf:null,shx:null};let selected=0;
 for(const part of ['shp','dbf','shx']){const original=await fs.readFile(prefix+'.'+part),bytes=gzipSync(original);input[part]=await readTigerPart({name:path.basename(prefix)+'.'+part+'.gz',size:bytes.length,arrayBuffer:async()=>bytes});selected+=bytes.length;if(part==='shp')assert.ok(original.length>16*1024*1024);}
 const result=engine.execute(tigerRequest(input));await fs.writeFile(root+'/ca-request.json',JSON.stringify(tigerRequest(input)));const options={encoding:'utf8',maxBuffer:512*1024*1024},host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/ca-request.json'],options));assert.deepEqual(host.records,result.records);
 const native=JSON.parse(execFileSync('target/debug/examples/tiger_reference.exe',['tracts','2020',prefix+'.shp'],options));for(const record of native){record.geometry_wkb_base64=Buffer.from(record.geometry_wkb_bytes).toString('base64');delete record.geometry_wkb_bytes;}assert.deepEqual(native,result.records);
 const text=createWorkbenchArchive({name:'California original TIGER sources',files:{tigerInputs:input},operation:'prepare-tiger-source'});assert.deepEqual(parseWorkbenchArchive(text).files.tigerInputs,input);await fs.writeFile(root+'/ca-inputs.bisect-workbench',text);await fs.writeFile(root+'/ca-receipt.json',JSON.stringify({units:result.record_count,selected_bytes:selected,shp:result.shp,shp_decoded:result.shp_decoded,records_sha256:result.records_sha256,records_json_bytes:Buffer.byteLength(result.records_json),scope:'Actual source reader parity; this does not run a graph or establish complete California workflow coverage.'}));console.log('Actual CA: '+result.record_count+' tract records, decoded SHP '+result.shp_decoded.decoded_byte_length+' bytes, selected '+selected+' gzip bytes; native-host/WASM/filesystem exact WKB parity and retained source-only archive passed.');
}
console.log('TIGER gzip: five modes/vintage fixtures, raw/decoded identity, unchanged records/graph/display map, inert source projects, CRC/truncation/128-MiB expansion guards passed.');

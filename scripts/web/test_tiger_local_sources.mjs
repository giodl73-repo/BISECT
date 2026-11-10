// Explicit local Census evidence; never downloads or discovers other sources.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tigerRequest,tigerDownload,readTigerPart} from '../../web/lab/tiger-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const [kind,yearText,shpPath]=process.argv.slice(2),year=Number(yearText);
if(!['tracts','block-groups'].includes(kind)||![2000,2010,2020].includes(year)||!shpPath?.endsWith('.shp'))throw new Error('Supply tracts|block-groups, Census year, and a local .shp path.');
const root='target/tiger-local-source-wasm';await fs.mkdir(root,{recursive:true});
const input={kind,year,shp:null,dbf:null,shx:null};
for(const part of ['shp','dbf','shx']){const filePath=shpPath.slice(0,-4)+'.'+part;const bytes=await fs.readFile(filePath);input[part]=await readTigerPart({name:path.basename(filePath),size:bytes.length,arrayBuffer:async()=>bytes});}
const request=tigerRequest(input),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),result=engine.execute(request);
await fs.writeFile(root+'/request.json',JSON.stringify(request));
const options={encoding:'utf8',maxBuffer:128*1024*1024};
const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],options));assert.deepEqual(host,result);
const native=JSON.parse(execFileSync('target/debug/examples/tiger_reference.exe',[kind,yearText,shpPath],options));for(const record of native){record.geometry_wkb_base64=Buffer.from(record.geometry_wkb_bytes).toString('base64');delete record.geometry_wkb_bytes;}assert.deepEqual(native,result.records);
assert.equal(Buffer.from(tigerDownload(result).bytes).toString(),result.records_json);
const project=createProject({name:'Local TIGER '+kind+' '+year,files:{tigerInputs:input},operation:'prepare-tiger-source',constraints:[],result,lastOperation:'prepare-tiger-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
await fs.writeFile(root+'/'+kind+'-'+year+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+kind+'-'+year+'-result.json',JSON.stringify(result));
console.log('Local TIGER: '+result.record_count+' actual '+kind+' records, exact native-host/WASM/filesystem WKB/areas/identities, original parts and inert project/download roundtrips passed.');

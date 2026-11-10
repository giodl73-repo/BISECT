// Explicit live-provider observation; never invoked by ordinary builds/tests.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {downloadPublicSource} from '../../web/lab/fetch-network.js';
import {decodeFetchSourceArchive} from '../../web/lab/fetch-source-archive.js';
import {publicFetchUrl} from '../../web/lab/fetch-acquisition.js';
assert.deepEqual(process.argv.slice(2),['--run'],'Pass --run for explicit live public-source requests.');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),engine=await instantiateEngine(await fs.readFile('dist/wasm-workspace-continuation-validated/toolkit/bisect_wasm.wasm'));
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI'],year:'2020',data_types:['school-districts','eia-861'],selected_paths:[]}}),root=path.resolve('target/fetch-archive-live','run-'+Date.now());await fs.mkdir(root,{recursive:true});const rows=[];
for(const item of plan.items){const url=publicFetchUrl(item),row={state:item.state_code,year:item.year,kind:item.kind,url,status:'requesting'};rows.push(row);console.log(item.kind+' '+url);
 try{const response=await downloadPublicSource(url,{maxBytes:64*1024*1024,timeoutMs:300000,retryAttempts:1});row.http_status=response.status;row.attempt_status=response.attempt_status;if(response.status===404)row.status='unavailable';else{row.bytes=response.bytes.length;row.sha256=sha(response.bytes);await fs.writeFile(path.join(root,item.kind+'.zip'),response.bytes,{flag:'wx'});const archive=await decodeFetchSourceArchive(response.bytes);row.status='archive-inspected';row.expanded_bytes=archive.files.reduce((n,file)=>n+file.bytes.length,0);row.members=archive.files.map(file=>({path:file.path,bytes:file.bytes.length}));row.planned_target_member_present=archive.files.some(file=>file.path===item.local_path.split('/').at(-1));}}
 catch(error){row.status=row.bytes?'archive-inspection-failed':'provider-request-failed';row.error=error.message;}
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({schema_version:'bisect-live-fetch-archive-observation-v1',observed_at_utc:new Date().toISOString(),rows,scope:'Explicit native-pinned public 2020 URLs with bounded credential-free Node transport and ZIP inspection. This does not test browser CORS, deployed relay, source authenticity, geometry/workbook validity or zone joins.'},null,2)+'\n');
}
console.log('Live observations '+path.join(root,'evidence.json'));

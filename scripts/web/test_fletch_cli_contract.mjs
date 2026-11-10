// Actual CLI orchestration, not the shared adapter used by older unit oracles.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {fletchSourcesRequest,fletchSourcesReportBytes} from '../../web/lab/fletch-source-files.js';
import {fletchCacheRequest,fletchCacheReportBytes} from '../../web/lab/fletch-cache-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=name=>process.argv.find(a=>a.startsWith('--'+name+'='))?.slice(name.length+3);
await fs.mkdir('target/fletch-cli-contract',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/fletch-cli-contract/run-'));
const native=path.join(root,'native.exe'),wasm=path.join(root,'engine.wasm');
await fs.copyFile(option('native')||'target/release/bisect.exe',native);
await fs.copyFile(option('wasm')||process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm',wasm);
const hash=b=>createHash('sha256').update(b).digest('hex');
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const engine=await instantiateEngine(await fs.readFile(wasm));
const manifest=await fs.readFile('data/manifest.json','utf8'),manifestPath=path.join(root,'manifest.json');await fs.writeFile(manifestPath,manifest);
const records=[];
const entry=(verified=true)=>({dataset_id:'bisect.2020.ri.tiger',source_url:'https://example.test/inert',cache_key:'opaque',relative_path:'../not-opened',sha256:'sha256:'+'g'.repeat(64),bytes:1,fetched_at_ms:1,verified});
const cases=[
 {name:'sources-default',kind:'sources'},
 {name:'sources-all-years',kind:'sources',year:'all'},
 {name:'sources-optional',kind:'sources',types:['lodes','school-districts','eia-861']},
 {name:'sources-empty',kind:'sources',states:['XX']},
 {name:'sources-duplicates',kind:'sources',states:['RI','RI']},
 {name:'cache-missing-expected',entries:[]},
 {name:'cache-verified',entries:[entry()]},
 {name:'cache-unverified-gated',entries:[entry(false)]},
 {name:'cache-unverified-ungated',entries:[entry(false)],gate:false},
 {name:'cache-unexpected',entries:[{...entry(),dataset_id:'unexpected'}]},
 {name:'cache-exact-u64',entries:[entry()],large:true},
 {name:'cache-bom',entries:[],bom:true},
 {name:'cache-invalid-json',invalid:true},
];
for(const row of cases){
 const dir=path.join(root,row.name);await fs.mkdir(dir);
 const selection={manifest_json:manifest,states:row.states||['RI'],year:row.year||'2020',data_types:row.types||[],selected_paths:[]};
 const selected={source:source('selection.json',JSON.stringify(selection))};
 let text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'C:/never-opened',entries:row.entries||[]});
 if(row.large)text=text.replace('"bytes":1','"bytes":9007199254740993');
 if(row.bom)text='\ufeff'+text;if(row.invalid)text='{invalid';
 const cache=source('cache.json',text),cachePath=path.join(dir,'cache.json');await fs.writeFile(cachePath,text);
 const out=path.join(dir,row.kind==='sources'?'report.csv':'report.json');
 const args=[row.kind==='sources'?'fletch-sources':'fletch-cache-index','--manifest',manifestPath,'--year',selection.year,'--states',...selection.states,'--output',out,'--details'];
 if(selection.data_types.length)args.push('--type',...selection.data_types);
 if(row.kind!=='sources')args.push('--cache-manifest',cachePath);if(row.gate!==false)args.push('--gate');
 const run=spawnSync(native,args,{encoding:'utf8',windowsHide:true});if(run.error)throw run.error;
 let request,result,error;
 try{request=row.kind==='sources'?fletchSourcesRequest({selection:selected,settings:{year:'selection'}}):fletchCacheRequest({selection:selected,manifest:cache});result=engine.execute(request);}catch(e){error=e.message;}
 const actual=await fs.readFile(out).catch(e=>{if(e.code==='ENOENT')return null;throw e;});
 const record={name:row.name,arguments:args,native_exit:run.status,stdout:run.stdout,stderr:run.stderr,request,result,error,output_sha256:actual?hash(actual):null};
 records.push(record);await fs.writeFile(path.join(root,'progress.json'),JSON.stringify(records,null,2)+'\n');
 if(row.invalid||row.bom){const project=createProject({name:row.name,operation:request.operation,files:{fetchPlanInputs:selected,fletchCacheManifest:cache,fletchCacheSettings:{year:'selection'}},constraints:[],result:null,lastOperation:null});await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));assert.equal(run.status,1,row.name);assert(error,row.name+' native-invalid source must refuse in browser');assert.equal(actual,null);continue;}
 assert(!error,error);assert.equal(run.status,row.gate===false||result.gate_passed?0:1,row.name+' gate exit');
 assert(actual,row.name+' native writes report before a failed gate');
 const bytes=row.kind==='sources'?fletchSourcesReportBytes(result):fletchCacheReportBytes(result);
 // These fixtures contain backslashes only in Windows-generated cache target paths.
 if(row.kind==='sources'){
  const nativeText=actual.toString('utf8');for(const line of nativeText.split('\n'))if(line.includes('\\'))assert(line.includes('data\\'),'Only declared Windows target paths differ');
  assert.deepEqual(Buffer.from(bytes),Buffer.from(nativeText.replaceAll('\\','/')),row.name+' native CSV apart from declared portable target separators');
 }else assert.deepEqual(Buffer.from(bytes),actual,row.name+' exact CLI output bytes');
 if(row.large)assert.equal(result.byte_count_text,'9007199254740993');
 const files=row.kind==='sources'?{fetchPlanInputs:selected,fletchSourceSettings:{year:'selection'}}:{fetchPlanInputs:selected,fletchCacheManifest:cache,fletchCacheSettings:{year:'selection'}};
 // Results remain data on Open; no paths, URLs or verification flags grant authority.
 const project=createProject({name:row.name,operation:request.operation,files,constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));
}
const sources=[];for(const file of ['crates/bisect-cli/src/main.rs','crates/bisect-cli/src/fletch.rs','crates/bisect-wasm/src/fletch_sources.rs','crates/bisect-wasm/src/fletch_cache.rs','web/lab/fletch-source-files.js','web/lab/fletch-cache-files.js','web/lab/fetch-plan-files.js','web/lab/project.js','scripts/web/test_fletch_cli_contract.mjs','data/manifest.json'])sources.push({path:file,sha256:hash(await fs.readFile(file))});
const evidence={status:'passed-actual-CLI-fletch-contract',sources,native:{path:native,sha256:hash(await fs.readFile(native))},wasm:{path:wasm,sha256:hash(await fs.readFile(wasm))},records,scope:'Actual fletch-sources/cache-index parser, source selection, report bytes and post-output gate outcomes. Explicit selected manifests; no automatic native path discovery, acquisition, selected-folder installation or full parent acceptance.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root}));

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readTutorialChecksums,tutorialRequest,verifyTutorialFiles,tutorialConsoleText,selectTutorialFiles} from '../../web/lab/tutorial-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),native=option('native'),wasm=option('wasm');if(!native||!wasm)throw Error('Pass explicit --native and --wasm paths.');
const hash=b=>createHash('sha256').update(b).digest('hex'),pin=async p=>({path:p,sha256:hash(await fs.readFile(p))});await fs.mkdir('target/tutorial-cli-contract',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/tutorial-cli-contract/run-')),cli=path.join(root,'native.exe');await fs.copyFile(native,cli);const engine=await instantiateEngine(await fs.readFile(wasm));
const a=Buffer.from([0,255,13,10,1]),b=Buffer.from('exact output\r\n'),empty=Buffer.alloc(0),template={schema_version:'tutorial-checksums v1',tutorial:'synthetic',pinned_inputs:[{name:'Raw input',local_path:'input.bin',source_url:'https://never-fetch.invalid/source',sha256:hash(a)}],expected_outputs:[{path:'output.json',sha256:hash(b),tract_count:193}],build_commit:'recorded-not-authenticated',pinned_at:'original-date'};
const cases=[
 ['pass',template,{'input.bin':a,'output.json':b}],['missing',template,{}],['drift',template,{'input.bin':b,'output.json':b}],['mixed',template,{'input.bin':a}],
 ['uppercase',{...template,pinned_inputs:[{...template.pinned_inputs[0],sha256:hash(a).toUpperCase()}]},{'input.bin':a,'output.json':b}],
 ['unrecorded-pin',{...template,pinned_inputs:[{...template.pinned_inputs[0],sha256:'PIN_ON_FIRST_RUN'}]},{'input.bin':a,'output.json':b}],
 ['empty-file',{...template,pinned_inputs:[{...template.pinned_inputs[0],sha256:hash(empty)}],expected_outputs:[]},{'input.bin':empty}],
 ['read-error',template,{'input.bin':'directory','output.json':b}],
 ['tutorial-warning',{...template,tutorial:"another' but requested tutorial='injected"},{'input.bin':a,'output.json':b}],
 ['wide-count',JSON.stringify(template).replace('"tract_count":193','"tract_count":9007199254740993'),{'input.bin':a,'output.json':b}],
 ['duplicate-reference',{...template,pinned_inputs:[template.pinned_inputs[0],template.pinned_inputs[0]]},{'input.bin':a,'output.json':b}],
 ['ignored-extra',template,{'input.bin':a,'output.json':b,'unrelated.bin':'directory'}],
 ['schema',{...template,schema_version:'bad'},{'input.bin':'directory'}],['malformed','{',{'input.bin':'directory'}],['shape','[]',{}],['bom','\ufeff'+JSON.stringify(template),{}],['empty','',{}],['crlf',JSON.stringify(template)+'\r\n',{'input.bin':a,'output.json':b}],
 ['limit',JSON.stringify(template)+' '.repeat(512*1024-Buffer.byteLength(JSON.stringify(template))),{'input.bin':a,'output.json':b}],
];
const records=[];
for(const [name,doc,bodies]of cases){
 const folder=path.join(root,name);await fs.mkdir(path.join(folder,'examples/synthetic-walkthrough'),{recursive:true});const text=typeof doc==='string'?doc:JSON.stringify(doc),bytes=Buffer.from(text),checksumsPath=path.join(folder,'examples/synthetic-walkthrough/checksums.json');await fs.writeFile(checksumsPath,bytes);
 const entries=[],fileSources=[];
 for(const [name,body]of Object.entries(bodies)){
  if(body==='directory'){await fs.mkdir(path.join(folder,name));entries.push({path:name,file:{size:1,slice:()=>({arrayBuffer:async()=>{throw Error('Controlled selected-file read refusal');}})}});fileSources.push({path:name,kind:'controlled-read-error'});}
  else{await fs.writeFile(path.join(folder,name),body);entries.push({path:name,file:new File([body],name)});fileSources.push({path:name,kind:'file',base64:body.toString('base64')});}
 }
 const hashedPaths=[],paths=new Map(entries.map(e=>[e.file,e.path]));
 const source=await readTutorialChecksums({name:'checksums.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)}),inputs={tutorial:'synthetic',checksums:source},request=tutorialRequest(inputs,entries),result=await verifyTutorialFiles({execute:r=>engine.execute(r),hashFile:file=>{hashedPaths.push(paths.get(file));return engine.hashFile(file);}},request),consoleText=tutorialConsoleText(result);
 assert.deepEqual(hashedPaths,[...new Set((result.rows??[]).map(r=>r.path))].filter(p=>Object.hasOwn(bodies,p)),'Parse before hashing; read only referenced files in native order');
 assert.equal(request.checksums_json,text);assert.equal(result.source_sha256,'sha256:'+hash(bytes));
 const ref=spawnSync(cli,['doctor','--check-tutorial-data','--tutorial','synthetic'],{cwd:folder,encoding:'utf8',windowsHide:true});assert.equal(ref.error,undefined);assert.equal(ref.stderr,'');assert.equal(result.native_exit_code,ref.status,name);
 let expected=ref.stdout.replace('=== bisect doctor --check-tutorial-data ===','=== bisect doctor --check-tutorial-data (WASM) ===').replace(/^Checksums: .*$/m,'Checksums: checksums.json');const adaptations=[];
 if(name==='read-error'){
  const nativeError=expected.match(/^          read error: .*$/m)?.[0],browserError=consoleText.match(/^          read error: .*$/m)?.[0];assert(nativeError&&browserError);expected=expected.replace(nativeError,browserError);adaptations.push({native_error:nativeError,browser_error:browserError,scope:'Different IO hosts; matching read-error class/status, not identical OS text.'});
 }
 assert.equal(consoleText,expected,name);
 const project=createProject({name,files:{tutorialInputs:inputs},operation:'verify-tutorial-files',constraints:[],result,lastOperation:'verify-tutorial-files'});assert.equal(tutorialRequest(parseProject(JSON.stringify(project)).files.tutorialInputs).checksums_json,text);await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));
 if(['pass','duplicate-reference'].includes(name)){const candidates=Object.keys(bodies).map(n=>({name:n,size:bodies[n].length,webkitRelativePath:'repo/'+n}));candidates.push({name:'unrelated',size:1,webkitRelativePath:'repo/unrelated'});assert.deepEqual(selectTutorialFiles(candidates,{folder:true,checksums:source}).map(e=>e.path),Object.keys(bodies));}
 records.push({name,source,inputs,result,console_text:consoleText,selected_files:fileSources,hashed_paths:hashedPaths,native:{exit_code:ref.status,stdout:ref.stdout,stderr:ref.stderr},adaptations});
}
const bom=records.find(r=>r.name==='bom'),old=engine.execute({operation:'verify-tutorial-data',tutorial:'synthetic',source_name:'checksums.json',checksums_json:new TextDecoder().decode(Buffer.from(bom.source.base64,'base64')),files:{}});assert.equal(old.native_exit_code,0);assert.equal(bom.result.native_exit_code,2);
const guards=[];for(const [name,bytes]of [['utf8',Buffer.from([255])],['oversized',Buffer.alloc(512*1024+1,32)]]){let reads=0;await assert.rejects(()=>readTutorialChecksums({name:name+'.json',size:bytes.length,arrayBuffer:async()=>{reads++;return bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);}}));if(name==='oversized')assert.equal(reads,0);guards.push({name,reads,status:'preflight-refused'});}
const sources=await Promise.all(['crates/bisect-cli/src/doctor.rs','crates/bisect-analysis/src/tutorial_checksums.rs','crates/bisect-wasm/src/tutorial.rs','web/lab/package-files.js','web/lab/tutorial-files.js','scripts/web/test_tutorial_cli_contract.mjs'].map(pin));const evidence={status:'passed-actual-tutorial-cli-contract',sources,native:await pin(cli),wasm:await pin(wasm),records,guards,historical_bom_loss:{native_exit:2,old_decoder_exit:old.native_exit_code,corrected_exit:bom.result.native_exit_code},scope:'Actual native tutorial parser/status/full console, streamed hashes and selected byte/project preservation. Only explicit WASM heading, displayed checksum path and different IO-host error text adapt. Safe relative <=2000 references, <=512 KiB JSON. No path authority, fetching, authentication, tract recount or full parent claim.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,cases:records.length}));

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {manifestRequest,readManifestSource,manifestConsoleText} from '../../web/lab/manifest-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),native=option('native'),wasm=option('wasm');
if(!native||!wasm)throw Error('Pass explicit --native and --wasm paths.');
const hash=b=>createHash('sha256').update(b).digest('hex'),pin=async p=>({path:p,sha256:hash(await fs.readFile(p))});
await fs.mkdir('target/manifest-cli-contract',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/manifest-cli-contract/run-'));
const cli=path.join(root,'native.exe');await fs.copyFile(native,cli);const engine=await instantiateEngine(await fs.readFile(wasm)),build=engine.execute({operation:'verify-run-manifest',source_name:'manifest.json',manifest_json:'{}'}).current_build;
const version=build.version,full=JSON.stringify({binary_version:version,binary_sha256:'not-a-digest',adjacency_sha256:'',adjacency_file:'C:\\never-open\\<script>.bin',created_at:'not-a-date',unknown:{integer:9007199254740993n.toString()}});
const cases=[['empty',''],['missing','{}'],['array','[]'],['null','null'],['number','12'],['string','"manifest"'],['version-null','{"binary_version":null}'],['version-number','{"binary_version":12}'],['version-empty','{"binary_version":""}'],['matches',JSON.stringify({binary_version:version})],['mismatch','{"binary_version":"999.999.999"}'],['untrimmed',JSON.stringify({binary_version:' '+version})],['full',full],['wrong-optional-types',JSON.stringify({binary_version:version,binary_sha256:12,adjacency_sha256:false,adjacency_file:[],created_at:{}})],['unicode',JSON.stringify({binary_version:version,adjacency_file:'élection.bin',created_at:'2020\r\noriginal'})],['crlf',full+'\r\n'],['bom','\ufeff'+full],['malformed','{'],['trailing','{}x'],['duplicate','{"binary_version":"wrong","binary_version":"'+version+'"}'],['limit',full+' '.repeat(1024*1024-Buffer.byteLength(full))]];
const records=[];
for(const [name,text]of cases){
 const bytes=Buffer.from(text),file=path.join(root,name+'.json');await fs.writeFile(file,bytes);
 const ref=spawnSync(cli,['doctor','--verify-manifest',file],{encoding:'utf8',windowsHide:true,maxBuffer:4*1024*1024});assert.equal(ref.error,undefined);assert.equal(ref.stderr,'');
 const source=await readManifestSource({name:name+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)}),request=manifestRequest(source),result=engine.execute({...request,runtime_artifact:engine.runtimeArtifacts.engine}),consoleText=manifestConsoleText(result);
 assert.equal(request.manifest_json,text);assert.equal(result.source_sha256,'sha256:'+hash(bytes));assert.equal(result.native_exit_code,ref.status);
 let expected=ref.stdout.replace(file,source.name).replace('=== bisect doctor --verify-manifest ===','=== bisect doctor --verify-manifest (WASM) ===');
 const allowances=[];
 if(result.status!=='invalid-json'){
  const nativeBlock=expected.match(/Running binary:\n([\s\S]*?)\n\nManifest records:/);assert(nativeBlock);
  const wasmBlock=consoleText.match(/Running WASM module:\n([\s\S]*?)\n\nManifest records:/);assert(wasmBlock);
  assert.equal(nativeBlock[1].match(/version:      (.*)/)[1],version,'Native and WASM package versions differ; no equivalent status claim.');
  expected=expected.replace(nativeBlock[0],wasmBlock[0]);allowances.push({kind:'actual-runtime-build-block',native:nativeBlock[0],wasm:wasmBlock[0]});
  const warning=/^\[WARN\] Running binary.*\n/gm,nativeWarnings=expected.match(warning)??[],wasmWarnings=consoleText.match(warning)??[];
  expected=expected.replace(warning,'');const withoutWarning=consoleText.replace(warning,'');assert.equal(withoutWarning,expected,name);allowances.push({kind:'actual-runtime-commit-warning',native:nativeWarnings,wasm:wasmWarnings});
 }else assert.equal(consoleText,expected,name);
 const project=createProject({name,files:{manifestSource:source},operation:'verify-run-manifest',constraints:[],result,lastOperation:'verify-run-manifest'});assert.deepEqual(manifestRequest(parseProject(JSON.stringify(project)).files.manifestSource),request);
 await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));records.push({name,source,request,result,console_text:consoleText,native:{exit_code:ref.status,stdout:ref.stdout,stderr:ref.stderr},allowances});
}
// Retain the original byte-loss counterexample without running it as the corrected path.
const bom=records.find(r=>r.name==='bom'),historical=engine.execute({...bom.request,manifest_json:new TextDecoder('utf-8',{fatal:true}).decode(Buffer.from(bom.source.base64,'base64'))});assert.equal(historical.native_exit_code,0);assert.equal(bom.result.native_exit_code,2);
const guards=[];for(const [name,bytes]of [['invalid-utf8',Buffer.from([255])],['oversized',Buffer.alloc(1024*1024+1,32)]]){
 const file=path.join(root,name+'.json');await fs.writeFile(file,bytes);const ref=spawnSync(cli,['doctor','--verify-manifest',file],{encoding:'utf8',windowsHide:true});let reads=0;await assert.rejects(()=>readManifestSource({name:name+'.json',size:bytes.length,arrayBuffer:async()=>{reads++;return bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);}}));if(name!=='invalid-utf8')assert.equal(reads,0);guards.push({name,native_exit:ref.status,stdout:ref.stdout,adapter:'preflight-refused',reads});
}
const sources=await Promise.all(['crates/bisect-cli/src/doctor.rs','crates/bisect-cli/src/provenance.rs','crates/bisect-analysis/src/manifest_provenance.rs','crates/bisect-wasm/src/manifest.rs','web/lab/manifest-files.js','scripts/web/test_manifest_cli_contract.mjs'].map(pin));
const evidence={status:'passed-actual-manifest-cli-contract',sources,native:await pin(cli),wasm:await pin(wasm),records,guards,historical_bom_loss:{native_exit:2,old_decoder_exit:historical.native_exit_code,corrected_exit:bom.result.native_exit_code},scope:'Actual native Doctor version/parse/status/complete console observations. Source-name display and actual native-vs-WASM build block/commit warnings are the only text adaptations. Selected nonempty UTF-8 <=1 MiB; native empty/non-UTF8/oversize paths are explicit browser preflight refusals. No referenced IO, authentication, OS picker or full parent acceptance.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,cases:records.length}));

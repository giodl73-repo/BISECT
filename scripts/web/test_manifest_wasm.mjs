import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {manifestRequest,readManifestSource} from '../../web/lab/manifest-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/manifest-wasm';await fs.mkdir(root,{recursive:true});let parity=0,cli=0,rejections=0;
const request=t=>({operation:'verify-run-manifest',source_name:'manifest.json',manifest_json:t});
async function native(r){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:16*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r){const out=engine.execute(r),ref=await native(r);const {current_build,...portable}=out,{current_build:host,...nativePortable}=ref;assert.deepEqual(portable,nativePortable);assert.equal(current_build.version,host.version);assert.ok(current_build.build_commit);assert.match(current_build.build_unix,/^\d+$/);assert.match(current_build.rustc_version,/rustc/);parity++;return out;}
const version=engine.execute(request('{}')).current_build.version;
const cases=['','{}','[]','null','12','{"binary_version":null}','{"binary_version":12}','{"binary_version":""}',JSON.stringify({binary_version:version}),'{"binary_version":"999.999.999"}','{','{"binary_version":" '+version+'"}',JSON.stringify({binary_version:version,binary_sha256:'not-a-hash',adjacency_sha256:'',adjacency_file:'https://attacker.invalid/<script>',created_at:'invalid',unexpected:{count:9007199254740993}})];
for(const text of cases){const out=await exact(request(text));await fs.writeFile(root+'/manifest.json',text);const result=spawnSync('target/debug/examples/manifest_reference.exe',[root+'/manifest.json'],{encoding:'utf8'});assert.equal(result.status,out.native_exit_code,result.stderr);cli++;if(out.assessment?.version_status==='missing')assert.match(result.stdout,/no binary_version field/);if(out.assessment?.version_status==='matches')assert.match(result.stdout,/binary_version matches/);if(out.assessment?.version_status==='mismatch')assert.ok(result.stdout.includes(out.assessment.version_error));}
const text=JSON.stringify({binary_version:version,adjacency_file:'C:\\not-supplied\\never-open',adjacency_sha256:'informational'}),output=await exact(request(text));assert.equal(output.status,'version-compatible');assert.equal(output.source_sha256,'sha256:'+createHash('sha256').update(text).digest('hex'));assert.match(output.scope,/No referenced path is opened/);
assert.equal((await exact(request('{}'))).status,'version-unavailable');assert.equal((await exact(request('{'))).status,'invalid-json');
const padded=text+' '.repeat(1024*1024-Buffer.byteLength(text));assert.equal((await exact(request(padded))).status,'version-compatible');
for(const r of [{...request(text),unexpected:true},{...request(text),source_name:''},{...request(text),source_name:'é'.repeat(101)},request(padded+' ')]){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
const file={name:'manifest.json',size:Buffer.byteLength(text),arrayBuffer:async()=>new TextEncoder().encode(text).buffer};const source=await readManifestSource(file);assert.equal(manifestRequest(source).manifest_json,text);
const project=createProject({name:'Manifest compatibility',files:{manifestSource:source},operation:'verify-run-manifest',constraints:[],result:output,lastOperation:'verify-run-manifest'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(manifestRequest(parseProject(JSON.stringify(project)).files.manifestSource).manifest_json,text);
assert.throws(()=>parseProject(JSON.stringify({...project,files:{manifestSource:{name:'x',base64:Buffer.from(padded+' ').toString('base64')}}})));
let read=false;await assert.rejects(()=>readManifestSource({...file,size:1024*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);await assert.rejects(()=>readManifestSource({...file,size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}));await assert.rejects(()=>readManifestSource({...file,size:2}));
assert.equal(countResultTables(output)[0].rows.find(r=>r[0]==='Status')[1],'version-compatible');
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,cli,rejections,scope:output.scope},null,2));console.log(`Manifest WASM: ${parity} native/WASM comparisons, ${cli} actual CLI exit/text checks and ${rejections} matching request rejections passed.`);

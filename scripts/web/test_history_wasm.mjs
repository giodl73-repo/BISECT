import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {encodeBytes,decodePackageFiles} from '../../web/lab/package-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/history-wasm-fixtures';await fs.mkdir(root,{recursive:true});
async function filesAt(dir,prefix='') {const files={};for(const entry of await fs.readdir(dir,{withFileTypes:true})){const key=prefix+entry.name,full=path.join(dir,entry.name);if(entry.isDirectory())Object.assign(files,await filesAt(full,key+'/'));else files[key]=[...await fs.readFile(full)];}return files;}
let sequence=0;
async function reference(request) {const file=path.join(root,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));const result=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(result.status,0,result.stderr);return JSON.parse(result.stdout);}
for(const name of ['l0-rename','l1-split-merge','l2-three-cycle','real-ri-tract-unchanged','l0-missing-unit','l1-bad-weights']) {
  const dir=`docs/fixtures/rhist/${name}`,files=await filesAt(dir),request={operation:'verify-history-files',files};
  const result=engine.execute(request);assert.deepEqual(result,await reference(request));
  const disk=spawnSync('target/release/rhist.exe',['verify',dir,'--format','json'],{encoding:'utf8'});
  assert.equal(disk.status,result.status==='pass'?0:1,disk.stderr);assert.deepEqual(result,JSON.parse(disk.stdout));
  const project=createProject({name,files:{packageFiles:Object.fromEntries(Object.entries(files).map(([key,value])=>[key,encodeBytes(Uint8Array.from(value))]))},operation:request.operation,constraints:[],result,lastOperation:request.operation});
  assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(decodePackageFiles(project.files.packageFiles),Object.assign(Object.create(null),files));
  await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));
  if(result.status==='pass')assert.ok(countResultTables(result)[0].rows.length>0);else assert.ok(countResultTables(result)[0].note);
}
const base=await filesAt('docs/fixtures/rhist/l0-rename');
const source=JSON.parse(Buffer.from(base['sources/source-index.json']).toString())[0].path;
for(const kind of ['source-tamper','missing-source','hash-drift','malformed-json','invalid-utf8','missing-manifest','unsafe-path']) {
  const files=structuredClone(base);
  if(kind==='source-tamper')files[source][0]^=1;
  if(kind==='missing-source')delete files[source];
  if(kind==='hash-drift'){const manifest=JSON.parse(Buffer.from(files['manifest.json']).toString());manifest.package_content_hash='sha256:'+'f'.repeat(64);files['manifest.json']=[...Buffer.from(JSON.stringify(manifest))];}
  if(kind==='malformed-json')files['manifest.json']=[123];
  if(kind==='invalid-utf8')files['units/cycles.ndjson']=[255];
  if(kind==='missing-manifest')delete files['manifest.json'];
  if(kind==='unsafe-path'){const entries=JSON.parse(Buffer.from(files['sources/source-index.json']).toString());entries[0].path='sources/..\\private';files['sources/source-index.json']=[...Buffer.from(JSON.stringify(entries))];}
  const request={operation:'verify-history-files',files},result=engine.execute(request);assert.equal(result.status,'fail',kind);assert.deepEqual(result,await reference(request));
}
const inert=countResultTables({status:'pass',package_id:'hostile',package_content_hash:'x',checks:Array(501).fill('<script>alert(1)</script>')});assert.equal(inert[0].rows.length,500);assert.equal(inert[0].rows[0][0],'<script>alert(1)</script>');
console.log('History verification: six exact disk/native/WASM fixture results, seven matching failed packages, exact bytes and Save/Open, bounded inert check tables passed.');

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {configInspectionRequest,readConfigSource} from '../../web/lab/config-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/config-inspection-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let matches=0,rejections=0;
async function exact(yaml_text){const request={operation:'inspect-algorithm-config',yaml_text,source_name:'example.yml'};const result=engine.execute(request);const path=root+`/request-${matches++}.json`;await fs.writeFile(path,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));assert.deepEqual(result,native);assert.equal(result.yaml_text,yaml_text);assert.equal(result.source_sha256,createHash('sha256').update(yaml_text).digest('hex'));return result;}
const minimum='name: example\nalgorithm:\n  structure: apportion-regions\n';
const defaults=await exact(minimum);assert.deepEqual(defaults.resolved_years,['2020','2010','2000']);assert.equal(defaults.resolved_workers,'4');assert.equal(defaults.declared_engine,'c-ffi');assert.equal(defaults.seed,null);
const maxSeed=await exact(minimum+'  seed: 18446744073709551615\n  engine: metis-core\n');assert.equal(maxSeed.seed,'18446744073709551615');assert.equal(maxSeed.document.algorithm.seed,maxSeed.seed);
await exact(minimum.replaceAll('\n','\r\n'));
const explicit=await exact(minimum+'  weights: county\n  alpha_county: 2.5\n  search: short-burst\n  percentile: 0.3\nworkers: 6\nyears: ["2010"]\nanalysis_types: [demographic, compactness]\n');assert.equal(explicit.resolved_workers,'6');assert.deepEqual(explicit.resolved_analysis_types,['demographic','compactness']);
// Names are declarations at this stage, and must not be presented as validated execution.
const unknown=await exact(minimum.replace('apportion-regions','future-custom-method'));assert.equal(unknown.document.algorithm.structure,'future-custom-method');assert.match(unknown.scope,/does not validate algorithm names/);
async function reject(yaml_text,source_name='example.yml'){const request={operation:'inspect-algorithm-config',yaml_text,source_name};assert.throws(()=>engine.execute(request));const path=root+`/reject-${rejections++}.json`;await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
for(const text of ['',minimum+'extra: true\n',minimum+'  bogus: true\n',minimum+'  seed: -1\n',minimum+'  seed: 18446744073709551616\n',minimum+'  percentile: .nan\n',minimum+'  w_vra: .inf\n','name: [broken',minimum+'name: duplicate\n',minimum+'  seed: 1\n  seed: 2\n',minimum+'workers: -1\n',' '.repeat(256*1024+1)])await reject(text);
await reject(minimum,'../escape.yml');
await reject('\ufeff'+minimum);
let deep='[0]';for(let i=0;i<200;i++)deep='['+deep+']';await reject(minimum+'years: '+deep+'\n');
// Recovery on the same WASM instance after all malformed inputs.
assert.equal(engine.execute({operation:'inspect-algorithm-config',yaml_text:minimum,source_name:'example.yml'}).resolved_workers,'4');
const source=await readConfigSource({name:'example.yml',size:Buffer.byteLength(maxSeed.yaml_text),arrayBuffer:async()=>new TextEncoder().encode(maxSeed.yaml_text).buffer});assert.equal(configInspectionRequest(source).yaml_text,maxSeed.yaml_text);
const project=createProject({name:'Native configuration inspection',files:{configSource:source},operation:'inspect-algorithm-config',constraints:[],result:maxSeed,lastOperation:'inspect-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/example.bisect',JSON.stringify(project));
assert.ok(countResultTables(maxSeed)[0].rows.some(r=>r[0]==='Exact seed'&&r[1]===maxSeed.seed));
let read=false;await assert.rejects(()=>readConfigSource({name:'big.yml',size:256*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readConfigSource({name:'changed.yml',size:3,arrayBuffer:async()=>new Uint8Array([65,66]).buffer}));
console.log(`Configuration inspection: ${matches} native/WASM comparisons, ${rejections} native/WASM rejections; defaults, exact u64 seed, source hashes, recovery and project retention passed.`);

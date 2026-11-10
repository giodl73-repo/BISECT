import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readConfigSource,configValidationRequest} from '../../web/lab/config-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {compareConfigCapability} from './fixtures/config-capability-comparison.mjs';
const root='target/config-validation-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const exec=(exe,args)=>execFileSync(exe,args,{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});
const nativeTool='target/release/examples/execute_tool_request.exe',nativeConfig='target/debug/examples/config_reference.exe';
const modes={'bisect':'edge-weighted','n-way':'metis-vra','geo-section':'geosection','area-section':'areasection','vra-section':'vra-section','apportion-regions':'apportion-regions','compact-bisect':'compact-bisect','moving-knife':'moving-knife','spectral':'spectral','regionalization':'regionalization','flow-construction':'flow-construction'};
const make=(structure='apportion-regions',extra='')=>`name: example\ndescription: "Native config parity"\nalgorithm:\n  structure: ${structure}\n${extra}`;
let matched=0,rejected=0,summaryMatches=0;
const comparisons=[];
const float=n=>Number.isInteger(n)?n+'.0':String(n);
function nativeSearch(s){switch(s.kind){case 'single':return 'Single';case 'multi':return `Multi { seeds: ${s.seeds} }`;case 'convergence-sweep':return `ConvergenceSweep { threshold: ${s.threshold} }`;default:return `${{'short-burst':'ShortBurst','short-burst-forest':'ShortBurstForest','short-burst-merge-split':'ShortBurstMergeSplit'}[s.kind]} { burst_length: ${s.burst_length}, n_bursts: ${s.n_bursts}, p: ${float(s.p)} }`;}}
async function exact(yaml_text,profile='open'){
 const request={operation:'validate-algorithm-config',yaml_text,source_name:'example.yml',profile};
 const result=engine.execute(request),path=root+`/request-${matched}.json`,yamlPath=root+`/source-${matched}.yml`;
 await fs.writeFile(path,JSON.stringify(request));await fs.writeFile(yamlPath,yaml_text);
 comparisons.push(compareConfigCapability(result,JSON.parse(exec(nativeTool,[path])),engine.supportsCMetis));
 const native=JSON.parse(exec(nativeConfig,[yamlPath,profile])),r=result.resolution;
 assert.equal(result.native_summary,native.summary.replace(/^\[OK\] [^\n]*\n/,'[OK] example.yml\n'));summaryMatches++;
 assert.equal(modes[r.split.kind],native.split);assert.equal(r.weights.geographic,native.geographic);assert.equal(r.weights.minority_weighting,native.minority_weighting);assert.equal(r.weights.alpha_county,native.alpha_county);
 assert.deepEqual(r.vertex_constraints.map(v=>v[0].toUpperCase()+v.slice(1)),native.vertex_constraints);
 assert.equal(nativeSearch(r.seeds),native.search);assert.equal({'c-ffi':'CFfi','redist-metis':'RedistMetis','gpmetis':'Gpmetis'}[r.metis.engine],native.engine);assert.equal(r.metis.seed===null?null:String(r.metis.seed),native.seed);assert.equal(result.seed,native.seed);assert.equal(r.metis.ufactor,native.ufactor);assert.equal(r.metis.niter,native.niter);
 const split=r.split.kind;const metisUsed=!['moving-knife','spectral','regionalization','flow-construction'].includes(split);assert.equal(result.metis_engine_used,metisUsed);assert.equal(result.browser_engine_available,!metisUsed||r.metis.engine==='redist-metis'||engine.supportsCMetis&&r.metis.engine==='c-ffi');assert.equal(result.compositor_valid,true);
 const declared=result.document.algorithm.balance_tolerance;
 assert.equal(result.balance_tolerance_percent,declared);
 assert.equal(result.balance_tolerance_source,declared==null?'state-chamber-policy-at-execution':'explicit-yaml-percent');
 if(declared==null) assert.ok(countResultTables(result)[0].rows.some(row=>row.includes('State/chamber policy at execution')));
 matched++;return result;
}
for(const name of ['standard-bisect','nway','ratio-optimal','geosection','ratio-optimal-area','areasection','ratio-optimal-vra','vra-section','prime-factor','apportion-regions','compact-polsby','compact-bisect','moving-knife','spectral','regionalization','flow-construction'])await exact(make(name));
for(const weights of ['unweighted','geographic','county','vra-aligned','proportional'])for(const search of ['single','multi','convergence','short-burst','short-burst-forest','short-burst-merge-split'])await exact(make('standard-bisect',`  weights: ${weights}\n  alpha_county: 2.5\n  search: ${search}\n  convergence_threshold: 17\n`));
for(const backend of ['c-ffi','c','metis-rs','metis-core','rust','gpmetis','subprocess'])await exact(make('apportion-regions',`  engine: ${backend}\n`));
const max=await exact(make('ratio-optimal-area','  area_swing: 1.25\n  seed: 18446744073709551615\n  engine: metis-core\n  balance_tolerance: 0.1\n'),'court');assert.equal(max.resolution.metis.seed,'18446744073709551615');assert.match(max.resolution_json,/18446744073709551615/);
for(const p of [-2,2]){const v=await exact(make('standard-bisect',`  search: short-burst\n  burst_length: 7\n  n_bursts: 3\n  percentile: ${p}\n`));assert.equal(v.resolution.seeds.p,p<0?0:1);}
await exact(make('moving-knife','  mka_orientations: 360\n  mka_metric: polsby-popper\n'));await exact(make('ratio-optimal-vra','  w_vra: 0.7\n'));await exact(make('apportion-regions','  search: convergence\n  convergence_threshold: 37\n'),'court');
// Preserve native parser/converter acceptance; runtime validation is a distinct gate.
await exact(make('standard-bisect','  search: multi\n  seeds: 0\nworkers: 0\n'));
async function reject(yaml_text,profile='open'){
 const request={operation:'validate-algorithm-config',yaml_text,source_name:'example.yml',profile};const path=root+`/reject-${rejected}.json`,yamlPath=root+`/invalid-${rejected}.yml`;await fs.writeFile(path,JSON.stringify(request));await fs.writeFile(yamlPath,yaml_text);
 assert.throws(()=>engine.execute(request));assert.throws(()=>exec(nativeTool,[path]));assert.throws(()=>exec(nativeConfig,[yamlPath,profile]));rejected++;
}
for(const text of [make('unknown'),make('apportion-regions','  weights: unknown\n'),make('apportion-regions','  search: unknown\n'),make('apportion-regions','  engine: unknown\n'),make('apportion-regions','  weights: county\n'),make('apportion-regions','  search: convergence\n'),make('standard-bisect','  weights: partisan\n')])await reject(text);
for(const search of ['multi','short-burst','short-burst-forest','short-burst-merge-split'])await reject(make('standard-bisect',`  search: ${search}\n`),'court');
for(const ratio of [0.10,0.25,0.99,2.01])await reject(make('ratio-optimal-area',`  area_swing: ${ratio}\n`));
await reject(make('apportion-regions','  weights: proportional\n'),'court');
assert.throws(()=>engine.execute({operation:'validate-algorithm-config',yaml_text:make(),source_name:'example.yml',profile:'legal-certificate'}));
const source=await readConfigSource({name:'example.yml',size:Buffer.byteLength(max.yaml_text),arrayBuffer:async()=>new TextEncoder().encode(max.yaml_text).buffer});assert.equal(configValidationRequest(source,{profile:'court'}).profile,'court');
const project=createProject({name:'Native YAML compositor validation',files:{configSource:source,configSettings:{profile:'court'}},operation:'validate-algorithm-config',constraints:[],result:max,lastOperation:'validate-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/example.bisect',JSON.stringify(project));
assert.ok(countResultTables(max)[0].rows.some(r=>r[0]==='Execution available in browser'&&r[1]==='true'));
await exact(make());
const witness=comparisons.find(row=>row.contract==='browser-c-metis-capability-v1');
if(witness)for(const mutate of [v=>v.yaml_text+='\n',v=>v.resolution.metis.seed='1',v=>v.browser_engine_available=false,v=>v.browser_execution_backend='metis-core',v=>v.extra=true]){
 const changed=structuredClone(witness.browser);mutate(changed);assert.throws(()=>compareConfigCapability(changed,witness.native,true),'Capability differences must not hide altered source, seed, backend or extra fields');
}
await fs.writeFile(root+'/capability-comparisons.json',JSON.stringify({supports_c_bridge:engine.supportsCMetis,comparisons},null,2)+'\n');
console.log(`Configuration validation: ${matched} native compositor comparisons with explicit browser-capability checks, ${summaryMatches} identical native summaries, ${rejected} three-way semantic/profile rejections; exact seed, aliases, defaults, clamping and inert project retention passed.`);

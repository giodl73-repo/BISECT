import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/optional-seed-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const size=6,n=36,adjacency=[],edges=[];
for(let i=0;i<n;i++){const ns=[];if(i%size)ns.push(i-1);if(i%size<size-1)ns.push(i+1);if(i>=size)ns.push(i-size);if(i<n-size)ns.push(i+size);adjacency.push(ns);for(const j of ns)if(i<j)edges.push([i,j,1+((i*17+j*13)%11)]);}
const graph={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:n},(_,i)=>'19001'+String(i).padStart(6,'0')),adjacency,population:Array(n).fill(100),edges,areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-93+i%size*.01,42+Math.floor(i/size)*.01])};
const base={weights:'geographic',seed:null,seeds:4,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:100};
const native=(exe,path)=>JSON.parse(execFileSync(exe,[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
const cases=[],plans=new Map();let exportChecked=false;
for(const [structure,search] of [['standard-bisect','single'],['standard-bisect','multi'],['nway','single'],['ratio-optimal','single'],['ratio-optimal','multi'],['ratio-optimal-area','single'],['ratio-optimal-area','multi']])for(const districts of [2,4])for(const seed of [null,0,42,'18446744073709551615']){
 const options={...base,structure,search,districts,seed},yaml_text=`name: seed\nalgorithm:\n  structure: ${structure}\n  weights: geographic\n  search: ${search}\n  seeds: 4\n  engine: metis-core\n  balance_tolerance: 25\n${seed==null?'':`  seed: ${seed}\n`}`;
 const request={graph,options},path=`${root}/case-${cases.length}.json`;await fs.writeFile(path,JSON.stringify({...request,yaml_text}));
 const result=engine.execute(request),reference=native('target/release/examples/config_seed_reference.exe',path);
 assert.deepEqual(result.assignments,reference.assignments);assert.equal(result.options.seed,seed);assert.equal(reference.declared_seed,seed==null?null:String(seed));
 const typedPath=`${root}/typed-${cases.length}.json`;await fs.writeFile(typedPath,JSON.stringify(request));assert.deepEqual(result,native('target/release/examples/execute_request.exe',typedPath));
 if(seed===null){const omitted=structuredClone(request);delete omitted.options.seed;assert.deepEqual(engine.execute(omitted),result);
  if(structure==='standard-bisect'){const tuned=engine.execute({...request,options:{...options,metis_objective:'cut',metis_trials:1}});assert.deepEqual(tuned.assignments,result.assignments);if(search==='single')assert.equal(tuned.metrics.structure_evidence.seed_schedule,'backend-default-at-each-node');}
 }
 plans.set(`${structure}/${search}/${districts}/${seed}`,result.assignments);
 if(!exportChecked&&seed===null){const q={operation:'export-engine-plan',request,assignments:result.assignments,label:'Native omitted seed',chamber:'congressional',created_at:'2026-10-07T00:00:00Z'},p=root+'/export.json';await fs.writeFile(p,JSON.stringify(q));assert.deepEqual(engine.execute(q),native('target/release/examples/execute_tool_request.exe',p));exportChecked=true;}
 cases.push({structure,search,districts,declared_seed:seed,assignments:result.assignments});
}
let differs=0;for(const [key,plan] of plans)if(key.endsWith('/null')&&JSON.stringify(plan)!==JSON.stringify(plans.get(key.replace(/null$/,'0'))))differs++;
assert.ok(differs>0,'None must not be silently converted to zero for optional METIS paths.');
const ensembleOptions={...base,structure:'standard-bisect',search:'bisection-ensemble',districts:4,steps:4};
const plainEnsemble=engine.execute({graph,options:ensembleOptions}),tunedEnsemble=engine.execute({graph,options:{...ensembleOptions,metis_objective:'cut',metis_trials:1}});
assert.deepEqual(plainEnsemble.assignments,tunedEnsemble.assignments);
let rejected=0;for(const seed of [-1,-0,1.5,NaN,Infinity,'01','-1','18446744073709551616',{},[]]){assert.throws(()=>engine.execute({graph,options:{...base,structure:'standard-bisect',search:'single',districts:2,seed}}));rejected++;}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,none_zero_differing_cases:differs,rejected,exportChecked},null,2));
console.log(`Optional seeds: ${cases.length} actual native YAML dispatcher/WASM matches, explicit/null/omitted seed fidelity, ${differs} None-versus-zero distinctions, exact u64, native plan export and ${rejected} boundary rejections passed.`);

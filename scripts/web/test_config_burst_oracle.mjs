import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/config-burst-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/config_seed_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const variants=['short-burst','short-burst-forest','short-burst-merge-split'],cases=[];let failures=0;
async function check(g,search,length,bursts,percentile,seed,weights,k=4,real=false){
 const yaml_text=`name: burst_oracle\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  weights: ${weights}\n  search: ${search}\n  burst_length: ${length}\n  n_bursts: ${bursts}\n  percentile: ${percentile}\n  balance_tolerance: 25\n${seed===null?'':`  seed: ${seed}\n`}`;
 const input={graph:g,yaml_text,chamber:'congressional',districts:k,source_name:'burst.yml',profile:'open'};
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,yaml_text,options:{districts:k}}));const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({operation:'run-algorithm-config',input});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error));failures++;cases.push({state:g.state,search,length,bursts,percentile,seed,weights,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.result.assignments,reference.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.declared_seed,seed);
 assert.equal(actual.result.metrics.weighted_boundary,reference.weighted_cut);assert.equal(actual.result.metrics.graph_boundary_m,reference.physical_cut);
 const o=actual.engine_request.options,e=actual.result.metrics.structure_evidence;assert.equal(o.burst_length,length);assert.equal(o.n_bursts,bursts);assert.equal(o.percentile,Math.max(0,Math.min(1,percentile)));assert.equal(e.total_proposals,length*bursts);assert.equal(e.chain,search);
 assert.equal(actual.configuration.yaml_text,yaml_text);assert.equal(actual.configuration.document.algorithm.percentile,percentile);
 if(!length||!bursts)assert.deepEqual(actual.result.assignments,engine.execute({...actual.engine_request,options:{...o,seed:o.seed??0,search:'single',burst_length:undefined,n_bursts:undefined}}).assignments);
 const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
 const project=createProject({name:'Burst oracle '+search,files:{configSource:source('burst.yml',yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(g))},configRunSettings:{chamber:'congressional',districts:k}},operation:'run-algorithm-config',constraints:[],result:actual,lastOperation:'run-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)).result,actual);
 if(!real&&search==='short-burst'&&length===2&&bursts===3&&percentile===.5&&seed==='18446744073709551615'&&weights==='geographic')await fs.writeFile(root+'/browser-fixture.bisect',JSON.stringify(project));
 cases.push({state:g.state,units:g.geoids.length,search,length,bursts,declared_percentile:percentile,effective_percentile:o.percentile,seed,weights,k,real,balanced:reference.balanced,contiguous:actual.result.metrics.contiguous});
}
for(const search of variants)for(const [length,bursts]of [[0,3],[2,0],[2,3],[5,3]])for(const percentile of [-.25,.5,1.25])for(const seed of [null,'42','18446744073709551615'])for(const weights of ['geographic','unweighted'])await check(graph,search,length,bursts,percentile,seed,weights);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const search of variants)for(const [length,bursts]of [[0,3],[2,3]])await check(g,search,length,bursts,.5,'18446744073709551615','geographic',k,true);}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_cli_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_cli_assignment_matches:cases.length-failures,matching_failures:failures,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,scope:'Native YAML conversion and actual burst dispatch on identical supplied graphs; geometry acquisition, arbitrary capacities and calibrated ensemble claims remain unproven.',cases_evidence:cases},null,2));
console.log(`Configured bursts: ${cases.length-failures} actual CLI/WASM assignment/balance/cut matches, ${failures} matching failures; three variants, zero work, percentile clamping, exact seeds and inert projects.`);

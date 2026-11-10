import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {engineReplayRequest,engineReplayReceiptBytes} from '../../web/lab/engine-replay-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/multiscale-seed-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input,g=base.request.graph;
const bg={schema_version:'bisect-block-group-graph-v1',state:g.state,year:g.year,geoids:g.geoids.flatMap(id=>[id+'1',id+'2']),population:g.population.flatMap(p=>[p/2,p/2]),adjacency:Array.from({length:g.geoids.length*2},(_,i)=>[i^1,...g.adjacency[Math.floor(i/2)].map(j=>j*2+i%2)].sort((a,b)=>a-b))};
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});let cases=0,nativeComparisons=0;
async function host(name,req,out){if(process.argv.includes('--skip-native'))return;const file=root+'/'+name+'.json';await fs.writeFile(file,JSON.stringify(req));assert.deepEqual(out,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024})));nativeComparisons++;}
async function check(name,input,build=false){
 const generation={operation:build?'run-multiscale-build':'run-multiscale',input},generated=engine.execute(generation);await host(name+'-generation',generation,generated);
 const artifact={kind:build?'multiscale-build':'multiscale',input},inputs={run:source('selected-run.json',JSON.stringify(artifact)),original:source('assignments.json',JSON.stringify(generated.assignments)),minSimilarity:1};
 const req=engineReplayRequest(inputs),out=engine.execute(req);await host(name+'-replay',req,out);assert.equal(out.status,'match',out.error);assert.deepEqual(out.replayed_result,generated);
 const declared=generated.evidence.seed;assert.equal(out.replayed_seed,declared);assert.equal(out.effective_seed,declared??0);assert.match(out.seed_policy,/zero when no seed/);assert.equal(artifact.input.request.options.seed,input.request.options.seed);
 if(build)assert.equal(String(out.effective_seed),String(generated.build_report.attempts.at(-1).seed??0));
 const rows=countResultTables(out)[0].rows;assert.equal(rows.find(([label])=>label==='Replayed seed')[1],declared===null?'—':String(declared));assert.equal(rows.find(([label])=>label==='Effective seed')[1],String(declared??0));
 const project=createProject({name,operation:'replay-engine-run',lastOperation:'replay-engine-run',constraints:[],files:{engineReplayInputs:inputs},result:out});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(JSON.parse(new TextDecoder().decode(engineReplayReceiptBytes(out))),out);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));cases++;return out;
}
for(const [fine,coarse]of [['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of [false,true])for(const seed of [null,42,'18446744073709551615'])for(const steps of [0,5]){
 const input=structuredClone(base);Object.assign(input,{fine_level:fine,coarse_level:coarse,total_steps:steps,alpha:.3,percentile:0,block_groups:fine==='bg'?bg:null,adaptive:adaptive?{target_accept:.3,adapt_interval:2,gamma_0:.1,coarse_tol_factor:1}:null});Object.assign(input.request.options,{districts:2,balance_tolerance:50,seed});
 const name=fine+'-'+coarse+'-'+adaptive+'-'+String(seed)+'-'+steps;await check(name,input);
 if(steps===0&&seed==='18446744073709551615')await check(name+'-build',input,true);
}
for(const seed of [null,'18446744073709551615']){
 const input=structuredClone(base);input.request.graph.population=g.population.map((_,i)=>100+(i*37)%67);Object.assign(input.request.options,{districts:4,balance_tolerance:.5,seed});Object.assign(input,{fine_level:'tract',coarse_level:'county',total_steps:0,block_groups:null,adaptive:null});
 const result=await check('accepted-retry-'+String(seed),input,true);assert.equal(result.build_report.accepted_attempt,seed===null?41:42);assert.equal(result.effective_seed,41);assert.equal(result.replayed_seed,41);assert.equal(JSON.parse(Buffer.from((JSON.parse(await fs.readFile(root+'/accepted-retry-'+String(seed)+'.bisect','utf8'))).files.engineReplayInputs.run.base64,'base64')).input.request.options.seed,seed);
}
for(const [name,seed]of [['raw-u64','18446744073709551615'],['missing-field',null]]){
 const fixture=JSON.parse(await fs.readFile(root+'/tract-county-false-'+String(seed)+'-0.bisect','utf8')),inputs=fixture.files.engineReplayInputs;
 if(seed===null){const artifact=JSON.parse(Buffer.from(inputs.run.base64,'base64').toString());delete artifact.input.request.options.seed;inputs.run=source('omitted-seed.json',JSON.stringify(artifact));}
 else inputs.run=source('numeric-seed.json',Buffer.from(inputs.run.base64,'base64').toString().replace('"seed":"18446744073709551615"','"seed":18446744073709551615'));
 const req=engineReplayRequest(inputs),out=engine.execute(req);await host(name,req,out);assert.equal(out.status,'match');assert.equal(out.replayed_seed,seed);assert.equal(out.effective_seed,seed??0);assert.deepEqual(out.replayed_result,fixture.result.replayed_result);cases++;
}
const invalid=structuredClone(base);invalid.total_steps=100001;const inputs={run:source('invalid.json',JSON.stringify({kind:'multiscale',input:invalid})),original:source('original.json',JSON.stringify(Object.fromEntries(g.geoids.map(id=>[id,1])))),minSimilarity:1},req=engineReplayRequest(inputs),out=engine.execute(req);await host('invalid',req,out);assert.equal(out.status,'failed');assert.equal(out.replayed_seed,null);assert.equal(Object.hasOwn(out,'effective_seed'),false);assert(!countResultTables(out)[0].rows.some(([label])=>label==='Effective seed'));
await fs.writeFile(root+'/evidence.json',JSON.stringify({replay_cases:cases,native_host_comparisons:nativeComparisons,all_resolution_pairs:true,fixed_adaptive_build:true,raw_numeric_max_u64:true,omitted_seed_field:true,declared_null_effective_zero:true,accepted_retry_cases:[{original_seed:null,accepted_attempt:41,effective_seed:41},{original_seed:'18446744073709551615',accepted_attempt:42,effective_seed:41}],failed_replay_claims_no_effective_seed:true,source_project_and_receipt_retention:true,scope:'Synthetic graph seed receipt parity; full native State/States orchestration and WASM capability coverage remain unfinished.'},null,2)+'\n');
console.log(`Multiscale seeds: ${cases} replay cases and independent generation receipts match native host/WASM across all resolution pairs, fixed/adaptive/build, omitted/42/max-u64 seeds and zero/nonzero sampling; declared null preserved, effective zero disclosed, failed runs do not claim a seed, project/source/receipt and visible tables passed (native comparisons when enabled).`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {doctorRequest,readDoctorContext,validateDoctorInputs} from '../../web/lab/doctor-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/doctor-wasm';await fs.mkdir(root,{recursive:true});
const database=JSON.parse(await fs.readFile('data/location_policy.json','utf8'));
let parity=0,registryChecks=0,cliChecks=0,rejections=0;
const settings={state:'WA',year:'2020',chamber:'house',resolution:'tract'};
const request=(s=settings,text=null,policy=null)=>({operation:'diagnose-location-inputs',settings:s,source_name:policy===null?null:'policy.json',policy_json:policy,context_source_name:text===null?null:'grid.rctx',context_json:text});
async function native(r){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:64*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r){const out=engine.execute(r);assert.deepEqual(out,await native(r));parity++;return out;}
async function reject(r){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
// Native CLI registry and runner are the metadata oracle, across all locations,
// available/invalid years, chamber aliases and the actual granularity boundary.
const cases=[];
for(const [state,entry]of Object.entries(database).filter(([,v])=>v?.name))for(const year of [...(entry.census_years?.length?entry.census_years:['2020']),'9999'])for(const chamber of ['congressional','house','senate'])for(const resolution of ['tract','block_group'])cases.push({state:state.toLowerCase(),year,chamber,resolution});
for(const chamber of ['lower','assembly','upper','unknown'])cases.push({...settings,chamber});
cases.push({...settings,state:'ZZ'});
const casesPath=resolve(root+'/cases.json');await fs.writeFile(casesPath,JSON.stringify(cases));
const reference=JSON.parse(execFileSync('target/debug/examples/doctor_reference.exe',[casesPath],{encoding:'utf8',maxBuffer:16*1024*1024,stdio:['ignore','pipe','pipe']}));
for(let i=0;i<cases.length;i++){
 const out=engine.execute(request(cases[i]));const {district_count_source,...metadata}=out.metadata;
 assert.deepEqual(metadata,reference[i],JSON.stringify(cases[i]));assert.equal(out.status,metadata.registered&&metadata.year_valid?'needs-inputs':'invalid-inputs');registryChecks++;
}
for(const state of ['WA','RI','NV','IE']){
 const s={...settings,state,year:database[state].census_years?.[0]??'2020'};
 await fs.writeFile(casesPath,JSON.stringify([s]));
 const text=execFileSync('target/debug/examples/doctor_reference.exe',[casesPath,resolve('data/location_policy.json'),'doctor'],{encoding:'utf8',maxBuffer:8*1024*1024,stdio:['ignore','pipe','pipe']});
 const out=await exact(request(s));assert.ok(text.includes(`Location '${state}' is registered.`));assert.ok(text.includes(`Year '${s.year}' is valid`));
 if(out.metadata.districts)assert.ok(text.includes(`${state} ${s.chamber} districts: ${out.metadata.districts}`));
 assert.ok(text.includes(`Balance tolerance: ${out.metadata.policy_tolerance_percent.toFixed(1)}%`));if(out.metadata.granularity_warning)assert.ok(text.includes(out.metadata.granularity_warning));cliChecks++;
}
const canonical=v=>typeof v==='bigint'?v.toString():Array.isArray(v)?'['+v.map(canonical).join(',')+']':v&&typeof v==='object'?'{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}':JSON.stringify(v);
const hash=v=>'sha256:'+createHash('sha256').update(canonical(v)).digest('hex');
const rehash=c=>{const projection={units:c.units,graph:c.graph??null,populations:c.populations??null,source_hashes:c.source_hashes};for(const k of ['subdivisions','demographics','geometry'])if(c[k]!=null)projection[k]=c[k];c.context_hash=hash(projection);return c;};
const context=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'));
assert.equal(rehash(structuredClone(context)).context_hash,context.context_hash);
const policy=JSON.stringify({WA:{name:'Synthetic Washington fixture',house_districts:3,census_years:['2020'],house_balance_tolerance:5}});
const text=canonical(context),out=await exact(request(settings,text,policy));assert.equal(out.status,'checks-passed');assert.equal(out.context.components,1);assert.equal(out.context.unit_count,9);assert.equal(out.context.total_population,String(context.populations.reduce((a,b)=>a+b,0)));
assert.equal(out.context_source_sha256,'sha256:'+createHash('sha256').update(text).digest('hex'));
assert.match(out.scope,/do not execute an engine/);assert.ok(out.checks.some(c=>c.check==='map-geometry'&&c.status==='not-checked'));
assert.equal((await exact(request(settings,text))).status,'invalid-inputs'); // 98 house districts exceed 9 units.
for(const change of [{state:'RI'},{year:'2010'},{resolution:'block_group'}])assert.equal((await exact(request({...settings,...change},text,policy))).status,'invalid-inputs');
for(const mutation of [c=>c.context_hash='sha256:wrong',c=>c.populations[0]++,c=>{c.units.unit_universe_hash='sha256:wrong';rehash(c);},c=>{c.graph.adjacency[0][0].to=99;rehash(c);},c=>{c.graph.adjacency[0].push(c.graph.adjacency[0][0]);rehash(c);},c=>{c.graph.adjacency[0].shift();rehash(c);},c=>{c.populations[0]=-1;rehash(c);},c=>{c.populations.fill(0);rehash(c);}]){const c=structuredClone(context);mutation(c);assert.equal((await exact(request(settings,canonical(c),policy))).status,'invalid-inputs');}
for(const key of ['graph','populations']){const c=structuredClone(context);delete c[key];rehash(c);assert.equal((await exact(request(settings,canonical(c),policy))).status,'needs-inputs');}
const isolated=structuredClone(context);isolated.graph.adjacency=Array.from({length:9},()=>[]);rehash(isolated);assert.equal((await exact(request(settings,canonical(isolated),policy))).status,'invalid-inputs');
const bridges=structuredClone(context);for(const row of bridges.graph.adjacency)for(const edge of row)edge.kind='bridge';rehash(bridges);assert.equal((await exact(request(settings,canonical(bridges),policy))).status,'checks-passed');
const huge=structuredClone(context);huge.populations=Array(9).fill(9007199254740993n);rehash(huge);assert.equal((await exact(request(settings,canonical(huge),policy))).context.total_population,'81064793292668937');
const bigText=text+' '.repeat(3*1024*1024);assert.equal((await exact(request(settings,bigText,policy))).status,'checks-passed');
const boundary=text+' '.repeat(8*1024*1024-Buffer.byteLength(text));assert.equal((await exact(request(settings,boundary,policy))).status,'checks-passed');
for(const r of [request({...settings,state:''}),request({...settings,unexpected:true}),{...request(),unexpected:true},{...request(),context_source_name:'x'},request(settings,boundary+' ',policy),request(settings,null,' '.repeat(512*1024+1)),request(settings,null,'[]')])await reject(r);
assert.equal((await exact(request(settings,'{',policy))).status,'invalid-inputs');
const file=(name,t)=>({name,size:Buffer.byteLength(t),arrayBuffer:async()=>new TextEncoder().encode(t).buffer});
const inputs={settings,policy:{name:'policy.json',base64:Buffer.from(policy).toString('base64')},context:await readDoctorContext(file('grid.rctx',bigText))};
assert.equal(doctorRequest(inputs).context_json,bigText);
const project=createProject({name:'Synthetic location preflight',files:{doctorInputs:inputs},operation:'diagnose-location-inputs',constraints:[],result:out,lastOperation:'diagnose-location-inputs'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(doctorRequest(parseProject(JSON.stringify(project)).files.doctorInputs).context_json,bigText);
assert.throws(()=>validateDoctorInputs({...inputs,url:'https://attacker.invalid'}));assert.throws(()=>validateDoctorInputs({...inputs,settings:{...settings,extra:true}}));
let read=false;await assert.rejects(()=>readDoctorContext({name:'big.rctx',size:8*1024*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readDoctorContext({name:'invalid.rctx',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}));await assert.rejects(()=>readDoctorContext({...file('changed.rctx',text),size:2}));
assert.ok(countResultTables(out).some(t=>t.rows.some(r=>r.includes('checks-passed'))));
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/report.json',JSON.stringify(out,null,2));
await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,registryChecks,cliChecks,rejections,scope:'Native metadata/CLI rendering; RCTX identities, schema/hashes, population and graph diagnosis, exact large integers, source bounds and inert project roundtrip. No balanced-feasibility or legal certification claim.'},null,2));
console.log(`Doctor WASM: ${registryChecks} native registry comparisons, ${cliChecks} actual CLI checks, ${parity} native/WASM comparisons and ${rejections} matching request rejections passed.`);

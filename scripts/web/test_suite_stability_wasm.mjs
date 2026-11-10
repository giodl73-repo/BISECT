import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/suite-stability-wasm';await fs.mkdir(root,{recursive:true});let matched=0,rejected=0,cliMatched=0;
const original=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
const text=p=>JSON.stringify(p);
const request=plans=>({operation:'analyze-suite-stability',plans_json:plans});
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
function rehash(p){const u=p.plan.units;u.unit_universe_hash='sha256:'+createHash('sha256').update(JSON.stringify(Object.fromEntries(Object.entries(u).filter(([k])=>k!=='unit_universe_hash').sort(([a],[b])=>a.localeCompare(b))))).digest('hex');return p;}
const base=request({'a.rplan':text(original),'b.rplan':text(original)});
const identical=await exact(base);assert.equal(identical.report.stable,true);assert.equal(identical.report.mean_similarity,1);
const renumbered=structuredClone(original);renumbered.plan.assignment=renumbered.plan.assignment.map(d=>(d+1)%renumbered.plan.k);
const renamedReq=request({'a.rplan':text(original),'b.rplan':text(renumbered)});const renamed=await exact(renamedReq);assert.equal(renamed.report.mean_similarity,0);assert.equal(renamed.report.stable,false);
const moved=structuredClone(original);moved.plan.assignment[0]=(moved.plan.assignment[0]+1)%moved.plan.k;
const threeReq=request({'a.rplan':text(original),'b.rplan':text(original),'c.rplan':text(moved)});const three=await exact(threeReq);assert.equal(three.report.num_pairs,3);assert.equal(three.report.unstable_pairs.length,2);
const reordered=structuredClone(original);reordered.plan.units.unit_ids.reverse();reordered.plan.assignment.reverse();reordered.plan.units.canonical_order='explicit-unit-ids';rehash(reordered);assert.equal((await exact(request({'a.rplan':text(original),'b.rplan':text(reordered)}))).report.mean_similarity,1);
for(const state of ['RI','IA','NC']){const p=JSON.parse(await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8'));const q=structuredClone(p);q.plan.assignment=q.plan.assignment.map(d=>(d+1)%q.plan.k);assert.equal((await exact(request({'a.rplan':text(p),'b.rplan':text(q)}))).report.mean_similarity,0);}
for(const req of [base,renamedReq,threeReq]){
 const dir=root+'/cli-'+cliMatched;for(const [label,source]of Object.entries(req.plans_json)){const p=JSON.parse(source).plan;const folder=dir+'/v1/2020/plans/'+label+'/data';await fs.mkdir(folder,{recursive:true});await fs.writeFile(folder+'/final_assignments.json',JSON.stringify(Object.fromEntries(p.units.unit_ids.map((id,i)=>[id,p.assignment[i]+1]))));}
 let stdout;try{stdout=execFileSync('target/debug/bisect.exe',['suite','stability','--labels',Object.keys(req.plans_json).join(','),'--version','v1','--year','2020','--output-base',dir],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});}catch(e){assert.equal(e.status,6);stdout=e.stdout;}
 assert.equal(stdout.trimEnd(),engine.execute(req).report_json);cliMatched++;
}
for(const mutate of [p=>p.plan.units.state='OR',p=>p.plan.units.year=2010,p=>p.plan.units.unit_ids[0]='unrelated',p=>{p.plan.units.unit_ids.pop();p.plan.assignment.pop();},p=>p.plan.units.unit_ids[1]=p.plan.units.unit_ids[0]]){const bad=structuredClone(original);mutate(bad);rehash(bad);await reject(request({'a.rplan':text(original),'b.rplan':text(bad)}));}
await reject(request({}));await reject(request({'one.rplan':text(original)}));await reject(request(Object.fromEntries(Array.from({length:33},(_,i)=>[i+'.rplan',text(original)]))));await reject(request({'../bad.rplan':text(original),'b.rplan':text(original)}));await reject(request({'a.rplan':text(original)+' '.repeat(3*1024*1024),'b.rplan':text(original)}));
const wrongHash=structuredClone(original);wrongHash.plan.units.unit_universe_hash='sha256:wrong';await reject(request({'a.rplan':text(original),'b.rplan':text(wrongHash)}));
await fs.writeFile(root+'/fixture.json',JSON.stringify({base:threeReq,output:three}));await fs.writeFile(root+'/stability.json',three.report_json);
console.log(`Suite stability: ${matched} native/WASM matches (three derived real-state label permutations), ${cliMatched} actual CLI report byte matches, ${rejected} rejected inputs. Exact-label metric, stable/unstable pairs, ID reordering and bounded source identity verified.`);

const {createProject}=await import('../../web/lab/project.js');const {stabilityRequest,validateStabilityInputs,readStabilityPlans}=await import('../../web/lab/stability-files.js');const {countResultTables}=await import('../../web/lab/count-results.js');
const sources={plans:Object.fromEntries(Object.entries(threeReq.plans_json).map(([n,s])=>[n,{name:n,base64:Buffer.from(s).toString('base64')}]))};
assert.deepEqual(stabilityRequest(sources),threeReq);assert.equal(countResultTables(three)[1].rows.length,3);assert.throws(()=>validateStabilityInputs({plans:{'../bad.rplan':sources.plans['a.rplan']}}));assert.throws(()=>stabilityRequest({plans:{}}));
let duplicateRead=false;const duplicate={name:'a.rplan',size:1,arrayBuffer:async()=>{duplicateRead=true;}};await assert.rejects(()=>readStabilityPlans([duplicate,duplicate]));assert.equal(duplicateRead,false);
const project=createProject({name:'Suite label stability fixture',files:{stabilityInputs:sources},operation:threeReq.operation,constraints:[],result:three,lastOperation:threeReq.operation});assert.deepEqual(stabilityRequest(project.files.stabilityInputs),threeReq);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
console.log('Stability project fixture: exact plan byte retention, all-pair tables, name/completeness and duplicate pre-read guards passed.');

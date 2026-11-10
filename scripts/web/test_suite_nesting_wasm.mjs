import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const {validateSuiteInputs,suiteNestingRequest,readSuiteSource}=await import('../../web/lab/suite-files.js');
const {createProject}=await import('../../web/lab/project.js');
const {countResultTables}=await import('../../web/lab/count-results.js');
const root='target/suite-nesting-wasm';await fs.mkdir(root,{recursive:true});let matched=0,rejected=0,cliMatched=0,real=0;
const original=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
function plan(assignment,chamber){const p=structuredClone(original);p.plan.assignment=assignment;p.plan.k=Math.max(...assignment)+1;p.metadata.chamber=chamber;return p;}
const house=plan([0,0,0,1,1,2,2,3,3],'house'),senate=plan([0,0,0,0,0,1,1,1,1],'senate');
const base={operation:'validate-suite-nesting',house_json:JSON.stringify(house),senate_json:JSON.stringify(senate),suite_name:'fixture',required_ratio:null};
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
function rehash(p){const u=p.plan.units;u.unit_universe_hash='sha256:'+createHash('sha256').update(JSON.stringify(Object.fromEntries(Object.entries(u).filter(([k])=>k!=='unit_universe_hash').sort(([a],[b])=>a.localeCompare(b))))).digest('hex');return p;}
const out=await exact(base);assert.equal(out.report.valid,true);assert.equal(out.required_ratio,2);assert.deepEqual(out.report.nesting.senate_to_house_map,{'1':[1,2],'2':[3,4]});
assert.equal((await exact({...base,required_ratio:1})).report.valid,false);
const split=structuredClone(senate);split.plan.assignment[1]=1;const splitReq={...base,senate_json:JSON.stringify(split)};assert.equal((await exact(splitReq)).report.valid,false);
const reordered=structuredClone(senate);reordered.plan.units.canonical_order='explicit-unit-ids';reordered.plan.units.unit_ids.reverse();reordered.plan.assignment.reverse();rehash(reordered);assert.equal((await exact({...base,senate_json:JSON.stringify(reordered)})).report.valid,true);
const threeHouse=plan([0,0,1,1,2,3,4,5,5],'house');const ratioThree={...base,house_json:JSON.stringify(threeHouse)};assert.equal((await exact(ratioThree)).required_ratio,3);
for(const req of [base,splitReq,ratioThree]){
 const dir=root+'/cli-'+cliMatched;for(const [label,text] of [['house',req.house_json],['senate',req.senate_json]]){const p=JSON.parse(text).plan;const folder=dir+'/v1/2020/plans/'+label+'/data';await fs.mkdir(folder,{recursive:true});await fs.writeFile(folder+'/final_assignments.json',JSON.stringify(Object.fromEntries(p.units.unit_ids.map((id,i)=>[id,p.assignment[i]+1]))));}
 let stdout;try{stdout=execFileSync('target/debug/bisect.exe',['suite','validate','--name','fixture','--version','v1','--year','2020','--output-base',dir,'--house-label','house','--senate-label','senate'],{encoding:'utf8',stdio:['ignore','pipe','pipe']});}catch(e){assert.equal(e.status,5);stdout=e.stdout;}
 assert.equal(stdout.trimEnd(),engine.execute(req).report_json);cliMatched++;
}
for(const state of ['RI','IA','NC']){const source=JSON.parse(await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8'));const s=structuredClone(source),h=structuredClone(source),seen={};s.metadata.chamber='senate';h.metadata.chamber='house';h.plan.assignment=s.plan.assignment.map(d=>2*d+((seen[d]=(seen[d]||0)+1)%2));h.plan.k=s.plan.k*2;const result=await exact({...base,house_json:JSON.stringify(h),senate_json:JSON.stringify(s)});assert.equal(result.report.valid,true);assert.equal(result.required_ratio,2);real++;}
for(const mutate of [p=>p.plan.units.state='OR',p=>p.plan.units.year=2010,p=>p.plan.units.unit_ids[0]='53001009999',p=>{p.plan.units.unit_ids.pop();p.plan.assignment.pop();},p=>p.plan.units.unit_ids[1]=p.plan.units.unit_ids[0]]){const p=structuredClone(senate);mutate(p);rehash(p);await reject({...base,senate_json:JSON.stringify(p)});}
await reject({...base,required_ratio:0});await reject({...base,required_ratio:501});await reject({...base,suite_name:''});
const badHash=structuredClone(senate);badHash.plan.units.unit_universe_hash='sha256:wrong';await reject({...base,senate_json:JSON.stringify(badHash)});
await reject({...base,house_json:base.house_json+' '.repeat(3*1024*1024)});
const empty=structuredClone(senate);empty.plan.units.unit_ids=[];empty.plan.assignment=[];rehash(empty);await reject({...base,senate_json:JSON.stringify(empty)});
await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output:out}));await fs.writeFile(root+'/validate-suite.json',out.report_json);
console.log(`Suite nesting: ${matched} native/WASM matches (${real} derived pairs using real state unit IDs), ${cliMatched} CLI report byte matches, ${rejected} rejections; whole-House containment, ratios, ID reordering, unit/state/year integrity and scope limits.`);

const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={house:source('house.rplan',base.house_json),senate:source('senate.rplan',base.senate_json),suite_name:base.suite_name,required_ratio:null};
assert.deepEqual(suiteNestingRequest(inputs),base);assert.equal(countResultTables(out)[1].rows.length,2);
assert.equal(countResultTables(engine.execute({...base,required_ratio:1}))[2].rows.length,2);
assert.throws(()=>validateSuiteInputs({...inputs,suite_name:'é'.repeat(101)}));assert.throws(()=>validateSuiteInputs({...inputs,required_ratio:0}));assert.throws(()=>suiteNestingRequest({...inputs,house:null}));
let oversizedRead=false;await assert.rejects(()=>readSuiteSource({size:3*1024*1024+1,arrayBuffer:async()=>{oversizedRead=true;}}));assert.equal(oversizedRead,false);
await assert.rejects(()=>readSuiteSource({name:'wrong.rplan',size:1,arrayBuffer:async()=>Buffer.from('two').buffer}));
const geometry={type:'FeatureCollection',features:house.plan.units.unit_ids.map((id,i)=>{const x=i%3,y=Math.floor(i/3);return {type:'Feature',properties:{geoid:id},geometry:{type:'Polygon',coordinates:[[[x,y],[x+0.9,y],[x+0.9,y+0.9],[x,y+0.9],[x,y]]]}};})};
const project=createProject({name:'House–Senate nesting fixture',files:{suiteInputs:inputs,planGeometry:source('synthetic-grid.geojson',JSON.stringify(geometry))},operation:base.operation,constraints:[],result:out,lastOperation:base.operation});
assert.deepEqual(suiteNestingRequest(project.files.suiteInputs),base);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
console.log('Suite browser fixture: exact project source bytes, native result tables, UTF-8 name limit, ratio/completeness and pre-read/dishonest size guards passed.');

const {planMapModel}=await import('../../web/lab/plan-map.js');
for(const text of [base.house_json,base.senate_json]){const validation=engine.execute({operation:'validate-rplan-text',document_json:text});assert.equal(planMapModel(JSON.parse(validation.document_json),geometry).units,9);}
console.log('Suite chamber map handoff: both exact plans validate and join all nine synthetic unit polygons.');

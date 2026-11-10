import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/plan-statistics-wasm';await fs.mkdir(root,{recursive:true});let matched=0,reports=0,rejected=0;
const plan=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8')),ids=plan.plan.units.unit_ids;
const header='GEOID,total_pop,white_non_hispanic,black_non_hispanic,asian_non_hispanic,hispanic,other\n';
const demo=p=>header+p.plan.units.unit_ids.map(id=>id+',100,40,25,5,20,10').join('\n')+'\n';
const election=p=>'geoid,dem_votes,rep_votes\n'+p.plan.units.unit_ids.map(id=>id+',60,40').join('\n');
const base={operation:'analyze-plan-statistics',document_json:JSON.stringify(plan),demographic_csv:demo(plan),urban_csv:`GEOID,place_name,place_pop\n${ids[0]},Alpha,1000\n${ids[1]},Beta,1000\n${ids[0]},Gamma,500\n`,election_csv:election(plan),tolerance_percent:0.5};
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;
 execFileSync('target/release/examples/plan_statistics_reference.exe',[root+'/request.json',root+'/native-'+matched],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});
 for(const [file,text]of Object.entries(out.files)){assert.equal(text,await fs.readFile(root+'/native-'+matched+'/'+file,'utf8'));reports++;}return out;
}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const out=await exact(base);assert.equal(out.demographic.districts[0].total_pop,'300');assert.equal(out.urban.districts[0].largest_city,'Beta');assert.equal(out.urban.districts[0].num_places,3);assert.equal(out.summary.districts[0].ideal_pop,'300');
await exact({...base,urban_csv:null,election_csv:null});
const urbanOnly=await exact({...base,demographic_csv:null,election_csv:null});assert.equal(urbanOnly.population_balance_evaluated,false);assert.equal(urbanOnly.summary.districts[0].ideal_pop,null);
await exact({...base,demographic_csv:null,urban_csv:null});
await exact({...base,urban_csv:'GEOID,place_name,place_pop\n',demographic_csv:demo(plan)+'unrelated,100,40,25,5,20,10\n'});
const zero=await exact({...base,demographic_csv:header+ids.map(id=>id+',0,0,0,0,0,0').join('\n')});assert.equal(zero.demographic.districts[0].pct_minority,1);assert.equal(zero.demographic.districts[0].is_majority_minority,true);
const huge=await exact({...base,demographic_csv:header+ids.map((id,i)=>id+(i===0?',9007199254740993,9007199254740993,0,0,0,0':',0,0,0,0,0,0')).join('\n'),urban_csv:`GEOID,place_name,place_pop\n${ids[0]},Large,18446744073709551615\n`});assert.equal(huge.demographic.districts[0].total_pop,'9007199254740993');assert.equal(huge.urban.districts[0].largest_city_pop,'18446744073709551615');assert.match(huge.files['demographic.json'],/9007199254740993/);assert.match(huge.files['urban.json'],/18446744073709551615/);
for(const state of ['RI','IA','NC']){const p=JSON.parse(await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8'));await exact({...base,document_json:JSON.stringify(p),demographic_csv:demo(p),urban_csv:null,election_csv:election(p)});}
for(const demographic_csv of [header+ids.slice(1).map(id=>id+',100,40,25,5,20,10').join('\n'),base.demographic_csv+ids[0]+',100,40,25,5,20,10\n',base.demographic_csv.replace('100,40','-1,40'),base.demographic_csv.replace('100,40','100,101'),base.demographic_csv.replace('100,40','100.5,40'),base.demographic_csv.replace('GEOID,','geoid,'),base.demographic_csv.replace('black_non_hispanic','white_non_hispanic'),header+ids.map(id=>id+',9223372036854775807,0,0,0,0,0').join('\n')])await reject({...base,demographic_csv});
const stateOverflow=header+ids.map((id,i)=>id+(i%3===0?',4000000000000000000,0,0,0,0,0':',0,0,0,0,0,0')).join('\n');await reject({...base,demographic_csv:stateOverflow});
await reject({...base,urban_csv:`GEOID,place_name,place_pop\n${ids[0]},Bad,-1\n`});await reject({...base,demographic_csv:null,urban_csv:null,election_csv:null});await reject({...base,tolerance_percent:-1});await reject({...base,tolerance_percent:101});await reject({...base,demographic_csv:base.demographic_csv+' '.repeat(3*1024*1024)});
const wrong=structuredClone(plan);wrong.plan.units.unit_universe_hash='sha256:wrong';await reject({...base,document_json:JSON.stringify(wrong)});
await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output:out}));for(const [name,text]of Object.entries(out.files))await fs.writeFile(root+'/'+name,text);
console.log(`Plan statistics: ${matched} native/WASM matches, ${reports} actual filesystem-backed native report byte matches, ${rejected} rejections. Full/sparse/optional data, urban ties/repeated-unit rows, exact i64/u64 counts, zero population and overflow verified.`);

const {createProject}=await import('../../web/lab/project.js');const {countResultTables}=await import('../../web/lab/count-results.js');const {statisticsRequest,validateStatisticsInputs}=await import('../../web/lab/statistics-files.js');const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={demographic:source('demographic.csv',base.demographic_csv),urban:source('urban.csv',base.urban_csv),election:source('election.csv',base.election_csv),tolerance_percent:base.tolerance_percent};const files={auditPlanText:source('fixture.rplan',base.document_json),statisticsInputs:inputs};assert.deepEqual(statisticsRequest(files,inputs),base);assert.throws(()=>validateStatisticsInputs({...inputs,tolerance_percent:-1}));assert.equal(countResultTables(urbanOnly)[0].rows[0][4],'Not evaluated');assert.equal(countResultTables(huge)[2].rows[0][1],'9007199254740993');
const project=createProject({name:'Native demographic/urban summary fixture',files,operation:base.operation,constraints:[],result:out,lastOperation:base.operation});await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
console.log('Statistics browser fixture: exact project/request bytes, large decimal count display and unknown-population rendering passed.');

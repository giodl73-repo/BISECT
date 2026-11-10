import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/comparison-fixtures/contexts.json','utf8'));
await fs.mkdir('target/report-plan-evidence-fixtures',{recursive:true});
const path='target/report-plan-evidence-fixtures/request.json';
const bytes=text=>Array.from(Buffer.from(text));
function files(label){return {'manifest.json':bytes(JSON.stringify({label,num_districts:3,state:'WA',year:2020})),'analysis/partisan.json':bytes('{"per_district_dem_share":[0.55,0.45,0.60]}'),'analysis/vra_analysis.json':bytes('{"mm_count":1}'),'analysis/compactness.json':bytes('{"mean_polsby_popper":0.5}')};}
const base={operation:'render-report-comparison',files_a:files('SYNTHETIC report A'),files_b:files('SYNTHETIC report B'),label_a:'A',label_b:'B',leaning_threshold:0.55,close_call_band:0.02,generated_at:'2026-10-04T00:00:00Z',plan_a:fixture.plan,plan_b:structuredClone(fixture.plan),context:fixture.contexts.full};
async function native(request){await fs.writeFile(path,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe'],maxBuffer:16*1024*1024}));}
let exact=0,rejected=0;
for(const context of [null,...['full','graph-only','zero-population','large-population'].map(key=>fixture.contexts[key])])for(const variant of ['identical','one-unit','renumbered']){
 const request=structuredClone(base);request.context=context;
 if(variant==='one-unit')request.plan_b.plan.assignment[0]=1;
 if(variant==='renumbered')request.plan_b.plan.assignment=request.plan_b.plan.assignment.map(d=>(d+1)%3);
 const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;
 const changed=request.plan_a.plan.assignment.map((d,i)=>d!==request.plan_b.plan.assignment[i]?i:null).filter(i=>i!==null);
 assert.equal(changed.length,{identical:0,'one-unit':1,renumbered:9}[variant]);
 const populations=context?.populations;
 const total=populations?populations.reduce((sum,n)=>sum+BigInt(n),0n).toString():null;
 const changedPopulation=populations?changed.reduce((sum,i)=>sum+BigInt(populations[i]),0n).toString():null;
 const touched=[...new Set(changed.flatMap(i=>[request.plan_a.plan.assignment[i],request.plan_b.plan.assignment[i]]))].sort((a,b)=>a-b);
 const evidence=result.plan_evidence;assert.equal(evidence.changed_units,changed.length);assert.deepEqual(evidence.district_ids_with_changes,touched);assert.equal(evidence.population_in_changed_units,changedPopulation);assert.equal(evidence.plan_a_total_population,total);assert.equal(evidence.plan_b_total_population,total);assert.equal(evidence.context_hash,context?.context_hash??null);
 assert.equal(result.report.diff,null);assert.equal(result.report.plan_a.total_population,total);assert.equal(result.report.plan_b.total_population,total);assert.deepEqual(result.receipt.plan_evidence,evidence);
 const plainCompare=engine.execute({operation:'compare-plans',plan_a:request.plan_a,plan_b:request.plan_b,context});assert.equal(evidence.plan_a_hash,plainCompare.plan_a.plan_hash);assert.equal(evidence.plan_b_hash,plainCompare.plan_b.plan_hash);
 assert.match(result.markdown,new RegExp(changed.length+' units'));assert.ok(result.markdown.includes('no population movement'));if(changedPopulation!==null)assert.ok(result.markdown.includes('Population in those units: '+changedPopulation));if(total!==null)assert.ok(result.html.replaceAll(',','').includes(total));
 const tables=countResultTables(result);assert.equal(tables[0].rows[5][1],total??'Unavailable');assert.equal(tables[0].rows[6][1],String(changed.length));assert.equal(tables[0].rows[7][1],changedPopulation??'Unavailable');
 const project=createProject({name:'Synthetic measured comparison report',files:{reportFiles:Object.fromEntries(Object.entries(request.files_a).map(([name,bytes])=>[name,Buffer.from(bytes).toString('base64')])),comparisonReportFiles:Object.fromEntries(Object.entries(request.files_b).map(([name,bytes])=>[name,Buffer.from(bytes).toString('base64')])),reportComparisonSettings:{label_a:request.label_a,label_b:request.label_b,leaning_threshold:request.leaning_threshold,close_call_band:request.close_call_band,generated_at:request.generated_at},reportPlanSettings:{include_plan_measurements:true},plan:request.plan_a,comparisonPlan:request.plan_b,...(context?{context}: {})},operation:'render-report-comparison',constraints:[],result,lastOperation:'render-report-comparison'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 if(context===fixture.contexts['large-population']&&variant==='renumbered')await fs.writeFile('target/report-plan-evidence-fixtures/browser-project.bisect',JSON.stringify(project));
}
for(const mutate of [r=>delete r.plan_a,r=>delete r.plan_b,r=>{delete r.plan_a;delete r.plan_b;},r=>r.files_a['manifest.json']=bytes('{"num_districts":2}'),r=>r.files_b['manifest.json']=bytes('{"num_districts":3,"state":"RI"}'),r=>r.files_a['manifest.json']=bytes('{"num_districts":3,"year":"2010"}'),r=>r.plan_b.plan.units.unit_ids.reverse(),r=>r.plan_b.plan.assignment[0]=99,r=>r.context.context_hash='sha256:'+'0'.repeat(64)]){
 const request=structuredClone(base);mutate(request);assert.throws(()=>engine.execute(request));await assert.rejects(native(request));rejected++;
}
await fs.writeFile('target/report-plan-evidence-fixtures/verification.json',JSON.stringify({exact_native_wasm:exact,matching_rejections:rejected,project_table_roundtrips:15,scope:'Separately supplied plan/context assignment and population measurements; imported analysis is not authenticated to plans.'},null,2));
console.log(`Report plan evidence: ${exact} exact native/WASM and project/table cases, ${rejected} matching rejections; independent assignment/BigInt population oracles passed.`);

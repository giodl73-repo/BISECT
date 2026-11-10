import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/comparison-fixtures/contexts.json','utf8'));
const original=fixture.plan,contexts={absent:null,...fixture.contexts};let cases=0,rejected=0,projects=0;
const variants=[original.plan.assignment,[2,2,2,0,0,0,1,1,1],[0,1,2,0,1,2,0,1,2],[0,1,0,1,0,1,0,1,0],[0,0,0,0,1,1,1,1,1],Array(9).fill(0),[0,0,0,0,0,0,0,0,1]];
function plan(assignment){const p=structuredClone(original);p.plan.assignment=assignment;p.plan.k=Math.max(...assignment)+1;p.plan.display_labels=Array.from({length:p.plan.k},(_,i)=>String(i+1));p.metadata.label='SYNTHETIC comparison '+assignment.join('');return p;}
function jaccard(a,b){const labels=[...new Set(a)].sort((x,y)=>x-y),bs=[...new Set(b)];return labels.reduce((sum,d)=>sum+Math.max(...bs.map(e=>{let intersection=0,union=0;for(let i=0;i<a.length;i++){if(a[i]===d&&b[i]===e)intersection++;if(a[i]===d||b[i]===e)union++;}return intersection/union;})),0)/labels.length;}
async function native(r){await fs.writeFile('target/comparison-fixtures/request.json',JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',['target/comparison-fixtures/request.json'],{encoding:'utf8',maxBuffer:16*1024*1024}));}
async function parity(r){const result=engine.execute(r);assert.deepEqual(result,await native(r));cases++;assert.equal(result.similarity.a_to_b,jaccard(r.plan_a.plan.assignment,r.plan_b.plan.assignment));assert.equal(result.similarity.b_to_a,jaccard(r.plan_b.plan.assignment,r.plan_a.plan.assignment));assert.equal(result.compactness.status,'not-evaluated');assert.equal(result.partisan.status,'not-evaluated');return result;}
let browserProject;
for(const assignment of variants)for(const [name,context]of Object.entries(contexts))for(const reverse of [false,true]){
 const p=plan(assignment),request={operation:'compare-plans',plan_a:reverse?p:original,plan_b:reverse?original:p,context};
 const result=await parity(request);
 const changed=request.plan_a.plan.assignment.flatMap((d,i)=>d!==request.plan_b.plan.assignment[i]?[i]:[]);
 assert.equal(result.assignment_change.changed_units,changed.length);
 assert.equal(result.assignment_change.common_units,request.plan_a.plan.assignment.length);
 assert.deepEqual(result.assignment_change.district_ids_with_changes,[...new Set(changed.flatMap(i=>[request.plan_a.plan.assignment[i],request.plan_b.plan.assignment[i]]))].sort((a,b)=>a-b));
 assert.equal(result.assignment_change.population_in_changed_units,context?.populations?changed.reduce((sum,i)=>sum+BigInt(context.populations[i]),0n).toString():null);

 if(context?.populations){const total=context.populations.reduce((s,p)=>s+BigInt(p),0n);assert.equal(result.plan_a.total_population,total.toString());assert.equal(result.population.status,total>0n?'measured':'not-evaluated');for(const [key,doc]of [['plan_a',request.plan_a],['plan_b',request.plan_b]])for(const d of result[key].district_measurements){const pop=context.populations.reduce((s,p,i)=>s+(doc.plan.assignment[i]===d.district_id?BigInt(p):0n),0n);assert.equal(d.population,pop.toString());if(total>0n)assert.ok(Math.abs(d.deviation_percent-Math.abs(Number(pop)-Number(total)/doc.plan.k)/(Number(total)/doc.plan.k)*100)<1e-12);}}
 else{assert.equal(result.population.status,'not-evaluated');assert.equal(result.plan_a.total_population,null);}
 assert.equal(result.plan_a.contiguity.status,context?.graph?'measured':'not-evaluated');
 if(name==='partial-subdivisions'){assert.equal(result.plan_a.county.missing_units,1);assert.equal(result.plan_a.municipal.known_units,0);}
 const files={plan:request.plan_a,comparisonPlan:request.plan_b,...(context?{context}:{})};const project=createProject({name:'Plan comparison',files,operation:'compare-plans',constraints:[],result,lastOperation:'compare-plans'});assert.deepEqual(parseProject(JSON.stringify(project)),project);projects++;
 const tables=countResultTables(result);assert.equal(tables.length,5);assert.ok(tables.every(t=>t.rows.length<=500));
 if(name==='full'&&assignment===variants[2]&&!reverse)browserProject=project;
}
const empty=plan(variants[3]);empty.plan.k=3;empty.plan.display_labels=['a','b','empty'];empty.plan.allow_empty_districts=true;
const er=await parity({operation:'compare-plans',plan_a:empty,plan_b:original,context:contexts.full});assert.equal(er.plan_a.district_measurements[2].components,0);assert.equal(er.plan_a.contiguity.all_nonempty_and_connected,false);
const valid={operation:'compare-plans',plan_a:original,plan_b:plan(variants[2]),context:contexts.full};
for(const mutate of [r=>r.extra=1,r=>r.plan_b.plan.assignment.pop(),r=>r.plan_b.plan.assignment[0]=50,r=>r.plan_b.plan.units.state='RI',r=>r.plan_b.plan.units.year=2010,r=>r.plan_b.plan.units.unit_ids.reverse(),r=>r.plan_b.plan.units.unit_kind='block-group',r=>r.context.context_hash='sha256:'+'0'.repeat(64),r=>r.context.units.state='RI',r=>r.context.populations.pop(),r=>r.context.populations[0]=-1,r=>r.context.graph.adjacency[0][0].to=999,r=>r.context.units.unit_universe_hash='sha256:wrong',r=>r.plan_a.plan.k=501]){
 const r=structuredClone(valid);mutate(r);assert.throws(()=>engine.execute(r));await fs.writeFile('target/comparison-fixtures/request.json',JSON.stringify(r));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',['target/comparison-fixtures/request.json'],{stdio:'pipe'}));rejected++;
}
const hostile=structuredClone(browserProject.result);hostile.plan_a.label='<img src=x onerror=alert(1)>';hostile.plan_a.district_measurements=Array(501).fill({label:'<script>evil</script>',population:'9007199254740993'});const tables=countResultTables(hostile);assert.equal(tables[0].rows[0][1],hostile.plan_a.label);assert.equal(tables[3].rows.length,500);assert.equal(tables[3].rows[0][3],'9007199254740993');
await fs.writeFile('target/comparison-fixtures/browser-project.bisect',JSON.stringify(browserProject));
await fs.writeFile('target/comparison-fixtures/verification.json',JSON.stringify({exact_native_wasm:cases,rejected,projects,scope:'Synthetic grid, native hashed contexts, missing/partial/zero/large populations, directional metrics, empty districts and scope/hash rejection'},null,2));
console.log(`Plan comparison: ${cases} exact native/WASM cases, ${projects} project roundtrips, ${rejected} matching rejections; bounded inert tables and exact population strings passed.`);

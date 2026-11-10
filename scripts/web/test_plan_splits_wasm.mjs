import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {retainAnalysis,analysisCommissionInputs} from '../../web/lab/analysis-collection.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {commissionRequest} from '../../web/lab/commission-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/plan-splits-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const planText=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'),plan=JSON.parse(planText),ids=plan.plan.units.unit_ids;
let matches=0,rejections=0;
async function exact(document_json,geography_json=null){const request={operation:'analyze-plan-splits',document_json,geography_json};await fs.writeFile(root+'/request.json',JSON.stringify(request));const out=engine.execute(request);const native=execFileSync('target/release/examples/plan_splits_reference.exe',[root+'/request.json'],{encoding:'utf8'});assert.equal(out.report_json,native);assert.deepEqual(out.report,JSON.parse(native));matches++;return out;}
const county=await exact(planText);assert.equal(county.report.municipalities.available,false);assert.equal(county.report.counties.split,1);
const geography={county_names:{[ids[0].slice(0,5)]:'Supplied county'},place_to_tracts:{split:ids,one:[ids[0]],unmatched:['unrelated'],overlap:[ids[0],ids.at(-1),'outside']},place_names:{split:'Example town <script>'}};
const sourceText=JSON.stringify(geography),out=await exact(planText,sourceText);assert.equal(out.report.municipalities.available,true);assert.equal(out.report.municipalities.total,4);assert.equal(out.report.municipalities.split,2);assert.equal(out.coverage.unmatched_links,2);assert.equal(out.report.counties.split_list[0].county_name,'Supplied county');assert.equal(countResultTables(out).length,3);
await exact(planText,'{}');for(const state of ['RI','IA','NC'])await exact(await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8'));
for(const bad of [{unexpected:1},{place_to_tracts:{x:[ids[0],ids[0]]}},{place_to_tracts:{x:[]}},{county_names:{x:''}},{place_to_tracts:{'':ids}}]){assert.throws(()=>engine.execute({operation:'analyze-plan-splits',document_json:planText,geography_json:JSON.stringify(bad)}));rejections++;}
assert.throws(()=>engine.execute({operation:'analyze-plan-splits',document_json:planText,geography_json:' '.repeat(3*1024*1024)}));rejections++;
const src=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});const collection=await retainAnalysis(undefined,src('grid.rplan',planText),out);assert.equal(Buffer.from(analysisCommissionInputs(collection).files['analysis/splits.json'].base64,'base64').toString(),out.report_json);
const prior=parseProject(await fs.readFile('target/analysis-collection-wasm/project.bisect','utf8'));const combined=await retainAnalysis(prior.files.analysisCollection,src('grid.rplan',planText),out);const nativeFixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8'));const inputs=analysisCommissionInputs(combined,{files:{'manifest.json':src('manifest.json',nativeFixture.files['manifest.json'])},generated_at:nativeFixture.generated_at});const report=engine.execute(commissionRequest(inputs));assert.deepEqual(JSON.parse(report.report_json).sections.geographic_constraints.splits,{...out.report,status:'ok'});
const fixture=createProject({name:'Native subdivision splits',operation:'analyze-plan-splits',files:{auditPlanText:src('grid.rplan',planText),splitsGeography:src('geography.json',sourceText)},constraints:[],result:out,lastOperation:'analyze-plan-splits'});assert.deepEqual(parseProject(JSON.stringify(fixture)),fixture);await fs.writeFile(root+'/project.bisect',JSON.stringify(fixture));await fs.writeFile(root+'/result.json',JSON.stringify(out));await fs.writeFile(root+'/evidence.json',JSON.stringify({native_report_byte_matches:matches,rejections,retention:true,project_roundtrip:true},null,2));console.log(`Native splits: ${matches} independent report byte matches, ${rejections} rejections; county/municipality coverage, retention, project and tables passed.`);

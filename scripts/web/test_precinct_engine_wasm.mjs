import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tuiRunRequest} from '../../web/lab/tui-run-files.js';
import {configTemplateRequest,CONFIG_TEMPLATE_DEFAULTS} from '../../web/lab/config-template.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/precinct-engine-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/tui-run-wasm/run.bisect','utf8'));
const original=JSON.parse(Buffer.from(base.files.configRunInputs.graph.base64,'base64').toString('utf8'));
const source=(name,value)=>({name,base64:Buffer.from(typeof value==='string'?value:JSON.stringify(value)).toString('base64')});
let cases=0,refusals=0;let first;
async function parity(request){const result=engine.execute(request);const scratch=root+'/request.json';await fs.writeFile(scratch,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[scratch],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));assert.deepEqual(result,native);cases++;return result;}
async function refusal(request){assert.throws(()=>engine.execute(request));const scratch=root+'/invalid.json';await fs.writeFile(scratch,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[scratch],{encoding:'utf8',windowsHide:true,stdio:'pipe'}));refusals++;}
for(const kind of ['precinct','imported'])for(const weights of ['geographic','county','unweighted'])for(const seed of ['42','18446744073709551615']){
 const census={...original,unit_kind:'tract',geoids:original.geoids.map((_,i)=>(i<18?'19001':'19003')+String(i).padStart(6,'0'))};
 const graph={...census,unit_kind:kind,geoids:original.geoids.map((_,i)=>i===0?'A':i===1?'é/2':'precinct-'+i),county_ids:census.geoids.map(id=>id.slice(0,5))};
 const template=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,structure:'standard-bisect',balance_tolerance:5,weights,alpha_county:weights==='county'?1.5:null,seed,workers:1,years:['2020']}));
 const settings={...base.files.tuiRunSettings,form:{...base.files.tuiRunSettings.form,resolution:kind,seed}};
 const graphSource=source(kind+'.json',graph),yamlSource=source(template.generated_filename,template.yaml_text);
 const request=tuiRunRequest(settings,yamlSource,base.files.configSettings,{graph:graphSource},base.files.configRunSettings);
 const result=await parity(request);assert.equal(result.status,'completed');
 const comparison=engine.execute({...request,input:{...request.input,form:{...request.input.form,resolution:'tract'},run:{...request.input.run,graph:census}}});
 assert.deepEqual(graph.geoids.map(id=>result.configured_run.result.assignments[id]),census.geoids.map(id=>comparison.configured_run.result.assignments[id]));assert.equal(result.configured_run.result.metrics.split_counties,comparison.configured_run.result.metrics.split_counties);assert.equal(result.configured_run.result.metrics.weighted_boundary,comparison.configured_run.result.metrics.weighted_boundary);
 const exported=result.practitioner_export;assert.equal(exported.document.plan.units.unit_kind,kind);assert.deepEqual(exported.document.plan.units.unit_ids,graph.geoids);assert.deepEqual(exported.context.subdivisions.county_ids,graph.county_ids);
 assert.throws(()=>engine.execute({operation:'run-multiscale',input:{request:result.configured_run.engine_request,fine_level:'tract',coarse_level:'county',total_steps:1,alpha:0.2,percentile:0,block_groups:null,adaptive:null}}),/tract parent graph/);
 const contiguity=await parity({operation:'analyze-plan-contiguity',document_json:exported.document_json,context_json:exported.context_json});assert(contiguity);
 if(weights==='county')assert.equal(result.configured_run.result.metrics.weighting_evidence.county_identity,'explicit-prepared-county-ids');
 const geometry=JSON.parse(Buffer.from(base.files.planGeometry.base64,'base64').toString());geometry.features.forEach((f,i)=>f.properties.GEOID=graph.geoids[i]);
 const files={...base.files,configSource:yamlSource,configRunInputs:{graph:graphSource},tuiRunSettings:settings,planGeometry:source(kind+'.geojson',geometry)};
 const project=createProject({name:kind+' configured Run',files,operation:'run-tui-configured',constraints:[],result,lastOperation:'run-tui-configured'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 if(weights==='geographic'&&seed==='42'){const map=await parity({operation:'render-native-district-map',input:{document_json:exported.document_json,unit_geometry_json:JSON.stringify(geometry),dpi:72,map_type:'districts',dissolve_method:'batch'}});await fs.writeFile(root+'/'+kind+'-map.json',JSON.stringify(map));}
 if(weights==='geographic'&&seed==='42'){
  const fixture=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')),profile={...fixture.legal_profile,population_tolerance:{type:'exact-ppm',max_deviation_ppm:50000},contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
  const constraints=['plan-shape','population','contiguity','splits'],generated_at_utc='2026-10-12T00:00:00Z';
  const audited=await parity({operation:'audit-rplan-text',document_json:exported.document_json,context_json:exported.context_json,profile_json:JSON.stringify(profile),constraints,generated_at_utc,lineage_json:null});assert.equal(audited.result,'pass');
  const verified=await parity({operation:'verify-audit-text',certificate_json:audited.certificate_json,document_json:exported.document_json,context_json:exported.context_json});assert.equal(verified.verified,true);
  const changed=structuredClone(exported.context);changed.subdivisions.county_ids[0]='changed';await refusal({operation:'verify-audit-text',certificate_json:audited.certificate_json,document_json:exported.document_json,context_json:JSON.stringify(changed)});
  const auditProject=createProject({name:kind+' text audit',operation:'audit-rplan-text',lastOperation:'audit-rplan-text',constraints,files:{auditPlanText:source(kind+'.rplan',exported.document_json),auditContextText:source(kind+'.rctx',exported.context_json),auditProfileText:source('profile.json',profile),textAuditSettings:{generated_at_utc}},result:audited});await fs.writeFile(root+'/'+kind+'-audit.bisect',JSON.stringify(auditProject));
 }
 if(weights==='geographic'&&seed==='42')await fs.writeFile(root+'/'+kind+'.bisect',JSON.stringify(project));
 first??={request,graph};
}
for(const kind of ['precinct','imported']){
 const request=structuredClone(first.request);request.input.form.year='2024';request.input.form.resolution=kind;request.input.run.graph.year='2024';request.input.run.graph.unit_kind=kind;request.input.run.yaml_text=request.input.run.yaml_text.replaceAll('2020','2024');
 const result=await parity(request);assert.equal(result.practitioner_export.document.plan.units.year,2024);assert.equal(result.practitioner_export.context.units.year,2024);assert.equal(result.form.year,'2024');
}
for(const mutate of [g=>delete g.county_ids,g=>g.county_ids.pop(),g=>g.county_ids[0]='',g=>g.county_ids[0]='x\n',g=>g.geoids[0]='',g=>g.geoids[0]='__proto__',g=>g.geoids[1]=g.geoids[0],g=>g.geoids[0]='x'.repeat(201),g=>g.state='XX',g=>g.unit_kind='tract',g=>g.county_ids.push('extra')]){
 const changed=structuredClone(first.request);mutate(changed.input.run.graph);await refusal(changed);
}
const changed=structuredClone(first.request);changed.input.form.resolution='tract';await refusal(changed);
const legacy=tuiRunRequest(base.files.tuiRunSettings,base.files.configSource,base.files.configSettings,base.files.configRunInputs,base.files.configRunSettings);assert.deepEqual(engine.execute(legacy),base.result);
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,refusals,wasm_sha256:engine.artifactIdentity.sha256,legacy_whole_result_unchanged:true,county_kernel_assignment_equivalence:true,exact_precinct_imported_context:true,declared_election_year_preserved:true,native_text_audit_verify:true,tampered_county_context_rejected:true,scope:'Synthetic prepared precinct/imported graphs through native configured Run/export/contiguity with explicit aligned county IDs, including short and Unicode IDs and max-u64 seeds. Does not acquire or authenticate precinct data or establish original-state precinct capacity.'},null,2));
console.log('Precinct/imported engine: '+cases+' whole native/WASM results, '+refusals+' matching refusals, exact county context, Census-kernel assignments and legacy identity passed.');

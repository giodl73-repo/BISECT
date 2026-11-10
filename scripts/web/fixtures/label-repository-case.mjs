import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../../web/lab/config-template.js';
export const sha=text=>createHash('sha256').update(text).digest('hex');
const source=input=>({source:{name:'repository-stage.json',base64:Buffer.from(JSON.stringify(input)).toString('base64')}});
export async function labelRepositoryCase(engine){
 const label='repository_fixture',year='2020',created='2026-10-23T00:00:00Z',config=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,name:label,structure:'standard-bisect',seed:'42',workers:1,years:[year]})).yaml_text;
 const registry=JSON.stringify({[label]:{built:[year],analyzed:[],reported:[]},unrelated:{built:['2000'],analyzed:[],reported:[]}},null,2)+'\r\n';
 const build=JSON.stringify({label,year,created,version:'0.1',config_path:'config/'+label+'.yaml',config_sha256:sha(config),command:'source-owned fixture index; no engine build performed',output_dir:'runs/'+label+'/'+year,metis_engine:'metis-core',algorithm:{structure:'standard-bisect'},states:{washington:{status:'ok',districts:3}},summary:{total:1,succeeded:1,failed:0}});
 const state={document_json:await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'),context_json:await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'),geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,tolerance_percent:0.5,election_settings:{bootstrap_seed:'18446744073709551615',bootstrap_replicates:1},compactness:null};
 const ids=JSON.parse(state.document_json).plan.units.unit_ids;
 state.compactness={unit_geometry_json:JSON.stringify({type:'FeatureCollection',features:ids.map((GEOID,i)=>{const x=i%3,y=Math.floor(i/3);return {type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[[[x,y],[x+1,y],[x+1,y+1],[x,y+1],[x,y]]]}};})}),coordinate_basis:'projected-metres',crs_label:'Source-owned synthetic grid metres'};
 const stageRequest={operation:'run-label-analysis',input:{build_index_json:build,registry_json:registry,created,states:{washington:state},types:['manifest','contiguity','statistics','compactness']}};
 const analysis=engine.execute(stageRequest);
 const original={'.bisect':registry,['runs/'+label+'/'+year+'/index.json']:build,['config/'+label+'.yaml']:config,'notes.txt':'Unrelated UTF-8 notes: café\r\n'};
 function merge(stage,current,sourceRegistry,upstream){
  const report=stage.schema_version==='bisect-label-report-v1';
  const input={reconciliation:{stage:report?'reported':'analyzed',source_registry_json:sourceRegistry,source_registry_sha256:sha(sourceRegistry),current_registry_json:current['.bisect'],proposed_registry_json:stage.registry_json,upstream_index_json:upstream,generated_index_json:stage.index_json},current_files:current,generated_files:stage.files,generated_file_sha256:stage.file_sha256,expected_file_sha256:Object.fromEntries(Object.keys(stage.files).filter(path=>path!=='.bisect').map(path=>[path,Object.hasOwn(current,path)?sha(current[path]):null]))};
  return {request:{operation:'reconcile-label-package',input},inputs:source(input),result:engine.execute({operation:'reconcile-label-package',input})};
 }
 const first=merge(analysis,original,registry,build),reportRequest={operation:'run-label-report',input:{analysis_index_json:analysis.index_json,registry_json:first.result.files['.bisect'],files:{...analysis.files,'.bisect':first.result.files['.bisect']},file_sha256:{...analysis.file_sha256,'.bisect':sha(first.result.files['.bisect'])},generated_at:created,format:'html',audit_with_report:true}};
 const report=engine.execute(reportRequest),second=merge(report,first.result.files,first.result.files['.bisect'],analysis.index_json);
 return {label,year,original,stageRequest,analysis,first,reportRequest,report,second};
}

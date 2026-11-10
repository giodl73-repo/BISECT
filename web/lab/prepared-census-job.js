import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {validatePreparedStateJob,executePreparedStateJob} from './prepared-state-job.js';
import {censusPreparationRequest} from './tract-input-files.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {characterStateFips} from './character-input.js';
import {validateStateSourceAcquisition} from './state-source-acquisition.js';

export const PREPARED_CENSUS_DEFAULTS={enabled:false,use_display_geometry:false,source:null,settings:{label:'Prepared Census run',chamber:'congressional',created_at:'',inspect_only:false}};
export function validatePreparedCensusTemplateInputs(value){
 if(!value||Object.keys(value).sort().join()!=='enabled,settings,source,use_display_geometry'||typeof value.enabled!=='boolean'||typeof value.use_display_geometry!=='boolean')throw new Error('Invalid Census request template selection.');
 validatePreparedEngineInputs({source:value.source,settings:value.settings});return value;
}
function preparation(input){
 if(!input||Object.keys(input).some(k=>!['state','projection','tiger','pl94','geometry_profile'].includes(k)))throw new Error('Invalid prepared Census source input.');
 return censusPreparationRequest({tractPreparationSettings:{state:input.state,projection:input.projection,...(input.geometry_profile?{geometry_profile:input.geometry_profile}:{})},tigerInputs:input.tiger,pl94Inputs:input.pl94});
}
export function validatePreparedCensusJob(job){
 if(!job||Object.keys(job).some(k=>!['prepared_engine_template','source_preparation','scope','geometry','acquisition'].includes(k))||!job.scope||Object.keys(job.scope).sort().join()!=='chamber,districts,state,year'||!characterStateFips(job.scope.state)||!['2000','2010','2020'].includes(job.scope.year)||job.scope.chamber!=='congressional'||job.scope.districts!==null&&(!Number.isSafeInteger(job.scope.districts)||job.scope.districts<1||job.scope.districts>500))throw new Error('Census template jobs require congressional native allocation scope.');
 const request=preparation(job.source_preparation);if(request.input.state!==job.scope.state||String(request.input.pl94.year)!==job.scope.year)throw new Error('Census source scope differs from the template job.');
 validatePreparedEngineInputs(job.prepared_engine_template);const settings=job.prepared_engine_template.settings;
 if(!job.prepared_engine_template.source||settings.inspect_only!==false||settings.chamber!=='congressional'||!settings.created_at.trim())throw new Error('Select an engine template and record congressional export metadata before queuing.');
 if(job.geometry)validateSourceFile(job.geometry,{maxBytes:64*1024*1024});
 if(job.acquisition){validateStateSourceAcquisition(job.acquisition);if(job.acquisition.receipts[0].state!==job.scope.state||job.acquisition.receipts[0].year!==job.scope.year||job.source_preparation.tiger.kind!=='tracts')throw new Error('Acquisition scope differs from the prepared template job.');}
 return job;
}
export function createPreparedCensusJob(files,inputs=files.preparedCensusTemplateInputs){
 validatePreparedCensusTemplateInputs(inputs);if(!inputs.enabled)throw new Error('Enable the full request template source mode first.');
 const selected=censusPreparationRequest(files).input,job={prepared_engine_template:{source:inputs.source,settings:{...inputs.settings,inspect_only:false}},source_preparation:selected,scope:{state:selected.state,year:String(selected.pl94.year),districts:null,chamber:'congressional'}};
 if(inputs.use_display_geometry){if(!files.planGeometry)throw new Error('Select the explicit fine-unit display geometry.');job.geometry=files.planGeometry;}
 return structuredClone(validatePreparedCensusJob(job));
}
export function executePreparedCensusJob(engine,job,{onProgress}={}){
 validatePreparedCensusJob(job);if(job.scope.districts===null)throw new Error('Resolve native allocation before executing a Census template job.');
 onProgress?.({stage:'source-preparation',message:'Preparing Census graph for the full request template'});
 const prepared=engine.execute(preparation(job.source_preparation),{onProgress});
 try{
  if(prepared.state!==job.scope.state||prepared.year!==job.scope.year)throw new Error('Prepared Census scope differs from selected allocation.');
  const binding=engine.execute({operation:'bind-prepared-engine-graph',input:{template_json:job.prepared_engine_template.source.text,graph_json:prepared.graph_json,districts:job.scope.districts}});
  if(binding.schema_version!=='bisect-prepared-engine-binding-v1'||binding.execution_performed!==false||binding.state!==job.scope.state||binding.year!==job.scope.year||binding.districts!==job.scope.districts)throw new Error('Native graph/template binding differs from the source job.');
  const geometry=job.geometry??{name:prepared.state+'-'+prepared.year+'-geometry.geojson',base64:encodeBytes(new TextEncoder().encode(prepared.geometry_json))};
  const bound=validatePreparedStateJob({prepared_engine:{source:{name:'bound-engine-request.json',text:binding.bound_request_json},settings:job.prepared_engine_template.settings},scope:job.scope,request_sha256:binding.bound_request_sha256,geometry});
  const result=executePreparedStateJob(engine,bound,{onProgress}),{bound_request_json,review,...receipt}=binding;
  result.template_source=structuredClone(job.prepared_engine_template);result.graph_binding=receipt;
  return {schema_version:'bisect-source-prepared-state-run-v1',source_preparation:prepared,result};
 }catch(error){error.source_preparation=prepared;throw error;}
}

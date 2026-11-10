import {nativeEngineInputRequest,NATIVE_ADJACENCY_SETTINGS} from './native-adjacency-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {validatePreparedStateJob,executePreparedStateJob} from './prepared-state-job.js';
import {validateSourceFile} from './package-files.js';
import {characterStateFips} from './character-input.js';

export function validatePreparedNativeJob(job){
 if(!job||Object.keys(job).some(k=>!['native_preparation','prepared_engine_template','scope','geometry'].includes(k))||!job.native_preparation||Object.keys(job.native_preparation).some(k=>!['sources','state','year','unit_kind','require_geometry'].includes(k)))throw new Error('Select original native inputs and a full request template.');
 const {sources,...settings}=job.native_preparation;nativeEngineInputRequest(sources,settings);
 const scope=job.scope;if(!scope||Object.keys(scope).sort().join()!=='chamber,districts,state,year'||!characterStateFips(scope.state)||!['2000','2010','2020'].includes(scope.year)||scope.state!==settings.state||scope.year!==settings.year||scope.chamber!=='congressional'||scope.districts!==null&&(!Number.isSafeInteger(scope.districts)||scope.districts<1||scope.districts>500))throw new Error('Native source jobs require matching congressional state/Census scope.');
 validatePreparedEngineInputs(job.prepared_engine_template);const input=job.prepared_engine_template;
 if(!input.source||input.settings.inspect_only!==false||input.settings.chamber!=='congressional'||!input.settings.created_at.trim())throw new Error('Select a full template and recorded congressional export metadata.');
 if(Object.hasOwn(job,'geometry'))validateSourceFile(job.geometry,{maxBytes:64*1024*1024});return job;
}
export function createPreparedNativeJob(files){
 const selected=files.preparedGraphInputs;if(!selected?.template)throw new Error('Select the full request template for the original native sources.');
 const request=nativeEngineInputRequest(files.nativeAdjacencyInputs,files.nativeAdjacencySettings??NATIVE_ADJACENCY_SETTINGS);
 const job={native_preparation:request.input,prepared_engine_template:{source:selected.template,settings:{...selected.settings,inspect_only:false}},scope:{state:request.input.state,year:request.input.year,chamber:'congressional',districts:null},...(files.planGeometry?{geometry:files.planGeometry}:{})};
 return structuredClone(validatePreparedNativeJob(job));
}
export function executePreparedNativeJob(engine,job,{onProgress}={}){
 validatePreparedNativeJob(job);if(job.scope.districts===null)throw new Error('Resolve native congressional allocation before source execution.');
 const {sources,...settings}=job.native_preparation;
 onProgress?.({stage:'native-source-preparation',message:'Preparing selected original native adjacency inputs'});
 const prepared=engine.execute(nativeEngineInputRequest(sources,settings),{onProgress});
 try{
  if(prepared.schema_version!=='bisect-native-engine-input-v1'||prepared.execution_performed!==false||prepared.state!==job.scope.state||prepared.year!==job.scope.year)throw new Error('Prepared native graph differs from selected state scope.');
  const binding=engine.execute({operation:'bind-prepared-engine-graph',input:{template_json:job.prepared_engine_template.source.text,graph_json:prepared.graph_json,districts:job.scope.districts}});
  if(binding.schema_version!=='bisect-prepared-engine-binding-v1'||binding.execution_performed!==false||binding.csv_loading_performed!==false||binding.state!==job.scope.state||binding.year!==job.scope.year||binding.districts!==job.scope.districts||binding.graph_sha256!==prepared.graph_sha256||binding.units!==prepared.units||binding.unit_kind!==(settings.unit_kind??'tract'))throw new Error('Native source/template binding differs from the prepared graph.');
  const bound=validatePreparedStateJob({prepared_engine:{source:{name:'bound-native-engine-request.json',text:binding.bound_request_json},settings:job.prepared_engine_template.settings},scope:job.scope,request_sha256:binding.bound_request_sha256,...(job.geometry?{geometry:job.geometry}:{})});
  const result=executePreparedStateJob(engine,bound,{onProgress}),{bound_request_json,review,...receipt}=binding;
  result.template_source=structuredClone(job.prepared_engine_template);result.graph_binding=receipt;result.native_input=structuredClone(job.native_preparation);
  return {schema_version:'bisect-native-source-prepared-state-run-v1',source_preparation:prepared,result};
 }catch(error){error.source_preparation=prepared;throw error;}
}

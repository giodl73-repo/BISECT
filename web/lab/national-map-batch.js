import {validateMapTigerSources} from './raw-map-files.js';
// Source selection and project opening validate envelopes only. Decompression and
// native geometry preparation happen on explicit Run, one state at a time.
import {validateSourceFile,encodeBytes} from './package-files.js';
import {parseNationalMapBatchJson,parseNativeMapPlanJson,parseGeometryJson,parseSafeJson} from './project.js';
import {validateNativeMapInputs} from './map-export-files.js';
const STATES=new Set('AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY'.split(' '));
const isTiger=job=>job?.source_type==='native-tiger-assignments';
const isRaw=job=>job?.source_type==='native-assignments'||isTiger(job);
const CAP=64*1024*1024;
export const MAX_NATIONAL_MAP_PACKAGE_BYTES=128*1024*1024;
export const NATIONAL_MAP_BATCH_DEFAULTS={source:null};
const keys=(v,expected)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===expected;
// Allocate the destination once. Array.from over a large string creates an
// intermediate boxed array that can exhaust the heap before native rendering.
const decode=source=>{const binary=atob(source.base64),bytes=new Uint8Array(binary.length);for(let i=0;i<binary.length;i++)bytes[i]=binary.charCodeAt(i);return bytes;};
const text=bytes=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
const parsedSources=new WeakMap();
export function validateNationalMapBatch(batch){
 if(!batch||typeof batch!=='object'||Array.isArray(batch)||!['dpi','map_type','states'].every(k=>Object.hasOwn(batch,k))||Object.keys(batch).some(k=>!['dpi','map_type','states','failure_policy','analysis_policy'].includes(k))||batch.analysis_policy!==undefined&&!['validated','native-defaults'].includes(batch.analysis_policy)||batch.failure_policy!==undefined&&!['stop','skip-state'].includes(batch.failure_policy)||![72,100,150,200,300].includes(batch.dpi)||!['districts','political','demographic','compactness'].includes(batch.map_type)||!batch.states||typeof batch.states!=='object'||Array.isArray(batch.states)||Object.keys(batch.states).length<1||Object.keys(batch.states).length>50)throw new Error('Choose a national map DPI/type and 1–50 supplied states.');
 let bytes=0,year;
 for(const [state,job] of Object.entries(batch.states)){
  const raw=isRaw(job),tiger=isTiger(job);
  if(!STATES.has(state)||!(tiger?keys(job,'analysis,assignments,dissolve_method,key_basis,source_type,tiger,unit_index,year'):raw?keys(job,'analysis,assignments,dissolve_method,geometry,geometry_format,key_basis,source_type,unit_index,year'):keys(job,'analysis,dissolve_method,geometry,geometry_format,plan'))||!tiger&&(!['json','gzip'].includes(job.geometry_format)||!job.geometry)||!['sequential','balanced','spatial-balanced','batch'].includes(job.dissolve_method))throw new Error('Each state needs exact plan or raw assignments, geometry, explicit format and native dissolve method.');
  if(raw&&(batch.analysis_policy!=='native-defaults'||![2000,2010,2020].includes(job.year)||!['unit-id','native-index-map'].includes(job.key_basis)||!job.assignments||job.key_basis==='unit-id'&&job.unit_index!==null||job.key_basis==='native-index-map'&&!job.unit_index))throw new Error('Raw national state sources require native-defaults analysis policy, explicit year and key mapping.');
  if(tiger){validateMapTigerSources({geometry_source:'tiger',tiger:job.tiger});if(!['tracts','block-groups'].includes(job.tiger?.kind)||job.tiger.year!==job.year||!job.tiger.shp||!job.tiger.dbf||job.dissolve_method!=='sequential')throw new Error('Direct TIGER national sources require matching tract/block-group vintage, SHP/DBF and native input order.');}
  const parts=tiger?[[job.assignments,3*1024*1024],[job.unit_index,3*1024*1024],[job.analysis,3*1024*1024],...['shp','dbf','shx'].map(k=>[job.tiger[k],CAP])]:raw?[[job.assignments,3*1024*1024],[job.unit_index,3*1024*1024],[job.geometry,CAP],[job.analysis,3*1024*1024]]:[[job.plan,3*1024*1024],[job.geometry,CAP],[job.analysis,3*1024*1024]];
  for(const [source,cap]of parts){if(source===null)continue;const empty=source?.base64===''&&(raw&&source!==job.geometry||batch.analysis_policy==='native-defaults'&&source===job.analysis);validateSourceFile(empty?{...source,base64:'AA=='}:source,{maxBytes:cap});bytes+=source.base64.length/4*3-(source.base64.endsWith('==')?2:source.base64.endsWith('=')?1:0);}
  if(raw){if(year!==undefined&&year!==String(job.year))throw new Error('National sources must share one year.');year=String(job.year);}
  else{const plan=parseNativeMapPlanJson(text(decode(job.plan))),units=plan?.plan?.units;if(units?.state!==state||!Array.isArray(units.unit_ids)||!Number.isSafeInteger(plan.plan.k)||plan.plan.k<1||plan.plan.k>500||!/^\d{4}$/.test(String(units.year))||year!==undefined&&year!==String(units.year))throw new Error('State plans must declare matching states, one year, unit IDs and 1–500 districts.');year=String(units.year);}

 }
 if(bytes>MAX_NATIONAL_MAP_PACKAGE_BYTES)throw new Error('Selected national map sources exceed 128 MiB; gzip geometry sources can reduce retained size.');
 return batch;
}
export function nationalMapBatchInput(source){
 validateSourceFile(source,{maxBytes:MAX_NATIONAL_MAP_PACKAGE_BYTES});const prior=parsedSources.get(source);
 if(prior?.base64===source.base64&&prior.name===source.name)return prior.batch;
 const batch=validateNationalMapBatch(parseNationalMapBatchJson(text(decode(source))));
 for(const job of Object.values(batch.states)){for(const part of [job.plan,job.geometry,job.analysis,job.assignments,job.unit_index,...Object.values(job.tiger||{})])if(part)Object.freeze(part);if(job.tiger)Object.freeze(job.tiger);Object.freeze(job);}Object.freeze(batch.states);Object.freeze(batch);
 parsedSources.set(source,{name:source.name,base64:source.base64,batch});return batch;
}
export function validateNationalMapBatchInputs(v){if(!keys(v,'source'))throw new Error('Invalid national map batch inputs.');if(v.source!==null)nationalMapBatchInput(v.source);return v;}
export function nationalMapBatchRequest(v){validateNationalMapBatchInputs(v);if(!v.source)throw new Error('Select a national map source package.');return {operation:'render-native-national-map-batch',input:nationalMapBatchInput(v.source)};}
export async function readNationalMapBatchSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_NATIONAL_MAP_PACKAGE_BYTES)throw new Error('Select a national map source package up to 128 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('National map package changed during selection.');
 const source={name:file.name,base64:encodeBytes(bytes)};nationalMapBatchInput(source);return source;
}
async function transformGeometry(original,compress){
 const Codec=compress?globalThis.CompressionStream:globalThis.DecompressionStream;if(typeof Codec!=='function')throw new Error('This browser cannot process gzip geometry.');
 const reader=new Blob([original]).stream().pipeThrough(new Codec('gzip')).getReader();let size=0;const chunks=[];
 try{
  for(;;){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>CAP)throw new Error('Geometry gzip transform exceeds 64 MiB.');chunks.push(value);}
 }catch(error){await reader.cancel().catch(()=>{});throw error;}finally{reader.releaseLock();}
 const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}return bytes;
}
async function geometryBytes(job){const original=decode(job.geometry);return job.geometry_format==='json'?original:transformGeometry(original,false);}
export async function addNativeMapToNationalBatch(previous,inputs){
 validateNativeMapInputs(inputs);const raw=inputs.source_mode==='native-assignments',tiger=raw&&inputs.geometry_source==='tiger';if((tiger?!inputs.tiger?.shp||!inputs.tiger?.dbf:!inputs.geometry)||inputs.map_type==='rounds'||inputs.round_assignments||inputs.round_index||raw&&!inputs.assignments||!raw&&!inputs.plan)throw new Error('Select final plan or original native assignments and geometry before adding a national state.');
 const plan=raw?null:parseNativeMapPlanJson(text(decode(inputs.plan))),state=raw?inputs.raw_state:plan.plan.units.state;if(!STATES.has(state))throw new Error('National map sources require one of the 50 states.');
 const map_type=inputs.map_type||'districts',prior=previous?.source?nationalMapBatchInput(previous.source):null;
 if(prior&&(prior.dpi!==inputs.dpi||prior.map_type!==map_type))throw new Error('Choose matching DPI and map type before adding another state.');
 if(raw&&prior&&prior.analysis_policy!=='native-defaults')throw new Error('Choose Native defaults in the national analysis policy before adding raw assignments.');
 const compressed=tiger?null:await transformGeometry(decode(inputs.geometry),true),geometry=tiger?null:{name:inputs.geometry.name,base64:encodeBytes(compressed)};
 const job=tiger?{source_type:'native-tiger-assignments',year:inputs.raw_year,key_basis:inputs.assignment_key_basis||'unit-id',assignments:inputs.assignments,unit_index:inputs.assignment_index||null,tiger:inputs.tiger,analysis:inputs.analysis||null,dissolve_method:'sequential'}:raw?{source_type:'native-assignments',year:inputs.raw_year,key_basis:inputs.assignment_key_basis||'unit-id',assignments:inputs.assignments,unit_index:inputs.assignment_index||null,geometry,geometry_format:'gzip',analysis:inputs.analysis||null,dissolve_method:inputs.dissolve_method||'sequential'}:{plan:inputs.plan,geometry,geometry_format:'gzip',analysis:inputs.analysis||null,dissolve_method:inputs.dissolve_method||'sequential'};
 const batch=validateNationalMapBatch({dpi:inputs.dpi,map_type,...(prior?.failure_policy?{failure_policy:prior.failure_policy}:{}),...(raw||prior?.analysis_policy?{analysis_policy:raw?'native-defaults':prior.analysis_policy}:{}),states:{...prior?.states,[state]:job}});
 const source={name:'selected-national-map-sources.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(batch)))};return validateNationalMapBatchInputs({source});
}
export function nationalMapBatchSummary(inputs){
 if(!inputs?.source)return [];
 return Object.entries(nationalMapBatchInput(inputs.source).states).sort().map(([state,job])=>{
  if(isRaw(job))return {state,source_type:job.source_type,units:null,districts:null,seed:null,geometry_format:isTiger(job)?job.tiger.kind==='block-groups'?'block groups (TIGER)':'native-tiger':job.geometry_format};const plan=parseNativeMapPlanJson(text(decode(job.plan)));return {state,units:plan.plan.units.unit_ids.length,districts:plan.plan.k,seed:plan.provenance?.producer?.engine_options?.seed??null,geometry_format:job.geometry_format};
 });
}
export function nationalMapFailurePolicy(inputs){return inputs?.source?nationalMapBatchInput(inputs.source).failure_policy||'stop':'stop';}
export function setNationalMapFailurePolicy(inputs,policy){
 if(!['stop','skip-state'].includes(policy)||!inputs?.source)throw new Error('Select national state sources and a failure policy.');
 const batch=validateNationalMapBatch({...nationalMapBatchInput(inputs.source),failure_policy:policy});return validateNationalMapBatchInputs({source:{name:'national-map-policy-sources.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(batch)))}});
}
export function nationalMapAnalysisPolicy(inputs){return inputs?.source?nationalMapBatchInput(inputs.source).analysis_policy||'validated':'validated';}
export function setNationalMapAnalysisPolicy(inputs,policy){
 if(!['validated','native-defaults'].includes(policy)||!inputs?.source)throw new Error('Select national state sources and an analysis policy.');
 const batch=validateNationalMapBatch({...nationalMapBatchInput(inputs.source),analysis_policy:policy});return validateNationalMapBatchInputs({source:{name:'national-map-analysis-policy-sources.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(batch)))}});
}
export async function executeNationalMapBatch(engine,batch,{onProgress}={}){
 validateNationalMapBatch(batch);const nativeAnalysis=batch.analysis_policy==='native-defaults',partial=batch.failure_policy==='skip-state',requested=Object.keys(batch.states).sort(),outcomes=[];
 const started=engine.execute({operation:'begin-native-national-map',input:{dpi:batch.dpi,map_type:batch.map_type,...(partial?{allow_empty:true}:{}),...(nativeAnalysis?{analysis_policy:'native-defaults'}:{}),...(requested.every(state=>isTiger(batch.states[state]))?{output_profile:'native-original'}:{})}}),session=started.session;
 if(started.schema_version!=='bisect-national-map-stream-v1'||started.status!=='started'||!Number.isSafeInteger(session)||session<1)throw new Error('Invalid native national map session.');
 let finished=false,accepted=0;
 const omitted=(state,stage,error)=>{const observed=String(error?.message??error),message=observed.trim()?observed:'State source could not be prepared.';outcomes.push({state,status:'omitted',stage,error:message.slice(0,16000)});onProgress?.({schema_version:'bisect-national-map-stream-v1',status:'state-omitted',session,state,states:accepted,requested_states:requested.length});};
 try{
  for(const state of requested){
   const job=batch.states[state];let input;
   try{
    if(isTiger(job)){input={session,state,input:{state,tiger:job.tiger,key_basis:job.key_basis,assignments_base64:job.assignments.base64,map_type:batch.map_type,dpi:batch.dpi,...(job.unit_index?{unit_index_base64:job.unit_index.base64}:{}),...(job.analysis?{analysis_base64:job.analysis.base64}:{})}};}else{
    const unit_geometry_json=text(await geometryBytes(job));parseGeometryJson(unit_geometry_json);
    if(job.source_type==='native-assignments'){input={session,state,input:{state,year:job.year,key_basis:job.key_basis,assignments_base64:job.assignments.base64,unit_geometry_json,map_type:batch.map_type,dpi:batch.dpi,dissolve_method:job.dissolve_method,...(job.unit_index?{unit_index_base64:job.unit_index.base64}:{}),...(job.analysis?{analysis_base64:job.analysis.base64}:{})}};}
    else{const document_json=text(decode(job.plan)),analysis_json=job.analysis&&!nativeAnalysis?text(decode(job.analysis)):null;parseNativeMapPlanJson(document_json);if(analysis_json!==null)parseSafeJson(analysis_json);input={session,state,input:{document_json,unit_geometry_json,dpi:batch.dpi,map_type:batch.map_type,dissolve_method:job.dissolve_method,...(nativeAnalysis&&job.analysis?{analysis_base64:job.analysis.base64}:analysis_json===null?{}:{analysis_json})}};}

    }
   }catch(error){if(!partial||error instanceof RangeError||error instanceof WebAssembly.RuntimeError)throw error;omitted(state,'source-preparation',error);continue;}
   if(partial||isTiger(job)){
    // Lifecycle/unknown-operation/runtime failures throw and abort; only a typed
    // native state-preparation failure can be omitted after an attempted append.
    const step=engine.execute({operation:isTiger(job)?'append-native-national-tiger-map-step':job.source_type==='native-assignments'?'append-native-national-raw-map-step':'append-native-national-map-step',input});
    if(!keys(step,'error,progress,schema_version,status')||step.schema_version!=='bisect-national-map-append-step-v1')throw new Error('Invalid national map state-step response.');
    if(step.status==='state-error'){
     if(typeof step.error!=='string'||new TextEncoder().encode(step.error).length>65536||step.progress!==null)throw new Error('Invalid national map state error.');if(!partial)throw new Error(step.error);omitted(state,'native-state-preparation',step.error);continue;
    }
    if(step.status!=='appended'||step.error!==null||step.progress?.schema_version!=='bisect-national-map-stream-v1'||step.progress.status!=='state-prepared'||step.progress.session!==session||step.progress.state!==state||step.progress.states!==accepted+1)throw new Error('Invalid native national state append receipt.');
    accepted++;outcomes.push({state,status:'included',stage:null,error:null});onProgress?.({...step.progress,requested_states:requested.length});
   }else{const progress=engine.execute({operation:job.source_type==='native-assignments'?'append-native-national-raw-map':'append-native-national-map',input});accepted++;onProgress?.({...progress,requested_states:requested.length});}
  }
  const result=engine.execute({operation:'finish-native-national-map',session});finished=true;
  if(partial){
   if(result?.schema_version!=='bisect-native-district-map-v1'||result.status!=='rendered'||!Array.isArray(result.provided_states)||result.provided_states.length!==accepted)throw new Error('Invalid partial national map coverage receipt.');
   result.national_coverage={failure_policy:'skip-state',requested_states:requested,included_states:outcomes.filter(row=>row.status==='included').map(row=>row.state),omitted_states:outcomes.filter(row=>row.status==='omitted'),state_outcomes:outcomes,all_requested_states_included:accepted===requested.length,empty:accepted===0};
   result.scope+=' Explicit skip-state policy: only successfully prepared supplied states are rendered. national_coverage lists all requested states, omissions and reasons. An empty map contains no successfully prepared state; it is not national coverage. Browser source, plan, identity and work limits remain distinct from native CLI filesystem inputs. No failed state is fabricated or silently substituted.';
  }
  if(nativeAnalysis){result.national_analysis_policy='native-defaults';result.scope+=' Explicit native national analysis policy: missing/unreadable/invalid analysis uses political 0.5, demographic 0.0 or compactness 0.3 defaults. Native numeric row filtering and last-valid-duplicate semantics apply. Declared analysis metadata is ignored as in the CLI, not validated or authenticated. For each included state, the supplied analysis source hash, observation status and fallback districts are recorded in provided_states. Defaults are display inputs, not measured statistics.';}
  const requestedRaw=requested.filter(state=>isRaw(batch.states[state]));if(requestedRaw.length){const included=result.provided_states.filter(state=>state.map_source==='native-assignments').map(state=>state.state);result.national_source_coverage={requested_raw_states:requestedRaw,included_raw_states:included,included_rplan_states:result.provided_states.filter(state=>state.map_source!=='native-assignments').map(state=>state.state)};result.map_source='native-assignment-stream';result.plan_hash=null;result.scope+=' Original native assignments are joined to supplied geometry without creating an RPLAN. National source coverage distinguishes raw snapshots and supplied RPLAN states; each included state retains its actual source hashes. No raw state inherits a retained plan, generated-plan identity or certification.';}
  return result;
 }finally{if(!finished){try{engine.execute({operation:'abort-native-national-map',session});}catch{/* A failed finish has already released native geometry. */}}}
}

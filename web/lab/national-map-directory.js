// Portable observations contain metadata only. Selected File snapshots stay in
// the current session and are cloned to the worker, never serialized in a project.
import {characterStateFips} from './character-input.js';
import {encodeBytes} from './package-files.js';
const names='alabama alaska arizona arkansas california colorado connecticut delaware florida georgia hawaii idaho illinois indiana iowa kansas kentucky louisiana maine maryland massachusetts michigan minnesota mississippi missouri montana nebraska nevada new_hampshire new_jersey new_mexico new_york north_carolina north_dakota ohio oklahoma oregon pennsylvania rhode_island south_carolina south_dakota tennessee texas utah vermont virginia washington west_virginia wisconsin wyoming'.split(' ');
const codes='AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY'.split(' ');
const states=new Map(names.map((name,i)=>[name,codes[i]]));
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===keys;
const safe=s=>typeof s==='string'&&s.length>0&&new TextEncoder().encode(s).length<=200&&!/[\\/\0]/.test(s)&&!['.','..','__proto__','constructor','prototype'].includes(s);
const compare=(a,b)=>{const x=new TextEncoder().encode(a),y=new TextEncoder().encode(b);for(let i=0;i<Math.min(x.length,y.length);i++)if(x[i]!==y[i])return x[i]-y[i];return x.length-y.length;};
const check=o=>{if(o.signal?.aborted)throw o.signal.reason||new Error('National folder selection cancelled.');if(o.isCurrent&&!o.isCurrent())throw new Error('Project changed during national folder selection. Select again.');};
class SourceLimitError extends Error {}
class SourceReadError extends Error {}
const message=e=>String(e?.message??e).slice(0,2000);
const allRefs=row=>[row.assignments,row.analysis,row.index,...Object.values(row.tiger||{})].filter(Boolean);
export const NATIONAL_DIRECTORY_DEFAULTS={version:'v1',year:2020,dpi:150,map_type:'districts',key_basis:'native-index-map',geometry_profile:'source-exact-large',dissolve_method:'sequential',force:false,observation:null};
export function validateNationalMapDirectory(v){
 if(!exact(v,'dissolve_method,dpi,force,geometry_profile,key_basis,map_type,observation,version,year')||!safe(v.version)||!['source-exact','source-exact-large','native-cli'].includes(v.geometry_profile)||!['unit-id','native-index-map'].includes(v.key_basis)||!['sequential','balanced','spatial-balanced','batch'].includes(v.dissolve_method)||![2000,2010,2020].includes(v.year)||![72,100,150,200,300].includes(v.dpi)||!['districts','political','demographic','compactness'].includes(v.map_type)||typeof v.force!=='boolean')throw new Error('Invalid native national directory settings.');
 if(v.geometry_profile==='native-cli'&&v.dissolve_method!=='sequential')throw new Error('Native CLI loading uses native input-order dissolve. Select that method explicitly.');
 if(v.observation===null)return v;
 const o=v.observation;if(!exact(o,'existing_output,key_basis,map_type,rows,version,year')||o.version!==v.version||o.year!==v.year||o.map_type!==v.map_type||o.key_basis!==v.key_basis||typeof o.existing_output!=='boolean'||!Array.isArray(o.rows)||o.rows.length>1000)throw new Error('Select national folders again after changing version, vintage or map type.');
 let previous=null;
 for(const row of o.rows){
  if(!exact(row,'analysis,analysis_error,assignments,error,folder,index,state,tiger')||!safe(row.folder)||row.state!==(states.get(row.folder)||null)||previous!==null&&compare(previous,row.folder)>=0||[row.error,row.analysis_error].some(e=>e!==null&&(typeof e!=='string'||e.length>2000)))throw new Error('Invalid native national state folder observation.');previous=row.folder;
  const base=v.version+'/'+v.year+'/states/'+row.folder;
  const validateRef=(r,root,paths,limit)=>{if(r===null)return;if(!exact(r,'byte_length,last_modified,name,path,root')||r.root!==root||!paths.includes(r.path)||r.name!==r.path.split('/').at(-1)||!Number.isSafeInteger(r.byte_length)||r.byte_length<0||r.byte_length>limit||!Number.isSafeInteger(r.last_modified)||r.last_modified<0)throw new Error('Invalid selected national source reference.');};
  validateRef(row.assignments,'outputs',[base+'/data/final_assignments.json'],3*1024*1024);
  validateRef(row.analysis,'outputs',[base+'/analysis/'+v.map_type+'.json'],3*1024*1024);
  const index=row.state?.toLowerCase()+'_adjacency_'+v.year+'_geoids.json';validateRef(row.index,'outputs',[v.version,'V3','V4'].map(version=>version+'/data/'+v.year+'/adjacency/'+index),3*1024*1024);
  if(row.tiger!==null){if(!row.state||!exact(row.tiger,'dbf,shp,shx'))throw new Error('Invalid selected TIGER reference set.');const stem='tl_'+v.year+'_'+characterStateFips(row.state)+'_tract';let bytes=0;for(const part of ['shp','dbf','shx']){validateRef(row.tiger[part],'data',[v.year+'/tiger/tracts/'+stem+'/'+stem+'.'+part],64*1024*1024);bytes+=row.tiger[part]?.byte_length||0;}if(!row.tiger.shp||!row.tiger.dbf||bytes>90*1024*1024)throw new Error('Selected national TIGER state exceeds source limits.');}
 }
 if(new TextEncoder().encode(JSON.stringify(v)).length>2*1024*1024)throw new Error('National directory observation exceeds 2 MiB.');return v;
}
async function handleAt(root,path,kind,optional=false){
 try{let handle=root;const parts=path.split('/');for(let i=0;i<parts.length;i++)handle=await handle[i===parts.length-1&&kind==='file'?'getFileHandle':'getDirectoryHandle'](parts[i]);if(handle.kind!==kind)throw new Error('Wrong entry type at '+path);return handle;}
 catch(error){if(optional&&error.name==='NotFoundError')return null;throw error;}
}
export async function observeNationalMapDirectories(outputs,data,settings,options={}){
 const v=validateNationalMapDirectory({...settings,observation:null});if(outputs?.kind!=='directory'||data?.kind!=='directory')throw new Error('Choose output-base and native data folders.');
 const files=Object.create(null),roots={outputs,data};
 const ref=async(root,path,cap,optional=false)=>{check(options);const h=await handleAt(roots[root],path,'file',optional);check(options);if(!h)return null;const file=await h.getFile();check(options);if(!(file instanceof File)||!Number.isSafeInteger(file.size)||file.size<0||file.size>cap)throw new SourceLimitError('Source exceeds its size limit: '+path);const r={root,path,name:file.name,byte_length:file.size,last_modified:file.lastModified};files[root+'/'+path]=file;return r;};
 check(options);const stateDir=await handleAt(outputs,v.version+'/'+v.year+'/states','directory');const entries=[];let entryCount=0;
 for await(const [name,h]of stateDir.entries()){check(options);if(++entryCount>1000)throw new Error('National discovery exceeds 1,000 entries.');if(h.kind==='directory'){if(!safe(name))throw new Error('Unsafe native state folder name.');entries.push(name);}}
 entries.sort(compare);const rows=[];
 for(const folder of entries){check(options);const state=states.get(folder)||null,base=v.version+'/'+v.year+'/states/'+folder,row={folder,state,assignments:null,analysis:null,analysis_error:null,tiger:null,index:null,error:null};rows.push(row);
  try{
   row.assignments=await ref('outputs',base+'/data/final_assignments.json',3*1024*1024,true);if(!row.assignments||!state)continue;
   if(v.map_type!=='districts')try{row.analysis=await ref('outputs',base+'/analysis/'+v.map_type+'.json',3*1024*1024,true);}catch(error){check(options);if(error instanceof SourceLimitError)throw error;row.analysis_error=message(error);}
   if(v.key_basis==='native-index-map')for(const version of [...new Set([v.version,'V3','V4'])]){row.index=await ref('outputs',version+'/data/'+v.year+'/adjacency/'+state.toLowerCase()+'_adjacency_'+v.year+'_geoids.json',3*1024*1024,true);if(row.index)break;}
   const stem='tl_'+v.year+'_'+characterStateFips(state)+'_tract',tiger={shp:null,dbf:null,shx:null};for(const part of ['shp','dbf','shx'])tiger[part]=await ref('data',v.year+'/tiger/tracts/'+stem+'/'+stem+'.'+part,64*1024*1024,part==='shx');if(Object.values(tiger).reduce((sum,r)=>sum+(r?.byte_length||0),0)>90*1024*1024)throw new SourceLimitError('Selected TIGER parts exceed 90 MiB for '+state);row.tiger=tiger;
  }catch(error){check(options);row.error=message(error);}
 }
 let existing_output=false;try{const maps=await handleAt(outputs,v.version+'/'+v.year+'/national/maps','directory',true);let mapsCount=0;if(maps)for await(const [name]of maps.entries()){check(options);if(++mapsCount>10000)throw new Error('Existing map directory exceeds 10,000 entries.');if(name===v.map_type+'.png'){existing_output=true;break;}}}catch(error){check(options);throw new Error('Cannot observe existing national output: '+message(error));}
 check(options);v.observation={version:v.version,year:v.year,map_type:v.map_type,key_basis:v.key_basis,existing_output,rows};validateNationalMapDirectory(v);
 const retained=Object.create(null);for(const row of rows)for(const r of allRefs(row))retained[r.root+'/'+r.path]=files[r.root+'/'+r.path];
 return {inputs:v,files:retained};
}
export function nationalDirectoryRequest(inputs,capture){
 validateNationalMapDirectory(inputs);if(!inputs.observation)throw new Error('Select native national folders first.');
 if(!capture||JSON.stringify(capture.inputs.observation)!==JSON.stringify(inputs.observation))throw new Error('Select both native folders again to grant this session source access.');
 return {operation:'render-native-national-directory',input:inputs,sources:capture.files};
}
function validateSnapshots(v,sources){
 if(!sources||typeof sources!=='object'||Array.isArray(sources))throw new Error('Missing selected File snapshots.');const refs=new Map(v.observation.rows.flatMap(allRefs).map(r=>[r.root+'/'+r.path,r]));
 if(Object.keys(sources).length!==refs.size)throw new Error('Selected files differ from the observed references.');for(const [id,r]of refs){const f=sources[id];if(!(f instanceof File)||f.name!==r.name||f.size!==r.byte_length||f.lastModified!==r.last_modified)throw new Error('Reselect changed native source: '+id);}
}
async function readSource(ref,sources,hashes){const file=sources[ref.root+'/'+ref.path];let buffer;try{buffer=await file.arrayBuffer();}catch(error){if(error instanceof RangeError||error instanceof WebAssembly.RuntimeError)throw error;throw new SourceReadError(message(error));}const bytes=new Uint8Array(buffer);if(bytes.length!==ref.byte_length)throw new SourceReadError('Selected native source changed during Run.');const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');hashes.push({...ref,sha256});return {name:ref.name,base64:encodeBytes(bytes)};}
export async function executeNationalMapDirectory(engine,request,{onProgress}={}){
 if(!exact(request,'input,operation,sources')||request.operation!=='render-native-national-directory')throw new Error('Invalid national directory request.');const v=validateNationalMapDirectory(request.input);if(!v.observation)throw new Error('Missing native national observation.');validateSnapshots(v,request.sources);
 const output_path=v.version+'/'+v.year+'/national/maps/'+v.map_type+'.png',base={schema_version:'bisect-native-national-directory-v1',output_path,status:'skipped-existing',rows:[],map:null,scope:'Read-only explicitly selected native roots. Observed paths and metadata are source claims; actual Run hashes bind bytes. No file was written. Folder references grant no access after Open; reselect both roots. No RPLAN, partition run, provenance authentication or certification is created.'};
 if(v.observation.existing_output&&!v.force)return base;
 const started=engine.execute({operation:'begin-native-national-map',input:{dpi:v.dpi,map_type:v.map_type,allow_empty:true,analysis_policy:'native-defaults',...(v.geometry_profile==='native-cli'?{output_profile:'native-original'}:{})}}),session=started.session;let finished=false,failureStage='state-processing';
 if(started.schema_version!=='bisect-national-map-stream-v1'||started.status!=='started'||!Number.isSafeInteger(session)||session<1)throw new Error('Invalid national directory session.');
 try{
  for(const row of v.observation.rows){
   const observed={folder:row.folder,state:row.state,status:'skipped-missing-assignments',error:null,analysis_error:row.analysis_error,source_geometry:null,source_hashes:[]};base.rows.push(observed);
   if(row.error){observed.status='omitted-source';observed.error=row.error;continue;}if(!row.assignments)continue;if(!row.state){observed.status='omitted-unknown-state';continue;}
   observed.status='preparing';let assignments,index,analysis=null,tiger;
   try{
    if(!row.tiger||v.key_basis==='native-index-map'&&!row.index)throw new SourceReadError('Missing tract geometry or explicit native index companion.');
    assignments=await readSource(row.assignments,request.sources,observed.source_hashes);index=v.key_basis==='native-index-map'?await readSource(row.index,request.sources,observed.source_hashes):null;
    if(row.analysis)try{analysis=await readSource(row.analysis,request.sources,observed.source_hashes);}catch(error){if(!(error instanceof SourceReadError))throw error;observed.analysis_error=message(error);}
    tiger={kind:'tracts',year:v.year,shp:null,dbf:null,shx:null};for(const part of ['shp','dbf','shx'])if(row.tiger[part])tiger[part]=await readSource(row.tiger[part],request.sources,observed.source_hashes);
   }catch(error){if(!(error instanceof SourceReadError))throw error;observed.status='omitted-source';observed.error=message(error);continue;}
   // Only typed source-geometry/state failures are omissions. Lifecycle, serde,
   // unknown-operation, allocation, runtime and malformed replies abort the run.
   let step;
   if(v.geometry_profile==='native-cli'){
    step=engine.execute({operation:'append-native-national-tiger-map-step',input:{session,state:row.state,input:{state:row.state,tiger,key_basis:v.key_basis,assignments_base64:assignments.base64,...(index?{unit_index_base64:index.base64}:{}),...(analysis?{analysis_base64:analysis.base64}:{}),map_type:v.map_type,dpi:v.dpi}}});
   }else{
   const preparedStep=engine.execute({operation:'prepare-native-map-tiger-step',input:{state:row.state,tiger,geometry_profile:v.geometry_profile}});
   if(!exact(preparedStep,'error,prepared,schema_version,status')||preparedStep.schema_version!=='bisect-native-map-tiger-step-v1')throw new Error('Invalid native map geometry step.');
   if(preparedStep.status==='geometry-error'){
    if(typeof preparedStep.error!=='string'||preparedStep.prepared!==null)throw new Error('Invalid native geometry failure.');observed.status='omitted-geometry';observed.error=preparedStep.error;continue;
   }
   const prepared=preparedStep.prepared;
   if(preparedStep.status!=='prepared'||preparedStep.error!==null||prepared?.schema_version!=='bisect-native-map-tiger-v1'||prepared.state!==row.state||prepared.year!==v.year||typeof prepared.geometry_json!=='string')throw new Error('Invalid native map geometry preparation receipt.');
   const {geometry_json,...receipt}=prepared;observed.source_geometry=receipt;
    step=engine.execute({operation:'append-native-national-raw-map-step',input:{session,state:row.state,input:{state:row.state,year:v.year,key_basis:v.key_basis,assignments_base64:assignments.base64,...(index?{unit_index_base64:index.base64}:{}),...(analysis?{analysis_base64:analysis.base64}:{}),unit_geometry_json:geometry_json,map_type:v.map_type,dpi:v.dpi,dissolve_method:v.dissolve_method}}});
   }
   if(!exact(step,'error,progress,schema_version,status')||step.schema_version!=='bisect-national-map-append-step-v1')throw new Error('Invalid national directory state-step receipt.');
   if(step.status==='state-error'){if(typeof step.error!=='string'||step.progress!==null)throw new Error('Invalid national directory state failure.');observed.status='omitted-preparation';observed.error=step.error;}
   else if(step.status==='appended'&&step.error===null&&step.progress?.schema_version==='bisect-national-map-stream-v1'&&step.progress.status==='state-prepared'&&step.progress.session===session&&step.progress.state===row.state)observed.status='included';else throw new Error('Invalid national directory state progress.');
   onProgress?.({schema_version:'bisect-native-national-directory-progress-v1',state:row.state,completed:base.rows.length,total:v.observation.rows.length,status:observed.status});
  }
  failureStage='render';base.map=engine.execute({operation:'finish-native-national-map',session});finished=true;base.status='rendered';
  if(base.map?.schema_version!=='bisect-native-district-map-v1'||base.map.status!=='rendered'||base.map.provided_states?.length!==base.rows.filter(r=>r.status==='included').length)throw new Error('Invalid national directory coverage.');
  base.map.map_source='native-assignment-directory';base.map.plan_hash=null;return base;
 }catch(error){
  const reason=message(error),receipt={...base,status:'failed',map:null,failure_stage:failureStage,error:reason,request_scope:{version:v.version,year:v.year,dpi:v.dpi,map_type:v.map_type,key_basis:v.key_basis,geometry_profile:v.geometry_profile,dissolve_method:v.dissolve_method,force:v.force,requested_folders:v.observation.rows.map(row=>row.folder)},runtime_artifact:engine.artifactIdentity??null,rows:base.rows.map(row=>({...row,status:row.status==='included'?'prepared-before-failure':row.status==='preparing'?'failed-runtime':row.status,error:row.status==='preparing'?reason:row.error})),scope:base.scope+' No completed map was produced. Prepared-state rows describe source processing only.'};
  error.source_preparation=receipt;throw error;
 }finally{if(!finished)try{engine.execute({operation:'abort-native-national-map',session});}catch{}}
}

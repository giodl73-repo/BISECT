import {encodeBytes} from './package-files.js';
import {stateCommandArguments} from './state-command-files.js';
import {nativeStateNameComponent} from './native-state-name-effects.js';

const FILE_LIMIT=8*1024*1024,TOTAL_LIMIT=16*1024*1024;
function validatePlan(plan){
 const paths=plan?.required_observation_paths,candidates=plan?.report_candidates,identity=plan?.identity;
 if(plan?.schema_version!=='bisect-native-state-report-sources-v1'||plan.status!=='selection-required'||plan.observed!==false||plan.filesystem_reads_performed!==false||plan.execution_performed!==false||plan.path_authority_granted!==false||plan.source_discovery_authorized!==false||!Array.isArray(paths)||paths.length<1||paths.length>5||new Set(paths).size!==paths.length||!Array.isArray(candidates)||candidates.length!==4||![plan.availability_path,...candidates].every(p=>paths.includes(p))||paths.some(p=>typeof p!=='string'||!/^data\/\d{4}\/demographics\/[A-Za-z0-9_ -]{1,110}\.csv$/.test(p)))throw Error('Invalid portable native report-source plan.');
 if(identity?.schema_version!=='bisect-native-state-identity-v1'||identity.native_year_valid!==true||!/^\d{4}$/.test(identity.effective_year)||!nativeStateNameComponent(identity.effective_state_name)||!nativeStateNameComponent(identity.state_code)||paths.some(p=>!p.startsWith('data/'+identity.effective_year+'/demographics/')))throw Error('Native report paths differ from the reviewed State scope.');
 return plan;
}
function scope(options){
 const current=()=>{options.signal?.throwIfAborted();if(options.isCurrent&&options.isCurrent()!==true)throw Error('State flags, project or selected source root changed. Load the snapshot again.');};
 return {current,async wait(action){current();const value=await action();current();return value;}};
}
function capture(command,options){
 stateCommandArguments(command);
 const original=structuredClone(command),unitIds=options.unitIds?structuredClone(options.unitIds):undefined;
 const bound={...options,...(unitIds?{unitIds}:{}),isCurrent:()=>command.arguments_json===original.arguments_json&&(!options.isCurrent||options.isCurrent()===true)};
 return {command:original,options:bound,guard:scope(bound)};
}
export async function planNativeReportSources(engine,command,options={}){
 const {command:original,guard}=capture(command,options),arguments_=stateCommandArguments(original);
 return validatePlan(await guard.wait(()=>engine.execute({operation:'inspect-native-state-report-sources',arguments:arguments_})));
}
async function snapshotResult(engine,command,plan,observations,origin,options,guard){
 guard.current();
 const request={operation:'inspect-native-state-report-sources',arguments:stateCommandArguments(command),observations,observation_origin:origin,...(options.unitIds?{unit_ids:structuredClone(options.unitIds)}:{})};
 const result=await guard.wait(()=>engine.execute(request));
 if(result?.schema_version!=='bisect-native-state-report-sources-v1'||result.observed!==true||JSON.stringify(result.identity)!==JSON.stringify(plan.identity)||result.observation_origin!==origin||!['sources-observed','prepared-report-counts','native-load-failure'].includes(result.status)||!/^[a-f0-9]{64}$/.test(result.source_bundle_sha256??'')||result.filesystem_reads_performed!==false||result.path_authority_granted!==false)throw Error('Native report-source preparation differs from its reviewed plan.');
 // Original bytes occur once, in the replayable request. No handle is returned.
 const {source_bundle,...receipt}=result;
 return {schema_version:'bisect-native-report-source-snapshot-v1',source_command:structuredClone(command),request,receipt,live_folder_authority:false};
}

/** Read a working-folder snapshot twice; no directory is created or retained. */
export async function readNativeReportSources(engine,command,root,options={}){
 const captured=capture(command,options),guard=captured.guard;command=captured.command;options=captured.options;
 if(root?.kind!=='directory'||typeof root.getDirectoryHandle!=='function')throw Error('Select a live native working folder explicitly.');
 const plan=await planNativeReportSources(engine,command,options);
 async function observe(){
  let directory=root;
  const parent=plan.required_observation_paths[0].split('/').slice(0,-1);
  for(const component of parent){
   try{directory=await guard.wait(()=>directory.getDirectoryHandle(component,{create:false}));}
   catch(error){if(['NotFoundError','TypeMismatchError'].includes(error.name))return plan.required_observation_paths.map(relative_path=>({relative_path,kind:'missing'}));throw error;}
   if(directory?.kind!=='directory')throw Error('Unexpected native source parent handle.');
  }
  const observations=[];let total=0;
  for(const relative_path of plan.required_observation_paths){
   guard.current();options.onProgress?.({relative_path,stage:'reading-source-snapshot'});guard.current();
   const basename=relative_path.split('/').at(-1);let handle;
   try{handle=await guard.wait(()=>directory.getFileHandle(basename,{create:false}));}
   catch(error){
    if(error.name==='NotFoundError'){observations.push({relative_path,kind:'missing'});continue;}
    if(error.name==='TypeMismatchError'){
     const folder=await guard.wait(()=>directory.getDirectoryHandle(basename,{create:false}));
     if(folder?.kind!=='directory')throw Error('Source candidate changed type while reading.');
     observations.push({relative_path,kind:'directory'});continue;
    }
    throw error;
   }
   if(handle?.kind!=='file'||typeof handle.getFile!=='function')throw Error('Unexpected native CSV handle.');
   const file=await guard.wait(()=>handle.getFile());
   if(!Number.isSafeInteger(file.size)||file.size<0||file.size>FILE_LIMIT)throw Error('A native report CSV exceeds 8 MiB.');
   const bytes=new Uint8Array(await guard.wait(()=>file.arrayBuffer()));total+=bytes.length;
   if(bytes.length!==file.size||bytes.length>FILE_LIMIT||total>TOTAL_LIMIT)throw Error('Native report-source snapshot changed or exceeds 16 MiB.');
   observations.push({relative_path,kind:'file',source_base64:encodeBytes(bytes)});
  }
  return observations;
 }
 const first=await observe(),second=await observe();guard.current();
 if(JSON.stringify(first)!==JSON.stringify(second))throw Error('Native report sources changed while loading. No snapshot was applied.');
 return snapshotResult(engine,command,plan,first,'selected-folder-snapshot',options,guard);
}

/** Explicit file bundles describe their own absence, never the computer's disk. */
export async function nativeReportSourcesFromFiles(engine,command,files,options={}){
 const captured=capture(command,options),guard=captured.guard;command=captured.command;options=captured.options;
 if(options.acceptUnselectedAsMissing!==true)throw Error('Explicitly choose to treat unselected candidates as absent in this portable bundle.');
 if(!Array.isArray(files)||files.length>5)throw Error('Select only the requested native report files.');
 const plan=await planNativeReportSources(engine,command,options),selected=new Map();let total=0;
 for(const file of files){
  guard.current();
  if(!file||!plan.required_observation_paths.includes(file.relative_path)||selected.has(file.relative_path)||!(file.bytes instanceof Uint8Array)||file.bytes.length>FILE_LIMIT)throw Error('Unexpected, duplicate or oversized native report file.');
  total+=file.bytes.length;if(total>TOTAL_LIMIT)throw Error('Native report-source bundle exceeds 16 MiB.');
  selected.set(file.relative_path,encodeBytes(file.bytes));
 }
 const observations=plan.required_observation_paths.map(relative_path=>selected.has(relative_path)?{relative_path,kind:'file',source_base64:selected.get(relative_path)}:{relative_path,kind:'missing'});
 return snapshotResult(engine,command,plan,observations,'explicit-file-bundle',options,guard);
}

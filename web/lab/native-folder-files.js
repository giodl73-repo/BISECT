import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {selectedFolderEntries,observedFolderPaths,portableRelativePath} from './run-target-files.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {parseSafeJson} from './project.js';
import {readPreparedEngineSource} from './prepared-engine-files.js';
import {createPreparedNativeJob} from './prepared-native-job.js';
import {stateRunJobs,validateStateRunBatchInputs} from './state-run-batch.js';
import {stateJobScope} from './state-source-job.js';

export const NATIVE_FOLDER_DEFAULTS={year:'2020',states:[],resolution:'tract',paths:null,manifest:null};
const decode=source=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));
export function validateNativeFolderInputs(value){
 if(!value||Object.keys(value).sort().join()!=='manifest,paths,resolution,states,year'||!['2000','2010','2020'].includes(value.year)||!['tract','block_group','block-group'].includes(value.resolution)||!Array.isArray(value.states)||value.states.length>100||value.states.some(s=>typeof s!=='string'||! /^[A-Za-z]{2}$/.test(s)))throw new Error('Select native folder Census year, state filters and resolution.');
 if(value.manifest!==null){validateSourceFile(value.manifest,{maxBytes:2*1024*1024});parseSafeJson(decode(value.manifest));}
 if(value.paths!==null){validateSourceFile(value.paths,{maxBytes:2*1024*1024});const paths=parseSafeJson(decode(value.paths));if(!paths||Object.keys(paths).join()!=='paths'||!Array.isArray(paths.paths)||paths.paths.length>10000||paths.paths.some(p=>!portableRelativePath(p)))throw new Error('Select bounded native folder path observations.');}
 return value;
}
export function nativeFolderSelection(files,previous=NATIVE_FOLDER_DEFAULTS){
 const {root,entries}=selectedFolderEntries(files);if(new TextEncoder().encode(root).length>150)throw new Error('Folder root name exceeds 150 UTF-8 bytes.');
 const paths={name:root+'-paths.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify({paths:observedFolderPaths(entries)})))};
 return validateNativeFolderInputs({...structuredClone(previous),paths});
}
export function nativeFolderRequest(inputs){validateNativeFolderInputs(inputs);if(!inputs.paths)throw new Error('Select the native output root folder.');return {operation:'inspect-native-input-folder',input:{paths_json:decode(inputs.paths),year:inputs.year,states:inputs.states,resolution:inputs.resolution,manifest_json:inputs.manifest?decode(inputs.manifest):null}};}
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function queueNativeFolderSources(fileList,inputs,result,graphInputs,batchInputs,{signal,onProgress=()=>{}}={}){
 inputs=structuredClone(validateNativeFolderInputs(inputs));graphInputs=structuredClone(graphInputs);batchInputs=structuredClone(validateStateRunBatchInputs(batchInputs));const selected=[...fileList],fresh=nativeFolderSelection(selected,inputs);
 const check=()=>{if(signal?.aborted)throw signal.reason||new Error('Native folder queue cancelled.');};check();
 if(fresh.paths.base64!==inputs.paths?.base64||fresh.paths.name!==inputs.paths.name||result?.schema_version!=='bisect-native-input-folder-v1'||result.execution_performed!==false||result.year!==inputs.year||result.resolution!==inputs.resolution||result.paths_sha256!==await digest(decode(inputs.paths)))throw new Error('Native folder observations changed. Inspect the selected folder again.');
 if(inputs.manifest&&result.manifest_sha256!==await digest(decode(inputs.manifest)))throw new Error('Native manifest changed. Inspect again.');
 const filters=[...new Set(inputs.states.map(s=>s.toUpperCase()))].sort();if(JSON.stringify(filters)!==JSON.stringify(result.requested_states))throw new Error('Native folder filters changed. Inspect again.');
 validatePreparedEngineInputs({source:graphInputs?.template??null,settings:graphInputs?.settings});if(graphInputs.settings.chamber!=='congressional'||!graphInputs.settings.created_at.trim())throw new Error('Record congressional export metadata before queueing native folder sources.');
 const entries=new Map(selectedFolderEntries(selected).entries.map(entry=>[entry.path,entry.file]));const plans=[];let minimumBytes=0;
 for(const row of result.states){
  if(row.status!=='binary-selected')continue;
  const stem=row.binary_path?.endsWith('.adj.bin')?row.binary_path.slice(0,-8):null;if(!stem||!['tract','block_group','block-group'].includes(row.effective_resolution)||row.year!==inputs.year)throw new Error('Invalid native folder selection row.');
  const paths={adjacency:row.binary_path,geoids:row.geoids_path,centroids:row.centroids_path,geometry:entries.has(stem+'_geometry.json')?stem+'_geometry.json':null};
  const template=entries.has(stem+'.request.json')?stem+'.request.json':null,display=entries.has(stem+'_display.geojson')?stem+'_display.geojson':null;
  for(const [key,path]of [...Object.entries(paths),['template',template],['display',display]])if(path){const file=entries.get(path),cap=(key==='adjacency'||key==='display'?64:key==='template'?25:16)*1024*1024;if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>cap)throw new Error('Missing or oversized selected native source: '+path);minimumBytes+=key==='template'?file.size:Math.ceil(file.size/3)*4;}
  if(!template){if(!graphInputs?.template)throw new Error('Select a shared full request template or supply per-state .request.json files.');minimumBytes+=new TextEncoder().encode(graphInputs.template.text).length;}
  plans.push({row,paths,template,display});
 }
 if(!plans.length)throw new Error('No browser-loadable native binary sources selected. Inspect missing or pickle-only states.');
 if(minimumBytes>64*1024*1024)throw new Error('Selected native sources exceed the 64 MiB job-source bound. Select fewer states.');
 const read=async path=>{check();const file=entries.get(path),bytes=new Uint8Array(await file.arrayBuffer());check();if(bytes.length!==file.size)throw new Error('Native source changed during reading: '+path);return {name:path,base64:encodeBytes(bytes)};};
 const jobs=[];
 for(const plan of plans){check();const sources={};for(const [key,path]of Object.entries(plan.paths))sources[key]=path?await read(path):null;let template=graphInputs.template;if(plan.template){const source=await readPreparedEngineSource(entries.get(plan.template));check();template={...source,name:plan.template};}
  const files={nativeAdjacencyInputs:sources,nativeAdjacencySettings:{state:plan.row.state,year:inputs.year,unit_kind:plan.row.effective_resolution==='tract'?'tract':'block-group'},preparedGraphInputs:{...graphInputs,template},...(plan.display?{planGeometry:await read(plan.display)}:{})};jobs.push(createPreparedNativeJob(files));onProgress({state:plan.row.state,completed:jobs.length,total:plans.length});
 }
 check();const old=batchInputs.jobs?stateRunJobs(batchInputs.jobs,batchInputs.all_years?153:51):[],keys=new Set(jobs.map(job=>job.scope.state+'/'+job.scope.year));const merged=[...old.filter(job=>{const scope=stateJobScope(job);return !keys.has(scope.state+'/'+scope.year);}),...jobs];const source={name:'native-folder-state-jobs.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(merged)))};const queued=validateStateRunBatchInputs({...batchInputs,jobs:source});check();return {inputs:queued,queued_states:jobs.map(job=>job.scope.state),skipped:result.states.filter(row=>row.status!=='binary-selected').map(row=>({state:row.state,status:row.status}))};
}

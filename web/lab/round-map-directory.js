// Read-only native state-folder observations. Handles never enter projects.
import {validateSourceFile,encodeBytes} from './package-files.js';
import {parseSafeJson} from './project.js';
import {nativeMapRequest,nativeMapDownload} from './map-export-files.js';
import {storedZipBytes} from './reproducibility-files.js';
const utf8=s=>new TextEncoder().encode(s);
class RoundOutputLimit extends Error {}
const safe=s=>typeof s==='string'&&utf8(s).length>0&&utf8(s).length<=200&&!/[\\/:\0]/.test(s)&&!['.','..','__proto__','constructor','prototype'].includes(s);
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===keys;
const checkpoint=options=>{if(options.signal?.aborted)throw options.signal.reason||new Error('Round selection cancelled.');if(options.isCurrent&&!options.isCurrent())throw new Error('Project changed while selecting rounds. Select again.');};
const decode=s=>Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0));
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
// Native names are compared lexically, never with locale/numeric collation.
const compare=(a,b)=>{const x=utf8(a),y=utf8(b);for(let i=0;i<Math.min(x.length,y.length);i++)if(x[i]!==y[i])return x[i]-y[i];return x.length-y.length;};
export function validateRoundMapDirectory(v){
 if(!exact(v,'existing_outputs,force,intermediate_present,rounds')||typeof v.force!=='boolean'||typeof v.intermediate_present!=='boolean'||!Array.isArray(v.rounds)||v.rounds.length>128||!Array.isArray(v.existing_outputs)||v.existing_outputs.length>1000||!v.intermediate_present&&v.rounds.length)throw new Error('Invalid native round-directory observation.');
 const names=new Set();let total=0;
 for(const row of v.rounds){if(!exact(row,'assignment,name,read_error')||!safe(row.name)||names.has(row.name)||row.read_error!==null&&(typeof row.read_error!=='string'||utf8(row.read_error).length>1024)||row.assignment!==null&&row.read_error!==null)throw new Error('Invalid or duplicate native round folder.');names.add(row.name);if(row.assignment!==null){validateSourceFile(row.assignment.base64===''?{...row.assignment,base64:'AA=='}:row.assignment,{maxBytes:3*1024*1024});total+=decode(row.assignment).length;}}
 if(total>16*1024*1024)throw new Error('Round assignment sources exceed 16 MiB.');
 if(new Set(v.existing_outputs).size!==v.existing_outputs.length||v.existing_outputs.some(n=>!/^round_[0-9]{2,}\.png$/.test(n)))throw new Error('Invalid existing round PNG names.');return v;
}
export async function readRoundMapDirectory(root,options={}){
 if(root?.kind!=='directory'||typeof root.entries!=='function')throw new Error('Choose a native state folder containing intermediate and maps.');
 let count=0,total=0;const list=async h=>{const rows=[];for await(const [name,child]of h.entries()){checkpoint(options);if(++count>10000)throw new Error('Round selection exceeds 10,000 entries.');if(!safe(name))throw new Error('Unsafe selected folder entry.');rows.push([name,child]);}return rows;};
 const top=await list(root),intermediate=top.find(([n])=>n==='intermediate')?.[1],maps=top.find(([n])=>n==='maps')?.[1],v={intermediate_present:!!intermediate,rounds:[],existing_outputs:[],force:false};
 if(intermediate&&intermediate.kind!=='directory')throw new Error('intermediate is not a directory.');
 if(intermediate){for(const [name,dir]of await list(intermediate)){if(dir.kind!=='directory')continue;if(v.rounds.length>=128)throw new Error('Select at most 128 round folders.');const row={name,assignment:null,read_error:null};const handle=(await list(dir)).find(([n])=>n==='assignments.json')?.[1];if(handle){try{if(handle.kind!=='file')throw new Error('assignments.json is not a readable file.');checkpoint(options);const file=await handle.getFile();checkpoint(options);if(!Number.isSafeInteger(file.size)||file.size<0||file.size>3*1024*1024)throw new RangeError('Each round assignment source must be at most 3 MiB.');total+=file.size;if(total>16*1024*1024)throw new RangeError('Round assignment sources exceed 16 MiB.');const body=new Uint8Array(await file.arrayBuffer());checkpoint(options);const current=await handle.getFile();checkpoint(options);if(current.size!==file.size||current.lastModified!==file.lastModified||body.length!==file.size)throw new RangeError('Round assignment source changed during selection.');row.assignment={name:'assignments.json',base64:encodeBytes(body)};}catch(error){checkpoint(options);if(error instanceof RangeError)throw error;row.read_error=String(error?.message||error).slice(0,200);}}v.rounds.push(row);}}
 if(maps?.kind==='directory'){const rounds=(await list(maps)).find(([n])=>n==='rounds')?.[1];if(rounds?.kind==='directory')for(const [name]of await list(rounds))if(/^round_[0-9]{2,}\.png$/.test(name))v.existing_outputs.push(name);}
 checkpoint(options);return validateRoundMapDirectory(v);
}
export async function executeRoundMapDirectory(engine,input,{onProgress}={}){
 if(!exact(input,'directory,map'))throw new Error('Invalid round map batch request.');const directory=validateRoundMapDirectory(input.directory);const settings={...input.map,round_directory:undefined};delete settings.round_directory;
 if(settings.map_type!=='rounds'||settings.round_assignments||settings.analysis)throw new Error('Round folder mode needs rounds type, no single assignments or analysis.');
 const base=nativeMapRequest(settings),ordered=[...directory.rounds].sort((a,b)=>compare(a.name,b.name));let fatal=false,outputBytes=0;
 const rows=[];
 for(const [index,source]of ordered.entries()){
  const filename='round_'+String(index).padStart(2,'0')+'.png',row={index,directory:source.name,output_path:'maps/rounds/'+filename,status:'not-attempted',error:null,failure_stage:null,assignment_sha256:null,map:null};rows.push(row);if(fatal)continue;
  if(directory.existing_outputs.includes(filename)&&!directory.force){row.status='skipped-existing';continue;}
  if(source.read_error!==null){row.status='failed-read';row.failure_stage='read';row.error=source.read_error;fatal=true;continue;}
  if(source.assignment===null){row.status='skipped-missing';continue;}
  const bytes=decode(source.assignment);row.assignment_sha256=await sha(bytes);let text;
  try{text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);const raw=parseSafeJson(text);if(!raw||typeof raw!=='object'||Array.isArray(raw)||Object.values(raw).some(v=>!Number.isInteger(v)||v<0||v>4294967295)||(text.match(/\"(?:\\.|[^\"\\])*\"|[^\s,:{}]+/g)||[]).some(token=>!token.startsWith('\"')&&!/^(0|[1-9][0-9]*)$/.test(token)))throw new Error('Assignments must map keys to unsigned wasm32 region labels.');}catch(error){row.status='failed-assignment';row.failure_stage='assignment';row.error=String(error.message);fatal=true;continue;}
  try{
   const step=engine.execute({operation:'render-native-district-map-step',input:{...base.input,round_assignments_json:text}});
   if(!exact(step,'error,failure_stage,map,schema_version,status')||step.schema_version!=='bisect-native-map-step-v1')throw new Error('Invalid native map stage response.');
   if(step.status==='failed'){
    if(!['geometry','render','rasterize','output-limit'].includes(step.failure_stage)||typeof step.error!=='string'||utf8(step.error).length>65536||step.map!==null)throw new Error('Invalid native map failure stage.');
    row.failure_stage=step.failure_stage;row.status=step.failure_stage==='geometry'?'map-error':'failed-'+step.failure_stage;row.error=step.error;fatal=step.failure_stage!=='geometry';
   }else{
    const map=step.map;if(step.status!=='rendered'||step.error!==null||step.failure_stage!==null||map?.schema_version!=='bisect-native-district-map-v1'||map.status!=='rendered'||typeof map.svg_text!=='string'||typeof map.png_base64!=='string')throw new Error('Invalid native rendered map stage.');
    const size=utf8(map.svg_text).length+atob(map.png_base64).length;if(outputBytes+size>64*1024*1024)throw new RoundOutputLimit('Round map outputs exceed 64 MiB.');outputBytes+=size;row.map=map;row.status='rendered';
   }
  }catch(error){row.status=error instanceof RoundOutputLimit?'failed-output-limit':'failed-runtime';row.failure_stage=error instanceof RoundOutputLimit?'output-limit':'runtime';row.error=String(error.message);fatal=true;}
  onProgress?.({schema_version:'bisect-round-map-directory-progress-v1',round:index+1,rounds:ordered.length});
 }
 return {schema_version:'bisect-round-map-directory-v1',status:fatal?'failed':'completed',rows,intermediate_present:directory.intermediate_present,force:directory.force,source_observation_sha256:await sha(utf8(JSON.stringify(directory))),rendered:rows.filter(r=>r.status==='rendered').length,output_bytes:outputBytes,execution_performed:false,scope:'Read-only selected native state-folder snapshot. Sorted immediate intermediate folders keep native round numbers; existing names are skipped unless forced. ZIP contains newly rendered PNGs only, never existing files. Invalid assignment/read errors stop later rounds and retain earlier outputs. Rust geometry-preparation failures are recorded as map-error and continued. Rust SVG rendering, PNG rasterization and output-limit failures stop later rounds and retain prior outputs. Runtime/invalid stage responses also stop. Geometry preparation includes explicit browser source/plan/identity bounds; it is not an assertion that every such failure exists in the native CLI. Explicit source key mapping and final RPLAN/full geometry are required. No local files are overwritten and no accepted sequence or lineage is certified.'};
}
export async function roundMapDirectoryZip(result){
 if(result?.schema_version!=='bisect-round-map-directory-v1'||!Array.isArray(result.rows)||result.rows.length>128)throw new Error('Render round folder maps first.');const entries=[];let total=0;for(const [index,row]of result.rows.entries()){if(row.index!==index||row.output_path!=='maps/rounds/round_'+String(index).padStart(2,'0')+'.png')throw new Error('Round output paths changed.');if(row.status!=='rendered')continue;const file=await nativeMapDownload(row.map,'png');total+=file.bytes.length;if(total>64*1024*1024)throw new Error('Round ZIP exceeds 64 MiB.');entries.push([row.output_path,file.bytes]);}if(!entries.length)throw new Error('No new round PNGs were rendered.');return storedZipBytes(entries,{maxSourceBytes:64*1024*1024});
}

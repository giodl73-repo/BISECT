import {nativeRegistryText} from './label-inventory-files.js';
import {validateEngineReplayInputs} from './engine-replay-files.js';
import {validatePackageVerificationInputs} from './package-verification-files.js';
import {validateManifestSource} from './manifest-files.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {parseSafeJson,parsePreparedJobsJson} from './project.js?v=label-build-archive-v1';
import {storedZipBytes} from './reproducibility-files.js';
export function labelBuildText(source){validateSourceFile(source,{maxBytes:16*1024*1024});return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));}
export const LABEL_BUILD_DEFAULTS={label:'state_build',profile:'open',years:['2020'],force:false,created_at:''};
const pathSafe=p=>typeof p==='string'&&p.length>0&&p.length<=240&&/^[A-Za-z0-9_./ -]+$/.test(p)&&!p.split('/').some(x=>!x||['.','..','__proto__','constructor','prototype'].includes(x));
export function validateLabelBuildInputs(v){
 if(!v||Object.keys(v).sort().join()!=='config,jobs,registry,settings')throw new Error('Invalid label build source fields.');
 const s=v.settings;if(!s||Object.keys(s).filter(k=>!['capture_rounds','write_manifest','retain_replay'].includes(k)).sort().join()!=='created_at,force,label,profile,years'||Object.hasOwn(s,'capture_rounds')&&typeof s.capture_rounds!=='boolean'||Object.hasOwn(s,'write_manifest')&&typeof s.write_manifest!=='boolean'||Object.hasOwn(s,'retain_replay')&&typeof s.retain_replay!=='boolean'||typeof s.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(s.label)||['__proto__','constructor','prototype','runs','analysis','reports','configs'].includes(s.label)||!['open','court'].includes(s.profile)||typeof s.force!=='boolean'||typeof s.created_at!=='string'||new TextEncoder().encode(s.created_at).length>100||!Array.isArray(s.years)||!s.years.length||s.years.length>10||new Set(s.years).size!==s.years.length||s.years.some(y=>typeof y!=='string'||!/^\d{4}$/.test(y)))throw new Error('Choose a label, profile and distinct four-digit years.');
 for(const [key,cap]of [['config',256*1024],['jobs',16*1024*1024],['registry',1024*1024]])if(v[key]!==null){if(key==='registry')nativeRegistryText(v[key]);else{validateSourceFile(v[key],{maxBytes:cap});labelBuildText(v[key]);}}
 if(v.jobs){const jobs=parsePreparedJobsJson(labelBuildText(v.jobs));if(!Array.isArray(jobs)||jobs.length>600)throw new Error('Prepared jobs JSON must be an array of at most 600 jobs.');for(const j of jobs){if(!j||Object.keys(j).some(k=>!['graph','districts','chamber','demographics','coi','electoral','vra_absent_file_fallback','compact_geometry'].includes(k))||Object.hasOwn(j,'vra_absent_file_fallback')&&(j.vra_absent_file_fallback!==true||j.demographics!=null)||Object.hasOwn(j,'compact_geometry')&&!['prepared','native-state-circular'].includes(j.compact_geometry)||!j.graph||!Number.isSafeInteger(j.districts)||j.districts<1||j.districts>500||!['congressional','house','senate'].includes(j.chamber))throw new Error('Each job requires a prepared graph, chamber and 1–500 districts.');}}
 return v;
}
export function labelBuildRequest(v){validateLabelBuildInputs(v);if(!v.config||!v.jobs||!v.settings.created_at)throw new Error('Select YAML and prepared jobs, and record a build timestamp.');const {years,...settings}=v.settings;return {operation:'run-label-build',input:{...settings,years_override:years,registry_json:v.registry?nativeRegistryText(v.registry):null,yaml_text:labelBuildText(v.config),source_name:v.config.name,jobs:parsePreparedJobsJson(labelBuildText(v.jobs))}};}
const policyJobCache=new WeakMap();
export function labelBuildJobPolicies(v){
 if(!v?.jobs)return [];let cached=policyJobCache.get(v.jobs);if(!cached||cached.base64!==v.jobs.base64){validateLabelBuildInputs(v);cached={base64:v.jobs.base64,jobs:parsePreparedJobsJson(labelBuildText(v.jobs))};policyJobCache.set(v.jobs,cached);}const jobs=cached.jobs;
 return jobs.map((j,index)=>({index,state:j.graph.state,year:j.graph.year,absent:j.vra_absent_file_fallback===true,has_demographics:j.demographics!=null}));
}
export function setLabelBuildJobAbsence(v,indices,absent){
 validateLabelBuildInputs(v);if(typeof absent!=='boolean'||!Array.isArray(indices)||!indices.length||indices.some(i=>!Number.isSafeInteger(i)))throw new Error('Select state jobs and an explicit source policy.');
 if(!v.jobs)throw new Error('Load prepared jobs first.');const jobs=parsePreparedJobsJson(labelBuildText(v.jobs)),selected=new Set(indices);if(selected.size!==indices.length||indices.some(i=>i<0||i>=jobs.length))throw new Error('Invalid state job selection.');
 const next=jobs.map((j,i)=>{if(!selected.has(i))return j;if(absent&&j.demographics!=null)throw new Error('A selected job already supplies demographics; absent-file policy cannot replace them.');const row={...j};if(absent)row.vra_absent_file_fallback=true;else delete row.vra_absent_file_fallback;return row;});
 return validateLabelBuildInputs({...v,jobs:{name:v.jobs.name,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(next)))}});
}
export async function readLabelBuildSource(f,kind){const cap={config:256*1024,jobs:16*1024*1024,registry:1024*1024}[kind];if(!cap||!f||!Number.isSafeInteger(f.size)||f.size<(kind==='registry'?0:1)||f.size>cap)throw new Error('Select a nonempty source within the displayed size limit.');const bytes=new Uint8Array(await f.arrayBuffer());if(bytes.length!==f.size)throw new Error('Build source size changed while reading.');const source={name:f.name,base64:encodeBytes(bytes)};if(kind==='registry'){const text=nativeRegistryText(source);parseSafeJson(text.trim()?text:'{}');}else{validateSourceFile(source,{maxBytes:cap});labelBuildText(source);if(kind==='jobs')parsePreparedJobsJson(labelBuildText(source));}return source;}
export function startLabelBuildCheckpoint(request){return {schema_version:'bisect-label-build-v1',status:'running',label:request.input.label,states:[],years:[],invalidated_downstream:[],files:{['configs/'+request.input.label+'.yml']:request.input.yaml_text,'.bisect':request.input.registry_json||'{}'},output_hashes:[],expected_jobs:request.input.jobs.filter(j=>request.input.force||!parseSafeJson(request.input.registry_json||'{}')[request.input.label]?.built?.includes(j.graph.year)).map(j=>({state:j.graph.state,year:j.graph.year})),checkpoint_sequence:0,checkpoint_request_sha256:null,scope:'Completed state outputs from this in-flight build. Unfinished years are not newly marked built. Registry records for unfinished existing years remain imported claims; incomplete force rebuilds require separate downstream review. Cancellation keeps received deltas; opening this saved checkpoint is inert and does not authenticate or resume execution.'};}
const checkpointFileSizes=new WeakMap();
export function retainLabelBuildCheckpoint(previous,event){
 if(!event||event.schema_version!=='bisect-label-build-progress-v1'||event.label!==previous.label||event.sequence!==previous.checkpoint_sequence+1||!/^[a-f0-9]{64}$/.test(event.request_sha256)||previous.checkpoint_request_sha256&&previous.checkpoint_request_sha256!==event.request_sha256)throw new Error('Unexpected build checkpoint identity or sequence.');
 const next={...structuredClone({...previous,files:null}),files:{...previous.files}};
 const cached=checkpointFileSizes.get(previous);const sizes=new Map(cached?.files===previous.files?cached.sizes:Object.entries(previous.files).map(([p,t])=>[p,new TextEncoder().encode(t).length]));
 if(event.kind==='state-completed'){
  const s=event.state;if(!s||!['ok','failed'].includes(s.status)||!/^\d{4}$/.test(s.year)||!/^[A-Z]{2}$/.test(s.state)||typeof s.state_name!=='string'||!/^[a-z0-9_]+$/.test(s.state_name)||next.states.some(r=>r.year===s.year&&r.state===s.state)||!event.files||Array.isArray(event.files))throw new Error('Invalid completed state checkpoint.');
  if(!next.expected_jobs.some(j=>j.state===s.state&&j.year===s.year))throw new Error('Checkpoint state was not scheduled.');const root=`runs/${next.label}/${s.year}/${s.state_name}/`;for(const [p,text]of Object.entries(event.files)){if(!pathSafe(p)||!p.startsWith(root)||typeof text!=='string')throw new Error('Invalid state checkpoint file.');next.files[p]=text;sizes.set(p,new TextEncoder().encode(text).length);}
  if(s.status==='failed'&&Object.keys(event.files).length||s.status==='ok'&&['plan.rplan','context.rctx','final_assignments.json'].some(p=>typeof event.files[root+p]!=='string'))throw new Error('Incomplete state checkpoint.');next.states.push(structuredClone(s));
 }else if(event.kind==='year-completed'){
  const y=event.year;if(!y||y.status!=='built'||!/^\d{4}$/.test(y.year)||event.index_path!==`runs/${next.label}/${y.year}/index.json`||typeof event.index_json!=='string'||typeof event.registry_json!=='string'||next.years.some(r=>r.year===y.year))throw new Error('Invalid completed year checkpoint.');
  const completed=next.states.filter(r=>r.year===y.year),expected=next.expected_jobs.filter(j=>j.year===y.year);if(!expected.length||completed.length!==expected.length||y.summary?.total!==completed.length||y.summary?.succeeded!==completed.filter(r=>r.status==='ok').length||y.summary?.failed!==completed.filter(r=>r.status==='failed').length)throw new Error('Year checkpoint precedes scheduled state completion.');// Inspect identity only; preserve native index bytes, including numeric u64 seeds.
  const index=JSON.parse(event.index_json),registry=parseSafeJson(event.registry_json);if(index.label!==next.label||index.year!==y.year||!registry[next.label]?.built?.includes(y.year))throw new Error('Checkpoint index/registry mismatch.');next.files[event.index_path]=event.index_json;next.files['.bisect']=event.registry_json;sizes.set(event.index_path,new TextEncoder().encode(event.index_json).length);sizes.set('.bisect',new TextEncoder().encode(event.registry_json).length);next.years.push(structuredClone(y));
 }else throw new Error('Unknown build checkpoint kind.');
 if([...sizes.values()].reduce((a,b)=>a+b,0)>64*1024*1024)throw new Error('Retained build files exceed 64 MiB.');next.checkpoint_sequence=event.sequence;next.checkpoint_request_sha256=event.request_sha256;Object.freeze(next.files);checkpointFileSizes.set(next,{files:next.files,sizes});return next;
}
export function labelBuildZip(result){if(result?.schema_version!=='bisect-label-build-v1'||!result.files||Object.keys(result.files).some(p=>!pathSafe(p)||typeof result.files[p]!=='string'))throw new Error('Run a label build first.');return storedZipBytes(Object.entries(result.files).map(([p,text])=>[p,new TextEncoder().encode(text)]),{maxSourceBytes:64*1024*1024});}

// Select exact generated files without parsing/re-serializing native integer fields.
export function labelBuildPlanChoices(result){
 if(result?.schema_version!=='bisect-label-build-v1'||typeof result.label!=='string'||!result.files)return [];
 const prefix='runs/'+result.label+'/';
 return Object.keys(result.files).filter(p=>pathSafe(p)&&p.startsWith(prefix)&&/^\d{4}\/[a-z_]+\/plan\.rplan$/.test(p.slice(prefix.length))&&typeof result.files[p]==='string'&&typeof result.files[p.replace(/plan\.rplan$/,'context.rctx')]==='string').sort();
}
export function labelBuildPlanSources(result,path){
 if(!labelBuildPlanChoices(result).includes(path))throw new Error('Select a retained build plan with its matching context.');
 const context=path.replace(/plan\.rplan$/,'context.rctx');
 const sources={};
 for(const [key,p] of [['auditPlanText',path],['auditContextText',context]]){
  const bytes=new TextEncoder().encode(result.files[p]);
  if(bytes.length>3*1024*1024)throw new Error('Selected plan/context exceeds the practitioner source limit of 3 MiB per file.');
  const source={name:p.split('/').slice(-3).join('-'),base64:encodeBytes(bytes)};
  validateSourceFile(source);sources[key]=source;
 }
 const manifestPath=path.replace(/plan\.rplan$/,'manifest.json');
 if(typeof result.files[manifestPath]==='string'){
  const source={name:'manifest.json',base64:encodeBytes(new TextEncoder().encode(result.files[manifestPath]))};validateSourceFile(source);
  validateManifestSource(source);sources.manifestSource=source;
  sources.packageVerificationInputs={manifest:structuredClone(source),files:{'plan.rplan':{...structuredClone(sources.auditPlanText),name:'plan.rplan'},'context.rctx':{...structuredClone(sources.auditContextText),name:'context.rctx'}}};
  validatePackageVerificationInputs(sources.packageVerificationInputs);sources.tuiVerifySettings={mode:'prechecks'};
 }
 const configuredPath=path.replace(/plan\.rplan$/,'accepted_configured_run.json');const runPath=typeof result.files[configuredPath]==='string'?configuredPath:path.replace(/plan\.rplan$/,'accepted_engine_run.json');
 if(typeof result.files[runPath]==='string'){
  const document=parseSafeJson(result.files[path]),p=document.plan,ids=p?.units?.unit_ids;
  if(!Array.isArray(ids)||!Array.isArray(p.assignment)||ids.length!==p.assignment.length||new Set(ids).size!==ids.length||p.assignment.some(d=>!Number.isInteger(d)||d<0||d>=p.k))throw new Error('Retained replay plan has an invalid unit/assignment index.');
  const original=Object.fromEntries(ids.map((id,i)=>[id,p.assignment[i]+1]));
  sources.engineReplayInputs={run:{name:runPath.split('/').at(-1),base64:encodeBytes(new TextEncoder().encode(result.files[runPath]))},original:{name:'original-final-assignments.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(original)))},minSimilarity:1};
  validateEngineReplayInputs(sources.engineReplayInputs);
 }
 return sources;
}

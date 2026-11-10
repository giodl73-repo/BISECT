import {electionAnalysisCsv} from './election-source-format.js';
import {nativeStateDataEntries} from './native-state-files.js';
import {validateSourceFile,exactSourceText,validatePackagePath} from './package-files.js';
import {parseGeometryJson} from './project.js';
import {validateElectionAnalysisSettings} from './election-analysis-settings.js';
import {storedZipBytes} from './reproducibility-files.js';
import {validateCommissionDeliverySettings} from './commission-files.js';
export function validateTuiRunWorkflowSettings(v){
 const keys=v&&Object.keys(v).filter(k=>!['map_types','delivery','include_exact_reock','population_centroids'].includes(k)).sort().join();
 if(!v||!['analysis,dpi,map,report','analysis,dissolve_method,dpi,map,report'].includes(keys)||v.dissolve_method!==undefined&&!['sequential','balanced','spatial-balanced','batch'].includes(v.dissolve_method)||['analysis','map','report'].some(k=>typeof v[k]!=='boolean')||!['72','100','150','200','300'].includes(v.dpi)||v.report&&!v.analysis)throw new Error('Choose analysis before report, and a native map DPI.');
 if(Object.hasOwn(v,'map_types')&&(!Array.isArray(v.map_types)||v.map_types.length>4||v.map&&v.map_types.length===0||new Set(v.map_types).size!==v.map_types.length||v.map_types.some(t=>!['districts','political','demographic','compactness'].includes(t))))throw new Error('Choose distinct supported workflow map types, including at least one when maps are enabled.');
 if(Object.hasOwn(v,'delivery'))validateCommissionDeliverySettings(v.delivery);
 if(Object.hasOwn(v,'include_exact_reock')&&typeof v.include_exact_reock!=='boolean')throw new Error('Workflow exact Reock option must be boolean.');
 if(v.population_centroids!==undefined&&v.population_centroids!==null){validateSourceFile(v.population_centroids,{maxBytes:3*1024*1024});exactSourceText(v.population_centroids);}return v;
}
export function stateWorkflowOptions(files,settings,createdAt){
 validateTuiRunWorkflowSettings(settings);if(!settings.analysis&&!settings.map&&!settings.report)return null;
 const source=files.planGeometry;let geometry=null,crsLabel='EPSG:4326',basis='wgs84-degrees';
 if(source&&(settings.analysis||settings.map)){validateSourceFile(source,{maxBytes:64*1024*1024});geometry=new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));const parsed=parseGeometryJson(geometry);if(['source-exact','source-exact-large','display-topology'].includes(parsed.geometry_profile)){basis='nad83-degrees';crsLabel='EPSG:4269';}}
 if(settings.map&&!geometry)throw new Error('Select matching unit geometry for workflow maps.');
 const statistics=files.statisticsInputs||{},primary=files.electionAnalysisSource,other=statistics.election;
 if(primary&&other&&primary.base64!==other.base64)throw new Error('The retained election selections differ. Choose one source.');
 const text=s=>s?exactSourceText(s):null;
 const election=files.electionAnalysisSettings||{bootstrap_seed:'42',bootstrap_replicates:1000};validateElectionAnalysisSettings(election);
 return {analysis:settings.analysis?{geography_json:text(files.splitsGeography),demographic_csv:text(statistics.demographic),urban_csv:text(statistics.urban),election_csv:electionAnalysisCsv(files,other),election_settings:election,compactness:geometry?{unit_geometry_json:geometry,coordinate_basis:basis,crs_label:crsLabel,include_exact_reock:settings.include_exact_reock||false,dissolve_method:settings.dissolve_method||'sequential',population_centroids_csv:text(settings.population_centroids)}:null}:null,map:settings.map?{unit_geometry_json:geometry,dpi:Number(settings.dpi),map_types:settings.map_types||['districts'],dissolve_method:settings.dissolve_method||'sequential'}:null,report:settings.report?{generated_at:createdAt,delivery:settings.delivery||null}:null};
}
export function acceptedStateWorkflowRequest(files,settings,createdAt){
 const options=stateWorkflowOptions(files,settings,createdAt);
 if(!options)throw new Error('Select at least one follow-up stage.');
 if(!files.auditPlanText||!files.auditContextText)throw new Error('Select the accepted RPLAN and matching RCTX exports.');
 // This operation admits native RCTX up to 8 MiB; other artifact readers
 // keep their own limits. Validate before allocating the decoded context.
 validateSourceFile(files.auditContextText,{maxBytes:8*1024*1024});
 const contextBytes=Uint8Array.from(atob(files.auditContextText.base64),c=>c.charCodeAt(0));
 const document_json=exactSourceText(files.auditPlanText),context_json=new TextDecoder('utf-8',{fatal:true}).decode(contextBytes);
 return {operation:'run-accepted-state-workflow',input:{document_json,context_json,options}};
}
export async function stateWorkflowEntries(w,accepted){
 if(w?.schema_version!=='bisect-state-workflow-v1'||!['completed','partial'].includes(w.status)||!w.files_base64||!w.file_sha256)throw new Error('Complete a fresh follow-up workflow first.');
 const entries=Object.entries(w.files_base64);if(!entries.length||entries.length>100||entries.length!==Object.keys(w.file_sha256).length)throw new Error('Invalid workflow file set.');let total=0;const files=[];
 for(const [name,base64]of entries){validatePackagePath(name);validateSourceFile({name,base64},{maxBytes:64*1024*1024});const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));total+=bytes.length;if(total>64*1024*1024)throw new Error('Workflow files exceed 64 MiB.');const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');if(hash!==w.file_sha256[name])throw new Error('Workflow file hash mismatch.');files.push([name,bytes]);}
 for(const [name,hash]of [['plan.rplan',w.accepted_plan_source_sha256],['context.rctx',w.accepted_context_source_sha256]])if(w.file_sha256[name]!==hash)throw new Error('Workflow accepted source identity mismatch.');
 if(accepted){for(const [name,text]of [['plan.rplan',accepted.document_json],['context.rctx',accepted.context_json]]){const bytes=new TextEncoder().encode(text),actual=files.find(([path])=>path===name)?.[1];if(!actual||actual.length!==bytes.length||actual.some((b,i)=>b!==bytes[i]))throw new Error('Workflow bytes differ from the accepted State exports.');}}
 return files;
}
export async function stateWorkflowZip(result){
 if(!['bisect-tui-run-v1','bisect-accepted-state-workflow-v1','bisect-prepared-state-run-v1'].includes(result?.schema_version))throw new Error('Complete a fresh follow-up workflow first.');
 const w=result.workflow,accepted=result.prepared?.export??result.practitioner_export;
 const files=await stateWorkflowEntries(w,accepted);
 if(result.configured_run&&!files.some(([name])=>name==='data/final_assignments.json'))files.push(['data/final_assignments.json',new TextEncoder().encode(JSON.stringify(result.configured_run.result.assignments,null,2)+'\n')]);
 if(result.configured_run?.result?.metrics?.native_state_vra)files.push(...nativeStateDataEntries(result.configured_run.engine_request,result.configured_run.result));
 files.push(['workflow-receipt.json',new TextEncoder().encode(JSON.stringify({...w,files_base64:undefined},null,2)+'\n')]);return storedZipBytes(files,{maxSourceBytes:65*1024*1024});
}

import {preparedEnginePlanSources} from './prepared-engine-results.js';
import {stateCommandArguments} from './state-command-files.js';
import {validateNativeReportSnapshot} from './native-report-source-inputs.js';

const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export function nativeReportBindingRequest(prepared,snapshot){
 preparedEnginePlanSources(prepared);
 validateNativeReportSnapshot(snapshot);
 if(snapshot?.schema_version!=='bisect-native-report-source-snapshot-v1'||snapshot.live_folder_authority!==false||snapshot.request?.operation!=='inspect-native-state-report-sources'||snapshot.receipt?.schema_version!=='bisect-native-state-report-sources-v1'||snapshot.receipt.observed!==true||snapshot.receipt.execution_performed!==false||snapshot.receipt.filesystem_reads_performed!==false||snapshot.receipt.path_authority_granted!==false||snapshot.receipt.source_discovery_authorized!==false||!/^[a-f0-9]{64}$/.test(snapshot.receipt.source_bundle_sha256??'')||JSON.stringify(stateCommandArguments(snapshot.source_command))!==JSON.stringify(snapshot.request.arguments))throw Error('Select a complete original native report-source snapshot; stored paths grant no authority.');
 const observations=snapshot.request.observations;let bytes=0;
 if(!Array.isArray(observations)||observations.length>5)throw Error('Select a bounded complete native report-source snapshot.');
 for(const entry of observations){if(!entry||Object.keys(entry).some(k=>!['relative_path','kind','source_base64'].includes(k))||typeof entry.relative_path!=='string'||entry.relative_path.length>256||!['missing','file','directory'].includes(entry.kind))throw Error('Invalid report-source observation.');if(entry.kind==='file'){if(typeof entry.source_base64!=='string'||entry.source_base64.length>Math.ceil(8*1024*1024/3)*4)throw Error('Native report CSV exceeds 8 MiB.');bytes+=Math.floor(entry.source_base64.length/4)*3-(entry.source_base64.endsWith('==')?2:entry.source_base64.endsWith('=')?1:0);}else if(entry.source_base64!=null)throw Error('A missing or directory source cannot carry file bytes.');}
 if(bytes>16*1024*1024)throw Error('Native report-source snapshot exceeds 16 MiB.');
 return {operation:'bind-native-state-report-sources',input:{arguments:structuredClone(snapshot.request.arguments),observations:structuredClone(snapshot.request.observations),observation_origin:snapshot.request.observation_origin,document_json:prepared.export.document_json,context_json:prepared.export.context_json}};
}

/** Produce a new result; preserve accepted generation/replay and original input. */
export async function bindNativeReportExport(engine,prepared,snapshot,{signal,isCurrent}={}){
 const current=()=>{signal?.throwIfAborted();if(isCurrent&&isCurrent()!==true)throw Error('Report binding request changed. Review the current sources and result again.');};current();
 const request=nativeReportBindingRequest(prepared,snapshot);
 prepared=structuredClone(prepared);snapshot=structuredClone(snapshot);
 const result=await engine.execute(request);current();
 if(result?.schema_version!=='bisect-native-state-report-binding-v1'||!['bound','native-load-failure'].includes(result.status)||result.artifact_units_bound!==true||result.partition_execution_performed!==false||result.historical_execution_authenticated!==false||result.original_plan_source_sha256!==await hash(request.input.document_json)||result.original_context_source_sha256!==await hash(request.input.context_json)||result.preparation?.source_bundle_sha256!==snapshot.receipt.source_bundle_sha256||JSON.stringify(result.preparation.identity)!==JSON.stringify(snapshot.receipt.identity)||result.preparation.observation_origin!==snapshot.request.observation_origin||result.preparation.requested_population_source!==snapshot.receipt.requested_population_source)throw Error('Native report-source binding differs from the selected source or original accepted export.');
 const {document_json,context_json,document,context,...receipt}=result;
 const next={...prepared,report_source_binding:{schema_version:'bisect-prepared-report-binding-v1',source_snapshot:structuredClone(snapshot),receipt,previous_export:{document_json:request.input.document_json,context_json:request.input.context_json}}};
 if(result.status==='native-load-failure'){
  next.export=null;next.export_error='Native report source failed: '+String(result.preparation.report_load_error??'No completed report context was produced.').slice(0,2000);
  current();
  return next;
 }
 if(result.graph_binding_verified!==true||result.graph_and_population_unchanged!==true||result.previous_context_hash!==prepared.export.context_hash||result.document_source_sha256!==await hash(document_json)||result.context_source_sha256!==await hash(context_json))throw Error('Native report binding changed or failed to bind accepted export identities.');
 next.export={...prepared.export,document_json,context_json,document,context,plan_hash:result.plan_hash,context_hash:result.context_hash,source_boundary:result.scope};
 next.export_error=null;
 preparedEnginePlanSources(next);
 current();
 return next;
}

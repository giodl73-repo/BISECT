import {stateCommandArguments} from './state-command-files.js';

const exact=(v,required,optional=[])=>v&&typeof v==='object'&&!Array.isArray(v)&&required.every(k=>Object.hasOwn(v,k))&&Object.keys(v).every(k=>required.includes(k)||optional.includes(k));
function boundedMetadata(root){
 const queue=[root],seen=new WeakSet();let nodes=0,bytes=0;
 while(queue.length){const v=queue.pop();if(++nodes>250000)throw Error('Report-source metadata exceeds its node limit.');
  if(typeof v==='string'){bytes+=new TextEncoder().encode(v).length;if(bytes>32*1024*1024)throw Error('Report-source metadata exceeds 32 MiB.');}
  else if(v&&typeof v==='object'){
   if(seen.has(v)||!Array.isArray(v)&&![Object.prototype,null].includes(Object.getPrototypeOf(v)))throw Error('Report-source metadata must be an acyclic JSON tree.');seen.add(v);
   for(const [key,property] of Object.entries(Object.getOwnPropertyDescriptors(v))){if(Array.isArray(v)&&key==='length')continue;if(!Object.hasOwn(property,'value')||['__proto__','prototype','constructor'].includes(key))throw Error('Invalid report-source metadata property.');queue.push(property.value);}
  }else if(v!==null&&typeof v!=='boolean'&&(typeof v!=='number'||!Number.isFinite(v)))throw Error('Report-source metadata contains a non-JSON value.');
 }
}
export function validateNativeReportSnapshot(v){
 if(!exact(v,['schema_version','source_command','request','receipt','live_folder_authority'])||v.schema_version!=='bisect-native-report-source-snapshot-v1'||v.live_folder_authority!==false)throw Error('Select a data-only native report-source snapshot.');
 const args=stateCommandArguments(v.source_command),r=v.request,p=v.receipt;
 if(!exact(r,['operation','arguments','observations','observation_origin'],['unit_ids'])||r.operation!=='inspect-native-state-report-sources'||!Array.isArray(r.arguments)||r.arguments.length>256||r.arguments.some(a=>typeof a!=='string'||a.length>4096)||JSON.stringify(r.arguments)!==JSON.stringify(args)||!['selected-folder-snapshot','explicit-file-bundle'].includes(r.observation_origin))throw Error('Report sources must retain their exact native argument tokens and observation realm.');
 if(!p||p.schema_version!=='bisect-native-state-report-sources-v1'||p.observed!==true||p.execution_performed!==false||p.filesystem_reads_performed!==false||p.path_authority_granted!==false||p.source_discovery_authorized!==false||!/^[a-f0-9]{64}$/.test(p.source_bundle_sha256??'')||p.observation_origin!==r.observation_origin||p.identity?.schema_version!=='bisect-native-state-identity-v1'||p.identity.native_year_valid!==true||p.identity.path_authority_granted!==false||typeof p.cvap_missing_warning!=='boolean'||typeof p.requested_population_source!=='string'||p.requested_population_source.length>4096)throw Error('Invalid native report-source inspection receipt.');
 boundedMetadata(p);
 if(r.unit_ids!==undefined&&(!Array.isArray(r.unit_ids)||!r.unit_ids.length||r.unit_ids.length>100000||r.unit_ids.some(id=>typeof id!=='string'||!id.length||id.length>200)||new Set(r.unit_ids).size!==r.unit_ids.length))throw Error('Invalid report-source inspection unit order.');
 if(!Array.isArray(r.observations)||!r.observations.length||r.observations.length>5)throw Error('Retain complete bounded native report-source observations.');
 let bytes=0;const paths=new Set();
 for(const o of r.observations){
  if(!exact(o,['relative_path','kind'],['source_base64'])||typeof o.relative_path!=='string'||!/^data\/\d{4}\/demographics\/[A-Za-z0-9_ -]{1,110}\.csv$/.test(o.relative_path)||paths.has(o.relative_path)||!['missing','directory','file'].includes(o.kind))throw Error('Invalid or duplicate report-source observation.');paths.add(o.relative_path);
  if(o.kind==='file'){if(typeof o.source_base64!=='string'||o.source_base64.length>Math.ceil(8*1024*1024/3)*4)throw Error('Native report CSV exceeds 8 MiB.');bytes+=Math.floor(o.source_base64.length/4)*3-(o.source_base64.endsWith('==')?2:o.source_base64.endsWith('=')?1:0);}
  else if(o.source_base64!=null)throw Error('Missing paths/directories cannot carry CSV bytes.');
 }
 if(bytes>16*1024*1024)throw Error('Native report-source snapshot exceeds 16 MiB.');
 return v;
}
export async function recheckNativeReportSnapshot(engine,snapshot,command){
 validateNativeReportSnapshot(snapshot);
 if(JSON.stringify(stateCommandArguments(command))!==JSON.stringify(snapshot.request.arguments))throw Error('Report sources belong to different original State flags. Load or apply the snapshot again.');
 const fresh=await engine.execute({operation:'inspect-native-state-report-sources',arguments:structuredClone(snapshot.request.arguments),observations:structuredClone(snapshot.request.observations),observation_origin:snapshot.request.observation_origin});
 if(fresh?.schema_version!=='bisect-native-state-report-sources-v1'||fresh.status!=='sources-observed'||fresh.source_bundle_sha256!==snapshot.receipt.source_bundle_sha256||JSON.stringify(fresh.identity)!==JSON.stringify(snapshot.receipt.identity)||fresh.requested_population_source!==snapshot.receipt.requested_population_source||fresh.cvap_missing_warning!==snapshot.receipt.cvap_missing_warning||fresh.availability_result_unused_by_partition!==snapshot.receipt.availability_result_unused_by_partition)throw Error('Native report sources, policy or inspection metadata changed. Review and apply again before Run.');
 return fresh;
}

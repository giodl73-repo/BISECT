import {createProject,parseSafeJson} from './project.js';
import {encodeBytes} from './package-files.js';
import {validateIlpAuditInputs} from './ilp-audit-files.js';
const record=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const digest=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
// Prepare the native artifact auditor's inputs only. Its Run is always explicit;
// a matching packet hash does not authenticate execution or certify a plan.
export async function ilpReportAuditProject(packet,name='Native ILP node artifact audit'){
 if(!packet||!['bisect-native-ilp-inspection-v1','bisect-native-ilp-run-reports-v1'].includes(packet.schema_version)||!record(packet.files)||!record(packet.file_sha256)||!Array.isArray(packet.nodes)||packet.nodes.length>512)throw new Error('Select a native ILP report packet.');
 const expected=new Map(),seen=new Set();let skipped=0;
 for(const n of packet.nodes){
  if(typeof n.path!=='string'||!/^[01]{0,20}$/.test(n.path)||n.depth!==n.path.length||seen.has(n.path))throw new Error('Invalid or duplicate native ILP node path.');seen.add(n.path);
  if(n.stage==='size-guard-skipped'){if(n.report_json!==null||n.lp_text!==null)throw new Error('Skipped node has unexpected solve artifacts.');skipped++;continue;}
  if(n.stage!=='native-solver-attempt'||typeof n.report_json!=='string'||typeof n.lp_text!=='string')throw new Error('Native ILP node artifacts are missing.');
  const base=`depth_${String(n.depth).padStart(2,'0')}/node_${n.path||'root'}`;
  for(const [suffix,text]of [['.json',n.report_json],['.lp',n.lp_text]]){const path=base+suffix;if(expected.has(path))throw new Error('Duplicate native ILP node artifact.');expected.set(path,text);}
 }
 if(!expected.size)throw new Error('This run has no solver artifacts to audit. Size-skipped nodes have no fabricated reports.');
 if(Object.keys(packet.files).length!==expected.size||Object.keys(packet.file_sha256).length!==expected.size)throw new Error('Native ILP artifact roster changed.');
 const inputs={reports:Object.create(null),models:Object.create(null),summary:null};let reportBytes=0,total=0;
 for(const [path,text]of expected){
  if(packet.files[path]!==text)throw new Error('Native ILP node and retained file bytes disagree.');
  if(text.length>8*1024*1024)throw new Error('An ILP audit file exceeds 8 MiB.');
  const bytes=new TextEncoder().encode(text);total+=bytes.length;if(path.endsWith('.json'))reportBytes+=bytes.length;
  if(bytes.length>8*1024*1024||reportBytes>8*1024*1024||total>16*1024*1024)throw new Error('ILP audit requires at most 8 MiB of reports and 16 MiB of combined artifacts.');
  if(await digest(bytes)!==packet.file_sha256[path])throw new Error('Native ILP artifact hash changed.');
  if(path.endsWith('.json'))parseSafeJson(text);
  inputs[path.endsWith('.json')?'reports':'models'][path]={name:path.split('/').at(-1),base64:encodeBytes(bytes)};
 }
 validateIlpAuditInputs(inputs);
 return createProject({name:`${name} (${Object.keys(inputs.reports).length} reports; ${skipped} size-skipped nodes)`,operation:'audit-ilp-artifacts',lastOperation:null,constraints:[],files:{ilpAuditInputs:inputs},result:null});
}

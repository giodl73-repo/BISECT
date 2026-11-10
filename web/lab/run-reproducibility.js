import {tuiRunSources,tuiRunAssignmentsText} from './tui-run-files.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {validateReproducibilityInputs,reproducibilityRequest} from './reproducibility-files.js';
const source=(name,text)=>({name,base64:encodeBytes(new TextEncoder().encode(text))});
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export async function runReproducibilityInputs(result,label){
 tuiRunAssignmentsText(result);
 const files=tuiRunSources(result),manifestText=result.report_manifest?.manifest_json;
 if(typeof manifestText!=='string')throw new Error('Missing accepted Run manifest.');
 const manifest=result.report_manifest;
 const [planHash,contextHash,manifestHash]=await Promise.all([result.practitioner_export.document_json,result.practitioner_export.context_json,manifestText].map(text=>hash(new TextEncoder().encode(text))));
 if(manifest.schema_version!=='bisect-plan-report-manifest-v1'||manifest.plan_source_sha256!==planHash||manifest.context_source_sha256!==contextHash||manifest.manifest_sha256!==manifestHash)throw new Error('Manifest source binding differs from the accepted Run.');
 const inputs={label,manifest:source('manifest.json',manifestText),analysis:{}};
 const w=result.workflow;
 if(w){
  if(w.schema_version!=='bisect-state-workflow-v1'||!['completed','partial'].includes(w.status)||w.accepted_plan_source_sha256!==planHash||w.accepted_context_source_sha256!==contextHash)throw new Error('Workflow source binding differs from the accepted Run.');
  for(const [path,base64]of Object.entries(w.files_base64||{})){
   if(!path.startsWith('analysis/')||!path.endsWith('.json'))continue;
   const name=path.slice(9),s={name,base64};validateSourceFile(s,{maxBytes:3*1024*1024});
   const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));if(await hash(bytes)!==w.file_sha256?.[path])throw new Error('Workflow analysis hash mismatch.');inputs.analysis[name]=s;
  }
 }
 validateReproducibilityInputs(inputs);reproducibilityRequest(files,inputs);
 return {files,inputs};
}

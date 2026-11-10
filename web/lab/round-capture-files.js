import {parseSafeJson,parseNativeMapPlanJson,parseBuildArchiveJson} from './project.js';
import {encodeBytes} from './package-files.js';
const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
const batchCache=new WeakMap();
// A view over retained native files, not a reconstructed engine request or rerun.
export function capturedBatchRoundResult(result,planPath){
 if(result?.schema_version!=='bisect-label-build-v1'||typeof planPath!=='string'||!Array.isArray(result.states))return null;
 const row=result.states.find(r=>r.status==='ok'&&r.plan_path===planPath);
 if(!row?.round_capture)return null;
 const expected=planPath.replace(/plan\.rplan$/,'round_capture.json'),text=result.files?.[expected],planText=result.files?.[planPath];
 if(!planPath.endsWith('/plan.rplan')||row.round_capture.path!==expected||typeof text!=='string'||typeof planText!=='string'||new TextEncoder().encode(text).length>64*1024*1024)throw new Error('Invalid retained batch capture source.');
 const cached=batchCache.get(result);if(cached?.path===planPath&&cached.text===text&&cached.planText===planText)return cached.view;
 const capture=parseBuildArchiveJson(text),document=parseNativeMapPlanJson(planText),plan=document.plan;
 if(capture.schema_version!=='bisect-native-round-capture-v1'||!Array.isArray(capture.events)||capture.events.length>128||row.round_capture.events!==capture.events.length||row.round_capture.status!==capture.status||row.round_capture.accepted_attempt!==row.build_report?.accepted_attempt||!Array.isArray(plan?.units?.unit_ids)||!Array.isArray(plan.assignment)||plan.assignment.length!==plan.units.unit_ids.length)throw new Error('Batch capture metadata or plan binding changed.');
 const view={schema_version:'bisect-configured-run-v1',engine_request:{graph:{geoids:plan.units.unit_ids}},result:{assignments:Object.fromEntries(plan.units.unit_ids.map((id,i)=>[id,plan.assignment[i]+1])),round_capture:capture},practitioner_export:{document_json:planText}};
 batchCache.set(result,{path:planPath,text,planText,view});return view;
}
export function capturedRoundEvents(result){const capture=result?.result?.round_capture;if(result?.schema_version!=='bisect-configured-run-v1'||capture?.schema_version!=='bisect-native-round-capture-v1'||!Array.isArray(capture.events)||capture.events.length>128)return [];return capture.events;}
export async function capturedRoundFiles(result,sequence){
 const events=capturedRoundEvents(result),capture=result?.result?.round_capture,graph=result?.engine_request?.graph;if(!capture||!graph||!Array.isArray(graph.geoids)||!Number.isInteger(sequence)||sequence<0||sequence>=events.length)throw new Error('Select a freshly captured native round event.');
 const event=events[sequence];if(event.sequence!==sequence||!Number.isInteger(event.units)||event.units<0||event.units>graph.geoids.length||typeof event.name!=='string'||!/^[A-Za-z0-9_]{1,80}$/.test(event.name))throw new Error('Invalid captured round identity.');
 for(const [text,digest]of [[capture.unit_index_json,capture.unit_index_sha256],[event.assignments_json,event.assignments_sha256]]){if(typeof text!=='string'||new TextEncoder().encode(text).length>3*1024*1024||await hash(text)!==digest)throw new Error('Captured round source bytes changed.');}
 const index=parseSafeJson(capture.unit_index_json),assignment=parseSafeJson(event.assignments_json);if(Object.keys(index).length!==graph.geoids.length||Object.keys(assignment).length!==event.units)throw new Error('Captured round coverage changed.');
 for(const [key,id]of Object.entries(index))if(!/^(0|[1-9]\d*)$/.test(key)||Number(key)>=graph.geoids.length||graph.geoids[Number(key)]!==id)throw new Error('Captured index differs from prepared graph order.');
 for(const [key,region]of Object.entries(assignment))if(!Object.hasOwn(index,key)||!Number.isInteger(region)||region<1||region>4294967295)throw new Error('Invalid captured assignment.');
 return {event,round:{name:'event-'+sequence+'-'+event.name+'-assignments.json',base64:encodeBytes(new TextEncoder().encode(event.assignments_json))},index:{name:'captured-unit-index.json',base64:encodeBytes(new TextEncoder().encode(capture.unit_index_json))}};
}
export async function capturedRoundMapInputs(result,sequence){
 const selected=await capturedRoundFiles(result,sequence);
 const text=result.practitioner_export?.document_json;if(typeof text!=='string')throw new Error('Prepare the native RPLAN before sending a captured round to the map.');
 const document=parseNativeMapPlanJson(text),graph=result.engine_request.graph,plan=document.plan;if(!plan||plan.units.unit_ids.length!==graph.geoids.length||plan.units.unit_ids.some((id,i)=>id!==graph.geoids[i]))throw new Error('Native export unit order differs from capture.');
 if(plan.assignment.some((district,i)=>district+1!==result.result.assignments[graph.geoids[i]]))throw new Error('Native export differs from returned final assignment.');
 return {plan:{name:'captured-run.rplan',base64:encodeBytes(new TextEncoder().encode(text))},geometry:null,analysis:null,round_assignments:selected.round,round_index:selected.index,round_key_basis:'native-index-map',map_type:'rounds',dpi:150,dissolve_method:'sequential'};
}

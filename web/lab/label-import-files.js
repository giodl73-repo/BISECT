import {validateSourceFile,encodeBytes,exactPlanText} from './package-files.js';
import {validateLabelInventoryInputs,nativeRegistryText} from './label-inventory-files.js';
import {parseSafeJson,inspectJsonText} from './project.js';
import {storedZipBytes} from './reproducibility-files.js';
const cap=16*1024*1024,encoder=new TextEncoder();
const byteLength=source=>source.base64.length/4*3-(source.base64.endsWith('==')?2:source.base64.endsWith('=')?1:0);
export const LABEL_IMPORT_DEFAULTS={label:'external_proposal',year:'2020',format:'auto',created:''};
export function validateLabelImportInputs(v){
 if(!v||Object.keys(v).sort().join()!=='dbf,registry,settings,shx,source'||!v.settings||Object.keys(v.settings).sort().join()!=='created,format,label,year')throw new Error('Invalid label import source/settings.');
 const s=v.settings;if(typeof s.label!=='string'||s.label.length>100||s.label&&!/^[A-Za-z0-9_-]+$/.test(s.label)||['__proto__','constructor','prototype'].includes(s.label)||typeof s.year!=='string'||!/^\d{4}$/.test(s.year)||!['auto','csv','geojson','shapefile','rplan'].includes(s.format)||typeof s.created!=='string'||encoder.encode(s.created).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(s.created))throw new Error('Choose a safe label, four-digit year, import format and recorded timestamp.');
 let total=0;for(const key of ['source','dbf','shx']){const source=v[key];if(source!==null){validateSourceFile(source,{maxBytes:cap});if(!source.name||encoder.encode(source.name).length>200||/[\\/:\u0000-\u001f\u007f-\u009f]/.test(source.name))throw new Error('Select a local filename up to 200 bytes.');total+=byteLength(source);}}
 if(total>cap)throw new Error('Assignment source parts exceed 16 MiB combined.');validateLabelInventoryInputs({label:'',registry:v.registry});return v;
}
export function labelImportRequest(v){validateLabelImportInputs(v);if(!v.source||!v.settings.label||!v.settings.created.trim())throw new Error('Select assignments, a label and an import timestamp.');return {operation:'import-label-assignments',input:{...v.settings,format:v.settings.format==='auto'?null:v.settings.format,source:v.source,dbf:v.dbf,shx:v.shx,registry_json:v.registry?nativeRegistryText(v.registry):null}};}
export async function readLabelImportSource(file,key,current){
 if(!['source','dbf','shx'].includes(key)||!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>cap)throw new Error('Select a nonempty assignment part up to 16 MiB.');
 const retained=['source','dbf','shx'].filter(k=>k!==key).reduce((size,k)=>size+(current?.[k]?byteLength(current[k]):0),0);if(retained+file.size>cap)throw new Error('Assignment source parts exceed 16 MiB combined.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Assignment file size changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};
 validateLabelImportInputs({...current,[key]:source});return source;
}
async function sha(bytes){return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');}
// Inspect the raw numeric token when JavaScript would otherwise round a native
// label. These values are only used to validate the file, never serialized.
function exactAssignmentFile(text){
 inspectJsonText(text);
 const value=JSON.parse(text,(_key,item,context)=>{
  if(typeof item!=='number')return item;
  const raw=context?.source;
  if(typeof raw==='string'&&!/^(0|[1-9][0-9]{0,19})$/.test(raw))throw new Error('Imported district labels must be unsigned decimal integers.');
  if(Number.isSafeInteger(item)&&item>=0)return item;
  if(typeof raw!=='string'||BigInt(raw)>18446744073709551615n)throw new Error('Imported district label exceeds the exact unsigned 64-bit range.');
  return BigInt(raw);
 });
 if(!value||typeof value!=='object'||Array.isArray(value))throw new Error('Imported assignments must be an object.');
 for(const [g,d] of Object.entries(value)){
  if(encoder.encode(g).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(g)||!(typeof d==='bigint'||typeof d==='number'))
   throw new Error('Imported assignments must have bounded GEOIDs and numeric district labels.');
 }
 return value;
}
export async function labelImportFiles(result,inputs){
 const request=labelImportRequest(inputs);if(result?.schema_version!=='bisect-label-import-v1'||result.status!=='imported-external-assignments'||result.execution_performed!==false||result.label!==request.input.label||result.year!==request.input.year||result.created!==request.input.created||!['csv','geojson','shapefile','rplan'].includes(result.format)||request.input.format&&result.format!==request.input.format)throw new Error('Run label import with these sources first.');
 for(const [key,source]of [['source',inputs.source],['dbf',inputs.dbf],['shx',inputs.shx]]){const identity=result[key];if(source){const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(identity?.name!==source.name||identity.bytes!==bytes.length||identity.sha256!==await sha(bytes))throw new Error('Imported source identity differs from selected bytes.');}else if(identity!==null)throw new Error('Unexpected imported sidecar identity.');}
 const root=`runs/${result.label}/${result.year}/`,indexPath=root+'index.json';if(result.index_path!==indexPath||result.files?.[indexPath]!==result.index_json||result.files?.['.bisect']!==result.registry_json)throw new Error('Imported index/registry files differ from native result.');
 const index=parseSafeJson(result.index_json),registry=parseSafeJson(result.registry_json);if(index.label!==result.label||index.year!==result.year||index.created!==result.created||index.algorithm?.structure!=='external'||index.algorithm.source!==result.source.sha256||index.algorithm.format!==result.format||index.algorithm.source_file!==inputs.source.name||!Array.isArray(registry[result.label]?.built)||!registry[result.label].built.includes(result.year)||!index.states||typeof index.states!=='object'||Array.isArray(index.states))throw new Error('Imported native index or built registry stage is inconsistent.');
 const states=Object.keys(index.states);if(!states.length||states.length>51||states.some(state=>!/^[a-z_]+$/.test(state)))throw new Error('Invalid imported state paths.');
 const expected=new Set(['.bisect',indexPath,...states.map(state=>root+state+'/assignments.json')]);if(Object.keys(result.files||{}).length!==expected.size||Object.keys(result.file_sha256||{}).length!==expected.size)throw new Error('Imported file set differs from native states.');
 let total=0,units=0;const files=[];for(const [path,text]of Object.entries(result.files)){if(!expected.has(path)||typeof text!=='string')throw new Error('Unexpected import file path or content.');const bytes=encoder.encode(text);total+=bytes.length;if(total>cap||await sha(bytes)!==result.file_sha256[path])throw new Error('Imported file size or digest mismatch.');if(path.endsWith('/assignments.json')){const assignments=exactAssignmentFile(text),state=path.slice(root.length,-'/assignments.json'.length),count=Object.keys(assignments).length;if(!count||index.states[state]?.tracts!==count||index.states[state]?.status!=='ok')throw new Error('Imported assignments differ from native state counts.');units+=count;}files.push([path,bytes]);}
 if(index.summary?.states!==states.length||index.summary?.total!==units)throw new Error('Imported summary differs from output assignment counts.');return files;
}
export async function labelImportZip(result,inputs){return storedZipBytes(await labelImportFiles(result,inputs));}
function labelImportContextText(v){
 if(!v||typeof v!=='object'||Array.isArray(v)||Object.keys(v).some(k=>!['state_name','index_sha256','district_basis','context'].includes(k))||!/^[a-z_]+$/.test(v.state_name)||!/^[a-f0-9]{64}$/.test(v.index_sha256)||!['one-based','zero-based'].includes(v.district_basis))throw new Error('Invalid imported-plan context selection.');
 // RCTX uses the same exact UTF-8 and 8 MiB bound as portable plan audits.
 return v.context===null?null:exactPlanText(v.context);
}
export function validateLabelImportContext(v){labelImportContextText(v);return v;}
export function labelImportPlanRequest(inputs,selection){
 const context_json=labelImportContextText(selection);if(context_json===null)throw new Error('Select the exact matching RCTX first.');
 return {operation:'prepare-label-import-plan',input:{input:labelImportRequest(inputs).input,state_name:selection.state_name,expected_index_sha256:selection.index_sha256,district_basis:selection.district_basis,context_json}};
}
export async function labelImportPlanZip(result,inputs,selection){
 const request=labelImportPlanRequest(inputs,selection);
 if(result?.schema_version!=='bisect-label-import-plan-v1'||result.execution_performed!==false||result.state_name!==selection.state_name||result.district_basis!==selection.district_basis||result.context_json!==request.input.context_json||result.bindings?.index_sha256!==selection.index_sha256||result.bindings?.source_sha256!==await sha(Uint8Array.from(atob(inputs.source.base64),c=>c.charCodeAt(0))))throw new Error('Run a fresh imported-plan preparation with these sources.');
 for(const key of ['dbf','shx'])if(inputs[key]&&result.bindings?.[key+'_sha256']!==await sha(Uint8Array.from(atob(inputs[key].base64),c=>c.charCodeAt(0))))throw new Error('Prepared sidecar identity differs from selected bytes.');
 const names=['imported.rplan','context.rctx','imported-assignments.json','import-index.json','import-record.json'];if(Object.keys(result.files||{}).length!==names.length||Object.keys(result.file_sha256||{}).length!==names.length||result.files['imported.rplan']!==result.document_json||result.files['context.rctx']!==result.context_json)throw new Error('Prepared native files are inconsistent.');
 const index=parseSafeJson(result.files['import-index.json']);if(index.label!==request.input.input.label||index.year!==request.input.input.year||index.created!==request.input.input.created||index.algorithm?.source!==result.bindings.source_sha256||await sha(encoder.encode(result.files['import-index.json']))!==selection.index_sha256||await sha(encoder.encode(result.context_json))!==result.bindings.context_sha256)throw new Error('Prepared import/context bindings are inconsistent.');
 const entries=[];for(const name of names){const text=result.files[name];if(typeof text!=='string')throw new Error('Missing prepared native file.');const bytes=encoder.encode(text);if(await sha(bytes)!==result.file_sha256[name])throw new Error('Prepared file digest mismatch.');entries.push([name,bytes]);}return storedZipBytes(entries);
}

import {validateSourceFile,encodeBytes,exactSourceText,exactContextText} from './package-files.js';
export const TEXT_COMPARISON_DEFAULTS={a:null,b:null,context:null};
export function validateTextComparisonInputs(v){
 if(!v||Object.keys(v).sort().join()!=='a,b,context')throw new Error('Invalid exact-text comparison inputs.');
 for(const key of ['a','b','context'])if(v[key]!==null){validateSourceFile(v[key],{maxBytes:(key==='context'?8:3)*1024*1024});(key==='context'?exactContextText:exactSourceText)(v[key]);}return v;
}
export function textComparisonRequest(v){validateTextComparisonInputs(v);if(!v.a||!v.b)throw new Error('Select both exact RPLAN sources.');return {operation:'compare-rplan-text',plan_a_json:exactSourceText(v.a),plan_b_json:exactSourceText(v.b),context_json:v.context?exactContextText(v.context):null};}
export async function readTextComparisonSource(file,key){if(!file)return null;const max=(key==='context'?8:3)*1024*1024;if(!['a','b','context'].includes(key)||!Number.isSafeInteger(file.size)||file.size<1||file.size>max)throw new Error('Comparison source exceeds its plan/context byte limit.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Comparison source changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};validateTextComparisonInputs({...TEXT_COMPARISON_DEFAULTS,[key]:source});return source;}

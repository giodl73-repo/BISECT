import {parseSafeJson} from './project.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
export const FETCH_PLAN_DEFAULTS={manifest_json:null,states:['RI'],year:'2020',data_types:[],selected_paths:[]};
export function validateFetchPlanInputs(v){if(!v||Object.keys(v).join()!=='source')throw new Error('Invalid Fetch selection source.');if(v.source!==null)fetchPlanInput(v.source);return v;}
function fetchPlanInput(source){validateSourceFile(source,{maxBytes:4*1024*1024});const value=parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0))));if(!value||Object.keys(value).sort().join()!=='data_types,manifest_json,selected_paths,states,year')throw new Error('Select a Fetch selection JSON with manifest_json, states, year, data_types and selected_paths.');return value;}
export function fetchPlanRequest(v){validateFetchPlanInputs(v);return {operation:'plan-fetch-sources',input:v.source?fetchPlanInput(v.source):structuredClone(FETCH_PLAN_DEFAULTS)};}
export async function readFetchPlanSource(file){if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>4*1024*1024)throw new Error('Select Fetch selection JSON up to 4 MiB.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Fetch source changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};fetchPlanInput(source);return source;}
export function fetchPlanTemplateBytes(){return new TextEncoder().encode(JSON.stringify(FETCH_PLAN_DEFAULTS,null,2)+'\n');}

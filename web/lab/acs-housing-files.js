import {inspectJsonText} from './project.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {validateCharacterInput} from './character-input.js';
export const ACS_HOUSING_DEFAULTS={state:'RI',year:'2020',data_year:'2020',source_label:'ACS housing API response'};
export function validateAcsHousingInputs(v){
 if(!v||Object.keys(v).sort().join()!=='settings,source'||!v.settings||Object.keys(v.settings).sort().join()!==Object.keys(ACS_HOUSING_DEFAULTS).sort().join())throw new Error('Invalid ACS housing preparation sources.');
 const s=v.settings;if(!/^[A-Z]{2}$/.test(s.state)||!['2000','2010','2020'].includes(s.year)||!/^\d{4}$/.test(s.data_year)||typeof s.source_label!=='string'||!s.source_label.trim()||new TextEncoder().encode(s.source_label).length>200)throw new Error('Select state, geometry vintage, observation year and source label.');
 if(v.source!==null)acsHousingSourceText(v.source);return v;
}
function acsHousingSourceText(source){validateSourceFile(source,{maxBytes:16*1024*1024});if(new TextEncoder().encode(source.name).length>200)throw new Error('ACS source name exceeds 200 bytes.');const text=new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));inspectJsonText(text);if(!Array.isArray(JSON.parse(text)))throw new Error('Select an ACS API JSON array response.');return text;}
export function acsHousingRequest(v){validateAcsHousingInputs(v);if(!v.source)throw new Error('Select a saved ACS housing API response.');return {operation:'prepare-acs-housing-source',input:{...v.settings,source_name:v.source.name,body_json:acsHousingSourceText(v.source)}};}
export async function readAcsHousingSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>16*1024*1024)throw new Error('Select nonempty ACS response JSON up to 16 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('ACS source changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};acsHousingSourceText(source);return source;
}
export function acsHousingDownload(result,kind){
 if(result?.schema_version!=='bisect-acs-housing-preparation-v1'||result.status!=='prepared'||result.execution_performed!==false)throw new Error('Prepare a fresh ACS housing response first.');
 if(kind==='csv'){if(typeof result.csv_text!=='string'||result.csv_text.length>32*1024*1024||!result.csv_text.startsWith('geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n'))throw new Error('Invalid native housing CSV.');return {name:'acs-housing.csv',type:'text/csv',bytes:new TextEncoder().encode(result.csv_text)};}
 if(kind!=='character'||result.character_status!=='ready')throw new Error('No validated housing character input is available.');validateCharacterInput(result.character_input,null,'housing');return {name:'housing-character-input.json',type:'application/json',bytes:new TextEncoder().encode(JSON.stringify(result.character_input,null,2)+'\n')};
}

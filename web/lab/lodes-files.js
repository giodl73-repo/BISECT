import {encodeBytes,validateSourceFile} from './package-files.js';
import {validateCharacterInput} from './character-input.js';
export const LODES_DEFAULTS={kind:'wac',state:'RI',year:'2020',data_year:'2020',source_label:'LODES block CSV'};
export function validateLodesInputs(v){
 if(!v||Object.keys(v).sort().join()!=='settings,source'||!v.settings||Object.keys(v.settings).sort().join()!==Object.keys(LODES_DEFAULTS).sort().join())throw new Error('Invalid LODES preparation sources.');
 const s=v.settings;if(!['wac','od'].includes(s.kind)||!/^[A-Z]{2}$/.test(s.state)||!['2000','2010','2020'].includes(s.year)||!/^\d{4}$/.test(s.data_year)||typeof s.source_label!=='string'||!s.source_label.trim()||new TextEncoder().encode(s.source_label).length>200)throw new Error('Select state, geometry vintage, observation year and source label.');
 if(v.source!==null)lodesSourceText(v.source);return v;
}
function lodesSourceText(source){validateSourceFile(source,{maxBytes:16*1024*1024});if(new TextEncoder().encode(source.name).length>200)throw new Error('LODES source name exceeds 200 bytes.');const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(bytes[0]===31&&bytes[1]===139)return null;return new TextDecoder('utf-8',{fatal:true}).decode(bytes);}
export function lodesRequest(v){validateLodesInputs(v);if(!v.source)throw new Error('Select a saved LODES block CSV or gzip.');const text=lodesSourceText(v.source);return {operation:'prepare-lodes-source',input:{...v.settings,source_name:v.source.name,...(text===null?{source_base64:v.source.base64}:{csv_text:text})}};}
export async function readLodesSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>16*1024*1024)throw new Error('Select nonempty LODES CSV or gzip up to 16 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('LODES source changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};lodesSourceText(source);return source;
}
export function lodesDownload(result,kind){
 if(result?.schema_version!=='bisect-lodes-preparation-v1'||result.status!=='prepared'||result.execution_performed!==false)throw new Error('Prepare a fresh LODES response first.');
 if(kind==='csv'){if(typeof result.csv_text!=='string'||result.csv_text.length>32*1024*1024||!result.csv_text.startsWith(result.kind==='wac'?'geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08\n':'home_geoid,work_geoid,s000\n'))throw new Error('Invalid native LODES CSV.');return {name:'lodes-'+result.kind+'-tract.csv',type:'text/csv',bytes:new TextEncoder().encode(result.csv_text)};}
 if(kind!=='character'||result.character_status!=='ready')throw new Error('No validated economic character input is available.');validateCharacterInput(result.character_input,null,'economic');return {name:'economic-character-input.json',type:'application/json',bytes:new TextEncoder().encode(JSON.stringify(result.character_input,null,2)+'\n')};
}

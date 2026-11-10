import {encodeBytes,validateSourceFile} from './package-files.js';
export function validateTigerInputs(v){
 if(!v||Object.keys(v).sort().join()!=='dbf,kind,shp,shx,year'||!['tracts','block-groups','blocks','block-centroids','counties'].includes(v.kind)||![2000,2010,2020].includes(v.year))throw new Error('Select TIGER unit type and Census vintage.');
 for(const kind of ['shp','dbf','shx'])if(v[kind]!==null){validateSourceFile(v[kind],{maxBytes:(v.kind==='counties'?96:64)*1024*1024});if(!v[kind].name||new TextEncoder().encode(v[kind].name).length>200)throw new Error('Invalid TIGER part name.');}return v;
}
export const TIGER_DEFAULTS={kind:'tracts',year:2020,shp:null,dbf:null,shx:null};
export function tigerRequest(v){validateTigerInputs(v);if(!v.shp||!v.dbf)throw new Error('Select SHP and matching DBF parts. SHX is optional.');return {operation:'prepare-tiger-source',input:v};}
export async function readTigerPart(file,{kind='tracts'}={}){
 const limit=(kind==='counties'?96:64)*1024*1024;
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>limit)throw new Error('Select a nonempty TIGER part up to '+limit/1024/1024+' MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('TIGER part changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};validateTigerInputs({...TIGER_DEFAULTS,kind,shp:source});return source;
}
export function tigerDownload(result){
 if(result?.schema_version!=='bisect-tiger-preparation-v1'||result.status!=='prepared'||result.execution_performed!==false||typeof result.records_json!=='string'||result.records_json.length>128*1024*1024)throw new Error('Prepare fresh TIGER records first.');
 return {name:'tiger-'+result.kind+'-'+result.year+'.json',type:'application/json',bytes:new TextEncoder().encode(result.records_json)};
}

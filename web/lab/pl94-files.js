import {encodeBytes,validateSourceFile} from './package-files.js';
export function validatePl94Inputs(v){
 if(!v||Object.keys(v).sort().join()!=='geography,population,year'||![2000,2010,2020].includes(v.year))throw new Error('Select a PL94 Census year.');
 for(const kind of ['geography','population'])if(v[kind]!==null){validateSourceFile(v[kind],{maxBytes:64*1024*1024});if(!v[kind].name||new TextEncoder().encode(v[kind].name).length>200)throw new Error('Invalid PL94 source name.');}
 return v;
}
export function pl94Request(v){validatePl94Inputs(v);if(!v.geography||v.year!==2000&&!v.population)throw new Error('Select geography and, for 2010/2020, population segment 1.');return {operation:'prepare-pl94-source',input:v};}
export async function readPl94Source(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>64*1024*1024)throw new Error('Select a nonempty PL94 source up to 64 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('PL94 source changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};validatePl94Inputs({year:2020,geography:source,population:null});return source;
}
export function pl94Download(result){
 if(result?.schema_version!=='bisect-pl94-preparation-v1'||result.status!=='prepared'||result.execution_performed!==false||typeof result.csv_text!=='string'||result.csv_text.length>64*1024*1024||!result.csv_text.startsWith('geoid,population\n'))throw new Error('Prepare a fresh PL94 source first.');
 return {name:'pl94-block-population-'+result.year+'.csv',type:'text/csv',bytes:new TextEncoder().encode(result.csv_text)};
}

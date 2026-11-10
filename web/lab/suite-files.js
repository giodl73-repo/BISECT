import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
const LIMIT=3*1024*1024;
export function validateSuiteInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==4||!['house','senate','suite_name','required_ratio'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid suite inputs.');
 if(typeof value.suite_name!=='string'||!value.suite_name.trim()||new TextEncoder().encode(value.suite_name).length>200)throw new Error('Supply a suite name of at most 200 UTF-8 bytes.');
 if(value.required_ratio!==null&&(!Number.isInteger(value.required_ratio)||value.required_ratio<1||value.required_ratio>500))throw new Error('Required ratio must be an integer from 1 to 500, or blank to infer.');
 let bytes=0;for(const key of ['house','senate'])if(value[key]!==null){const source=value[key];validateSourceFile(source);if(source.base64.length>4*1024*1024)throw new Error('Suite sources exceed 3 MiB.');exactSourceText(source);bytes+=atob(source.base64).length;}
 if(bytes>LIMIT)throw new Error('House and Senate sources must total at most 3 MiB.');return value;
}
export function suiteNestingRequest(inputs){
 validateSuiteInputs(inputs);if(!inputs.house||!inputs.senate)throw new Error('Select both House and Senate RPLAN files.');
 return {operation:'validate-suite-nesting',house_json:exactSourceText(inputs.house),senate_json:exactSourceText(inputs.senate),suite_name:inputs.suite_name,required_ratio:inputs.required_ratio};
}
export async function readSuiteSource(file){
 if(!file)return null;
 if(!Number.isSafeInteger(file.size)||file.size<1||file.size>LIMIT)throw new Error('Select a nonempty source of at most 3 MiB.');
 const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('Suite source size changed while reading.');exactSourceText(source);return source;
}

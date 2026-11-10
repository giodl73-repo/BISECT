import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
const valid=n=>typeof n==='string'&&n.length<=120&&/^[A-Za-z0-9_-][A-Za-z0-9_.-]*\.json$/.test(n);
export function validateNationalInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==1||!Object.hasOwn(value,'reports')||!value.reports||typeof value.reports!=='object'||Array.isArray(value.reports)||Object.keys(value.reports).length>60)throw new Error('Supply at most 60 state report files.');
 let bytes=0;for(const [name,source]of Object.entries(value.reports)){if(!valid(name))throw new Error('Reports need unique plain JSON basenames.');validateSourceFile(source);exactSourceText(source);bytes+=name.length+atob(source.base64).length;}
 if(bytes>8*1024*1024)throw new Error('State reports exceed 8 MiB combined.');return value;
}
export function nationalRequest(value){validateNationalInputs(value);if(!Object.keys(value.reports).length)throw new Error('Select at least one state report.');return {operation:'aggregate-national-analysis',reports_json:Object.fromEntries(Object.entries(value.reports).map(([name,source])=>[name,exactSourceText(source)]))};}
export async function readNationalReports(selected){
 const list=Array.from(selected);if(list.length>60)throw new Error('Select at most 60 state reports.');let bytes=0;const names=new Set();
 for(const file of list){if(!valid(file.name)||names.has(file.name)||!Number.isSafeInteger(file.size)||file.size<1||file.size>3*1024*1024)throw new Error('Select uniquely named nonempty JSON reports of at most 3 MiB each.');names.add(file.name);bytes+=file.name.length+file.size;}
 if(bytes>8*1024*1024)throw new Error('State reports exceed 8 MiB combined.');const reports=Object.create(null);
 for(const file of list){const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('Report size changed while reading.');exactSourceText(source);reports[file.name]=source;}
 return validateNationalInputs({reports});
}

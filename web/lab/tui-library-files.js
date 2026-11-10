import {validateSourceFile,exactSourceText,encodeBytes} from './package-files.js';
import {TUI_COMPARE_DEFAULTS,validateTuiCompareInputs} from './tui-compare-files.js';
import {validatePackageVerificationInputs} from './package-verification-files.js';
export const TUI_LIBRARY_DEFAULTS={filter:'',sort_column:'label',sort_direction:'asc',selected_directory:''};
// Exact App::cycle_sort sequence: deviation descending is not in the native cycle.
export function nextTuiLibrarySort(column,direction){
 const cycle=[['label','asc'],['label','desc'],['splits','asc'],['splits','desc'],['pp','asc'],['pp','desc'],['deviation','asc']];
 const index=cycle.findIndex(([c,d])=>c===column&&d===direction),next=cycle[(index+1)%cycle.length];
 return {sort_column:next[0],sort_direction:next[1]};
}
const safePart=s=>typeof s==='string'&&s.length>0&&new TextEncoder().encode(s).length<=200&&!/[\\/:\u0000-\u001f\u007f-\u009f]/.test(s)&&!['.','..','__proto__','constructor','prototype'].includes(s);
const sourceText=source=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));
const analyses=['compactness.json','summary.json','splits.json','contiguity.json'];
export function validateTuiLibraryInputs(value){
 if(!value||Object.keys(value).sort().join()!=='plans,settings'||!Array.isArray(value.plans)||value.plans.length>100)throw new Error('Select at most 100 native plan folders.');
 const s=value.settings;if(!s||Object.keys(s).sort().join()!=='filter,selected_directory,sort_column,sort_direction'||typeof s.filter!=='string'||new TextEncoder().encode(s.filter).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(s.filter)||!['label','splits','pp','deviation'].includes(s.sort_column)||!['asc','desc'].includes(s.sort_direction)||typeof s.selected_directory!=='string'||new TextEncoder().encode(s.selected_directory).length>240||s.selected_directory!==''&&!s.selected_directory.split('/').every(safePart))throw new Error('Invalid plan library preferences.');
 const paths=new Set();let bytes=0;
 for(const p of value.plans){
  if(!p||Object.keys(p).filter(k=>k!=='assignments').sort().join()!=='analysis,directory,folder_name,manifest,present'||typeof p.directory!=='string'||new TextEncoder().encode(p.directory).length>240||!p.directory.split('/').every(safePart)||paths.has(p.directory)||!safePart(p.folder_name)||!p.analysis||Array.isArray(p.analysis)||Object.keys(p.analysis).some(k=>!analyses.includes(k))||!Array.isArray(p.present)||new Set(p.present).size!==p.present.length||p.present.some(k=>!['audit.json','maps/districts.png','maps/compactness.png','maps/political.png',p.folder_name+'.rplan'].includes(k)))throw new Error('Invalid native plan library source inventory.');paths.add(p.directory);
  for(const source of [p.manifest,...Object.values(p.analysis)]){if(source===null)continue;validateSourceFile(source,{maxBytes:1024*1024});exactSourceText(source);bytes+=atob(source.base64).length;}
  if(p.assignments!==undefined){validateTuiCompareInputs({a:{label:p.folder_name,sources:{'data/final_assignments.json':p.assignments}},b:TUI_COMPARE_DEFAULTS.b});bytes+=atob(p.assignments.base64).length;}
 }
 if(bytes>16*1024*1024)throw new Error('Library JSON sources exceed 16 MiB.');return value;
}
export function tuiLibraryRequest(value){validateTuiLibraryInputs(value);return {operation:'inspect-tui-plan-library',input:{filter:value.settings.filter,sort_column:value.settings.sort_column,sort_direction:value.settings.sort_direction,plans:value.plans.map(p=>({directory:p.directory,folder_name:p.folder_name,manifest_json:p.manifest?sourceText(p.manifest):null,analysis_json:Object.fromEntries(Object.entries(p.analysis).map(([name,s])=>[name,sourceText(s)]))}))}};}
export function tuiLibraryDoctorInputs(value,directory){validateTuiLibraryInputs(value);const p=value.plans.find(p=>p.directory===directory);if(!p)throw new Error('Select a retained plan folder.');return {plans:[{label:p.folder_name,manifest:p.manifest,analysis:p.analysis,present:p.present}]};}
export function tuiLibraryCompareInputs(value,directory,label,b=TUI_COMPARE_DEFAULTS.b){
 validateTuiLibraryInputs(value);const p=value.plans.find(p=>p.directory===directory);if(!p)throw new Error('Select a retained plan folder.');
 return structuredClone(validateTuiCompareInputs({a:{label,sources:{...Object.fromEntries(Object.entries(p.analysis).map(([name,source])=>['analysis/'+name,source])),...(p.assignments?{'data/final_assignments.json':p.assignments}:{})}},b}));
}
export function tuiLibraryVerificationInputs(value,directory){
 validateTuiLibraryInputs(value);const p=value.plans.find(p=>p.directory===directory);if(!p?.manifest)throw new Error('Select a retained plan manifest.');
 const packageVerificationInputs=validatePackageVerificationInputs({manifest:p.manifest,files:{...Object.fromEntries(Object.entries(p.analysis).map(([name,source])=>['analysis/'+name,source])),...(p.assignments?{'data/final_assignments.json':p.assignments}:{})}});
 return structuredClone({packageVerificationInputs,tuiVerifySettings:{mode:'prechecks'}});
}
export async function readTuiLibraryFolder(selected,settings=TUI_LIBRARY_DEFAULTS){
 const groups=new Map(),seen=new Set();let total=0;
 for(const file of selected){
  const parts=(file.webkitRelativePath||'').split('/');if(parts.length<2||!parts.every(safePart))throw new Error('Select a folder with safe relative paths.');
  const name=parts.pop();let kind,key,folderParts=parts;
  if(name==='manifest.json'){kind='manifest';key=name;}
  else if(parts.at(-1)==='data'&&name==='final_assignments.json'){folderParts=parts.slice(0,-1);kind='assignments';key='data/'+name;}
  else if(parts.at(-1)==='analysis'&&analyses.includes(name)){folderParts=parts.slice(0,-1);kind='analysis';key=name;}
  else if(parts.at(-1)==='maps'&&['districts.png','compactness.png','political.png'].includes(name)){folderParts=parts.slice(0,-1);kind='present';key='maps/'+name;}
  else if(name==='audit.json'||name===parts.at(-1)+'.rplan'){kind='present';key=name;}
  else continue;
  if(!folderParts.length)throw new Error('Invalid plan artifact location.');const directory=folderParts.join('/'),folder_name=folderParts.at(-1),identity=directory+'/'+kind+'/'+key;if(seen.has(identity))throw new Error('Duplicate selected library artifact.');seen.add(identity);
  if(!groups.has(directory)){if(groups.size>=100)throw new Error('Select at most 100 plan folders.');groups.set(directory,{directory,folder_name,manifest:null,analysis:{},present:[],pending:[]});}
  const group=groups.get(directory);if(kind==='present'){group.present.push(key);continue;}
  const limit=(kind==='assignments'?8:1)*1024*1024;if(!Number.isSafeInteger(file.size)||file.size<(kind==='assignments'?0:1)||file.size>limit)throw new Error('Library JSON supports 1 MiB per summary and 8 MiB per assignment source.');total+=file.size;if(total>16*1024*1024)throw new Error('Library JSON sources exceed 16 MiB.');group.pending.push({kind,key,file});
 }
 if(!groups.size)throw new Error('No native plan manifests or analysis artifacts found.');
 for(const group of groups.values()){for(const {kind,key,file}of group.pending){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Library source size changed while reading.');new TextDecoder('utf-8',{fatal:true}).decode(bytes);const source={name:file.name,base64:encodeBytes(bytes)};if(kind==='manifest')group.manifest=source;else if(kind==='assignments')group.assignments=source;else group.analysis[key]=source;}delete group.pending;}
 return validateTuiLibraryInputs({plans:[...groups.values()],settings:{...settings,selected_directory:''}});
}

import {encodeBytes,isBase64Encoding} from './package-files.js';
import {validateDoctorCommand} from './doctor-command.js';
const analyses=['summary.json','compactness.json','contiguity.json','splits.json','demographic.json','partisan.json','vra_analysis.json','comparison.json','nesting.json'];
const utf8=s=>new TextEncoder().encode(s).length;
const safe=(s,max=200)=>typeof s==='string'&&utf8(s)>0&&utf8(s)<=max&&!/[\\/:\0]/.test(s)&&!['.','..','__proto__','constructor','prototype'].includes(s);
export function validatePlanDoctorInputs(input){
 if(!input||typeof input!=='object'||Array.isArray(input)||!Object.hasOwn(input,'plans')||Object.keys(input).some(k=>!['plans','command'].includes(k))||!Array.isArray(input.plans)||input.plans.length>100)throw new Error('Select up to 100 plan folders.');
 if(input.command!=null)validateDoctorCommand(input.command);
 let bytes=0;const labels=new Set();
 for(const p of input.plans){
  if(!p||Object.keys(p).sort().join()!=='analysis,label,manifest,present'||!safe(p.label)||labels.has(p.label)||!p.analysis||typeof p.analysis!=='object'||Array.isArray(p.analysis)||!Array.isArray(p.present)||p.present.length>5||new Set(p.present).size!==p.present.length)throw new Error('Invalid plan inventory.');labels.add(p.label);
  if(Object.keys(p.analysis).some(k=>!analyses.includes(k))||p.present.some(k=>!['audit.json','maps/districts.png','maps/compactness.png','maps/political.png',p.label+'.rplan'].includes(k)))throw new Error('Unknown Doctor artifact.');
  for(const source of [p.manifest,...Object.values(p.analysis)]){if(source===null)continue;validateDoctorSource(source);bytes+=atob(source.base64).length;}
 }
 if(bytes>16*1024*1024)throw new Error('Selected JSON sources exceed 16 MiB.');return input;
}
function validateDoctorSource(source){
 if(!source||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||utf8(source.name)>200||typeof source.base64!=='string'||source.base64.length>1398104||!isBase64Encoding(source.base64))throw new Error('Invalid Doctor source bytes.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(bytes.length>1024*1024||encodeBytes(bytes)!==source.base64)throw new Error('Doctor sources must be canonical Base64, at most 1 MiB.');
}
function sourceText(source){return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));}
export function planDoctorRequest(input){
 validatePlanDoctorInputs(input);if(input.command?.mode==='label'&&!input.command.label)throw Error('Choose a plan label before running the Doctor report.');
 const plans=input.command?.mode==='label'?input.plans.filter(p=>p.label===input.command.label):input.plans;
 return {operation:'diagnose-plan-artifacts',plans:plans.map(p=>{
  const v={label:p.label,manifest_json:null,analysis_json:{},present:p.present};
  if(p.manifest){try{v.manifest_json=sourceText(p.manifest);}catch{v.manifest_base64=p.manifest.base64;}}
  for(const [name,source]of Object.entries(p.analysis)){if(source===null)v.analysis_json[name]=null;else try{v.analysis_json[name]=sourceText(source);}catch{(v.analysis_base64??={})[name]=source.base64;}}
  return v;
 })};
}
function checkpoint(options){if(options.signal?.aborted)throw options.signal.reason||new Error('Doctor selection cancelled.');if(options.isCurrent&&!options.isCurrent())throw new Error('Project changed while reading plan folders. Select again.');}
export async function readPlanDoctorFolder(selected,options={}){
 let observed=0;const groups=new Map((options.directoryLabels||[]).map(label=>['plans/'+label,{label,manifest:null,analysis:Object.create(null),present:[],sources:[]}]));const paths=new Set();let bytes=0;
 for(const file of selected){
  checkpoint(options);if(++observed>10000)throw new Error('Doctor selection exceeds 10,000 file entries.');
  const parts=(file.webkitRelativePath||'').split('/');if(parts.length<2||parts.some((p,i)=>!safe(p,i===parts.length-1?240:200)))throw new Error('Select a folder with safe relative file paths.');
  const root=parts.shift(),name=parts.pop();let folder=parts.join('/'),kind,path;
  if(name==='manifest.json'){kind='manifest';path=name;}
  else if(parts.at(-1)==='analysis'&&analyses.includes(name)){parts.pop();folder=parts.join('/');kind='analysis';path=name;}
  else if(parts.at(-1)==='maps'&&['districts.png','compactness.png','political.png'].includes(name)){parts.pop();folder=parts.join('/');kind='present';path='maps/'+name;}
  else if(name==='audit.json'||name===(parts.at(-1)||root)+'.rplan'){kind='present';path=name;}
  else continue;
  const label=parts.at(-1)||root;const key=root+'/'+folder;const full=key+'/'+kind+'/'+path;
  if(paths.has(full))throw new Error('Duplicate selected artifact path.');paths.add(full);
  if(!groups.has(key)){if(groups.size>=100)throw new Error('Select at most 100 plan folders.');groups.set(key,{label,manifest:null,analysis:Object.create(null),present:[],sources:[]});}
  const g=groups.get(key);if(kind==='present'){g.present.push(path);continue;}
  if(!Number.isSafeInteger(file.size)||file.size<0||file.size>1024*1024)throw new Error('Each JSON source must be at most 1 MiB.');bytes+=file.size;if(bytes>16*1024*1024)throw new Error('Selected JSON sources exceed 16 MiB.');g.sources.push({kind,path,file});
 }
 if(!groups.size&&!options.directoryLabels)throw new Error('No native Doctor artifacts found in the selected folder.');
 // Validate selection limits before reading any JSON; binary artifacts are presence-only.
 const labels=new Set();for(const g of groups.values()){if(labels.has(g.label))throw new Error('Plan folder names must be unique.');labels.add(g.label);}
 for(const g of groups.values()){
  for(const {kind,path,file} of g.sources){
   checkpoint(options);let body;
   try{body=await file.arrayBuffer();}catch(error){checkpoint(options);if(error?.name==='DoctorSelectionChanged')throw error;if(kind==='manifest')g.manifest=null;else g.analysis[path]=null;continue;}
   checkpoint(options);const bytes=new Uint8Array(body);if(bytes.length!==file.size)throw new Error('Source size changed while reading.');
   const source={name:file.name,base64:encodeBytes(bytes)};validateDoctorSource(source);
   if(kind==='manifest')g.manifest=source;else g.analysis[path]=source;
  }delete g.sources;
 }
 return validatePlanDoctorInputs({plans:[...groups.values()].sort((a,b)=>a.label.localeCompare(b.label))});
}

// Select the native `plans` collection, inspecting immediate plan directories only.
// Handles never enter the returned project input, and no write methods are used.
export async function readPlanDoctorDirectory(root,options={}){
 if(root?.kind!=='directory'||typeof root.entries!=='function')throw new Error('Choose the native plans directory.');
 const selected=[],labels=[];let entries=0;
 const list=async handle=>{const out=[];for await(const [name,child]of handle.entries()){checkpoint(options);if(++entries>10000)throw new Error('Doctor selection exceeds 10,000 directory entries.');out.push([name,child]);}return out;};
 async function artifact(label,relative,handle){
  checkpoint(options);const path='plans/'+label+'/'+relative;
  if(relative==='audit.json'||relative.endsWith('.rplan')||relative.startsWith('maps/')){selected.push({name:relative.split('/').at(-1),webkitRelativePath:path,size:0});return;}
  if(handle.kind!=='file'){selected.push({name:relative.split('/').at(-1),webkitRelativePath:path,size:0,arrayBuffer:async()=>{throw new Error('Artifact is a directory.');}});return;}
  let original;try{original=await handle.getFile();}catch{checkpoint(options);selected.push({name:relative,webkitRelativePath:path,size:0,arrayBuffer:async()=>{throw new Error('Artifact is unreadable.');}});return;}
  checkpoint(options);selected.push({name:relative.split('/').at(-1),webkitRelativePath:path,size:original.size,arrayBuffer:async()=>{
   checkpoint(options);const current=await handle.getFile();checkpoint(options);
   if(current.size!==original.size||current.lastModified!==original.lastModified)throw Object.assign(new Error('Doctor source changed since selection.'),{name:'DoctorSelectionChanged'});
   const body=await current.arrayBuffer();checkpoint(options);return body;
  }});
 }
 for(const [label,plan]of await list(root)){
  if(plan.kind!=='directory')continue;
  if(!safe(label)||labels.includes(label))throw new Error('Plan folder names must be unique safe names up to 200 UTF-8 bytes.');labels.push(label);if(labels.length>100)throw new Error('Select at most 100 plan folders.');
  for(const [name,handle]of await list(plan)){
   if(name==='manifest.json'||name==='audit.json'||name===label+'.rplan')await artifact(label,name,handle);
   else if((name==='analysis'||name==='maps')&&handle.kind==='directory')for(const [child,h]of await list(handle)){
    if(name==='analysis'&&analyses.includes(child)||name==='maps'&&['districts.png','compactness.png','political.png'].includes(child))await artifact(label,name+'/'+child,h);
   }
  }
 }
 checkpoint(options);options.onProgress?.('Selected '+labels.length+' immediate plan directories. Reading supported JSON only.');
 return readPlanDoctorFolder(selected,{...options,directoryLabels:labels});
}

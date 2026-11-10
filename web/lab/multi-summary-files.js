import {validateSourceFile,exactSourceText,encodeBytes} from './package-files.js';
export const MULTI_SUMMARY_DEFAULTS={labels:[],plans:{}};
const names=['manifest.json','summary.json','compactness.json','splits.json','contiguity.json'];
const label=s=>typeof s==='string'&&/^[A-Za-z0-9_-]{1,100}$/.test(s)&&!['__proto__','constructor','prototype'].includes(s);
export function validateMultiSummaryInputs(v){
 if(!v||Object.keys(v).sort().join()!=='labels,plans'||!Array.isArray(v.labels)||v.labels.length>60||v.labels.some(s=>!label(s))||!v.plans||Array.isArray(v.plans)||typeof v.plans!=='object'||Object.keys(v.plans).length>60)throw new Error('Select up to 60 portable plan labels.');
 let total=0;for(const [name,files]of Object.entries(v.plans)){if(!label(name)||!files||Array.isArray(files)||typeof files!=='object'||Object.keys(files).length>5)throw new Error('Invalid supplied plan folder.');for(const [path,source]of Object.entries(files)){if(!names.includes(path))throw new Error('Unknown native summary source.');validateSourceFile(source,{maxBytes:3*1024*1024});total+=new TextEncoder().encode(exactSourceText(source)).length;}}
 if(total>8*1024*1024)throw new Error('Summary sources exceed 8 MiB.');return v;
}
export function multiSummaryRequest(v){validateMultiSummaryInputs(v);if(v.labels.length<2)throw new Error('Supply at least two plan labels.');return {operation:'summarize-plan-labels',input:{labels:v.labels,plans:Object.fromEntries(Object.entries(v.plans).filter(([name])=>v.labels.includes(name)).map(([name,files])=>[name,Object.fromEntries(Object.entries(files).map(([p,s])=>[p,exactSourceText(s)]))]))}};}
export async function readMultiSummaryFolder(selection,labels=[]){
 validateMultiSummaryInputs({labels,plans:{}});const selected=labels.length?new Set(labels):null;
 const list=[...selection];if(!list.length||list.length>10000)throw new Error('Select one plans collection with at most 10,000 files.');
 let root,total=0;const pending=[],plans=Object.create(null),seen=new Set();
 for(const file of list){const parts=(file.webkitRelativePath||'').split('/');if(parts.length<2||parts.some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p)||/[\\\u0000-\u001f]/.test(p)))throw new Error('Unsafe plan collection path.');if(root&&root!==parts[0])throw new Error('Select one plans collection.');root=parts[0];if(parts.length<3)continue;
  const name=parts[1];if(selected&&!selected.has(name))continue;if(!label(name))throw new Error('Invalid plan label.');plans[name]??=Object.create(null);if(Object.keys(plans).length>60)throw new Error('Too many plan folders.');
  const source=parts.length===3&&parts[2]==='manifest.json'?'manifest.json':parts.length===4&&parts[2]==='analysis'&&names.includes(parts[3])&&parts[3]!=='manifest.json'?parts[3]:null;if(!source)continue;
  const key=name+'/'+source;if(seen.has(key.toLowerCase()))throw new Error('Duplicate native summary source.');seen.add(key.toLowerCase());if(!Number.isSafeInteger(file.size)||file.size<1||file.size>3*1024*1024)throw new Error('Summary source exceeds 3 MiB.');total+=file.size;if(total>8*1024*1024)throw new Error('Summary sources exceed 8 MiB.');pending.push({name,source,file});
 }
 for(const {name,source,file}of pending){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Summary source changed while reading.');new TextDecoder('utf-8',{fatal:true}).decode(bytes);plans[name][source]={name:file.name,base64:encodeBytes(bytes)};}
 return validateMultiSummaryInputs({labels:selected?[...labels]:Object.keys(plans).sort(),plans});
}

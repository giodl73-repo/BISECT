import {validateSourceFile,encodeBytes} from './package-files.js';
export const TUI_COMPARE_PATHS=['data/final_assignments.json','analysis/compactness.json','analysis/summary.json','analysis/splits.json','analysis/contiguity.json'];
export const TUI_COMPARE_DEFAULTS={a:{label:'Plan A',sources:{}},b:{label:'Plan B',sources:{}}};
const bytes=s=>Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0));
const text=s=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes(s));
const safe=s=>s&&new TextEncoder().encode(s).length<=200&&!/[\\/:\u0000-\u001f\u007f-\u009f]/.test(s)&&!['.','..','__proto__','constructor','prototype'].includes(s);
export function validateTuiCompareInputs(v){
 if(!v||Object.keys(v).sort().join()!=='a,b')throw new Error('Supply two native comparison sides.');let total=0;
 for(const side of [v.a,v.b]){if(!side||Object.keys(side).sort().join()!=='label,sources'||typeof side.label!=='string'||new TextEncoder().encode(side.label).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(side.label)||!side.sources||Array.isArray(side.sources)||Object.keys(side.sources).some(k=>!TUI_COMPARE_PATHS.includes(k)))throw new Error('Invalid native comparison label or source path.');
  for(const s of Object.values(side.sources)){if(s?.base64===''){if(Object.keys(s).sort().join()!=='base64,name'||typeof s.name!=='string'||!safe(s.name))throw new Error('Invalid empty comparison source.');}else validateSourceFile(s,{maxBytes:8*1024*1024});text(s);total+=bytes(s).length;}
 }if(total>16*1024*1024)throw new Error('Comparison JSON sources exceed 16 MiB.');return v;
}
export function tuiCompareRequest(v){validateTuiCompareInputs(v);return {operation:'compare-tui-plans',input:Object.fromEntries(['a','b'].map(k=>[k,{label:v[k].label,sources:Object.fromEntries(Object.entries(v[k].sources).map(([path,s])=>[path,text(s)]))}]))};}
export async function readTuiCompareFolder(selected,label){
 const files=Array.from(selected);if(files.length>5000)throw new Error('Select one plan folder with at most 5,000 files.');const pending=new Map();let root=null,total=0;
 for(const file of files){const parts=(file.webkitRelativePath||'').split('/');if(parts.length<2||!parts.every(safe))throw new Error('Select one plan folder with safe relative paths.');if(root!==null&&root!==parts[0])throw new Error('Select one plan folder per side.');root=parts[0];const path=parts.slice(1).join('/');if(!TUI_COMPARE_PATHS.includes(path))continue;if(pending.has(path))throw new Error('Duplicate native comparison source.');if(!Number.isSafeInteger(file.size)||file.size<0||file.size>8*1024*1024)throw new Error('Each comparison JSON source supports up to 8 MiB.');total+=file.size;if(total>16*1024*1024)throw new Error('Comparison JSON sources exceed 16 MiB.');pending.set(path,file);}
 if(root===null)throw new Error('Select a native plan folder.');const sources={};for(const [path,file]of pending){const b=new Uint8Array(await file.arrayBuffer());if(b.length!==file.size)throw new Error('Comparison source size changed while reading.');new TextDecoder('utf-8',{fatal:true}).decode(b);sources[path]={name:file.name,base64:encodeBytes(b)};}const side={label:label??root,sources};validateTuiCompareInputs({a:side,b:TUI_COMPARE_DEFAULTS.b});return side;
}

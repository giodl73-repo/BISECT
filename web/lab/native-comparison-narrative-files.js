import {encodeBytes,isBase64Encoding} from './package-files.js';

export const NATIVE_NARRATIVE_OPERATION='render-native-comparison-narrative';
export const NATIVE_NARRATIVE_SCHEMA='bisect-native-comparison-narrative-v1';
export const NATIVE_NARRATIVE_DEFAULTS=Object.freeze({a:null,b:null,format:'narrative',leaning_threshold:0.55,close_call_band:0.02,source_date_epoch:null});
const allowed=new Set(['manifest.json','data/final_assignments.json','final_assignments.json',
  'analysis/partisan.json','analysis/vra_analysis.json','analysis/compactness.json']);
const mime={'narrative.md':'text/markdown','narrative_manifest.json':'application/json',
  'comparison.html':'text/html','comparison.txt':'text/plain'};
const max=4*1024*1024;
const encode=new TextEncoder(),decode=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});

export function nativeNarrativePlain(value){
  return typeof value==='string'&&value.trim().length>0&&encode.encode(value).length<=200
    && !/[\p{Cc}<>\[\]`\\]/u.test(value);
}

function folderName(value){
  if(!nativeNarrativePlain(value)||value.includes('/')||value==='.'||value==='..')throw new Error('Use a plain native plan folder name.');
  return value;
}

function validateFolder(folder){
  if(!folder||Object.keys(folder).sort().join()!=='files,name')throw new Error('Invalid comparison plan folder.');
  folderName(folder.name);
  if(!folder.files||typeof folder.files!=='object'||Array.isArray(folder.files)||Object.keys(folder.files).length>6)throw new Error('Invalid comparison plan files.');
  let size=0;
  for(const [key,text] of Object.entries(folder.files)){
    if(!allowed.has(key)||typeof text!=='string'||text.length>Math.ceil(max/3)*4||!isBase64Encoding(text))throw new Error('Invalid bounded comparison file.');
    size+=text.length/4*3-(text.endsWith('==')?2:text.endsWith('=')?1:0);
    if(size>max)throw new Error('Comparison plan files exceed 4 MiB.');
  }
  if(!Object.hasOwn(folder.files,'manifest.json')||!['data/final_assignments.json','final_assignments.json'].some(key=>Object.hasOwn(folder.files,key)))
    throw new Error('Select a plan folder with manifest.json and final_assignments.json.');
  return folder;
}

export function validateNativeNarrativeInputs(value){
  if(!value||Object.keys(value).sort().join()!=='a,b,close_call_band,format,leaning_threshold,source_date_epoch')throw new Error('Invalid comparison narrative inputs.');
  for(const side of ['a','b'])if(value[side]!==null)validateFolder(value[side]);
  if(!['narrative','both','html'].includes(value.format))throw new Error('Unknown native report format.');
  for(const key of ['leaning_threshold','close_call_band'])if(!Number.isFinite(value[key])||value[key]<0||value[key]>1)throw new Error('Use threshold and close-call fractions from 0 to 1.');
  const epoch=value.source_date_epoch;
  if(epoch!==null&&(typeof epoch!=='string'||epoch.length>20||!/^(0|-?[1-9][0-9]*)$/.test(epoch)||BigInt(epoch)<-(1n<<63n)||BigInt(epoch)>=(1n<<63n)))
    throw new Error('Use a canonical signed integer for the optional source date epoch.');
  return value;
}

export async function readNativeNarrativeFolder(fileList){
  const files=Array.from(fileList??[]);
  if(!files.length)return null;
  if(files.length>10000)throw new Error('The selected folder contains too many entries.');
  const pending=new Map();let name,size=0;
  for(const file of files){
    const segments=file.webkitRelativePath?.split('/')??[];
    const root=folderName(segments.shift());
    if(name&&name!==root)throw new Error('Select one native plan folder at a time.');
    name=root;
    const key=segments.join('/');
    if(!allowed.has(key))continue;
    if(pending.has(key)||!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Duplicate or invalid comparison file.');
    size+=file.size;if(size>max)throw new Error('Comparison plan files exceed 4 MiB.');
    pending.set(key,file);
  }
  const source={name,files:{}};
  if(!pending.has('manifest.json')||!['data/final_assignments.json','final_assignments.json'].some(key=>pending.has(key)))
    throw new Error('Select a plan folder with manifest.json and final_assignments.json.');
  for(const [key,file] of pending){
    const bytes=new Uint8Array(await file.arrayBuffer());
    if(bytes.length!==file.size)throw new Error('Comparison file changed while reading.');
    decode.decode(bytes);
    source.files[key]=encodeBytes(bytes);
  }
  return validateFolder(source);
}

export function nativeNarrativeRequest(inputs,{allowCrossYear=false,signoff=null}={}){
  validateNativeNarrativeInputs(inputs);
  if(!inputs.a||!inputs.b)throw new Error('Select both native plan folders.');
  if(typeof allowCrossYear!=='boolean'||(signoff!==null&&!nativeNarrativePlain(signoff)))throw new Error('Confirm a plain sign-off name for this run.');
  const side=folder=>({name:folder.name,files:Object.fromEntries(Object.entries(folder.files).map(([key,text])=>
    [key,decode.decode(Uint8Array.from(atob(text),c=>c.charCodeAt(0)))]))});
  return {operation:NATIVE_NARRATIVE_OPERATION,input:{plan_a:side(inputs.a),plan_b:side(inputs.b),
    format:inputs.format,leaning_threshold:inputs.leaning_threshold,close_call_band:inputs.close_call_band,
    source_date_epoch:inputs.source_date_epoch,allow_cross_year:allowCrossYear,
    signoff:signoff===null?null:{name:signoff,confirmed_for_this_run:true}}};
}

export async function nativeNarrativeDownload(result,name){
  if(result?.schema_version!==NATIVE_NARRATIVE_SCHEMA||!Object.hasOwn(mime,name)||!Object.hasOwn(result.files??{},name)
    ||typeof result.files[name]!=='string')throw new Error('Run the selected native report format before downloading.');
  const bytes=encode.encode(result.files[name]);
  if(bytes.length>2*1024*1024)throw new Error('Native report file exceeds the download limit.');
  const sha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
  if(sha!==result.file_sha256?.[name])throw new Error('Native report bytes differ from the fresh result.');
  return {bytes,mime:mime[name]};
}

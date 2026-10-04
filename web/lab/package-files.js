// Exact selected bytes, never extracted to disk or interpreted as markup.
export const MAX_PACKAGE_BYTES=8*1024*1024;
function validatePackagePath(key){
  if(key.length>240 || !/^[a-zA-Z0-9_./ -]+$/.test(key) || key.split('/').some(p=>!p || p==='.' || p==='..' || ['__proto__','constructor','prototype'].includes(p)))throw new Error('Invalid package path.');
}
export function validatePackageFiles(files) {
  if(!files || typeof files!=='object' || Array.isArray(files))throw new Error('Invalid package files.');
  const keys=Object.keys(files);let bytes=0;
  if(!keys.length || keys.length>2000)throw new Error('Select between 1 and 2,000 package files.');
  for(const key of keys){
    validatePackagePath(key);
    const value=files[key];
    if(typeof value!=='string' || value.length>Math.ceil(MAX_PACKAGE_BYTES/3)*4 || (value.length%4!==0 || !/^[A-Za-z0-9+/]*={0,2}$/.test(value)))throw new Error('Invalid package file encoding.');
    bytes+=value.length/4*3-(value.endsWith('==')?2:value.endsWith('=')?1:0);
    if(bytes>MAX_PACKAGE_BYTES)throw new Error('Package exceeds the 8 MiB limit.');
  }
  if(!Object.hasOwn(files,'manifest.json'))throw new Error('Package is missing manifest.json.');
  return files;
}
export function encodeBytes(bytes){
  let binary='';for(let i=0;i<bytes.length;i+=8192)binary+=String.fromCharCode(...bytes.subarray(i,i+8192));
  return btoa(binary);
}
export function decodePackageFiles(files){
  validatePackageFiles(files);const decoded=Object.create(null);
  for(const [path,value]of Object.entries(files))decoded[path]=Array.from(atob(value),c=>c.charCodeAt(0));
  return decoded;
}
export async function readSelectedPackage(selected){
  const entries=Array.from(selected);let total=0;
  if(entries.length>2000)throw new Error('Too many package files.');
  for(const file of entries){if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid package file size.');total+=file.size;if(total>MAX_PACKAGE_BYTES)throw new Error('Package exceeds the 8 MiB limit.');}
  const paths=entries.map(file=>file.webkitRelativePath||file.name);
  const manifests=paths.filter(path=>path==='manifest.json'||path.endsWith('/manifest.json'));
  if(manifests.length!==1)throw new Error('Select one package folder containing one manifest.json.');
  const root=manifests[0].slice(0,-'manifest.json'.length), files=Object.create(null);
  // Reject paths and duplicates before reading or encoding any selected bytes.
  const relative=paths.map(path=>{if(!path.startsWith(root))throw new Error('Selected files do not belong to one package.');const value=path.slice(root.length);validatePackagePath(value);return value;});
  if(new Set(relative).size!==relative.length)throw new Error('Duplicate package file.');
  total=0;
  for(let i=0;i<entries.length;i++){
    if(!paths[i].startsWith(root))throw new Error('Selected files do not belong to one package.');
    const path=paths[i].slice(root.length);
    if(Object.hasOwn(files,path))throw new Error('Duplicate package file.');
    const bytes=new Uint8Array(await entries[i].arrayBuffer());total+=bytes.byteLength;
    if(total>MAX_PACKAGE_BYTES)throw new Error('Package exceeds the 8 MiB limit.');
    files[path]=encodeBytes(bytes);
  }
  return validatePackageFiles(files);
}

export function validateImportSettings(value,{requireComplete=true}={}){
  const keys=['country','state','county','date','election_type','scope','status'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(key=>!Object.hasOwn(value,key)||typeof value[key]!=='string'||(requireComplete&&!value[key].trim())||new TextEncoder().encode(value[key]).length>200))throw new Error('Supply explicit election and jurisdiction metadata.');
  return value;
}
export function validateDemographicSettings(value,{requireComplete=true}={}){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'basis')||!Object.hasOwn(value,'source_label')||!['voting-age-population','citizen-voting-age-population'].includes(value.basis)||typeof value.source_label!=='string'||new TextEncoder().encode(value.source_label).length>200||(requireComplete&&!value.source_label.trim()))throw new Error('Select VAP/CVAP basis and a source label of 1–200 bytes.');
  return value;
}
export function validateSourceFile(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'name')||!Object.hasOwn(value,'base64')||typeof value.name!=='string'||value.name.length>200)throw new Error('Invalid election source.');
  validatePackageFiles({'manifest.json':value.base64});
  if(!value.base64.length)throw new Error('Select a nonempty election source.');
  return value;
}
export async function readElectionSource(file){
  if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_PACKAGE_BYTES)throw new Error('Select a nonempty election source of at most 8 MiB.');
  const bytes=new Uint8Array(await file.arrayBuffer());
  if(!bytes.length||bytes.length>MAX_PACKAGE_BYTES)throw new Error('Election source exceeds 8 MiB.');
  return validateSourceFile({name:file.name,base64:encodeBytes(bytes)});
}
export function validatePackageArchive(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||value.schema_version!=='bisect-rcount-files-v1'||!Object.hasOwn(value,'files'))throw new Error('Invalid RCOUNT package archive.');
  return validatePackageFiles(value.files);
}
export function validateRiSources(value,{requireComplete=true}={}){
  const keys=['auditReport','ballotManifest','ballotRetrieval'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(key=>!keys.includes(key))||(requireComplete&&keys.some(key=>!Object.hasOwn(value,key))))throw new Error('Select the RI audit report, ballot manifest and ballot retrieval CSV files.');
  let bytes=0;
  for(const source of Object.values(value)){validateSourceFile(source);bytes+=source.base64.length/4*3-(source.base64.endsWith('==')?2:source.base64.endsWith('=')?1:0);}
  if(bytes>MAX_PACKAGE_BYTES)throw new Error('Combined RI audit sources exceed 8 MiB.');
  return value;
}
export function validateAggregationSettings(value,{requireComplete=true}={}){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==3||!Object.hasOwn(value,'contest_id')||typeof value.contest_id!=='string'||(requireComplete&&!value.contest_id.trim())||new TextEncoder().encode(value.contest_id).length>200||!['unofficial','canvassed','recounted','amended','certified','withdrawn','superseded'].includes(value.status)||!['json','pretty-json'].includes(value.output_format))throw new Error('Supply a contest ID, count status and transcript format.');return value;
}

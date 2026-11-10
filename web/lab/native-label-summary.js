import {parseSafeJson} from './project.js';
import {encodeBytes,validatePackagePath} from './package-files.js';
const MAX=25*1024*1024,ONE=1024*1024;
const bytes=text=>new TextEncoder().encode(text);
const sha=async data=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',data)),x=>x.toString(16).padStart(2,'0')).join('');
const safeLabel=value=>typeof value==='string'&&/^[A-Za-z0-9_-]{1,100}$/.test(value)&&!['__proto__','constructor','prototype'].includes(value);
export function nativeSummaryOutputDirectory(value){
 if(value===''||value==null)return null;if(value==='.')return '.';if(typeof value!=='string'||value.length>200)throw new Error('Choose a bounded relative output directory.');validatePackagePath(value);
 if(['.bisect','.bisect.lock','.bisect.tmp'].includes(value.split('/')[0])||value.split('/').some(part=>part.endsWith('.')||part.endsWith(' ')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)))throw new Error('Unsafe relative report output directory.');return value;
}
export function validateNativeSummarySource(source){
 if(!source||source.schema_version!=='bisect-native-label-summary-sources-v1'||Object.keys(source).sort().join()!=='label,registry_json,schema_version,sources'||!safeLabel(source.label)||typeof source.registry_json!=='string'||bytes(source.registry_json).length>ONE||!source.sources||typeof source.sources!=='object'||Array.isArray(source.sources)||Object.keys(source.sources).length>100)throw new Error('Select a bounded native summary source snapshot.');
 parseSafeJson(source.registry_json);let total=bytes(source.registry_json).length;
 for(const [year,value]of Object.entries(source.sources)){
  if(!/^\d{4}$/.test(year)||!value||Object.keys(value).sort().join()!=='analysis_directory_present,analysis_index_json'||typeof value.analysis_directory_present!=='boolean'||value.analysis_index_json!==null&&typeof value.analysis_index_json!=='string'||value.analysis_index_json!==null&&bytes(value.analysis_index_json).length>ONE)throw new Error('Invalid native summary year source.');
  total+=value.analysis_index_json===null?0:bytes(value.analysis_index_json).length;if(total>MAX)throw new Error('Native summary sources exceed 25 MiB.');
 }return source;
}
export function nativeSummarySourceFromAnalysis(source){
 if(source?.schema_version==='bisect-native-label-summary-sources-v1')return validateNativeSummarySource(source);
 const batches=source.batches||{single:source};let label,registry=source.registry_json;const sources={};
 for(const batch of Object.values(batches)){
  const index=parseSafeJson(batch.analysis_index_json);if(!safeLabel(index.label)||!/^\d{4}$/.test(index.year)||label!==undefined&&label!==index.label)throw new Error('Choose analyses for one label with explicit years.');label=index.label;
  registry??=batch.registry_json??batch.files?.['.bisect'];sources[index.year]={analysis_directory_present:true,analysis_index_json:batch.analysis_index_json};
 }
 if(typeof registry!=='string')throw new Error('Native summaries require the original registry. Read the matching repository folder.');
 return validateNativeSummarySource({schema_version:'bisect-native-label-summary-sources-v1',label,registry_json:registry,sources});
}
export async function snapshotNativeSummaryRoot(root,label,{signal,assertCurrent}={}){
 const check=()=>{signal?.throwIfAborted();assertCurrent?.();};check();if(root?.kind!=='directory'||!safeLabel(label))throw new Error('Choose the repository and its safe label name.');
 if(root.queryPermission&&await root.queryPermission({mode:'read'})!=='granted')throw new Error('Repository read permission is unavailable.');
 async function text(file){const value=await file.getFile();if(value.size>ONE)throw new Error('Registry/index exceeds 1 MiB.');const data=new Uint8Array(await value.arrayBuffer());check();if(data.length!==value.size)throw new Error('Source changed while reading.');return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(data);}
 const registry_json=await text(await root.getFileHandle('.bisect')),registry=parseSafeJson(registry_json),years=registry[label]?.analyzed??[];
 if(!Array.isArray(years)||years.length>100||years.some(year=>typeof year!=='string'||!/^\d{4}$/.test(year)))throw new Error('Choose at most 100 recorded four-digit analyzed years.');
 const sources={};let total=bytes(registry_json).length;
 for(const year of years){check();if(Object.hasOwn(sources,year))continue;let directory;
  try{directory=await(await(await root.getDirectoryHandle('analysis')).getDirectoryHandle(label)).getDirectoryHandle(year);}catch(error){if(error.name!=='NotFoundError')throw error;sources[year]={analysis_directory_present:false,analysis_index_json:null};continue;}
  let index=null;try{index=await text(await directory.getFileHandle('index.json'));}catch(error){if(error.name!=='NotFoundError')throw error;}
  total+=index===null?0:bytes(index).length;if(total>MAX)throw new Error('Summary snapshots exceed 25 MiB.');sources[year]={analysis_directory_present:true,analysis_index_json:index};
 }
 check();const value=validateNativeSummarySource({schema_version:'bisect-native-label-summary-sources-v1',label,registry_json,sources}),data=bytes(JSON.stringify(value));if(data.length>MAX)throw new Error('Encoded summary snapshot exceeds 25 MiB.');return {name:label+'-native-summary-sources.json',base64:encodeBytes(data)};
}
export async function executeNativeSummaryBatch(engine,input,{onProgress,signal}={}){
 if(!input||Object.keys(input).some(key=>!['source','year','format','generated_at','output_dir'].includes(key))||!['html','json'].includes(input.format)||typeof input.generated_at!=='string'||!input.generated_at.trim()||input.generated_at.length>100||input.year!=null&&(typeof input.year!=='string'||!/^\d{4}$/.test(input.year)))throw new Error('Select native summary format, year and recorded timestamp.');
 const source=validateNativeSummarySource(input.source),registry=parseSafeJson(source.registry_json),declared=registry[source.label]?.analyzed,years=input.year!=null?[input.year]:Array.isArray(declared)?declared:[];
 if(years.length>100||years.some(year=>typeof year!=='string'||!/^\d{4}$/.test(year)))throw new Error('Choose at most 100 recorded four-digit years.');
 const output=nativeSummaryOutputDirectory(input.output_dir),files={},hashes={},outcomes=[];let current=source.registry_json,error=null;
 const result=()=>({schema_version:'bisect-label-report-v1',report_variant:'native-summary-batch',label:source.label,year:years.length===1?years[0]:null,format:input.format,status:error?outcomes.some(row=>row.status==='completed')?'partial':'failed':outcomes.length<years.length?'partial':'generated',execution_status:error?'failed':outcomes.length<years.length?'running':'completed',requested_years:[...years],years:structuredClone(outcomes),files:{...files},file_sha256:{...hashes},registry_promoted:outcomes.some(row=>row.status==='completed'),registry_json:current,registry_source_sha256:null,output_dir:output,error,states:[],scope:'Native summary commands in original registry year order, stopping on first failure and retaining completed files. A shared output directory overwrites index.json in native order. Prepared bytes only; no filesystem authority, state-analysis success or certification. Native filesystem application remains separate.'});
 if(!years.length){try{await engine.execute({operation:'run-native-label-summary',input:{label:source.label,year:input.year||'0000',registry_json:current,analysis_directory_present:false,analysis_index_json:null,format:input.format,generated_at:input.generated_at}});}catch(e){error=e.message;}if(!error)throw new Error('Native empty-year guard did not refuse the request.');return result();}
 for(const [ordinal,year]of years.entries()){
  signal?.throwIfAborted();const directory=`analysis/${source.label}/${year}`,original=source.sources[year]||{analysis_directory_present:false,analysis_index_json:null};
  const snapshot={analysis_directory_present:original.analysis_directory_present||Object.keys(files).some(path=>path.startsWith(directory+'/')),analysis_index_json:Object.hasOwn(files,directory+'/index.json')?files[directory+'/index.json']:original.analysis_index_json};
  try{
   const generated=await engine.execute({operation:'run-native-label-summary',input:{label:source.label,year,registry_json:current,...snapshot,format:input.format,generated_at:input.generated_at}});signal?.throwIfAborted();
   const prefix=`reports/${source.label}/${year}/`,mapped={};for(const [name,text]of Object.entries(generated.files)){const destination=output&&name!=='.bisect'?(output==='.'?'':output+'/')+name.slice(prefix.length):name;validatePackagePath(destination);mapped[destination]=text;}
   const candidate={...files,...mapped};if(Object.values(candidate).reduce((n,text)=>n+bytes(text).length,0)>MAX)throw new Error('Prepared summary files exceed 25 MiB. Completed prefix retained.');
   for(const [name,text]of Object.entries(mapped)){files[name]=text;hashes[name]=await sha(bytes(text));}current=generated.registry_json;
   outcomes.push({ordinal,year,status:'completed',analysis_index_present:generated.analysis_index_present,analysis_index_sha256:generated.analysis_index_sha256,files:Object.keys(mapped).filter(path=>path!=='.bisect')});
  }catch(e){if(signal?.aborted)throw e;error=e.message;outcomes.push({ordinal,year,status:'failed',error});}
  await onProgress?.(result());if(error)break;
 }
 const value=result();value.registry_source_sha256=await sha(bytes(source.registry_json));return value;
}

export async function nativeSummaryOutputFiles(result){
 if(result?.schema_version!=='bisect-label-report-v1'||result.report_variant!=='native-summary-batch'||!safeLabel(result.label)||!['html','json'].includes(result.format)||!Array.isArray(result.requested_years)||result.requested_years.length>100||result.requested_years.some(year=>!/^\d{4}$/.test(year))||!Array.isArray(result.years)||result.years.length>result.requested_years.length||!result.files||!result.file_sha256)throw new Error('Invalid native summary output.');
 const directory=nativeSummaryOutputDirectory(result.output_dir),expected=new Set(),indexes=new Map();let completed=0,failed=false;
 for(const [ordinal,row]of result.years.entries()){
  if(failed||row.ordinal!==ordinal||row.year!==result.requested_years[ordinal]||!['completed','failed'].includes(row.status))throw new Error('Native summary step order differs.');
  if(row.status==='failed'){failed=true;continue;}completed++;
  const prefix=directory?(directory==='.'?'':directory+'/'):`reports/${result.label}/${row.year}/`,report=`${result.label}_${row.year}_report.${result.format}`,index=prefix+'index.json';
  const names=[index,prefix+report];if(!Array.isArray(row.files)||JSON.stringify([...row.files].sort())!==JSON.stringify(names.sort()))throw new Error('Native summary step file roster differs.');
  for(const path of names)expected.add(path);indexes.set(index,{row,report});
 }
 if(completed)expected.add('.bisect');if(result.registry_promoted!==(completed>0)||Object.keys(result.files).length!==expected.size||Object.keys(result.file_sha256).length!==expected.size)throw new Error('Native summary output roster differs.');
 const entries=[];let total=0;for(const path of [...expected].sort()){
  validatePackagePath(path);const text=result.files[path];if(typeof text!=='string'||(total+=bytes(text).length)>MAX||await sha(bytes(text))!==result.file_sha256[path])throw new Error('Native summary output hash or byte limit mismatch.');entries.push([path,bytes(text)]);
 }
 if(completed&&result.files['.bisect']!==result.registry_json)throw new Error('Native summary registry bytes differ.');
 for(const [path,{row,report}]of indexes){const index=parseSafeJson(result.files[path]);if(index.label!==result.label||index.year!==row.year||index.format!==result.format||index.analysis_index_sha256!==row.analysis_index_sha256||JSON.stringify(index.files)!==JSON.stringify([report]))throw new Error('Native summary index differs from its last writer.');}
 return entries;
}

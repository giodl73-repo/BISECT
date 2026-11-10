import {parseSafeJson,inspectJsonText,parseNativeMapPlanJson} from './project.js';
import {encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
import {labelImportPlanRequest,validateLabelImportInputs} from './label-import-files.js';
const cap=25*1024*1024;
export function labelBatchText(s){
 if(!s||Object.keys(s).sort().join()!=='base64,name'||typeof s.name!=='string'||s.name.length>200||typeof s.base64!=='string'||!s.base64.length||s.base64.length>Math.ceil(cap/3)*4||s.base64.length%4!==0||/[^A-Za-z0-9+/=]/.test(s.base64))throw new Error('Invalid bounded analysis batch encoding.');
 const padding=s.base64.indexOf('=');if(padding!==-1&&(padding<s.base64.length-2||!/^={1,2}$/.test(s.base64.slice(padding))))throw new Error('Invalid analysis batch padding.');
 const bytes=Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0));if(bytes.length>cap)throw new Error('Batch exceeds 25 MiB.');
 return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}
const text=labelBatchText;
export const labelAnalysisTypes=['manifest','contiguity','splits','statistics','compactness','election'];
function validateBatch(input){
 if(!input||Object.keys(input).some(k=>!['build_index_json','created','states','registry_json','external_import'].includes(k))||!['build_index_json','created','states'].every(k=>Object.hasOwn(input,k))||typeof input.build_index_json!=='string'||new TextEncoder().encode(input.build_index_json).length>1024*1024||typeof input.created!=='string'||!input.states||Array.isArray(input.states)||Object.keys(input.states).length<1||Object.keys(input.states).length>60)throw new Error('Select a native analysis batch with a build index and 1–60 state sources.');
 inspectJsonText(input.build_index_json);JSON.parse(input.build_index_json);validateRegistry(input.registry_json);
 if(input.external_import!=null){const proof=input.external_import,native=proof.input;if(!proof||Object.keys(proof).sort().join()!=='district_bases,input'||!native||Object.keys(native).sort().join()!=='created,dbf,format,label,registry_json,shx,source,year'||!proof.district_bases||Array.isArray(proof.district_bases)||Object.keys(proof.district_bases).length>60||Object.entries(proof.district_bases).some(([name,basis])=>!/^[a-z_]+$/.test(name)||!['one-based','zero-based'].includes(basis))||Object.keys(input.states).some(name=>!Object.hasOwn(proof.district_bases,name)))throw new Error('Select an original import with explicit state numbering bases.');validateRegistry(native.registry_json);validateLabelImportInputs({source:native.source,dbf:native.dbf,shx:native.shx,registry:null,settings:{label:native.label,year:native.year,created:native.created,format:native.format??'auto'}});}
}
export function labelAnalysisFromExternalPlan(result,inputs,selection,previous=null){
 const request=labelImportPlanRequest(inputs,selection);
 if(result?.schema_version!=='bisect-label-import-plan-v1'||result.state_name!==selection.state_name||result.bindings?.index_sha256!==selection.index_sha256||result.context_json!==request.input.context_json||typeof result.document_json!=='string'||typeof result.files?.['import-index.json']!=='string')throw new Error('Prepare this imported state with its exact context first.');
 let batch={build_index_json:result.files['import-index.json'],created:'',states:{},external_import:{input:request.input.input,district_bases:{}}};
 if(previous){validateLabelAnalysisInputs(previous);if(!previous.source)throw new Error('Prepare a batch before adding a state.');batch=parseSafeJson(text(previous.source));if(batch.batches||batch.build_index_json!==result.files['import-index.json']||JSON.stringify(batch.external_import?.input)!==JSON.stringify(request.input.input))throw new Error('Add states from the same original import and year.');}
 batch.created='';batch.states[selection.state_name]={document_json:result.document_json,context_json:result.context_json,geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,tolerance_percent:0.5,election_settings:{bootstrap_seed:'42',bootstrap_replicates:100},compactness:null};batch.external_import.district_bases[selection.state_name]=selection.district_basis;
 if(!previous&&typeof result.import_registry_json==='string')batch.registry_json=result.import_registry_json;
 const source={name:inputs.settings.label+'-'+inputs.settings.year+'-external-analysis.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(batch)))};
 return validateLabelAnalysisInputs({source,created:''});
}
function validateRegistry(value){if(value!=null){if(typeof value!=='string'||new TextEncoder().encode(value).length>1024*1024)throw new Error('Registry exceeds 1 MiB.');parseSafeJson(value);}}
function batches(source){return source.batches||{[JSON.parse(source.build_index_json).year]:source};}
export function validateLabelAnalysisInputs(v){
 if(!v||Object.keys(v).some(k=>!['created','source','states','types','years'].includes(k))||!Object.hasOwn(v,'source')||typeof v.created!=='string'||v.created.length>100)throw new Error('Invalid label analysis inputs.');
 if(Object.hasOwn(v,'types')&&(!Array.isArray(v.types)||v.types.length>6||v.types.some(name=>typeof name!=='string'||!labelAnalysisTypes.includes(name))||new Set(v.types).size!==v.types.length))throw new Error('Choose unique implemented analysis types: '+labelAnalysisTypes.join(', ')+'.');
 if(Object.hasOwn(v,'states')&&(!Array.isArray(v.states)||v.states.length>60||v.states.some(name=>typeof name!=='string'||!/^[A-Za-z_]{1,100}$/.test(name))||new Set(v.states.map(name=>name.toLowerCase())).size!==v.states.length))throw new Error('Choose unique supplied state names, with at most 60 selections.');
 if(Object.hasOwn(v,'years')&&(!Array.isArray(v.years)||v.years.length>10||v.years.some(year=>typeof year!=='string'||!/^\d{4}$/.test(year))||new Set(v.years).size!==v.years.length))throw new Error('Choose unique supplied four-digit analysis years, at most 10.');
 if(v.source!==null){const input=parseSafeJson(text(v.source));if(Object.hasOwn(input,'batches')){
  if(Object.keys(input).some(k=>!['created','batches','registry_json'].includes(k))||typeof input.created!=='string'||!input.batches||Array.isArray(input.batches)||Object.keys(input.batches).length<1||Object.keys(input.batches).length>10)throw new Error('Select 1–10 original year batches.');
  let label;for(const [year,batch]of Object.entries(input.batches)){validateBatch(batch);const build=JSON.parse(batch.build_index_json);if(!/^\d{4}$/.test(year)||build.year!==year||label!==undefined&&label!==build.label||batch.registry_json!=null)throw new Error('Year batches need matching original indexes, one label and one shared registry.');label=build.label;}validateRegistry(input.registry_json);
 }else validateBatch(input);}
 if(v.years?.length&&v.years.some(year=>!labelAnalysisYearNames(v).includes(year)))throw new Error('Selected year is absent from the supplied analysis batch.');
 if(v.states?.length&&v.states.some(name=>!labelAnalysisStateNames(v).includes(name.toLowerCase())))throw new Error('Selected state is absent from the supplied analysis batch.');
 return v;
}
export function labelAnalysisStateNames(v){return v?.source?[...new Set(Object.values(batches(parseSafeJson(text(v.source)))).flatMap(batch=>Object.keys(batch.states)))].sort():[];}
export function labelAnalysisYearNames(v){return v?.source?Object.keys(batches(parseSafeJson(text(v.source)))).sort():[];}
export function labelAnalysisRequest(v){
 validateLabelAnalysisInputs(v);if(!v.source||!v.created.trim())throw new Error('Select a batch and record an analysis timestamp.');
 const source=parseSafeJson(text(v.source)),filter=batch=>{const input={...batch,created:v.created};if(v.states?.length){const selected=new Set(v.states.map(name=>name.toLowerCase()));input.states=Object.fromEntries(Object.entries(input.states).filter(([name])=>selected.has(name)));}return input;};
 if(source.batches){const selected=new Set(v.years?.length?v.years:Object.keys(source.batches)),input={created:v.created,batches:Object.fromEntries(Object.entries(source.batches).filter(([year])=>selected.has(year)).map(([year,batch])=>[year,filter(batch)]))};if(v.types?.length)input.types=[...v.types];if(source.registry_json!=null)input.registry_json=source.registry_json;return {operation:'run-label-analysis-years',input};}
 const input=filter(source);if(v.types?.length)input.types=[...v.types];return {operation:'run-label-analysis',input};
}
export async function readLabelAnalysisSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>cap)throw new Error('Select a nonempty batch up to 25 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Batch size changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};const input=parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes));validateLabelAnalysisInputs({source,created:input.created});return {source,created:input.created};
}
export function labelAnalysisFromBuild(result,planPath){
 if(result?.schema_version!=='bisect-label-build-v1'||typeof planPath!=='string')throw new Error('Run a label build and select a plan first.');
 const match=/^runs\/([A-Za-z0-9_-]+)\/(\d{4})\/[a-z_]+\/plan\.rplan$/.exec(planPath);if(!match||match[1]!==result.label)throw new Error('Select a generated plan.');
 const root=`runs/${match[1]}/${match[2]}`,buildText=result.files?.[root+'/index.json'];if(typeof buildText!=='string')throw new Error('The selected build year is incomplete.');
 inspectJsonText(buildText);const build=JSON.parse(buildText),states={}; // Only state metadata is inspected; original index bytes, including numeric u64 seeds, remain untouched.
 for(const [name,entry]of Object.entries(build.states||{})){if(entry.status!=='ok')continue;const document_json=result.files[`${root}/${name}/plan.rplan`],context_json=result.files[`${root}/${name}/context.rctx`];if(typeof document_json!=='string'||typeof context_json!=='string')throw new Error('Completed build is missing a plan or context.');
  const producer=parseNativeMapPlanJson(document_json).provenance?.producer,recorded=[producer?.algorithm_configuration?.execution_tolerance_percent,producer?.engine_options?.balance_tolerance].filter(value=>value!==undefined),tolerance_percent=recorded[0]??0.5;
  if(recorded.some(value=>typeof value!=='number'||!Number.isFinite(value)||value<0||value>100||value!==tolerance_percent))throw new Error('Build plan has invalid or conflicting recorded population tolerances.');
  states[name]={document_json,context_json,geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,tolerance_percent,election_settings:{bootstrap_seed:'42',bootstrap_replicates:100},compactness:null};}
 const input={build_index_json:buildText,created:'',states};if(typeof result.files['.bisect']==='string')input.registry_json=result.files['.bisect'];const source={name:`${result.label}-${match[2]}-analysis-batch.json`,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(input)))};return validateLabelAnalysisInputs({source,created:''});
}
export function labelAnalysisFromBuildYears(result){
 if(result?.schema_version!=='bisect-label-build-v1')throw new Error('Run a label build first.');
 const source={created:'',batches:{}};
 for(const item of result.years||[]){if(item.status!=='built')continue;const prefix=`runs/${result.label}/${item.year}/`,path=Object.keys(result.files||{}).find(path=>path.startsWith(prefix)&&path.endsWith('/plan.rplan'));if(!path)throw new Error('Completed build year has no retained successful plan.');const batch=parseSafeJson(text(labelAnalysisFromBuild(result,path).source));delete batch.registry_json;source.batches[item.year]=batch;}
 if(typeof result.files['.bisect']==='string')source.registry_json=result.files['.bisect'];
 return validateLabelAnalysisInputs({source:{name:`${result.label}-all-years-analysis.json`,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(source)))},created:''});
}
export function labelAnalysisYearResult(result,year){
 if(result?.schema_version==='bisect-label-analysis-v1'){if(result.year!==year)throw new Error('Select an available analysis year.');return result;}
 const item=result?.schema_version==='bisect-label-analysis-years-v1'?result.years?.find(item=>item.year===year):null;if(item?.status!=='completed'||item.result?.year!==year||item.result.label!==result.label)throw new Error('Select a completed analysis year.');return item.result;
}
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');
export async function startLabelAnalysisYearsCheckpoint(request){
 if(request?.operation!=='run-label-analysis-years'||!request.input?.batches)throw new Error('Select a multi-year analysis request.');
 const years=Object.keys(request.input.batches).sort();if(!years.length||years.length>10||years.some(year=>!/^\d{4}$/.test(year)))throw new Error('Invalid checkpoint years.');
 const hashes={};let label;
 for(const year of years){const text=request.input.batches[year].build_index_json;inspectJsonText(text);const index=JSON.parse(text);if(index.year!==year||label!==undefined&&index.label!==label)throw new Error('Checkpoint source identities differ.');label=index.label;hashes[year]=await sha(new TextEncoder().encode(text));}
 const supplied=request.input.types,types=!supplied||supplied.length===1&&supplied[0]==='all'?labelAnalysisTypes:labelAnalysisTypes.filter(type=>supplied.includes(type));
 return {label,created:request.input.created,years,hashes,types};
}
export function retainLabelAnalysisYearsCheckpoint(previous,event,binding){
 const snapshot=event?.snapshot,sequence=previous?(previous.checkpoint_sequence||0)+1:binding?.resume_prefix_count||1;
 if(event?.schema_version!=='bisect-label-analysis-years-progress-v1'||event.sequence!==sequence||!previous&&binding?.resume_prefix_count&&event.resume_verified!==true||JSON.stringify(event.requested_years)!==JSON.stringify(binding?.years)||snapshot?.schema_version!=='bisect-label-analysis-years-v1'||snapshot.label!==binding.label||snapshot.created!==binding.created||JSON.stringify(snapshot.types)!==JSON.stringify(binding.types)||!Array.isArray(snapshot.years)||snapshot.years.length!==sequence||sequence>binding.years.length)throw new Error('Unexpected analysis checkpoint identity or sequence.');
 for(const [i,item]of snapshot.years.entries()){
  if(item.year!==binding.years[i]||!['completed','failed'].includes(item.status))throw new Error('Unscheduled analysis checkpoint year.');
  if(item.status==='completed'){const result=item.result,index=parseSafeJson(result.index_json);if(result.label!==binding.label||result.year!==item.year||index.run_index_sha256!==binding.hashes[item.year]||index.label!==binding.label||index.year!==item.year||result.files?.[result.index_path]!==result.index_json)throw new Error('Analysis checkpoint source/index mismatch.');}
  if(previous&&i<sequence-1&&JSON.stringify(item)!==JSON.stringify(previous.years[i]))throw new Error('Completed analysis checkpoint changed.');
 }
 if(!snapshot.files||!snapshot.file_sha256||Object.keys(snapshot.files).length>8421||Object.values(snapshot.files).some(text=>typeof text!=='string')||Object.values(snapshot.files).reduce((size,text)=>size+new TextEncoder().encode(text).length,0)>cap)throw new Error('Analysis checkpoint files exceed bounds.');
 return {...snapshot,checkpoint_sequence:sequence,requested_years:[...binding.years],execution_status:sequence<binding.years.length?'running':'completed'};
}
export async function labelAnalysisFiles(result){
 if(result?.schema_version==='bisect-label-analysis-years-v1'){
  if(!/^[A-Za-z0-9_-]{1,100}$/.test(result.label)||!Array.isArray(result.years)||result.years.length<1||result.years.length>10||new Set(result.years.map(item=>item.year)).size!==result.years.length)throw new Error('Invalid multi-year analysis result.');
  const expected=new Map();for(const item of result.years){if(!/^\d{4}$/.test(item.year)||!['completed','failed'].includes(item.status))throw new Error('Invalid analysis year outcome.');if(item.status==='completed'){const year=labelAnalysisYearResult(result,item.year);for(const [path,bytes]of await labelAnalysisFiles(year))if(path!=='.bisect')expected.set(path,bytes);}}
  if(typeof result.registry_json==='string'){parseSafeJson(result.registry_json);expected.set('.bisect',new TextEncoder().encode(result.registry_json));}
  const path=`analysis/${result.label}/years.json`,receipt=result.files?.[path];if(typeof receipt!=='string')throw new Error('Missing multi-year analysis receipt.');const record=parseSafeJson(receipt);
  if(record.schema_version!=='bisect-label-analysis-years-record-v1'||record.label!==result.label||record.created!==result.created||record.status!==result.status||JSON.stringify(record.requested_years)!==JSON.stringify(result.requested_years)||JSON.stringify(record.types)!==JSON.stringify(result.types)||!Array.isArray(record.years)||record.years.length!==result.years.length||record.years.some((row,i)=>row.year!==result.years[i].year||row.status!==result.years[i].status||row.error!==(result.years[i].error??null)||row.index_sha256!==(result.years[i].result?.index_sha256??null)))throw new Error('Multi-year receipt differs from recorded outcomes.');
  expected.set(path,new TextEncoder().encode(receipt));if(Object.keys(result.files||{}).length!==expected.size||Object.keys(result.file_sha256||{}).length!==expected.size)throw new Error('Multi-year analysis file count mismatch.');let total=0;for(const [path,bytes]of expected){if(result.files[path]!==new TextDecoder().decode(bytes)||await sha(bytes)!==result.file_sha256[path])throw new Error('Multi-year analysis file or hash mismatch.');total+=bytes.length;if(total>cap)throw new Error('Multi-year outputs exceed 25 MiB.');}return [...expected];
 }

 if(result?.schema_version!=='bisect-label-analysis-v1'||!/^[A-Za-z0-9_-]{1,100}$/.test(result.label)||!/^\d{4}$/.test(result.year)||!result.files||!result.file_sha256)throw new Error('Run label analyses first.');
 const root=`analysis/${result.label}/${result.year}/`,indexPath=root+'index.json';
 if(result.index_path!==indexPath||result.files[indexPath]!==result.index_json||await sha(new TextEncoder().encode(result.index_json))!==result.index_sha256)throw new Error('Analysis index hash or path mismatch.');
 const index=parseSafeJson(result.index_json);if(index.label!==result.label||index.year!==result.year)throw new Error('Analysis index identity mismatch.');
 const names=new Set(['plan.rplan','context.rctx','manifest.json','analysis-package.json','summary.json','demographic.json','urban.json','political.json','proportionality.json','partisan.json','partisan-metrics.json','contiguity.json','compactness.json','splits.json']);
 const entries=Object.entries(result.files);if(entries.length>842||Object.keys(result.file_sha256).length!==entries.length)throw new Error('Invalid analysis file count.');let total=0;const files=[];
 for(const [path,value]of entries){
  if(typeof value!=='string')throw new Error('Analysis file must be text.');
  let isContext=false;
  if(path==='.bisect'){
   if(result.registry_promoted!==true||value!==result.registry_json)throw new Error('Registry output does not match the recorded update.');
   parseSafeJson(value);
  }else if(path!==indexPath){
   const parts=path.slice(root.length).split('/');
   if(!path.startsWith(root)||parts.length!==2||!/^[a-z_]{1,100}$/.test(parts[0])||!Object.hasOwn(index.states,parts[0])||!names.has(parts[1]))throw new Error('Unsafe or unknown analysis output path.');
   isContext=parts[1]==='context.rctx';
  }
  const bytes=new TextEncoder().encode(value);total+=bytes.length;
  if(total>cap)throw new Error('Analysis outputs exceed 25 MiB.');
  if(isContext&&bytes.length>8*1024*1024)throw new Error('Analysis RCTX exceeds 8 MiB.');
  if(await sha(bytes)!==result.file_sha256[path])throw new Error('Analysis file hash mismatch.');
  // RCTX graphs contain many small adjacency records. Keep the larger
  // structural budget specific to this bounded, hash-checked file.
  inspectJsonText(value,isContext?2_000_000:500_000);
  files.push([path,bytes]);
 }
 return files;
}
export async function labelAnalysisZip(result){return storedZipBytes(await labelAnalysisFiles(result));}

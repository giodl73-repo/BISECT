// Exact selected bytes, never extracted to disk or interpreted as markup.
export const MAX_PACKAGE_BYTES=8*1024*1024;
// Avoid repeated-group regular expressions: valid multi-megabyte base64 can
// exhaust the JavaScript regexp stack. Canonical pad bits are checked by the
// existing decode/encode comparison at callers that require canonical bytes.
export function isBase64Encoding(encoded){
  if(typeof encoded!=='string'||encoded.length%4!==0||/[^A-Za-z0-9+/=]/.test(encoded))return false;
  const padding=encoded.indexOf('=');
  return padding===-1||(padding>=encoded.length-2&&/^={1,2}$/.test(encoded.slice(padding)));
}
export function validatePackagePath(key){
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
export async function readSelectedPackage(selected,{maxBytes=MAX_PACKAGE_BYTES}={}){
  if(!Number.isSafeInteger(maxBytes)||maxBytes<1||maxBytes>MAX_PACKAGE_BYTES)throw new Error("Invalid package byte limit.");
  const entries=Array.from(selected);let total=0;
  if(entries.length>2000)throw new Error('Too many package files.');
  for(const file of entries){if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid package file size.');total+=file.size;if(total>maxBytes)throw new Error(`Package exceeds the ${maxBytes/1024/1024} MiB limit.`);}
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
    if(total>maxBytes)throw new Error(`Package exceeds the ${maxBytes/1024/1024} MiB limit.`);
    files[path]=encodeBytes(bytes);
  }
  return validatePackageFiles(files);
}

export function validateImportSettings(value,{requireComplete=true}={}){
  const keys=['country','state','county','date','election_type','scope','status'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(key=>!Object.hasOwn(value,key)||typeof value[key]!=='string'||(requireComplete&&!value[key].trim())||new TextEncoder().encode(value[key]).length>200))throw new Error('Supply explicit election and jurisdiction metadata.');
  return value;
}
export function validateDemographicSettings(value,{requireComplete=true,allowTotalPopulation=false}={}){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'basis')||!Object.hasOwn(value,'source_label')||![...(allowTotalPopulation?['total-population']:[]),'voting-age-population','citizen-voting-age-population'].includes(value.basis)||typeof value.source_label!=='string'||new TextEncoder().encode(value.source_label).length>200||(requireComplete&&!value.source_label.trim()))throw new Error('Select a supported demographic basis and a source label of 1–200 bytes.');
  return value;
}
export function validateSourceFile(value,{maxBytes=MAX_PACKAGE_BYTES,allowEmpty=false}={}){
  if(typeof allowEmpty!=='boolean')throw new Error('Invalid empty-source policy.');
  if(!Number.isSafeInteger(maxBytes)||maxBytes<1||maxBytes>128*1024*1024)throw new Error('Invalid source byte limit.');
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'name')||!Object.hasOwn(value,'base64')||typeof value.name!=='string'||value.name.length>200)throw new Error('Invalid election source.');
  const encoded=value.base64;
  if(typeof encoded!=='string'||encoded.length>Math.ceil(maxBytes/3)*4||!isBase64Encoding(encoded))throw new Error('Invalid package file encoding.');
  const bytes=encoded.length/4*3-(encoded.endsWith('==')?2:encoded.endsWith('=')?1:0);
  if(bytes>maxBytes)throw new Error(`Source exceeds the ${maxBytes/1024/1024} MiB limit.`);
  if(!allowEmpty&&!value.base64.length)throw new Error('Select a nonempty election source.');
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

const REPORT_NAMES=['manifest.json','analysis/partisan.json','analysis/vra_analysis.json','analysis/compactness.json'];
export function validateReportFiles(files){
  validatePackageFiles(files);
  if(Object.keys(files).some(key=>!REPORT_NAMES.includes(key)))throw new Error('Unsupported report file name.');
  const total=Object.values(files).reduce((sum,value)=>sum+value.length/4*3-(value.endsWith('==')?2:value.endsWith('=')?1:0),0);
  if(total>4*1024*1024)throw new Error('Report inputs exceed 4 MiB.');
  return files;
}
export function validateReportSettings(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'label')||!Object.hasOwn(value,'leaning_threshold')||typeof value.label!=='string'||new TextEncoder().encode(value.label).length>200||!Number.isFinite(value.leaning_threshold)||value.leaning_threshold<0||value.leaning_threshold>1)throw new Error('Supply a report label of at most 200 bytes and a leaning threshold from 0 to 100%.');
  return value;
}
export async function readSelectedReport(selected){
  const entries=Array.from(selected);
  if(entries.length>2000)throw new Error('Too many report folder entries.');
  const paths=entries.map(file=>file.webkitRelativePath||file.name);
  const manifests=paths.filter(path=>path==='manifest.json'||path.endsWith('/manifest.json'));
  if(manifests.length!==1)throw new Error('Select one report folder with one manifest.json.');
  const root=manifests[0].slice(0,-'manifest.json'.length);
  const relative=paths.map(path=>{if(!path.startsWith(root))throw new Error('Report files must share one root.');const name=path.slice(root.length);validatePackagePath(name);return name;});
  if(new Set(relative).size!==relative.length)throw new Error('Duplicate report file.');
  return validateReportFiles(await readSelectedPackage(entries.filter((file,index)=>REPORT_NAMES.includes(relative[index])),{maxBytes:4*1024*1024}));
}

export function validateReportComparisonSettings(value){
  const keys=['label_a','label_b','leaning_threshold','close_call_band','generated_at'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(key=>!Object.hasOwn(value,key)))throw new Error('Invalid report comparison settings.');
  validateReportSettings({label:value.label_a,leaning_threshold:value.leaning_threshold});
  validateReportSettings({label:value.label_b,leaning_threshold:value.close_call_band});
  if(typeof value.generated_at!=='string'||new TextEncoder().encode(value.generated_at).length>100)throw new Error('Generation timestamp must be at most 100 bytes.');
  return value;
}

export function validateCeremonySettings(value){
  const keys=['published_unix_seconds','witness_receipts','reason','include_tree'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(k=>!Object.hasOwn(value,k)))throw new Error('Invalid ceremony settings.');
  const time=value.published_unix_seconds;
  if(typeof time!=='string'||time.length>20||time!==''&&(!/^(0|-?[1-9][0-9]*)$/.test(time)||BigInt(time)<-(1n<<63n)||BigInt(time)>(1n<<63n)-1n))throw new Error('Recorded timestamp must be a canonical decimal i64.');
  if(!Array.isArray(value.witness_receipts)||value.witness_receipts.length>100||value.witness_receipts.some(s=>typeof s!=='string'||new TextEncoder().encode(s).length>2000))throw new Error('Supply at most 100 receipt IDs of at most 2000 bytes.');
  if(typeof value.reason!=='string'||new TextEncoder().encode(value.reason).length>2000||typeof value.include_tree!=='boolean')throw new Error('Invalid ceremony halt reason or verification scope.');
  return value;
}
export function exactSourceText(source){
  validateSourceFile(source);
  const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));
  if(bytes.length>3*1024*1024)throw new Error('Artifact exceeds 3 MiB.');
  return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}
// Full-state native RPLAN exports can exceed the smaller exact-proof budget.
// This boundary retains original UTF-8/BOM bytes and does not widen proof inputs.
export function exactPlanText(source){
  validateSourceFile(source,{maxBytes:8*1024*1024});
  return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));
}
export function textAuditSourceLimit(key,operation){
  if(key==='auditContextText')return 8*1024*1024;
  if(key==='auditPlanText'&&!['validate-native-rplan','improve-rplan-text','analyze-plan-election'].includes(operation))return 8*1024*1024;
  return 3*1024*1024;
}
// Native JSON readers consume these exact UTF-8 bytes with serde_json. Preserve a BOM
// rather than silently removing it and accepting a native-invalid artifact.
export function exactNativeJsonText(source,{allowEmpty=false}={}){
  validateSourceFile(source,{maxBytes:3*1024*1024,allowEmpty});
  const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));
  return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
}
// Explicit native JSON readers can inspect a zero-byte file as a parse outcome.
// Generic election/project readers retain their nonempty default.
export async function readNativeJsonSource(file,{maxBytes}={}){
  if(!Number.isSafeInteger(maxBytes)||maxBytes<1||maxBytes>3*1024*1024)throw new Error('Invalid native JSON byte limit.');
  if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>maxBytes)throw new Error('Select native JSON within the '+maxBytes+' byte limit.');
  if(typeof file.name!=='string'||!file.name||new TextEncoder().encode(file.name).length>200)throw new Error('Use a source name of 1–200 UTF-8 bytes.');
  const bytes=new Uint8Array(await file.arrayBuffer());
  if(bytes.length!==file.size||bytes.length>maxBytes)throw new Error('Native JSON size changed while reading.');
  const source={name:file.name,base64:encodeBytes(bytes)};
  validateSourceFile(source,{maxBytes,allowEmpty:true});exactNativeJsonText(source,{allowEmpty:true});return source;
}
// Native YAML readers need the same original UTF-8 bytes, including a BOM.
export const exactNativeUtf8Text=exactNativeJsonText;
// Retain the existing ceremony API and its byte-preserving behavior.
export const exactCeremonyText=exactNativeJsonText;
// RCTX adjacency is larger than other text artifacts; retain exact native bytes.
export function exactContextText(source){
  validateSourceFile(source,{maxBytes:8*1024*1024});
  return new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));
}
export function ceremonyRequest(operation,files,settings){
  validateCeremonySettings(settings);
  const source=key=>{if(!files[key])throw new Error('Select the ceremony '+key.replace('ceremony','').toLowerCase()+' file.');return exactCeremonyText(files[key]);};
  if(operation==='start-ceremony')return {operation,tree_json:source('ceremonyTree'),config_json:source('ceremonyConfig')};
  const transcript_json=source('ceremonyTranscript');
  if(operation==='inspect-ceremony-prefix')return {operation,transcript_json};
  if(operation==='verify-ceremony')return {operation,transcript_json,tree_json:settings.include_tree?source('ceremonyTree'):null};
  const tree_json=source('ceremonyTree');
  if(operation==='announce-ceremony'){
    if(!settings.published_unix_seconds||!settings.witness_receipts.length)throw new Error('Supply a recorded timestamp and witness receipt IDs.');
    return {operation,tree_json,transcript_json,published_unix_seconds:settings.published_unix_seconds,witness_receipts:settings.witness_receipts};
  }
  if(operation==='halt-ceremony'){
    if(!settings.reason.trim())throw new Error('Supply the halt reason.');
    return {operation,tree_json,transcript_json,reason:settings.reason};
  }
  throw new Error('Unsupported ceremony operation.');
}

export function validateColumnSettings(value){
  const keys=['k','tolerance','formulation_only','exact_fixture_limit','label','chamber','created_at'];
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(key=>!Object.hasOwn(value,key)))throw new Error('Invalid column benchmark settings.');
  if(!Number.isInteger(value.k)||value.k<1||value.k>20||!Number.isFinite(value.tolerance)||value.tolerance<0||value.tolerance>1||typeof value.formulation_only!=='boolean'||!Number.isInteger(value.exact_fixture_limit)||value.exact_fixture_limit<0||value.exact_fixture_limit>12)throw new Error('Supply 1 through 20 districts, tolerance from 0% through 100%, and exact master limit from 0 through 12.');
  for(const key of ['label','chamber','created_at'])if(typeof value[key]!=='string'||!value[key].trim()||new TextEncoder().encode(value[key]).length>200)throw new Error('Plan metadata must be nonempty and at most 200 bytes.');
  return value;
}
export function columnContextRequest(source,settings){
  if(!source)throw new Error('Select an RCTX context.');
  return {operation:'solve-branch-price-context',context_json:exactContextText(source),settings:validateColumnSettings(settings)};
}

export function validateTextAuditSettings(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==1||typeof value.generated_at_utc!=='string'||new TextEncoder().encode(value.generated_at_utc).length>200)throw new Error('Invalid recorded audit timestamp.');return value;
}
export function textAuditRequest(operation,files,constraints,settings){
  const source=(key,required)=>{if(!files[key]){if(required)throw new Error('Select the '+key+' file.');return null;}return key==='auditContextText'?exactContextText(files[key]):key==='auditPlanText'?exactPlanText(files[key]):exactSourceText(files[key]);};
  const document_json=source('auditPlanText',operation==='audit-rplan-text'),context_json=source('auditContextText',false);
  if(operation==='verify-audit-text')return {operation,certificate_json:source('auditCertificateText',true),document_json,context_json};
  if(operation!=='audit-rplan-text')throw new Error('Unsupported text audit operation.');
  validateTextAuditSettings(settings);if(!settings.generated_at_utc.trim())throw new Error('Supply a recorded audit timestamp.');
  return {operation,document_json,context_json,profile_json:source('auditProfileText',true),constraints,generated_at_utc:settings.generated_at_utc,lineage_json:source('auditLineageText',false)};
}

export function validateImprovementSettings(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join(',')!=='generated_at_utc,label,tolerance_percent'||!Number.isFinite(value.tolerance_percent)||value.tolerance_percent<0||!(value.label===null||typeof value.label==='string')||typeof value.generated_at_utc!=='string')throw new Error('Invalid improvement settings.');
  for(const text of [value.label,value.generated_at_utc])if(text!==null&&new TextEncoder().encode(text).length>200)throw new Error('Improvement metadata exceeds 200 bytes.');return value;
}
export function validatePlanImportSettings(s){
 if(!s||typeof s!=='object'||Array.isArray(s)||Object.keys(s).sort().join(',')!=='as_civic_counter_proposal,created_at,format,label,source_name,state,submitted_at,submitted_by,year'||!['gerrychain','dra-csv','geojson','geojson-polygons'].includes(s.format)||!Number.isInteger(s.year)||s.year<1900||s.year>2100||typeof s.state!=='string'||!/^[A-Z]{2}$/.test(s.state)||typeof s.as_civic_counter_proposal!=='boolean')throw new Error('Invalid plan import format/state/year settings.');
 for(const value of [s.label,s.created_at,s.source_name,...[s.submitted_by,s.submitted_at].filter(v=>v!==null)])if(typeof value!=='string'||!value.trim()||new TextEncoder().encode(value).length>200)throw new Error('Import metadata must be nonempty and at most 200 bytes.');
 if(s.as_civic_counter_proposal&&(s.submitted_by===null||s.submitted_at===null))throw new Error('Civic counter-proposal requires submitter and recorded submission timestamp.');return s;
}
export function planImportRequest(files,settings){
 validatePlanImportSettings(settings);if(!files.planImportSource)throw new Error('Select an external plan source.');validateSourceFile(files.planImportSource);
 const text=exactSourceText(files.planImportSource);if(new TextEncoder().encode(text).length>3*1024*1024)throw new Error('Plan import exceeds 3 MiB.');
 const context_json=files.auditContextText?exactContextText(files.auditContextText):null;
 const request={operation:'import-plan-interchange',source_text:text,context_json,settings};
 if(settings.format==='geojson-polygons'){if(!files.planImportCentroids)throw new Error('Select explicit tract centroids.');validateSourceFile(files.planImportCentroids);request.centroids_json=exactSourceText(files.planImportCentroids);if(new TextEncoder().encode(request.centroids_json).length>3*1024*1024)throw new Error('Centroid source exceeds 3 MiB.');}
 return request;
}
export function improvementRequest(files,settings){
  validateImprovementSettings(settings);if(!settings.generated_at_utc.trim())throw new Error('Supply a recorded creation timestamp.');
  if(!files.auditPlanText||!files.auditContextText)throw new Error('Select both RPLAN and RCTX files.');
  return {operation:'improve-rplan-text',document_json:exactSourceText(files.auditPlanText),context_json:exactContextText(files.auditContextText),settings};
}

export function validateSmcSettings(value){
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join(',')!=='base_seed,k,particles,population_tolerance,resample_threshold'||!Number.isInteger(value.k)||value.k<1||value.k>500||!Number.isInteger(value.particles)||value.particles<1||value.particles>1000)throw new Error('SMC requires 1–500 districts and 1–1,000 particles.');
  for(const field of ['population_tolerance','resample_threshold'])if(!Number.isFinite(value[field])||value[field]<0||value[field]>1)throw new Error('SMC tolerance and resampling threshold must be fractions from 0 to 1.');
  if(typeof value.base_seed!=='string'||! /^(0|[1-9][0-9]{0,19})$/.test(value.base_seed)||BigInt(value.base_seed)>18446744073709551615n)throw new Error('SMC seed must be a canonical unsigned 64-bit decimal integer.');return value;
}
export function smcRequest(files,settings){if(!files.auditContextText)throw new Error('Select an RCTX file.');return {operation:'sample-smc-context',context_json:exactContextText(files.auditContextText),settings:validateSmcSettings(settings)};}

export function validateSmcSelection(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join(',')!=='chamber,created_at,label,particle_index'||!Number.isInteger(value.particle_index)||value.particle_index<0||value.particle_index>=1000)throw new Error('Select a particle index from 0 through 999.');
 for(const key of ['label','chamber','created_at'])if(typeof value[key]!=='string'||new TextEncoder().encode(value[key]).length>200)throw new Error('Invalid SMC plan metadata.');return value;
}
export function smcSelectionRequest(files,settings){
 validateSmcSelection(settings);for(const key of ['label','chamber','created_at'])if(!settings[key].trim())throw new Error('Supply all selected-plan metadata.');
 if(!files.smcEnsemble||!files.auditContextText)throw new Error('Select both the ensemble NDJSON and its RCTX.');validateSourceFile(files.smcEnsemble);
 const bytes=Uint8Array.from(atob(files.smcEnsemble.base64),c=>c.charCodeAt(0));
 return {operation:'select-smc-plan',context_json:exactContextText(files.auditContextText),ensemble_ndjson:new TextDecoder('utf-8',{fatal:true}).decode(bytes),settings};
}
export function validateParetoSettings(v){
 if(!v||typeof v!=='object'||Array.isArray(v)||Object.keys(v).sort().join(',')!=='base_seed,democratic_shares,generations,k,minority_shares,population,tolerance'||!Number.isInteger(v.k)||v.k<1||v.k>500||!Number.isInteger(v.population)||v.population<2||v.population>1000||!Number.isInteger(v.generations)||v.generations<0||v.generations>1000||!Number.isFinite(v.tolerance)||v.tolerance<0||v.tolerance>1)throw new Error('Invalid Pareto district, population, generation or tolerance settings.');
 if(typeof v.base_seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(v.base_seed)||BigInt(v.base_seed)>18446744073709551615n)throw new Error('Supply a canonical unsigned 64-bit Pareto seed.');
 for(const key of ['democratic_shares','minority_shares'])if(v[key]!==null&&(!Array.isArray(v[key])||v[key].length>10000||v[key].some(x=>!Number.isFinite(x)||x<0||x>1)))throw new Error('Optional shares must be arrays in exact context unit order, with values from 0 through 1.');return v;
}
export function paretoRequest(files,settings){if(!files.auditContextText)throw new Error('Select an RCTX file.');return {operation:'sample-pareto-context',context_json:exactContextText(files.auditContextText),settings:validateParetoSettings(settings)};}
export function validateParetoSelection(v){
 if(!v||typeof v!=='object'||Array.isArray(v)||Object.keys(v).sort().join(',')!=='frontier_index,generated_at,label'||!Number.isInteger(v.frontier_index)||v.frontier_index<0||v.frontier_index>=1000)throw new Error('Select a frontier index from 0 through 999.');
 for(const key of ['label','generated_at'])if(typeof v[key]!=='string'||new TextEncoder().encode(v[key]).length>200)throw new Error('Invalid Pareto selection metadata.');return v;
}
export function paretoSelectionRequest(files,settings,selection){
 validateParetoSelection(selection);if(!selection.label.trim()||!selection.generated_at.trim())throw new Error('Supply selection label and timestamp.');
 if(!files.paretoFrontier||!files.auditContextText)throw new Error('Select frontier NDJSON and original RCTX.');validateSourceFile(files.paretoFrontier);
 return {operation:'select-pareto-plan',context_json:exactContextText(files.auditContextText),frontier_ndjson:new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(files.paretoFrontier.base64),c=>c.charCodeAt(0))),settings:validateParetoSettings(settings),selection};
}

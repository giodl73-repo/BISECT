import {validateSourceFile,encodeBytes} from './package-files.js';
import {inspectJsonText} from './project.js';
const TYPES={
 'bisect-plan-analysis-package-v1':['summary.json','demographic.json','urban.json','political.json','proportionality.json','partisan.json','partisan-metrics.json','compactness.json','contiguity.json','splits.json'],
 'bisect-plan-splits-v1':['splits.json'],
 'bisect-explicit-plan-splits-v1':['splits.json'],
 'bisect-explicit-plan-splits-v2':['splits.json'],
 'bisect-plan-contiguity-v1':['contiguity.json'],
 'bisect-plan-statistics-v1':['summary.json','demographic.json','urban.json','political.json'],
 'bisect-plan-election-analysis-v1':['partisan.json','partisan-metrics.json','political.json','proportionality.json'],
 'bisect-plan-compactness-v1':['compactness.json']
};
const allowed=new Set(Object.values(TYPES).flat());
const bytes=s=>Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0));
const text=s=>new TextDecoder('utf-8',{fatal:true}).decode(bytes(s));
const sha=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),x=>x.toString(16).padStart(2,'0')).join('');
export const canCollectAnalysis=r=>Boolean(TYPES[r?.schema_version]);
export function validateAnalysisCollection(v){
 if(!v||Object.keys(v).sort().join()!=='files,plan,plan_source_sha256'||!v.files||Array.isArray(v.files)||!/^[a-f0-9]{64}$/.test(v.plan_source_sha256))throw new Error('Invalid retained analysis collection.');
 validateSourceFile(v.plan);inspectJsonText(text(v.plan));
 let total=bytes(v.plan).length;
 if(!Object.keys(v.files).length||Object.keys(v.files).length>allowed.size)throw new Error('Select bounded native analysis files.');
 for(const [name,source]of Object.entries(v.files)){if(!allowed.has(name))throw new Error('Unknown retained analysis filename.');validateSourceFile(source,{maxBytes:8*1024*1024});total+=bytes(source).length;inspectJsonText(text(source));}
 if(total>12*1024*1024)throw new Error('Plan and retained analyses exceed 12 MiB.');return v;
}
export async function retainAnalysis(collection,plan,result){
 if(!canCollectAnalysis(result))throw new Error('Run a supported native plan analysis first.');
 validateSourceFile(plan);const hash=await sha(bytes(plan));
 if(hash!==result.plan_source_sha256)throw new Error('Analysis does not match the selected exact plan source.');
 if(collection){validateAnalysisCollection(collection);if(collection.plan.base64!==plan.base64||collection.plan_source_sha256!==hash)throw new Error('Clear retained analyses before switching plans.');}
 const files={...(collection?.files||{})};
 const generated=['bisect-plan-compactness-v1','bisect-plan-contiguity-v1','bisect-plan-splits-v1','bisect-explicit-plan-splits-v1','bisect-explicit-plan-splits-v2'].includes(result.schema_version)?{[result.schema_version==='bisect-plan-compactness-v1'?'compactness.json':['bisect-plan-splits-v1','bisect-explicit-plan-splits-v1','bisect-explicit-plan-splits-v2'].includes(result.schema_version)?'splits.json':'contiguity.json']:result.report_json}:result.files;
 for(const name of TYPES[result.schema_version])if(typeof generated?.[name]==='string'){
  const b=new TextEncoder().encode(generated[name]);const expected=result.schema_version==='bisect-plan-analysis-package-v1'?result.file_sha256?.[name]:['compactness.json','contiguity.json','splits.json'].includes(name)?result.report_sha256:result.file_sha256?.[name];
  if(await sha(b)!==expected)throw new Error('Native analysis file hash mismatch.');
  files[name]={name,base64:encodeBytes(b)};
 }
 return validateAnalysisCollection({plan:{...plan},plan_source_sha256:hash,files});
}
export function analysisCommissionInputs(collection,existing){
 validateAnalysisCollection(collection);
 const files={...(existing?.files||{})};let count=0;
 // Only native commission-schema files are mapped into report sections.
 for(const name of ['summary.json','compactness.json','contiguity.json','partisan.json','splits.json'])if(collection.files[name]){
  const path='analysis/'+name,source=collection.files[name];
  if(files[path]&&files[path].base64!==source.base64)throw new Error('Existing report analysis differs; clear or replace its report folder first.');
  files[path]={...source};count++;
 }
 if(!count)throw new Error('Retained files have no compatible required report section.');
 return {files,generated_at:existing?.generated_at||'',...(existing?.opaque_manifest?{opaque_manifest:true}:{})};
}

export async function planManifestCommissionInputs(result,plan,context,collection,existing){
 if(result?.schema_version!=='bisect-plan-report-manifest-v1')throw new Error('Prepare a fresh native report manifest first.');
 validateSourceFile(plan);validateSourceFile(context);
 if(await sha(bytes(plan))!==result.plan_source_sha256||await sha(bytes(context))!==result.context_source_sha256)throw new Error('Manifest sources no longer match the selected plan/context.');
 const manifestBytes=new TextEncoder().encode(result.manifest_json);if(await sha(manifestBytes)!==result.manifest_sha256)throw new Error('Native manifest output hash mismatch.');inspectJsonText(result.manifest_json);
 if(collection&&collection.plan.base64!==plan.base64)throw new Error('Retained analyses belong to a different exact plan.');
 const inputs=collection?analysisCommissionInputs(collection,existing):{files:{...(existing?.files||{})},generated_at:existing?.generated_at||''};
 for(const [path,source]of [['manifest.json',{name:'manifest.json',base64:encodeBytes(manifestBytes)}],['plan.rplan',plan],['context.rctx',context]]){
  if(inputs.files[path]&&inputs.files[path].base64!==source.base64)throw new Error('Existing report package differs; clear its sources before preparing another plan.');
  inputs.files[path]={...source};
 }
 return inputs;
}

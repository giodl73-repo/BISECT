import {electionAnalysisCsv} from './election-source-format.js';
import {exactSourceText,encodeBytes,validateSourceFile} from './package-files.js';
import {compactnessRequest} from './compactness-files.js';
import {validateStatisticsInputs} from './statistics-files.js';
import {validateElectionAnalysisSettings} from './election-analysis-settings.js';
import {storedZipBytes} from './reproducibility-files.js';
import {parseGeometryJson,inspectJsonText} from './project.js';
const analysisNames=['summary.json','demographic.json','urban.json','political.json','proportionality.json','partisan.json','partisan-metrics.json','contiguity.json','compactness.json','splits.json'];
const allowed=new Set([...analysisNames,'plan.rplan','context.rctx','manifest.json','analysis-package.json']);
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');
export function analysisPackageRequest(files,statistics,compactness,electionSettings){
 if(!files.auditPlanText)throw new Error('Select an exact RPLAN.');
 validateStatisticsInputs(statistics);validateElectionAnalysisSettings(electionSettings);
 const primary=files.electionAnalysisSource,other=statistics.election;
 if(primary&&other&&primary.base64!==other.base64)throw new Error('The two election CSV selections differ. Remove one or select the same source for both.');
 const geometry=compactness?.geometry?compactnessRequest(files,compactness):null;
 if(geometry){delete geometry.operation;delete geometry.document_json;}
 const optional=source=>source?exactSourceText(source):null;
 const input={document_json:exactSourceText(files.auditPlanText),context_json:files.auditContextText?new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(validateSourceFile(files.auditContextText,{maxBytes:8*1024*1024}).base64),c=>c.charCodeAt(0))):null,geography_json:optional(files.splitsGeography),demographic_csv:optional(statistics.demographic),urban_csv:optional(statistics.urban),election_csv:electionAnalysisCsv(files,other),tolerance_percent:statistics.tolerance_percent,election_settings:electionSettings,compactness:geometry};
 const total=[input.document_json,input.context_json,input.geography_json,input.demographic_csv,input.urban_csv,input.election_csv,geometry?.unit_geometry_json,geometry?.population_centroids_csv].filter(v=>typeof v==='string').reduce((n,t)=>n+new TextEncoder().encode(t).length,0);const cap=(geometry&&/"geometry_profile"\s*:\s*"(?:source-exact-large|display-topology)"/.test(geometry.unit_geometry_json)&&['source-exact-large','display-topology'].includes(parseGeometryJson(geometry.unit_geometry_json).geometry_profile)?64:25)*1024*1024;if(total>cap)throw new Error('Analysis package source bytes exceed the selected 25/64 MiB profile.');
 return {operation:'analyze-plan-package',input};
}
export async function analysisPackageFiles(result){
 if(result?.schema_version!=='bisect-plan-analysis-package-v1'||!result.files||!result.file_sha256||!['computed','partial','partial-failure'].includes(result.status))throw new Error('Run a native analysis package first.');
 const entries=Object.entries(result.files);if(!entries.length||entries.length>allowed.size)throw new Error('Invalid analysis package files.');let total=0;
 for(const [name,text]of entries){if(!allowed.has(name)||typeof text!=='string')throw new Error('Unknown analysis package file.');const bytes=new TextEncoder().encode(text);total+=bytes.length;if(total>25*1024*1024)throw new Error('Analysis package exceeds 25 MiB.');if(await sha(bytes)!==result.file_sha256[name])throw new Error('Analysis package file hash mismatch.');inspectJsonText(text);}
 if(typeof result.files['plan.rplan']!=='string'||await sha(new TextEncoder().encode(result.files['plan.rplan']))!==result.plan_source_sha256)throw new Error('Analysis package plan hash mismatch.');return entries;
}
export async function analysisPackageZip(result){return storedZipBytes((await analysisPackageFiles(result)).map(([name,text])=>[name,new TextEncoder().encode(text)]));}
export async function analysisPackageCommissionInputs(result,plan,context){
 if(!result?.report_ready)throw new Error('The analysis package is missing required report files. Supply sources and Run again.');
 const entries=await analysisPackageFiles(result);if(!plan||exactSourceText(plan)!==result.files['plan.rplan'])throw new Error('Selected plan no longer matches the analysis package.');
 if(result.files['context.rctx']){validateSourceFile(context,{maxBytes:8*1024*1024});const text=new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(context.base64),c=>c.charCodeAt(0)));if(text!==result.files['context.rctx'])throw new Error('Selected context no longer matches the analysis package.');}
 const files={};for(const [name,text]of entries){if(!['summary.json','contiguity.json','compactness.json','partisan.json','splits.json','manifest.json','plan.rplan','context.rctx'].includes(name))continue;const path=analysisNames.includes(name)?'analysis/'+name:name;files[path]={name,base64:encodeBytes(new TextEncoder().encode(text))};}return {files,generated_at:''};
}

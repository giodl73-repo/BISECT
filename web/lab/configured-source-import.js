import {verifyContextEngineOrigin} from './context-engine-files.js';
import {parseConfiguredSourceZip} from './fletch-cache-backup-zip.js';
import {parseSafeJson,parsePreparedGraphJson,validateWorkbenchArchive,WORKBENCH_ARCHIVE_SCHEMA,readValidatedFile} from './project.js';
import {encodeBytes} from './package-files.js';
import {validateConfiguredRecipe} from './configured-run-recipe.js';
import {validateConfiguredRuntimeArtifacts} from './configured-runtime-artifacts.js';
const text=bytes=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
const digest=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
const source=(name,bytes)=>({name,base64:encodeBytes(bytes)});
const exact=(value,keys)=>value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===keys.length&&keys.every(k=>Object.hasOwn(value,k));
export async function importConfiguredSourceBundle(bytes){
 const members=parseConfiguredSourceZip(bytes),get=name=>{const entry=members.get(name);if(!entry)throw new Error('Source bundle missing '+name);return entry.data;};
 const manifest=parseSafeJson(text(get('bundle.json')));
 if(!exact(manifest,['schema_version','files','scope'])||manifest.schema_version!=='bisect-configured-source-bundle-v1'||!manifest.files||typeof manifest.scope!=='string'||Object.keys(manifest.files).length!==members.size-1||Object.hasOwn(manifest.files,'bundle.json'))throw new Error('Invalid source bundle manifest.');
 let total=0;
 for(const [name,entry]of Object.entries(manifest.files)){const data=get(name);total+=data.length;if(!exact(entry,['bytes','sha256'])||entry.bytes!==data.length||!/^[a-f0-9]{64}$/.test(entry.sha256||'')||await digest(data)!==entry.sha256)throw new Error('Source bundle byte hash/length mismatch: '+name);}
 if(total>25*1024*1024)throw new Error('Source bundle contents exceed 25 MiB.');
 const plan=parseSafeJson(text(get('plan.rplan'))),context=parseSafeJson(text(get('context.rctx'))),graph=parsePreparedGraphJson(text(get('sources/graph.json')));
 const ids=graph.geoids;if(!Array.isArray(ids)||!Array.isArray(plan.plan?.units?.unit_ids)||JSON.stringify(ids)!==JSON.stringify(plan.plan.units.unit_ids)||JSON.stringify(ids)!==JSON.stringify(context.units?.unit_ids)||!Number.isSafeInteger(plan.plan?.k)||plan.plan.k<1)throw new Error('Source graph/native plan/context units disagree.');
 if(plan.plan.units.state!==graph.state||context.units.state!==graph.state||String(plan.plan.units.year)!==graph.year||String(context.units.year)!==graph.year)throw new Error('Source graph/native plan/context scope disagrees.');
 const files={auditPlanText:source('plan.rplan',get('plan.rplan')),auditContextText:source('context.rctx',get('context.rctx')),configSource:source(plan.provenance?.producer?.algorithm_configuration?.source_name||'config.yml',get('sources/config.yml')),configSettings:{profile:plan.provenance?.producer?.algorithm_configuration?.profile||'open'},configRunInputs:{graph:source('graph.json',get('sources/graph.json'))},configRunSettings:{chamber:plan.metadata?.chamber,districts:plan.plan.k,...(plan.provenance?.producer?.engine_options?.vra_absent_file_fallback===true?{vra_absent_file_fallback:true}:{})}};
 // Older bundles have no explicit settings member. Recover the observable
 // native settings; omitted/default field spelling cannot be reconstructed.
 const producer=plan.provenance?.producer;
 if(Object.hasOwn(producer?.engine_options??{},'compact_geometry'))files.configRunSettings.compact_geometry=producer.engine_options.compact_geometry;
 if(producer?.engine_options?.capture_rounds===true)files.configRunSettings.capture_rounds=true;
 if(producer?.algorithm_configuration?.execution_attempt!=null)files.configRunSettings.native_build=true;
 if(producer?.electoral_settings){files.configRunSettings.seats_per_district=producer.electoral_settings.seats_per_district;if(producer.electoral_settings.total_seats!=null)files.configRunSettings.total_seats=producer.electoral_settings.total_seats;}
 if(members.has('sources/demographics.json'))files.configRunInputs.demographics=source('demographics.json',get('sources/demographics.json'));
 if(members.has('sources/run-settings.json'))files.configRunSettings=validateConfiguredRecipe(parseSafeJson(text(get('sources/run-settings.json'))),plan).settings;
 if(members.has('runtime/engine-artifacts.json'))files.configuredRuntimeArtifacts=validateConfiguredRuntimeArtifacts(parseSafeJson(text(get('runtime/engine-artifacts.json'))));
 if(members.has('sources/coi.json'))files.configRunInputs.coi=source(plan.provenance?.producer?.coi_input?.source_name||'coi.json',get('sources/coi.json'));
 const receiptPresent=members.has('sources/native-demographic-adoption.json');if(receiptPresent!==members.has('sources/native-demographics.csv'))throw new Error('Native CSV and adoption receipt must be paired.');
 if(receiptPresent){
  const receipt=parseSafeJson(text(get('sources/native-demographic-adoption.json'))),input=parseSafeJson(text(get('sources/demographics.json')));
  if(receipt.schema_version!=='bisect-native-demographic-adoption-v1'||receipt.source_sha256!==await digest(get('sources/native-demographics.csv'))||receipt.graph_source_sha256!==await digest(get('sources/graph.json'))||receipt.demographics_source_sha256!==await digest(get('sources/demographics.json'))||receipt.state!==graph.state||receipt.year!==graph.year||receipt.basis!==input.basis)throw new Error('Native source bundle adoption binding disagrees.');
  files.demographicSource=source(receipt.source_name,get('sources/native-demographics.csv'));files.nativeDemographicReceipt=source('native-demographic-adoption.json',get('sources/native-demographic-adoption.json'));files.demographicSettings={basis:input.basis,source_label:input.source_label};
 }
 const contextOriginPresent=members.has('sources/context-engine-origin.json');if(contextOriginPresent!==members.has('sources/original-context.rctx'))throw new Error('Original RCTX source and engine origin must be paired.');
 if(members.has('sources/context-attributes.json')&&!contextOriginPresent)throw new Error('Attributes require the paired original context and origin.');
 if(contextOriginPresent){const origin=parseSafeJson(text(get('sources/context-engine-origin.json'))),inputs={source:source(origin.source_name,get('sources/original-context.rctx')),physical_edge_weights_metres:true,...(members.has('sources/context-attributes.json')?{attributes:source(origin.attributes_source?.source_name,get('sources/context-attributes.json'))}:{})};await verifyContextEngineOrigin(inputs,origin,get('sources/graph.json'));files.contextEngineInputs=inputs;files.contextEngineOrigin=origin;}
 return validateWorkbenchArchive({schema_version:WORKBENCH_ARCHIVE_SCHEMA,name:'Imported plan and sources',saved_at_utc:new Date().toISOString(),operation:'audit-rplan-text',constraints:['plan-shape','population','contiguity'],files,result:null,lastOperation:null});
}
export async function readConfiguredSourceBundle(file,{signal}={}){
 if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>32*1024*1024)throw new Error('Select a plan-and-sources ZIP of at most 32 MiB.');
 return readValidatedFile(file,new URL('./configured-source-import-worker.js',import.meta.url),{signal});
}

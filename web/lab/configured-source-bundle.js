import {verifyContextEngineOrigin,contextSourceBytes} from './context-engine-files.js';
import {validateSourceFile} from './package-files.js';
import {parseSafeJson} from './project.js';
import {storedZipBytes} from './reproducibility-files.js';
import {validateConfiguredRecipe} from './configured-run-recipe.js';
import {validateConfiguredRuntimeArtifacts} from './configured-runtime-artifacts.js';
const encode=text=>new TextEncoder().encode(text);
const decode=source=>{validateSourceFile(source);return Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));};
const parsed=source=>parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(decode(source)));
const hash=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
const canonical=v=>JSON.stringify(v,(_key,value)=>value&&typeof value==='object'&&!Array.isArray(value)?Object.fromEntries(Object.keys(value).sort().map(k=>[k,value[k]])):value);

export async function configuredSourceBundle(result,files){
 if(result?.schema_version!=='bisect-configured-run-v1'||typeof result.practitioner_export?.document_json!=='string'||typeof result.practitioner_export?.context_json!=='string')throw new Error('Prepare native RPLAN/RCTX from a fresh configured run first.');
 const request=result.engine_request,configuration=result.configuration;
 if(!request?.graph||!configuration?.yaml_text||!files.configRunInputs?.graph||!files.configSource)throw new Error('Configured run sources are incomplete.');
 if(canonical(parsed(files.configRunInputs.graph))!==canonical(request.graph))throw new Error('Selected graph differs from the exported run.');
 if(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(decode(files.configSource))!==configuration.yaml_text)throw new Error('Selected YAML differs from the exported run.');
 const entries=[['plan.rplan',encode(result.practitioner_export.document_json)],['context.rctx',encode(result.practitioner_export.context_json)],['sources/config.yml',decode(files.configSource)],['sources/graph.json',decode(files.configRunInputs.graph)]];
 const document=parseSafeJson(result.practitioner_export.document_json);
 const recipe=validateConfiguredRecipe({schema_version:'bisect-configured-run-recipe-v1',settings:files.configRunSettings,execution_attempt:result.build_report?.accepted_attempt??null},document);
 entries.push(['sources/run-settings.json',encode(JSON.stringify(recipe))]);
 if(files.configuredRuntimeArtifacts){const artifacts=validateConfiguredRuntimeArtifacts(files.configuredRuntimeArtifacts);if(artifacts.export===null)throw new Error('Prepare a fresh native export with measured engine identity.');entries.push(['runtime/engine-artifacts.json',encode(JSON.stringify(artifacts))]);}
 const context=parseSafeJson(result.practitioner_export.context_json);
 if(canonical(document.plan?.assignment)!==canonical(request.graph.geoids.map(id=>result.result.assignments[id]-1)))throw new Error('Exported plan assignments differ from the completed run.');
 if(request.demographics){
  const nativeHash=context.source_hashes?.['bisect.demographic-input'];
  if(!files.configRunInputs.demographics||canonical(parsed(files.configRunInputs.demographics))!==canonical(request.demographics)||!/^sha256:[a-f0-9]{64}$/.test(nativeHash||'')||document.provenance?.source_hashes?.['bisect.demographic-input']!==nativeHash)throw new Error('Demographic inputs differ from the exported native plan.');
  entries.push(['sources/demographics.json',decode(files.configRunInputs.demographics)]);
 }
 if(request.coi){
  const selected=files.configRunInputs.coi;
  if(!selected||selected.name!==request.coi.source_name||new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(decode(selected))!==request.coi.weights_json)throw new Error('COI source differs from the exported run.');
  entries.push(['sources/coi.json',decode(selected)]);
 }
 if(files.nativeDemographicReceipt&&request.options.vra_absent_file_fallback!==true){
  if(!request.demographics||!files.demographicSource)throw new Error('Native demographic source receipt has no matching input.');
  const receipt=parsed(files.nativeDemographicReceipt);
  if(receipt.schema_version!=='bisect-native-demographic-adoption-v1'||receipt.source_name!==files.demographicSource.name||receipt.source_sha256!==await hash(decode(files.demographicSource))||receipt.graph_source_sha256!==await hash(decode(files.configRunInputs.graph))||receipt.demographics_source_sha256!==await hash(decode(files.configRunInputs.demographics))||receipt.state!==request.graph.state||receipt.year!==request.graph.year||receipt.basis!==request.demographics.basis)throw new Error('Native demographic receipt or source binding changed. Preview and adopt again.');
  entries.push(['sources/native-demographics.csv',decode(files.demographicSource)],['sources/native-demographic-adoption.json',decode(files.nativeDemographicReceipt)]);
 }
 if(files.contextEngineOrigin){await verifyContextEngineOrigin(files.contextEngineInputs,files.contextEngineOrigin,decode(files.configRunInputs.graph));entries.push(['sources/original-context.rctx',contextSourceBytes(files.contextEngineInputs.source)],['sources/context-engine-origin.json',encode(JSON.stringify(files.contextEngineOrigin))]);if(files.contextEngineInputs.attributes)entries.push(['sources/context-attributes.json',contextSourceBytes(files.contextEngineInputs.attributes)]);}
 const manifest={schema_version:'bisect-configured-source-bundle-v1',files:Object.fromEntries(await Promise.all(entries.map(async([name,bytes])=>[name,{bytes:bytes.length,sha256:await hash(bytes)}]))),scope:'Exact native RPLAN/RCTX plus selected run sources. Source hashes establish byte integrity; receipt diagnostics and source authenticity are recorded claims. No optimality or legal certification.'};
 entries.push(['bundle.json',encode(JSON.stringify(manifest,null,2))]);
 return {bytes:storedZipBytes(entries),manifest};
}

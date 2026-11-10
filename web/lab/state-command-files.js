import {renderStateIdentityPreview} from './native-state-identity-preview.js';
import {preparedEngineRequest,validatePreparedEngineInputs} from './prepared-engine-files.js';
import {parseSafeJson} from './project.js';
import {NATIVE_STATE_PREPARED_REFUSED_FIELDS} from './native-state-boundary.js';
export const RUN_COMMAND_DEFAULTS={arguments_json:'[]'};
export const STATES_COMMAND_DEFAULTS={arguments_json:'["--year","2020","--version","v1","--output-dir","outputs/v1"]'};
export const STATE_COMMAND_DEFAULTS={arguments_json:'["--state", "RI", "--metis-engine", "metis-core"]'};
export function stateCommandArguments(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).join()!=='arguments_json'||typeof value.arguments_json!=='string'||new TextEncoder().encode(value.arguments_json).length>1024*1024)throw new Error('Supply a State argument JSON array of at most 1 MiB.');
 const args=parseSafeJson(value.arguments_json);if(!Array.isArray(args)||args.length>256||args.some(v=>typeof v!=='string'||v.includes('\0')||new TextEncoder().encode(v).length>4096)||args.reduce((n,v)=>n+new TextEncoder().encode(v).length,0)>64*1024)throw new Error('Use up to 256 string arguments, 4096 UTF-8 bytes each and 64 KiB total, without NUL bytes.');return args;
}
export function validateStateCommandInputs(value){stateCommandArguments(value);return value;}
export function stateCommandRequest(value,{bulk=false,run=false}={}){return {operation:run?'inspect-native-run-command':bulk?'inspect-native-states-command':'inspect-native-state-command',arguments:stateCommandArguments(value)};}
export function statePreparedFlagRefusals(value){
 if(value?.schema_version!=='bisect-native-state-command-v1'||value.status!=='parsed'||!Array.isArray(value.fields))return [];
 return value.fields.filter(f=>f.source==='command-line'&&NATIVE_STATE_PREPARED_REFUSED_FIELDS.includes(f.id)).map(f=>'--'+f.long);
}
export function renderStateCommand(target,value){
 const selectedSource=value?.single_source_selection;
 const verifiedAdjacency=value?.native_state?.compilation?.adjacency_override?.path_accessed===false;
 if(value?.native_state||value?.native_states||value?.native_run)value=(value.native_state??value.native_states??value.native_run).compilation.native_command;
 const prepared=value?.schema_version==='bisect-native-state-run-input-v1'?value:null;if(prepared)value=prepared.native_command;
 target.replaceChildren();const run=value?.schema_version==='bisect-native-run-command-v1',bulk=run||value?.schema_version==='bisect-native-states-command-v1';target.hidden=!bulk&&value?.schema_version!=='bisect-native-state-command-v1';if(target.hidden)return;
 const title=document.createElement('h3');title.textContent=run?'Native Run command review':bulk?'Native States bulk command review':'Native State command review';target.append(title);const note=document.createElement('p');note.textContent=`${value.target_pointer_width}-bit count fields. Inspection parses flags and converts the native compositor without running an engine or reading paths.`;target.append(note);
 if(value.status!=='parsed'){const message=document.createElement('pre');message.textContent=value.message;target.append(message);return;}
 if(selectedSource?.schema_version==='bisect-native-selected-state-source-v1'&&Number.isSafeInteger(selectedSource.scope?.districts)){const p=document.createElement('p'),kind=selectedSource.district_count?.allocation_source,chamber=selectedSource.scope.chamber;p.textContent='Single State Run: '+chamber+' · '+selectedSource.scope.districts+' districts. Count source: '+({'explicit-native-flag':'explicit --districts','policy':'native policy','congressional-fallback':'native congressional allocation'}[kind]??'recorded native allocation')+'.';target.append(p);}
 const refused=statePreparedFlagRefusals(value).filter(flag=>!verifiedAdjacency||flag!=='--adjacency');if(refused.length){const p=document.createElement('p');p.textContent='Selected graph/template preparation refuses these explicit orchestration flags: '+refused.join(', ')+'. Use the separate source, output and workflow tools. Other flags still require matching selected sources and native policy validation.';target.append(p);}
 if(verifiedAdjacency){const p=document.createElement('p');p.textContent='This Run used the selected adjacency bytes and companions. The recorded --adjacency path was not opened.';target.append(p);}
 if(prepared?.prepared_review){const p=document.createElement('p'),o=prepared.prepared_review.engine_request.options;p.textContent=`Prepared run: ${o.districts} districts, ${o.balance_tolerance}% population tolerance, native population retries. No partition has run. Select Use compiled native request to continue.`;target.append(p);}
 if(!bulk&&!run)renderStateIdentityPreview(target,value.browser_identity_preview);
 const a=bulk?value.effective_configuration:value.effective_algorithm,summary=document.createElement('p');summary.textContent=run?`${value.years.join(', ')} · native Run configuration · configured seed budget ${a.seeds_per_split} · requested workers ${value.workers} · ${value.print_only?'print-only intent':'execution intent'}. This inspection never reads output paths or schedules states.`:bulk?`${value.year} · native ${run?'Run':'States'} configuration · configured seed budget ${a.seeds_per_split} · requested workers ${value.workers} · profile ${value.profile_filter?.allowed===false?'rejected':'passed'} · multiscale levels ${value.multiscale_levels?.allowed===false?'rejected':'passed'} · ${value.print_only?'print-only':'execution intent'}. This operation never reads output paths or schedules states.`:`${value.state} ${value.year} · ${value.resolution} · structure ${a.structure} · configured search ${a.search} · native budget ${a.seed_compositor_budget} · engine ${a.metis.engine} · seed ${a.metis.seed??'native default'} · profile filter ${value.profile_filter?.allowed===false?'rejected':'passed'}`;target.append(summary);
 if(run){const p=document.createElement('p');p.textContent='Selected years: '+value.years.join(', ')+'. Native parsed but unused flags: '+value.native_unused_fields.join(', ')+'. This tool never executes or deletes files. Browser explicit Run honors print-only intent.';target.append(p);}if(value.profile_filter?.allowed===false){const p=document.createElement('p');p.textContent=value.profile_filter.error;target.append(p);}
 const layers=document.createElement('pre');layers.textContent=JSON.stringify(a,null,2);target.append(layers);const native=document.createElement('details'),nativeTitle=document.createElement('summary'),nativeText=document.createElement('pre');nativeTitle.textContent='Exact native conversion details';nativeText.textContent=bulk?value.native_debug:value.native_debug.algorithm;native.append(nativeTitle,nativeText);target.append(native);const details=document.createElement('details'),caption=document.createElement('summary');caption.textContent=run?'All Run flags and defaults':bulk?'All States flags and defaults':'All State flags and defaults';details.append(caption);const table=document.createElement('table'),head=document.createElement('tr');for(const text of ['Flag','Values','Source']){const th=document.createElement('th');th.textContent=text;head.append(th);}table.append(head);for(const field of value.fields){const row=document.createElement('tr');for(const text of [field.long?'--'+field.long:field.id,field.values.length?field.values.map(v=>JSON.stringify(v)).join(' '):'unset',field.source]){const td=document.createElement('td');td.textContent=text;row.append(td);}table.append(row);}const wrap=document.createElement('div');wrap.append(table);details.append(wrap);target.append(details);
}

export function stateCommandPreparationRequest(command,prepared,{bulk=false,run=false}={}){
 validatePreparedEngineInputs(prepared);
 const request=preparedEngineRequest({...prepared,settings:{...prepared.settings,inspect_only:true}});
 return {operation:run?'prepare-native-run-run':bulk?'prepare-native-states-run':'prepare-native-state-run',input:{arguments:stateCommandArguments(command),prepared:{...request.input,inspect_only:true}}};
}
export async function preparedEngineFromStateCommand(command,prepared,result,{bulk=false,run=false}={}){
 const original=stateCommandPreparationRequest(command,prepared,{bulk,run});
 if(result?.schema_version!=='bisect-native-state-run-input-v1'||result.execution_performed!==false||typeof result.prepared_request_json!=='string')throw new Error('Prepare a fresh native State request first.');
 const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
 if(JSON.stringify(result.native_command?.arguments)!==JSON.stringify(original.input.arguments)||await hash(original.input.prepared.request_json)!==result.original_template_sha256||await hash(result.prepared_request_json)!==result.prepared_request_sha256)throw new Error('Native State source or compiled request changed. Prepare it again.');
 return validatePreparedEngineInputs({...prepared,source:{name:'native-state-compiled.json',text:result.prepared_request_json},settings:{...prepared.settings,inspect_only:true}});
}

const bulkCvdFlags={method:['--partition-mode','-m'],iters:['--cvd-iters'],metric:['--cvd-metric']};
function optionToken(token,aliases){for(const alias of aliases){if(token===alias)return {separate:true};if(token.startsWith(alias+'='))return {value:token.slice(alias.length+1)};if(alias==='-m'&&token.startsWith('-m')&&token.length>2)return {value:token.slice(2).replace(/^=/,'')};}return null;}
function bulkOption(args,aliases,fallback){let value=fallback;for(let i=0;i<args.length;i++){if(args[i]==='--')break;const match=optionToken(args[i],aliases);if(match)value=match.separate?args[i+1]??fallback:match.value;}return value;}
export function bulkCvdControls(command){
 const args=stateCommandArguments(command),rawMetric=bulkOption(args,bulkCvdFlags.metric,'graph-distance');
 return {selected:bulkOption(args,bulkCvdFlags.method,'edge-weighted')==='centroidal-voronoi',iterations:bulkOption(args,bulkCvdFlags.iters,'20'),metric:['geographic','Geographic'].includes(rawMetric)?'geographic':'graph-distance'};
}
export function setBulkCvdControl(command,control,value){
 const args=stateCommandArguments(command),aliases=bulkCvdFlags[control];if(!aliases)throw new Error('Unknown CVD control.');if(args.includes('--'))throw new Error('Edit the raw arguments to remove the -- terminator before using CVD controls.');
 if(control==='method')value='centroidal-voronoi';
 if(control==='iters'){if(!['string','number'].includes(typeof value))throw new Error('CVD iterations must be a number.');const count=Number(value);if(value===''||!Number.isSafeInteger(count)||count<0||count>10000)throw new Error('Choose 0–10,000 CVD iterations; the previous flag value is retained.');value=String(count);}
 if(control==='metric'&&!['graph-distance','geographic'].includes(value))throw new Error('Choose graph distance or geographic centroids.');
 const next=[];for(let i=0;i<args.length;i++){const match=optionToken(args[i],aliases);if(!match){next.push(args[i]);continue;}if(match.separate&&i+1<args.length&&(!args[i+1].startsWith('-')||/^-\d/.test(args[i+1])))i++;}
 next.push(aliases[0],value);return validateStateCommandInputs({arguments_json:JSON.stringify(next,null,2)});
}

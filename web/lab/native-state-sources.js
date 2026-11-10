import {nativeStateFollowup} from './native-state-followup.js';
import {RUN_COMMAND_DEFAULTS,STATES_COMMAND_DEFAULTS,STATE_COMMAND_DEFAULTS,validateStateCommandInputs} from './state-command-files.js';
import {STATE_GRAPH_DEFAULTS,validateStateGraphInputs} from './state-graph-files.js';
import {nativeEngineInputRequest,NATIVE_ADJACENCY_SETTINGS} from './native-adjacency-files.js';
import {censusPreparationRequest,tractPreparationRequest} from './tract-input-files.js';
import {validateSourceFile} from './package-files.js';
import {selectedStateWorkflowBinding} from './native-state-workflow.js';
export const NATIVE_STATE_SOURCE_DEFAULTS={enabled:false};
export function validateNativeStateSourceSettings(v){if(!v||Object.keys(v).some(k=>!['enabled','command_kind'].includes(k))||typeof v.enabled!=='boolean'||Object.hasOwn(v,'command_kind')&&!['state','states','run'].includes(v.command_kind))throw new Error('Invalid native flags source mode.');return v;}
export function nativeStatePreparationScope(request){
 const input=request?.input;
 if(request?.operation==='prepare-native-engine-input')return {state:input.state,year:input.year,resolution:({'tract':'tract','block-group':'block_group','block':'block'})[input.unit_kind??'tract']};
 return {state:input?.state,year:String(input?.pl94?.year),resolution:({'tracts':'tract','block-groups':'block_group','blocks':'block'})[input?.tiger?.kind]};
}
export function validateNativeStateSourceInputs(v){
 if(!v||Object.keys(v).sort().join()!=='auxiliary,preparation,settings'||!v.preparation||Object.keys(v.preparation).sort().join()!=='input,operation')throw new Error('Retain selected source preparation, auxiliary data and export settings.');
 validateStateGraphInputs({graph:null,auxiliary:v.auxiliary,settings:v.settings});const r=v.preparation,p=r.input;
 if(r.operation==='prepare-native-engine-input'){
  if(!p||Object.keys(p).some(k=>!['sources','state','year','unit_kind','require_geometry'].includes(k)))throw new Error('Invalid native State source preparation.');const {sources,...settings}=p;nativeEngineInputRequest(sources,settings);
 }else if(['prepare-census-engine-input','prepare-tract-engine-input'].includes(r.operation)){
  if(!p||Object.keys(p).some(k=>!['state','projection','tiger','pl94','geometry_profile'].includes(k)))throw new Error('Invalid Census State source preparation.');const files={tigerInputs:p.tiger,pl94Inputs:p.pl94,tractPreparationSettings:{state:p.state,projection:p.projection,...(p.geometry_profile?{geometry_profile:p.geometry_profile}:{})}};(r.operation==='prepare-census-engine-input'?censusPreparationRequest:tractPreparationRequest)(files);
 }else throw new Error('State source jobs support selected native binary or Census/tract preparation inputs.');
 const scope=nativeStatePreparationScope(r),custom=r.operation==='prepare-native-engine-input'&&['precinct','imported'].includes(p.unit_kind);if(!scope.resolution&&!custom||!['congressional','house','senate'].includes(v.settings.chamber)||!v.settings.created_at.trim())throw new Error('State source jobs require supported selected inputs and recorded native chamber export metadata.');return v;
}
export async function createNativeStateSourceJob(files,kind,{acquisition,selectedScope=false}={}){
 const preparation=kind==='native'?nativeEngineInputRequest(files.nativeAdjacencyInputs,files.nativeAdjacencySettings??NATIVE_ADJACENCY_SETTINGS):kind==='tract'?tractPreparationRequest(files):censusPreparationRequest(files);
 const kindCommand=files.nativeStateSourceSettings?.command_kind??'state',bulk=kindCommand!=='state',selected=files.stateGraphInputs??STATE_GRAPH_DEFAULTS,command=structuredClone(validateStateCommandInputs(kindCommand==='run'?(files.runCommandInputs??RUN_COMMAND_DEFAULTS):bulk?(files.statesCommandInputs??STATES_COMMAND_DEFAULTS):(files.stateCommandInputs??STATE_COMMAND_DEFAULTS)));
 if(bulk&&(selectedScope||nativeStatePreparationScope(preparation).resolution!=='tract')||(!selectedScope||bulk)&&selected.settings.chamber!=='congressional')throw new Error('National source queues and native Run/States require congressional scope. Use a single State Run for House/Senate sources.');
 const custom=kind==='native'&&['precinct','imported'].includes(preparation.input.unit_kind);
 if(custom&&(!selectedScope||bulk))throw new Error('Custom adjacency sources require a single State Run.');
 const source_inputs=structuredClone(validateNativeStateSourceInputs({preparation,auxiliary:selected.auxiliary,settings:selected.settings})),scope=nativeStatePreparationScope(preparation),recipe={command,source_inputs,...(bulk?{command_kind:kindCommand}:{}),...(selectedScope?{source_scope_policy:custom?'match-native-adjacency-flags':'match-native-flags'}:{})},followup=nativeStateFollowup(files);if(followup)recipe.followup=followup;
 if(!bulk){const binding=selectedStateWorkflowBinding(files,command);if(binding){if(binding.source_override&&(kind!=='native'||binding.source_override.preparation_sha256!==Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(preparation)))),b=>b.toString(16).padStart(2,'0')).join('')))throw new Error('Selected native adjacency or companions changed. Reapply the --adjacency handoff before queueing.');if(binding.name_effects){const actual=source_inputs.auxiliary?Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(source_inputs.auxiliary)))),b=>b.toString(16).padStart(2,'0')).join(''):null;if(actual!==binding.name_effects.auxiliary_sha256)throw new Error('Selected auxiliary source changed after native naming handoff; review and apply again before queueing.');}recipe.workflow_binding=binding;}}
 if(files.planGeometry&&kind==='native')validateSourceFile(files.planGeometry,{maxBytes:64*1024*1024});
 const source_sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(recipe)))),b=>b.toString(16).padStart(2,'0')).join('');
 return {native_state:recipe,scope:{state:scope.state,year:scope.year,chamber:selectedScope?selected.settings.chamber:'congressional',districts:null},request_sha256:null,source_sha256,...(files.planGeometry&&kind==='native'?{geometry:structuredClone(files.planGeometry)}:{}),...(acquisition?{acquisition:structuredClone(acquisition)}:{})};
}
export function nativeStateSourceDefaultCommand(files,kind){
 const r=kind==='native'?nativeEngineInputRequest(files.nativeAdjacencyInputs,files.nativeAdjacencySettings??NATIVE_ADJACENCY_SETTINGS):kind==='tract'?tractPreparationRequest(files):censusPreparationRequest(files),s=nativeStatePreparationScope(r);
 if(!s.resolution){if(kind!=='native'||!['precinct','imported'].includes(r.input.unit_kind)||['run','states'].includes(files.nativeStateSourceSettings?.command_kind))throw new Error('Choose supported single-State sources.');return {arguments_json:JSON.stringify(['--state',s.state,'--year',s.year,'--adjacency',r.input.sources.adjacency.name,'--chamber',(files.stateGraphInputs??STATE_GRAPH_DEFAULTS).settings.chamber,'--metis-engine','metis-core'])};}
 if(files.nativeStateSourceSettings?.command_kind==='run'){if(s.resolution!=='tract')throw new Error('Native Run source jobs require tracts.');return {arguments_json:JSON.stringify(['--year',s.year,'--states',s.state])};}
 if(files.nativeStateSourceSettings?.command_kind==='states'){if(s.resolution!=='tract')throw new Error('Native Run/States source jobs require tracts.');return {arguments_json:JSON.stringify(['--year',s.year,'--version','v1','--output-dir','outputs/v1'])};}
 return {arguments_json:JSON.stringify(['--state',s.state,'--year',s.year,'--resolution',s.resolution,'--chamber',(files.stateGraphInputs??STATE_GRAPH_DEFAULTS).settings.chamber,'--metis-engine','metis-core'])};
}

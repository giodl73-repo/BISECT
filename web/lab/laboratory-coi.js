import {validateCoiInput} from './coi-input.js';
export function laboratoryCoiRequest(config){
 if(!Object.hasOwn(config,'coi'))return {};
 if(['multiscale','multiscale-adaptive'].includes(config.search))throw new Error('Use the multiscale tract-initialization COI source for this search.');
 return {coi:validateCoiInput(config.coi)};
}
// Mirror the native computed map: sparse partisan/minority maps contain only
// boosted edges; an implicit unit reporting weight is not a computed map entry.
export function laboratoryCoiWeights(graph,config,baseWeight){
 if(!config.coi)return {weight:baseWeight,expected:null};
 const input=laboratoryCoiRequest(config).coi,values=JSON.parse(input.weights_json),ids=graph.geoids;
 const present=(u,v)=>config.weights==='unweighted'?false:config.weights==='partisan'?baseWeight(u,v,1)>1:config.weights==='vra-aligned'?config.demographics[graph.state].minority_fractions[ids[u]]>=.4&&config.demographics[graph.state].minority_fractions[ids[v]]>=.4:true;
 const weight=(u,v,length)=>{const original=baseWeight(u,v,length);return present(u,v)?original*Math.sqrt((values[ids[u]]??1)*(values[ids[v]]??1)):original;};
 let applied=0,changed=0,total=0;
 for(const [u,v,length]of graph.edges)if(present(u,v)){applied++;const adjusted=weight(u,v,length);total+=adjusted;if(!Number.isFinite(adjusted)||!Number.isFinite(total))throw new Error('COI adjusted weights exceed the native numeric range.');if(adjusted!==baseWeight(u,v,length))changed++;}
 const matched=ids.filter(id=>Object.hasOwn(values,id)).length,active=config.weights!=='unweighted';
 return {weight,expected:{schema_version:'bisect-coi-weighting-v1',source_name:input.source_name,supplied_units:Object.keys(values).length,matched_units:matched,defaulted_units:ids.length-matched,ignored_units:Object.keys(values).length-matched,applied_edges:applied,changed_edges:changed,status:!active?'no-effect-unweighted-native-empty-map':changed?'applied':'applied-no-edge-change',formula:'resolved_edge_weight * sqrt(unit_weight_u * unit_weight_v)',missing_unit_weight:1}};
}
export function verifyLaboratoryCoiCounts(state,expected){
 const e=state.metrics.coi_weighting_evidence;
 if(!expected){if(e!==undefined)throw new Error('Unrequested laboratory COI evidence.');return;}
 if(!e||Object.keys(e).length!==Object.keys(expected).length+2||Object.entries(expected).some(([key,value])=>e[key]!==value)||typeof e.source_sha256!=='string'||!/^[a-f0-9]{64}$/.test(e.source_sha256)||typeof e.scope!=='string'||e.scope.length>2000)throw new Error('Laboratory COI evidence disagrees with the source or native computed weight map.');
}
export async function verifyLaboratoryCoiSource(state,config){
 if(!config.coi)return;
 const input=laboratoryCoiRequest(config).coi,hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(input.weights_json))),b=>b.toString(16).padStart(2,'0')).join('');
 if(state.metrics.coi_weighting_evidence?.source_sha256!==hash)throw new Error('Laboratory COI source bytes changed.');
}

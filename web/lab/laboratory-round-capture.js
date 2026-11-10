import {parseSafeJson} from './project.js';
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
// Only call on a freshly captured or verified imported trace.
export function laboratorySnapshot(capture,sequence){
 const event=capture?.events?.[sequence];
 if(!Number.isInteger(sequence)||sequence<0||!event||event.sequence!==sequence)throw new Error('Select a captured snapshot.');
 const index=parseSafeJson(capture.unit_index_json),values=parseSafeJson(event.assignments_json),assignments=new Map();
 for(const [key,value]of Object.entries(values)){
  if(typeof index[key]!=='string'||!Number.isInteger(value)||value<1)throw new Error('Invalid captured snapshot assignment.');
  assignments.set(index[key],value);
 }
 return {event,assignments,unit_universe:capture.unit_universe??null};
}
export function laboratorySnapshotDistrict(snapshot,id){return snapshot.assignments.get(snapshot.unit_universe?.returned_unit_kind==='bg'?id.slice(0,11):id)??null;}
export function laboratorySnapshotGeoJSON(geometry,snapshot){
 if(geometry?.type!=='FeatureCollection'||!Array.isArray(geometry.features))throw new Error('Load state geometry first.');
 return {...geometry,bisect_snapshot:{sequence:snapshot.event.sequence,name:snapshot.event.name,complete_universe:snapshot.event.complete_universe,matches_returned_assignment:snapshot.event.matches_returned_assignment,...(snapshot.unit_universe?{unit_universe:snapshot.unit_universe}:{}),scope:'Native emitted snapshot; unassigned units are neutral. Final-plan metrics do not describe this snapshot.'},features:geometry.features.map(f=>({...f,properties:{...f.properties,district:laboratorySnapshotDistrict(snapshot,f.properties.geoid)}}))};
}
// Check retained capture bytes against the prepared universe and returned plan.
// This proves consistency only, never authenticity or a complete search history.
export async function verifyLaboratoryCapture(graph,state,assignments,config){
 const capture=state.metrics?.round_capture;
 if(config.capture_rounds!==true){if(capture!==undefined)throw new Error('Unrequested laboratory round capture.');return;}
 if(!capture||capture.schema_version!=='bisect-native-round-capture-v1'||!Array.isArray(capture.events)||capture.events.length>128||typeof capture.scope!=='string'||typeof capture.terminal_matches_returned_assignment!=='boolean'||capture.status!==(capture.events.length?'captured':'no-events-emitted'))throw new Error('Invalid laboratory round capture.');
 const multiscale=['multiscale','multiscale-adaptive'].includes(config.search),bg=multiscale&&config.multiscale.fine_level==='bg';
 if(multiscale){const u=capture.unit_universe,expected={unit_kind:'tract',stage:'multiscale-initialization',returned_unit_kind:bg?'bg':'tract',final_comparison:bg?'parent-tract-projection-to-every-returned-block-group':'exact-tract-labels'};if(!u||Object.keys(u).length!==4||Object.keys(expected).some(k=>u[k]!==expected[k]))throw new Error('Multiscale capture universe changed.');}
 else if(capture.unit_universe!==undefined)throw new Error('Unexpected capture universe.');
 const checked=async(text,hash)=>{if(typeof text!=='string'||new TextEncoder().encode(text).length>3*1024*1024||await digest(text)!==hash)throw new Error('Laboratory round source bytes changed.');return parseSafeJson(text);};
 const index=await checked(capture.unit_index_json,capture.unit_index_sha256);
 if(!index||typeof index!=='object'||Array.isArray(index)||Object.keys(index).length!==graph.geoids.length||graph.geoids.some((id,i)=>index[String(i)]!==id))throw new Error('Laboratory round unit order changed.');
 let entries=0,last=false;
 for(let i=0;i<capture.events.length;i++){
  const e=capture.events[i];if(!e||e.sequence!==i||typeof e.name!=='string'||!/^[A-Za-z0-9_]{1,80}$/.test(e.name)||typeof e.complete_universe!=='boolean'||typeof e.matches_returned_assignment!=='boolean')throw new Error('Invalid laboratory round event.');
  const values=await checked(e.assignments_json,e.assignments_sha256);
  if(!values||typeof values!=='object'||Array.isArray(values))throw new Error('Invalid laboratory round assignments.');
  const keys=Object.keys(values);entries+=keys.length;
  if(entries>2000000||e.units!==keys.length||e.regions!==new Set(Object.values(values)).size||e.complete_universe!==(keys.length===graph.geoids.length))throw new Error('Laboratory round coverage changed.');
  for(const key of keys)if(!/^(0|[1-9]\d*)$/.test(key)||!Object.hasOwn(index,key)||!Number.isInteger(values[key])||values[key]<1||values[key]>4294967295)throw new Error('Invalid laboratory round assignment.');
  const projected=new Map(keys.map(key=>[index[key],values[key]]));
  last=e.complete_universe&&(bg?config.multiscale.inputs[state.code??graph.state].graph.geoids.every(id=>projected.get(id.slice(0,11))===assignments[id]):keys.every(key=>assignments[index[key]]===values[key]));
  if(e.matches_returned_assignment!==last)throw new Error('Laboratory round final comparison changed.');
 }
 if(capture.terminal_matches_returned_assignment!==last)throw new Error('Laboratory round terminal comparison changed.');
}

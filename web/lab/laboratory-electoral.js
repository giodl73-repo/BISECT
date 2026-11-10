const record=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const u32=v=>Number.isSafeInteger(v)&&v>=0&&v<=4294967295&&!Object.is(v,-0);
export function validateLaboratoryElectoral(config,districts=config?.districts){
 if(!Object.hasOwn(config??{},'electoral'))return;
 const e=config.electoral;
 if(!record(e)||!Object.hasOwn(e,'seats_per_district')||Object.keys(e).some(k=>!['seats_per_district','total_seats'].includes(k))||!u32(e.seats_per_district)||Object.hasOwn(e,'total_seats')&&!u32(e.total_seats))throw new Error('Record unsigned 32-bit seats per district and an optional total seat count.');
 if(Number.isInteger(districts)&&!Object.hasOwn(e,'total_seats')&&e.seats_per_district*districts>4294967295)throw new Error('Derived total seats exceed the native WASM manifest range. Record a supported total explicitly.');
}
export function electoralRequest(config){
 validateLaboratoryElectoral(config);
 if(!Object.hasOwn(config,'electoral'))return {};
 const e=config.electoral;return {electoral:{seats_per_district:e.seats_per_district,...(Object.hasOwn(e,'total_seats')?{total_seats:e.total_seats}:{})}};
}
export function electoralConfigRecord(config){return Object.hasOwn(config,'electoral')?{electoral_settings:JSON.stringify(electoralRequest(config).electoral)}:{};}
const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function verifyLaboratoryElectoral(graph,state,config){
 const e=state.metrics?.electoral_evidence;
 if(!Object.hasOwn(config,'electoral')){if(e!==undefined)throw new Error('Unrequested laboratory electoral evidence.');return;}
 const k=state.metrics.district_count??state.metrics.districts;validateLaboratoryElectoral(config,k);
 const requested=electoralRequest(config).electoral,total=requested.total_seats??requested.seats_per_district*k,seats=Math.max(1,requested.seats_per_district);
 // The existing native multiscale path computes this metadata on the supplied
 // tract input, even when the final plan uses separately supplied block groups.
 const population=graph.population.reduce((a,b)=>a+b,0);
 const expected={schema_version:'bisect-electoral-settings-v1',requested_seats_per_district:requested.seats_per_district,requested_total_seats:requested.total_seats??null,seats_per_district:seats,total_seats:total,electoral_system:requested.seats_per_district<=1?'single_member':'multi_member_uniform',districts:k,uniform_total_consistent:total===seats*k,ideal_population_per_seat:population/Math.max(1,total),ideal_population_per_district:population/k,settings_sha256:'sha256:'+await hash(JSON.stringify(requested))};
 if(!record(e)||Object.keys(e).length!==Object.keys(expected).length+1||Object.keys(expected).some(key=>e[key]!==expected[key])||typeof e.scope!=='string'||e.scope.length>2000)throw new Error('Laboratory electoral evidence disagrees with requested settings or native population basis.');
 if(state.metrics.multiscale_evidence){const nested=state.metrics.multiscale_evidence.electoral_settings;if(!record(nested)||Object.keys(nested).length!==Object.keys(e).length||Object.keys(e).some(k=>nested[k]!==e[k]))throw new Error('Multiscale electoral evidence changed.');}
}

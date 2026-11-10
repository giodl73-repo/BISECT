import {countyPreGraph,countyPreLength} from './county-augmentation.js';
import {validateJsonTree,parseSafeJson} from './project.js';
import {validateNativeCharacterInputs,nativeCharacterRequest} from './native-character-files.js';
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
const hash=v=>typeof v==='string'&&/^[a-f0-9]{64}$/.test(v);
const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
const close=(a,b)=>Number.isFinite(a)&&Number.isFinite(b)&&Math.abs(a-b)<=Math.max(1e-9,Math.abs(b)*1e-10);
export const nativeNumberClass=v=>Number.isNaN(v)?'nan':v===Infinity?'positive-infinity':v===-Infinity?'negative-infinity':Object.is(v,-0)?'negative-zero':'finite';
export function nativeMetricMatches(actual,expected,kind){
 if(!['finite','negative-zero','nan','positive-infinity','negative-infinity'].includes(kind))return false;
 if(Number.isFinite(expected))return close(actual,expected)&&(expected===0?['finite','negative-zero'].includes(kind):kind==='finite');
 return actual===null&&kind===nativeNumberClass(expected);
}
export function nativeSourceScope(input,config,code){
 validateJsonTree(input);validateNativeCharacterInputs(input);const s=input.settings;
 if(!input.source||s.state!==code||s.year!==config.year||s.kind+'-character'!==config.weights)throw new Error('Original native CSV scope must match selected state, Census year and weights.');return s;
}
export function validateNativeCharacterSources(config){
 if(!Object.hasOwn(config,'native_character_sources'))return;
 const map=config.native_character_sources;if(!map||Array.isArray(map)||typeof map!=='object'||!Object.keys(map).length||Object.keys(map).some(code=>!config.states.includes(code)))throw new Error('Select original native CSVs only for selected states.');
 for(const [code,input]of Object.entries(map)){nativeSourceScope(input,config,code);if(config.characters?.[code]||config.character_preparations?.[code]||config.character_fallback_sources?.[code])throw new Error('Original native CSV conflicts with another character selection for this state.');}
}
export function nativeCharacterLabRequest(config,graph){const input=config.native_character_sources?.[graph.state];if(!input)return null;nativeSourceScope(input,config,graph.state);return nativeCharacterRequest(input).input;}
function decodeNumber(value,kind){
 if(kind==='finite'&&Number.isFinite(value)&&value>=0&&value<=1&&!Object.is(value,-0))return value;
 if(kind==='negative-zero'&&value===0)return -0;
 if(value===null&&kind==='nan')return NaN;if(value===null&&kind==='positive-infinity')return Infinity;if(value===null&&kind==='negative-infinity')return -Infinity;
 throw new Error('Invalid numeric class in native character record.');
}
// Source bytes are checked asynchronously. This verifies recorded geometry and
// vector/edge mathematics without replaying an imported CSV transformation.
export function nativeCharacterLabWeights(graph,config,evidence,scope='edge-weighting'){
 const physicalGraph=graph;graph=countyPreGraph(graph,config);
 validateJsonTree(evidence);const input=config.native_character_sources?.[graph.state],s=nativeSourceScope(input,config,graph.state);
 if(!exact(evidence,['method','evaluation','scope'])||evidence.method!=='native-character-file-v1'||evidence.scope!==scope)throw new Error('Native source weighting evidence has the wrong scope.');
 const r=evidence.evaluation,keys=['schema_version','state','year','data_year','kind','alpha','source_name','source_sha256','source_byte_length','target_json','target_sha256','native_status','native_error','geographic_fallback','matched_tracts','missing_tract_ids','ignored_source_ids','records','edges','execution_performed','scope'];
 // Keep the conditional explicit; diagnostic status and nullable values differ.
 if(!exact(r,keys)||r.schema_version!=='bisect-native-character-weights-v1'||r.execution_performed!==false||['state','year','data_year','kind'].some(k=>r[k]!==s[k])||r.alpha!==config.character_alpha||r.source_name!==input.source.name||!hash(r.source_sha256)||!hash(r.target_sha256)||!Number.isSafeInteger(r.source_byte_length)||r.source_byte_length<0||typeof r.target_json!=='string'||typeof r.scope!=='string'||!['loaded','empty','error'].includes(r.native_status)||r.geographic_fallback!==(r.native_status!=='loaded')||(r.native_status==='error'?(typeof r.native_error!=='string'||!r.native_error):r.native_error!==null)||r.geographic_fallback&&config.character_absent_file_fallback!==true)throw new Error('Native source report disagrees with selection or fallback policy.');
 const target=parseSafeJson(r.target_json);if(!exact(target,['state','year',...(Object.hasOwn(target,'unit_kind')?['unit_kind']:[]),...(Object.hasOwn(target,'county_ids')?['county_ids']:[]),'geoids','edges'])||(target.unit_kind??'tract')!==(graph.unit_kind??'tract')||!same(target.county_ids,graph.county_ids)||target.state!==graph.state||target.year!==graph.year||!same(target.geoids,graph.geoids)||!Array.isArray(target.edges)||target.edges.length!==graph.edges.length||target.edges.some((e,i)=>!Array.isArray(e)||e.length!==3||e[0]!==graph.edges[i][0]||e[1]!==graph.edges[i][1]||!close(e[2],graph.edges[i][2])))throw new Error('Native weighting target differs from selected graph.');
 const targetIds=new Set(graph.geoids),rustCompare=(a,b)=>{const x=[...a],y=[...b];for(let i=0;i<Math.min(x.length,y.length);i++){const c=x[i].codePointAt(0)-y[i].codePointAt(0);if(c)return c;}return x.length-y.length;};
 const sortedUnique=v=>Array.isArray(v)&&v.every(x=>typeof x==='string')&&new Set(v).size===v.length&&same(v,[...v].sort(rustCompare));
 if(!sortedUnique(r.missing_tract_ids)||!sortedUnique(r.ignored_source_ids)||r.ignored_source_ids.some(id=>targetIds.has(id))||r.geographic_fallback&&r.ignored_source_ids.length>0||!Array.isArray(r.records)||r.records.length!==(r.geographic_fallback?0:graph.geoids.length)||!Array.isArray(r.edges)||r.edges.length!==graph.edges.length)throw new Error('Native alignment coverage is inconsistent.');
 const fields=s.kind==='economic'?3:4,vectors=[],missing=[];
 for(const [i,row]of r.records.entries()){if(!exact(row,['geoid','supplied','values','classes'])||row.geoid!==graph.geoids[i]||typeof row.supplied!=='boolean'||!Array.isArray(row.values)||!Array.isArray(row.classes)||row.values.length!==fields||row.classes.length!==fields)throw new Error('Invalid aligned native character record.');const values=row.values.map((v,j)=>decodeNumber(v,row.classes[j]));if(!row.supplied){missing.push(row.geoid);if(row.classes.some(c=>c!=='finite')||values.some(v=>v!==(s.kind==='economic'?0:.5)))throw new Error('Native missing-tract defaults were changed.');}vectors.push(values);}
 if(r.geographic_fallback)missing.push(...graph.geoids);
 if(!same(missing.sort(),r.missing_tract_ids)||r.matched_tracts!==(r.geographic_fallback?0:graph.geoids.length-missing.length))throw new Error('Native matched/missing counts disagree.');
 const sourceWeight=(u,v,length)=>{if(r.geographic_fallback)return length;const a=vectors[u],b=vectors[v],dot=a.reduce((sum,x,i)=>sum+x*b[i],0),ma=Math.sqrt(a.reduce((sum,x)=>sum+x*x,0)),mb=Math.sqrt(b.reduce((sum,x)=>sum+x*x,0)),similarity=ma<1e-15||mb<1e-15?(s.kind==='housing'||ma<1e-15&&mb<1e-15?1:.5):Math.min(1,Math.max(0,dot/(ma*mb)));return length*(r.alpha+(1-r.alpha)*similarity);};
 for(const [i,e]of r.edges.entries()){const [u,v,w]=graph.edges[i];if(!exact(e,['u','v','weight','class'])||e.u!==u||e.v!==v||!nativeMetricMatches(e.weight,sourceWeight(u,v,w),e.class))throw new Error('Native edge weights disagree with recorded vectors and physical target.');}
 return {native:true,input,kind:s.kind,alpha:r.alpha,report:r,weight:(u,v,length)=>sourceWeight(u,v,countyPreLength(physicalGraph,config,u,v,length))};
}
export async function verifyNativeCharacterLabSource(graph,state,config){
 const scope=config.search?.startsWith('multiscale')?'parent-tract-initialization-only; no fine-unit character observations inferred':'edge-weighting',evidence=config.search?.startsWith('multiscale')?state.metrics.multiscale_evidence?.character_initialization:state.metrics.weighting_evidence,weights=nativeCharacterLabWeights(graph,config,evidence,scope),bytes=Uint8Array.from(atob(weights.input.source.base64),c=>c.charCodeAt(0)),digest=async b=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',b))].map(x=>x.toString(16).padStart(2,'0')).join('');
 if(bytes.length!==weights.report.source_byte_length||await digest(bytes)!==weights.report.source_sha256||await digest(new TextEncoder().encode(weights.report.target_json))!==weights.report.target_sha256)throw new Error('Original native CSV or target bytes disagree with recorded hashes.');return weights;
}

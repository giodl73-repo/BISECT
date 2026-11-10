import {parseSafeJson} from './project.js';
const record=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const exact=(v,keys)=>record(v)&&Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
const canonical=v=>JSON.stringify(v&&typeof v==='object'?Array.isArray(v)?v.map(x=>JSON.parse(canonical(x))):Object.fromEntries(Object.keys(v).sort().map(k=>[k,JSON.parse(canonical(v[k]))])):v);
const same=(a,b)=>canonical(a)===canonical(b);
// Reopening establishes byte and data consistency, never authentic execution or
// whole-plan optimality. No report is executed, extracted to disk, or evaluated.
export async function verifyLaboratoryIlpReports(graph,state,assignments,config){
 const p=state.metrics?.ilp_report_capture;
 if(config.capture_ilp_reports!==true){if(p!==undefined)throw new Error('Unrequested ILP reports.');return;}
 const keys=['schema_version','status','error','source_json','source_sha256','returned_assignments_json','returned_assignments_sha256','accepted_attempt','unit_ids','nodes','files','file_sha256','scope'];
 if(config.structure!=='ilp'||!exact(p,keys)||p.schema_version!=='bisect-native-ilp-run-reports-v1'||p.status!=='completed'||p.error!==null||typeof p.scope!=='string'||p.scope.length>2000||!same(p.unit_ids,graph.geoids)||!Array.isArray(p.nodes)||p.nodes.length>512||!record(p.files)||!record(p.file_sha256))throw new Error('Invalid laboratory ILP reports.');
 const checked=async(text,hash,limit)=>{if(typeof text!=='string'||new TextEncoder().encode(text).length>limit||await digest(text)!==hash)throw new Error('ILP report source bytes changed.');return parseSafeJson(text);};
 const request=await checked(p.source_json,p.source_sha256,8*1024*1024),returned=await checked(p.returned_assignments_json,p.returned_assignments_sha256,25*1024*1024);
 const normalized=structuredClone(graph);for(const key of ['unit_kind','county_ids'])if(normalized[key]===null)delete normalized[key];
 if(!same(request.graph,normalized)||!same(request.options,state.metrics.recorded_options)||!same(request.electoral??null,config.electoral??null)||!same(request.coi??null,config.coi??null)||!same(request.partisan??null,config.partisans?.[graph.state]??null)||!same(request.demographics??null,config.demographics?.[graph.state]??null)||!same(request.character??null,config.characters?.[graph.state]??null)||!same(request.elections??null,config.elections?.[graph.state]??null)||request.options?.capture_ilp_reports!==true||!same(returned,assignments)||p.accepted_attempt!==(config.native_build===true?state.metrics.balance_build?.accepted_attempt:null))throw new Error('ILP reports disagree with the returned run.');
 const leaves=[],schedule=new Map();
 const visit=(k,path)=>{if(k===1){leaves.push(path);return;}schedule.set(path,[]);visit(Math.floor(k/2),path+'0');visit(Math.ceil(k/2),path+'1');};
 if(!Number.isInteger(request.options.districts)||request.options.districts<1||request.options.districts>500)throw new Error('Invalid ILP district count.');
 visit(request.options.districts,'');leaves.sort((a,b)=>a.length-b.length||a.localeCompare(b));
 let mapped=0;for(const [i,id]of graph.geoids.entries()){const leaf=leaves[assignments[id]-1];if(typeof leaf!=='string')throw new Error('Invalid ILP returned district.');for(let depth=0;depth<leaf.length;depth++){if(++mapped>2000000)throw new Error('ILP unit references exceed bounds.');schedule.get(leaf.slice(0,depth)).push(i);}}
 if(p.nodes.length!==schedule.size)throw new Error('ILP report node roster changed.');
 let references=0,bytes=0;const expectedFiles={},seen=new Set();let previous=null;
 for(const n of p.nodes){
  if(!exact(n,['depth','path','unit_indices','metis_multiplier','ilp_population_tolerance','stage','report_json','lp_text'])||typeof n.path!=='string'||!/^[01]{0,20}$/.test(n.path)||n.depth!==n.path.length||seen.has(n.path)||!Array.isArray(n.unit_indices))throw new Error('Invalid ILP node mapping.');
  const order=[n.depth,n.path];if(previous&&(order[0]<previous[0]||order[0]===previous[0]&&order[1]<=previous[1]))throw new Error('ILP nodes changed order.');previous=order;seen.add(n.path);
  let k=request.options.districts;for(const bit of n.path){if(k<2)throw new Error('ILP node path exceeds native tree.');k=bit==='0'?Math.floor(k/2):Math.ceil(k/2);}
  const tolerance=request.options.balance_tolerance/100/k;
  if(k<2||n.ilp_population_tolerance!==tolerance||n.metis_multiplier!==1+tolerance)throw new Error('ILP node population settings changed.');
  let last=-1;for(const i of n.unit_indices){if(!Number.isInteger(i)||i<=last||i>=graph.geoids.length)throw new Error('Invalid ILP original unit index.');last=i;}
  if(!same(n.unit_indices,schedule.get(n.path)))throw new Error('ILP node mapping disagrees with final native tree labels.');
  references+=n.unit_indices.length;if(references>2000000)throw new Error('ILP unit references exceed bounds.');
  if(n.path===''&&!same(n.unit_indices,graph.geoids.map((_,i)=>i)))throw new Error('ILP root unit mapping changed.');
  const base=`depth_${String(n.depth).padStart(2,'0')}/node_${n.path||'root'}`;
  if(n.stage==='size-guard-skipped'){if(n.report_json!==null||n.lp_text!==null||n.unit_indices.length<=(request.options.ilp_max_tracts??500))throw new Error('Invalid skipped ILP node.');continue;}
  if(n.stage!=='native-solver-attempt'||typeof n.report_json!=='string'||typeof n.lp_text!=='string'||n.unit_indices.length>(request.options.ilp_max_tracts??500))throw new Error('Invalid ILP solve artifact.');
  bytes+=new TextEncoder().encode(n.report_json).length+new TextEncoder().encode(n.lp_text).length;if(bytes>16*1024*1024)throw new Error('ILP artifact bytes exceed bounds.');
  const r=parseSafeJson(n.report_json);if(r.schema_version!=='ilp-solve-report-v1'||r.formulation?.n_tracts!==n.unit_indices.length||r.formulation?.k!==2||r.model_artifact?.format!=='cplex-lp'||r.model_artifact.path!==`node_${n.path||'root'}.lp`||await digest(n.lp_text)!==r.model_artifact.sha256)throw new Error('ILP report and LP model disagree.');
  expectedFiles[base+'.json']=n.report_json;expectedFiles[base+'.lp']=n.lp_text;
 }
 if(!same(p.files,expectedFiles)||!same(Object.keys(p.file_sha256).sort(),Object.keys(expectedFiles).sort()))throw new Error('ILP report file roster changed.');
 for(const [name,text]of Object.entries(expectedFiles))if(await digest(text)!==p.file_sha256[name])throw new Error('ILP report file bytes changed.');
}

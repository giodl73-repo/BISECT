import {validateJsonTree,readValidatedFile} from './project.js';
export const isMultiscale=c=>['multiscale','multiscale-adaptive'].includes(typeof c==='string'?c:c?.search);
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
const integer=(v,min,max)=>Number.isSafeInteger(v)&&v>=min&&v<=max;
const fraction=v=>Number.isFinite(v)&&!Object.is(v,-0)&&v>=0&&v<=1;
const canonical=v=>JSON.stringify(sort(v));
function sort(v){return Array.isArray(v)?v.map(sort):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,sort(v[k])])):v;}
async function digest(bytes){return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(v=>v.toString(16).padStart(2,'0')).join('');}
export const bundleIdentity=b=>digest(new TextEncoder().encode(canonical({graph:b.graph,boundary_edges:b.boundary_edges,geometry:b.geometry})));
export function validateMultiscaleBundle(b,tract=null){
 validateJsonTree(b);
 if(!exact(b,['schema_version','source_label','graph','boundary_edges','geometry'])||b.schema_version!=='bisect-multiscale-input-v1'||typeof b.source_label!=='string'||!b.source_label.trim()||b.source_label.length>200)throw new Error('Invalid multiscale input bundle.');
 const g=b.graph,n=g?.geoids?.length;
 if(!exact(g,['schema_version','state','year','geoids','adjacency','population'])||g.schema_version!=='bisect-block-group-graph-v1'||!/^[A-Z]{2}$/.test(g.state)||!['2000','2010','2020'].includes(g.year)||!integer(n,1,100000)||g.adjacency.length!==n||g.population.length!==n||new Set(g.geoids).size!==n||g.geoids.some(id=>typeof id!=='string'||!/^\d{12}$/.test(id))||g.population.some(p=>!integer(p,0,Number.MAX_SAFE_INTEGER)))throw new Error('Invalid block-group graph.');
 const arcs=new Set();let total=0;
 for(let i=0;i<n;i++){total+=g.population[i];if(!Number.isSafeInteger(total)||!Array.isArray(g.adjacency[i]))throw new Error('Invalid fine population or adjacency.');for(const j of g.adjacency[i]){const key=`${i}:${j}`;if(!integer(j,0,n-1)||i===j||arcs.has(key)||arcs.size>=2000000)throw new Error('Invalid fine adjacency.');arcs.add(key);}}
 if(total===0||[...arcs].some(key=>!arcs.has(key.split(':').reverse().join(':'))))throw new Error('Fine adjacency must be symmetric with positive population.');
 const connected=(indices,parent)=>{if(!indices.length)return false;const seen=new Set([indices[0]]),queue=[indices[0]];while(queue.length)for(const j of g.adjacency[queue.pop()])if((!parent||g.geoids[j].slice(0,11)===parent)&&!seen.has(j)){seen.add(j);queue.push(j);}return seen.size===indices.length;};
 if(!connected(Array.from({length:n},(_,i)=>i)))throw new Error('Fine graph must be connected.');
 if(!Array.isArray(b.boundary_edges)||b.boundary_edges.length*2!==arcs.size)throw new Error('Complete fine physical boundaries are required.');
 const edges=new Set();for(const e of b.boundary_edges){if(!Array.isArray(e)||e.length!==3||!integer(e[0],0,n-1)||!integer(e[1],e[0]+1,n-1)||!arcs.has(`${e[0]}:${e[1]}`)||edges.has(`${e[0]}:${e[1]}`)||!Number.isFinite(e[2])||e[2]<=0)throw new Error('Invalid fine boundary edge.');edges.add(`${e[0]}:${e[1]}`);}
 if(!Number.isFinite(b.boundary_edges.reduce((s,e)=>s+e[2],0)))throw new Error('Fine boundary total exceeds numeric range.');
 const geometry=b.geometry;
 if(!exact(geometry,['type','features'])||geometry.type!=='FeatureCollection'||!Array.isArray(geometry.features)||geometry.features.length!==n)throw new Error('Complete block-group geometry is required.');
 const ids=new Set(g.geoids),seen=new Set();let points=0;
 for(const f of geometry.features){if(!exact(f,['type','properties','geometry'])||f.type!=='Feature'||!exact(f.properties,['geoid'])||!ids.has(f.properties.geoid)||seen.has(f.properties.geoid)||!exact(f.geometry,['type','coordinates'])||!['Polygon','MultiPolygon'].includes(f.geometry.type))throw new Error('Invalid fine map feature.');seen.add(f.properties.geoid);
  const polygons=f.geometry.type==='Polygon'?[f.geometry.coordinates]:f.geometry.coordinates;if(!Array.isArray(polygons)||!polygons.length)throw new Error('Empty fine polygon.');
  for(const polygon of polygons){if(!Array.isArray(polygon)||!polygon.length)throw new Error('Empty fine polygon.');for(const ring of polygon){if(!Array.isArray(ring)||ring.length<4||ring.some(p=>!Array.isArray(p)||p.length!==2||!Number.isFinite(p[0])||!Number.isFinite(p[1])||Math.abs(p[0])>180||Math.abs(p[1])>90)||ring[0][0]!==ring.at(-1)[0]||ring[0][1]!==ring.at(-1)[1]||(points+=ring.length)>2000000)throw new Error('Invalid or oversized fine polygon ring.');}}
 }
 if(tract){if(g.state!==tract.state||g.year!==tract.year)throw new Error('Fine/tract scope mismatch.');const parents=new Map(tract.geoids.map((id,i)=>[id,i])),pop=Array(tract.geoids.length).fill(0),groups=new Map(),quotient=new Set();
  for(let i=0;i<n;i++){const parent=g.geoids[i].slice(0,11);if(!parents.has(parent))throw new Error('Fine unit has no selected tract parent.');pop[parents.get(parent)]+=g.population[i];if(!groups.has(parent))groups.set(parent,[]);groups.get(parent).push(i);for(const j of g.adjacency[i]){const other=g.geoids[j].slice(0,11);if(parent!==other)quotient.add(`${parent}:${other}`);}}
  if(pop.some((v,i)=>v!==tract.population[i])||groups.size!==parents.size||[...groups].some(([id,indices])=>!connected(indices,id)))throw new Error('Fine inputs must cover connected tracts with exact population sums.');
  const expected=new Set(tract.adjacency.flatMap((ns,i)=>ns.map(j=>`${tract.geoids[i]}:${tract.geoids[j]}`)));if(expected.size!==quotient.size||[...expected].some(e=>!quotient.has(e)))throw new Error('Fine adjacency quotient differs from tract graph.');
 }
 return b;
}
export function validateMultiscaleConfig(c){
 const m=c.multiscale;
 if(!exact(m,['fine_level','coarse_level','total_steps','alpha','percentile','adaptive','inputs'])||c.structure!=='standard-bisect'||c.weights!=='geographic'||c.alpha_county!==0||Object.hasOwn(c,'metis_objective')||Object.hasOwn(c,'metis_trials')||!['tract','bg'].includes(m.fine_level)||!['tract','county'].includes(m.coarse_level)||(m.fine_level==='tract'&&m.coarse_level!=='county')||!integer(m.total_steps,0,100000)||!fraction(m.alpha)||!fraction(m.percentile)||!m.inputs||Array.isArray(m.inputs))throw new Error('Invalid multiscale configuration.');
 if(c.search==='multiscale-adaptive'){const a=m.adaptive;if(!exact(a,['target_accept','adapt_interval','gamma_0','coarse_tol_factor'])||!fraction(a.target_accept)||!integer(a.adapt_interval,1,100000)||!fraction(a.gamma_0)||!Number.isFinite(a.coarse_tol_factor)||a.coarse_tol_factor<1||a.coarse_tol_factor>10||a.coarse_tol_factor*c.balance_tolerance>100)throw new Error('Invalid adaptation parameters.');}else if(m.adaptive!==null)throw new Error('Fixed multiscale search must not contain adaptation parameters.');
 const codes=Object.keys(m.inputs);if(m.fine_level==='tract'?codes.length!==0:codes.length!==c.states.length||c.states.some(code=>!Object.hasOwn(m.inputs,code)))throw new Error('Explicit fine input is required for every selected state.');
 if(c.steps!==m.total_steps||c.percentile!==m.percentile||c.seeds!==1)throw new Error('Multiscale step/percentile settings disagree with the run controls.');
 for(const code of codes){const b=validateMultiscaleBundle(m.inputs[code]);if(b.graph.state!==code||b.graph.year!==c.year)throw new Error('Fine input scope mismatch.');}
}
export function fineGraph(tract,c){if(!isMultiscale(c)||c.multiscale.fine_level==='tract')return tract;const b=validateMultiscaleBundle(c.multiscale.inputs[tract.state],tract);return {...tract,...b.graph,schema_version:1,edges:b.boundary_edges,areas:Array(b.graph.geoids.length).fill(0),exterior_perimeters:Array(b.graph.geoids.length).fill(0),centroids:Array.from({length:b.graph.geoids.length},()=>[0,0])};}
export function multiscaleRequest(graph,options,c){const m=c.multiscale;return {operation:'run-multiscale',input:{request:{graph,options:{...options,search:'single',seeds:1,steps:1,percentile:0}},fine_level:m.fine_level,coarse_level:m.coarse_level,total_steps:m.total_steps,alpha:m.alpha,percentile:m.percentile,adaptive:m.adaptive,block_groups:m.fine_level==='bg'?validateMultiscaleBundle(m.inputs[graph.state],graph).graph:null}};}
export async function graphIdentity(g,b){const bytes=[],encoder=new TextEncoder();const append=v=>bytes.push(...v),u64=v=>{const a=new Uint8Array(8);new DataView(a.buffer).setBigUint64(0,BigInt(v),true);append(a);},text=v=>append(encoder.encode(v));
 text('BISECT_MULTISCALE_GRAPHS_V1\0');text(g.state);append([0]);text(g.year);append([0]);
 const units=x=>{u64(x.geoids.length);x.geoids.forEach((id,i)=>{u64(id.length);text(id);u64(x.population[i]);u64(x.adjacency[i].length);x.adjacency[i].forEach(u64);});};units(g);u64(g.edges.length);g.edges.forEach(([i,j,v])=>{u64(i);u64(j);const a=new Uint8Array(8);new DataView(a.buffer).setFloat64(0,v,true);append(a);});append([b?1:0]);if(b)units(b);return digest(Uint8Array.from(bytes));}
export function adaptMultiscaleResult(result,g,options,c){const fine=fineGraph(g,c),k=options.districts,plan=fine.geoids.map(id=>result.assignments[id]),populations=Array(k).fill(0),units=Array(k).fill(0),components=Array(k).fill(0),counties=new Map(),seen=new Set();
 fine.geoids.forEach((id,i)=>{const d=plan[i]-1;populations[d]+=fine.population[i];units[d]++;const county=id.slice(0,5);if(!counties.has(county))counties.set(county,new Set());counties.get(county).add(plan[i]);});
 for(let i=0;i<plan.length;i++)if(!seen.has(i)){components[plan[i]-1]++;const todo=[i];seen.add(i);while(todo.length)for(const j of fine.adjacency[todo.pop()])if(plan[j]===plan[i]&&!seen.has(j)){seen.add(j);todo.push(j);}}
 const total=fine.population.reduce((a,b)=>a+b,0),districts=populations.map((p,i)=>({district:i+1,population:p,units:units[i],deviation_percent:(p/(total/k)-1)*100,components:components[i]})),max=Math.max(...districts.map(d=>Math.abs(d.deviation_percent))),boundary=fine.edges.reduce((sum,[i,j,length])=>sum+(plan[i]!==plan[j]?length:0),0);
 return {backend:'metis-core-rust',options,assignments:result.assignments,metrics:{districts:k,district_metrics:districts,units:plan.length,population:total,max_deviation_percent:max,graph_boundary_m:boundary,weighted_boundary:boundary,split_counties:[...counties.values()].filter(ds=>ds.size>1).length,contiguous:components.every(n=>n===1),within_requested_tolerance:max<=c.balance_tolerance,root_split:null,structure_evidence:null,multiscale_evidence:result.evidence,execution:'heuristic',optimality:'unproved'}};
}
export async function verifyMultiscaleEvidence(g,state,c){
 if(state.metrics.root_split!==null||state.metrics.structure_evidence!==null)throw new Error('Multiscale projects must not claim recursive root or unrelated structure evidence.');
 const e=state.metrics.multiscale_evidence,m=c.multiscale,b=m.fine_level==='bg'?validateMultiscaleBundle(m.inputs[g.state],g):null;
 if(!exact(e,['method','rng','graph_sha256','coarse_level','total_steps','alpha','percentile','initialization','sampling','ranking','coarse_assignment','diagnostic_policy','seed','diagnostics','execution','optimality'])||e.execution!=='heuristic'||e.optimality!=='unproved'||e.method!==c.search||e.rng!=='chacha12-u64-portable-wilson-v1'||e.graph_sha256!==await graphIdentity(g,b?.graph)||e.coarse_level!==m.coarse_level||e.total_steps!==m.total_steps||e.alpha!==m.alpha||e.percentile!==m.percentile||e.seed!==c.seed||e.initialization!=='weighted-geographic-metis'||e.sampling!=='unweighted-wilson-recom'||e.ranking!=='fine-unweighted-edge-cut-then-step'||e.coarse_assignment!=='last-fine-index-wins-native-policy'||e.diagnostic_policy!=='native-counters: fine-attempts-counted-as-accepted; coarse-rebalance-success')throw new Error('Multiscale evidence disagrees with inputs/settings.');
 if(b&&state.metrics.fine_input_sha256!==await bundleIdentity(b))throw new Error('Fine map/boundary identity mismatch.');
 if(!b&&state.metrics.fine_input_sha256!==null)throw new Error('Unexpected fine input identity.');
 if(m.adaptive){const d=e.diagnostics;if(!exact(d,['final_alpha','alpha_trace','fine_acceptance_rate','coarse_acceptance_rate'])||!fraction(d.final_alpha)||!fraction(d.fine_acceptance_rate)||!fraction(d.coarse_acceptance_rate)||!Array.isArray(d.alpha_trace)||d.alpha_trace.length!==Math.floor(m.total_steps/m.adaptive.adapt_interval)||d.alpha_trace.some(v=>!Number.isFinite(v)||v<.05||v>.95)||d.final_alpha!==(d.alpha_trace.at(-1)??m.alpha))throw new Error('Invalid multiscale adaptation evidence.');}else if(e.diagnostics!==null)throw new Error('Unexpected adaptation evidence.');
}
export const readMultiscaleBundle=(file,options)=>readValidatedFile(file,new URL('./multiscale-input-worker.js',import.meta.url),options);

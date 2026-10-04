import {validateElectionInput,verifyElectionResult} from './election-input.js';
import {validatePartisanInput,partisanWeights,partisanIdentity} from './partisan-input.js';
import {parseSafeJson, validateJsonTree, readValidatedFile} from './project.js';
import {effectiveConfig,engineOptions,isSeed} from './static.js';
import {validateDemographicInput,tractMeanFractions,demographicIdentity} from './demographic-input.js';
export const LAB_PROJECT_SCHEMA = 'bisect-laboratory-project-v1';
const configKeys = ['name','mode','states','year','chamber','districts','structure','weights','search','seed','seeds','steps','percentile','alpha_county','balance_tolerance','area_swing','iterations','timeout_seconds'];
const record = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const exact = (value, keys) => record(value) && Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value,key));
const hash = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const integer = (value,min,max) => Number.isSafeInteger(value) && value >= min && value <= max;
const bounded = (value,min,max) => Number.isFinite(value) && value >= min && value <= max;
export function validateLabConfig(config) {
  const sa=config?.structure==='simulated-annealing';
  const baseKeys=config?.search==='smc-percentile'?[...configKeys,'smc_particles','smc_resample_threshold']:sa?[...configKeys,'sa_steps_per_tract','sa_t0_factor','sa_t_final']:config?.structure==='flow-construction'&&Object.hasOwn(config,'flow_repair')?[...configKeys,'flow_repair']:configKeys;
  const nway=['nway','standard-bisect','ratio-optimal','ratio-optimal-area','ratio-optimal-vra'].includes(config?.structure)&&(Object.hasOwn(config,'metis_objective')||Object.hasOwn(config,'metis_trials'));
  if(nway&&((['ratio-optimal','ratio-optimal-area','ratio-optimal-vra'].includes(config.structure)&&!['single','multi'].includes(config.search))||(config.structure==='standard-bisect'&&!['single','multi','percentile','convergence','bisection-ensemble'].includes(config.search))||!['cut','volume'].includes(config.metis_objective)||!integer(config.metis_trials,1,100)))throw new Error('Invalid METIS objective/trial controls or search.');
  const cvd=config?.structure==='centroidal-voronoi'&&(Object.hasOwn(config,'cvd_iters')||Object.hasOwn(config,'cvd_metric'));
  const mka=config?.structure==='moving-knife'&&(Object.hasOwn(config,'mka_orientations')||Object.hasOwn(config,'mka_metric'));
  const compact=config?.structure==='compact-polsby'&&Object.hasOwn(config,'compact_epsilon');
  const area=config?.structure==='ratio-optimal-area'&&Object.hasOwn(config,'area_init');
  if(area&&!['ratio-optimal','moving-knife'].includes(config.area_init))throw new Error('Invalid AreaSection initializer.');
  const burst=config?.structure==='standard-bisect'&&config.search?.startsWith('short-burst')&&(Object.hasOwn(config,'burst_length')||Object.hasOwn(config,'n_bursts'));
  if(burst&&(!['short-burst','short-burst-forest','short-burst-merge-split'].includes(config.search)||!integer(config.burst_length,0,100000)||!integer(config.n_bursts,0,10000)||config.burst_length*config.n_bursts>100000))throw new Error('Invalid short-burst work budget.');
  const pt=config?.structure==='standard-bisect'&&config.search==='parallel-tempering'&&['pt_replicas','pt_swap_interval','pt_cold_tol','pt_hot_tol'].some(key=>Object.hasOwn(config,key));
  if(pt&&(!integer(config.pt_replicas,1,32)||!integer(config.pt_swap_interval,1,100000)||!bounded(config.pt_cold_tol,0.0001,0.25)||!bounded(config.pt_hot_tol,config.pt_cold_tol,1)||Math.abs(config.pt_cold_tol*100-config.balance_tolerance)>1e-9||config.pt_replicas*config.seeds>100000))throw new Error('Invalid parallel-tempering ladder or work budget.');
  const vra=config?.search==='vra-recom',vraSection=config?.structure==='ratio-optimal-vra';
  const specialKeys=vraSection?[...baseKeys,'w_vra','demographics']:vra?[...baseKeys,'vra_threshold','demographics']:pt?[...baseKeys,'pt_replicas','pt_swap_interval','pt_cold_tol','pt_hot_tol']:burst?[...baseKeys,'burst_length','n_bursts']:area?[...baseKeys,'area_init']:compact?[...baseKeys,'compact_epsilon']:mka?[...baseKeys,'mka_orientations','mka_metric']:cvd?[...baseKeys,'cvd_iters','cvd_metric']:baseKeys;
  const electionMethod=['proportional-bisect','proportional-section'].includes(config?.structure);
  const electionKeys=electionMethod?[...specialKeys,'elections',...(config.structure==='proportional-section'?['proportional_eta']:[])]:specialKeys;
  const tuningKeys=nway?[...electionKeys,'metis_objective','metis_trials']:electionKeys;
  const keys=config?.weights==='partisan'?[...tuningKeys,'dem_threshold','rep_threshold','partisans']:tuningKeys;
  if(compact&&!bounded(config.compact_epsilon,0,1))throw new Error('Invalid CompactBisect slack.');
  if(mka&&(!integer(config.mka_orientations,0,10000)||!['reock','polsby'].includes(config.mka_metric)))throw new Error('Invalid moving-knife parameters.');
  if(cvd&&(!integer(config.cvd_iters,0,10000)||!['graph-distance','geographic'].includes(config.cvd_metric)))throw new Error('Invalid CVD parameters.');
  if (!exact(config,keys) || typeof config.name !== 'string' || !config.name.trim() || config.name.length > 100) throw new Error('Invalid laboratory configuration.');
  if(Object.hasOwn(config,'flow_repair')&&!['none','bfs'].includes(config.flow_repair))throw new Error('Invalid flow repair method.');
  if(config.search==='smc-percentile'&&(config.structure!=='standard-bisect'||!integer(config.smc_particles,1,10000)||!bounded(config.smc_resample_threshold,0,1)||config.balance_tolerance!==0.5))throw new Error('Invalid SMC settings.');
  if (!['state','national'].includes(config.mode) || !['2000','2010','2020'].includes(config.year) || !['congressional','house','senate'].includes(config.chamber)) throw new Error('Invalid experiment scope.');
  if (!Array.isArray(config.states) || !integer(config.states.length,1,50) || new Set(config.states).size !== config.states.length || config.states.some(code => typeof code !== 'string' || !/^[A-Z]{2}$/.test(code)) || (config.mode === 'state' && config.states.length !== 1)) throw new Error('Invalid state selection.');
  if (config.districts !== null && !integer(config.districts,1,500)) throw new Error('Invalid district count.');
  if (config.mode === 'national' && config.districts !== null) throw new Error('National experiments use chamber allocations.');
  for (const key of ['structure','weights','search']) if (typeof config[key] !== 'string' || !/^[a-z-]{1,40}$/.test(config[key])) throw new Error('Invalid engine setting.');
  if(!['geographic','unweighted','county','partisan'].includes(config.weights))throw new Error('Unsupported project boundary weights.');
  if (!isSeed(config.seed) || !integer(config.seeds,pt?0:1,10000) || !integer(config.steps,['vra-recom','forest-recom','merge-split','flip','bisection-ensemble'].includes(config.search)?0:1,100000) || !integer(config.iterations,1,1000) || !bounded(config.percentile,0,1) || !bounded(config.alpha_county,0,100) || !bounded(config.balance_tolerance,0.01,25) || !bounded(config.area_swing,1.01,2) || !bounded(config.timeout_seconds,1,86400)) throw new Error('Invalid engine option bounds.');
  if(electionMethod){
    if(!record(config.elections)||Object.keys(config.elections).length!==config.states.length||!['single',...(config.structure==='proportional-section'?['multi']:[])].includes(config.search)||config.weights==='partisan'||(config.structure==='proportional-section'&&!bounded(config.proportional_eta,1,2)))throw new Error('Proportional methods require supported search, election counts per selected state and valid vote tolerance.');
    for(const code of config.states){const input=validateElectionInput(config.elections[code]);if(input.state!==code||input.year!==config.year)throw new Error('Election input scope must match the experiment.');}
  }
  if(config.weights==='partisan'){
    if(config.alpha_county!==0||config.structure!=='standard-bisect'||!['single','multi','percentile','convergence','bisection-ensemble'].includes(config.search)||!bounded(config.dem_threshold,0,1)||!bounded(config.rep_threshold,0,config.dem_threshold)||!record(config.partisans)||Object.keys(config.partisans).length!==config.states.length)throw new Error('Partisan weights require standard Single/Multi/percentile/convergence/local-ensemble, ordered thresholds and explicit shares for every selected state.');
    for(const code of config.states){const input=validatePartisanInput(config.partisans[code]);if(input.state!==code||input.year!==config.year)throw new Error('Partisan input scope must match the experiment.');}
  }
  if(vraSection){
    if(!['single','multi'].includes(config.search)||!bounded(config.w_vra,0,1)||!record(config.demographics)||Object.keys(config.demographics).length!==config.states.length)throw new Error('VRASection requires single/multi search, an alignment weight and demographic inputs.');
  }
  if(vra||vraSection){
    if(vra&&(config.structure!=='standard-bisect'||!bounded(config.vra_threshold,0,1)||config.balance_tolerance!==0.5||!record(config.demographics)||Object.keys(config.demographics).length!==config.states.length))throw new Error('VRA ReCom requires standard bisection, a threshold, 0.5% tolerance and demographic data for every selected state.');
    for(const code of config.states){if(!config.demographics[code])throw new Error(`Load demographic data for ${code} ${config.year}.`);const input=validateDemographicInput(config.demographics[code]);if(vraSection&&input.basis!=='total-population')throw new Error('VRASection requires total-population fractions for its native mass proxy.');if(input.state!==code||input.year!==config.year)throw new Error('Demographic scope must match the experiment.');}
  }
  if(sa&&(!integer(config.sa_steps_per_tract,0,10000)||!bounded(config.sa_t0_factor,0,1000)||!bounded(config.sa_t_final,1e-15,1e6)))throw new Error('Invalid simulated annealing parameters.');
  return config;
}
export function validateLabProject(project) {
  validateJsonTree(project);
  if (!exact(project,['schema_version','saved_at_utc','experiment','assignments','inputs']) || project.schema_version !== LAB_PROJECT_SCHEMA || typeof project.saved_at_utc !== 'string' || project.saved_at_utc.length > 40 || !Number.isFinite(Date.parse(project.saved_at_utc))) throw new Error('Invalid laboratory project format.');
  const job = project.experiment;
  if (!exact(job,['id','config','states','created_unix','status','logs']) || typeof job.id !== 'string' || !/^[a-zA-Z0-9-]{1,100}$/.test(job.id) || !integer(job.created_unix,0,8640000000000) || !['completed','partial','failed','cancelled','interrupted'].includes(job.status)) throw new Error('Invalid saved experiment.');
  validateLabConfig(job.config);
  if (!Array.isArray(job.logs) || job.logs.length > 500 || job.logs.some(line => typeof line !== 'string' || line.length > 2000)) throw new Error('Invalid experiment log.');
  if (!Array.isArray(job.states) || job.states.length !== job.config.states.length || !record(project.assignments) || !record(project.inputs) || Object.keys(project.inputs).length !== job.states.length) throw new Error('Invalid experiment results.');
  const completed = [];
  for (let index = 0; index < job.states.length; index++) {
    const state = job.states[index], code = job.config.states[index];
    if (!exact(state,['code','status','metrics','error','elapsed_seconds','command']) || state.code !== code || !['completed','failed','cancelled','interrupted'].includes(state.status) || !bounded(state.elapsed_seconds,0,1e9) || !Array.isArray(state.command) || state.command.length || (state.error !== null && (typeof state.error !== 'string' || state.error.length > 2000))) throw new Error('Invalid saved state result.');
    const input = project.inputs[code];
    if (!exact(input,['prepared_graph_sha256','geometry_sha256']) || !hash(input.prepared_graph_sha256) || !hash(input.geometry_sha256)) throw new Error('Invalid project input identity.');
    if (state.status !== 'completed') { if (state.metrics !== null || Object.hasOwn(project.assignments,code)) throw new Error('Incomplete state has a result.'); continue; }
    completed.push(code);
    const m = state.metrics, assignments = project.assignments[code];
    if (!record(m) || !record(assignments) || !integer(m.district_count,1,500) || !integer(m.units,1,1000000) || Object.keys(assignments).length !== m.units || !integer(m.population,1,Number.MAX_SAFE_INTEGER) || !Array.isArray(m.districts) || m.districts.length !== m.district_count) throw new Error('Invalid district metrics or assignments.');
    for (const key of ['max_deviation_percent','graph_boundary_m','weighted_boundary','split_counties']) if (!bounded(m[key],0,Number.MAX_SAFE_INTEGER)) throw new Error('Invalid district metric.');
    if (typeof m.contiguous !== 'boolean' || typeof m.balance_passed !== 'boolean' || m.balance_tolerance_percent !== job.config.balance_tolerance || m.prepared_graph_sha256 !== input.prepared_graph_sha256 || !hash(m.graph_sha256) || m.execution !== 'heuristic' || m.optimality !== 'unproved') throw new Error('Invalid result evidence.');
    const districtKeys=['district','population','units','deviation_percent','components'];
    if(m.within_requested_tolerance!==m.balance_passed||!Array.isArray(m.district_metrics)||m.district_metrics.length!==m.district_count||m.district_metrics.some((d,i)=>!exact(d,districtKeys)||!record(m.districts[i])||districtKeys.some(key=>d[key]!==m.districts[i][key])))throw new Error('Inconsistent recorded district metrics.');
    if (!record(m.engine_provenance) || m.engine_provenance.runtime !== 'wasm-browser' || m.engine_provenance.backend !== 'metis-core-rust' || !hash(m.engine_provenance.wasm_sha256)) throw new Error('Invalid recorded engine identity.');
    const c=job.config, options={...engineOptions(c),structure:c.structure,weights:c.weights,search:c.search,districts:m.district_count,seed:c.seed,seeds:c.seeds,steps:c.steps,percentile:c.percentile,alpha_county:c.alpha_county,balance_tolerance:c.balance_tolerance,area_swing:c.area_swing,iterations:c.iterations};
    const same=(actual,expected)=>exact(actual,Object.keys(expected))&&Object.keys(expected).every(key=>actual[key]===expected[key]);
    if(!same(m.requested_options,options)||!same(m.recorded_options,options)||(!same(m.effective_config,effectiveConfig(c,m.district_count))&&!(c.search==='flip'&&m.structure_evidence==null&&same(m.effective_config,{...effectiveConfig(c,m.district_count),iterations:c.iterations}))))throw new Error('Recorded options disagree with the experiment.');
    for (const [id,district] of Object.entries(assignments)) if (!/^\d{11}$/.test(id) || !integer(district,1,m.district_count)) throw new Error('Invalid tract assignment.');
    for (let i = 0; i < m.districts.length; i++) {
      const d = m.districts[i];
      if (!record(d) || d.district !== i+1 || !integer(d.population,0,m.population) || !integer(d.units,0,m.units) || !integer(d.components,0,m.units) || !bounded(d.deviation_percent,-100,Number.MAX_SAFE_INTEGER)) throw new Error('Invalid district result.');
    }
  }
  if (Object.keys(project.assignments).length !== completed.length) throw new Error('Unexpected assignment results.');
  if ((job.status === 'completed' && completed.length !== job.states.length) || (job.status === 'partial' && (!completed.length || completed.length === job.states.length)) || (job.status === 'failed' && completed.length)) throw new Error('Inconsistent experiment status.');
  return project;
}
export function parseLabProject(text) { return validateLabProject(parseSafeJson(text)); }
export function readLabProjectFile(file, options) { return readValidatedFile(file,new URL('./laboratory-project-worker.js',import.meta.url),options); }
export async function createLabProject(lab, id) {
  const source = lab.jobs.get(id);
  if (!source) throw new Error('Select an experiment first.');
  const job = {id:source.id,config:structuredClone(source.config),states:structuredClone(source.states),created_unix:source.created_unix,status:source.status,logs:source.logs.slice(-500)};
  const assignments = Object.create(null), inputs = Object.create(null);
  if (['queued','running'].includes(job.status)) { job.status='interrupted'; job.logs.push('Snapshot of an unfinished run. Opening it does not resume execution.'); }
  for (const state of job.states) {
    if (['queued','running'].includes(state.status)) { state.status='interrupted'; state.metrics=null; state.error='No completed result was recorded before this snapshot.'; }
    const entry = lab.manifest.graphs[`${state.code}:${job.config.year}`];
    inputs[state.code]={prepared_graph_sha256:lab.manifest.assets[entry.graph_ref],geometry_sha256:lab.manifest.assets[lab.manifest.geometries[`${state.code}:${job.config.year}`]]};
    if (state.metrics) assignments[state.code]=await lab.api(`/api/runs/${id}/${state.code}/assignments`);
  }
  job.logs=job.logs.slice(-500);
  return parseLabProject(JSON.stringify({schema_version:LAB_PROJECT_SCHEMA,saved_at_utc:new Date().toISOString(),experiment:job,assignments,inputs}));
}

// Independently check stored measurements against hashed prepared inputs. This
// establishes data consistency, not that the claimed engine generated the plan.
export function verifyLabAssignments(graph, state, assignments, config) {
  const m=state.metrics, k=m.district_count, ids=graph.geoids;
  const partisan=config.weights==='partisan'?partisanWeights(graph,config):null;
  const edgeWeight=(u,v,length)=>partisan?partisan.weight(u,v):config.weights==='unweighted'?1:config.weights==='county'&&ids[u].slice(0,5)===ids[v].slice(0,5)?length*Math.max(1,config.alpha_county):length;
  if(partisan){const e=m.weighting_evidence;if(!exact(e,['method','dem_threshold','rep_threshold','strong_tracts','tracts','alpha','baseline','boost','shares_sha256','scope'])||e.method!=='partisan-adaptive-boost'||e.dem_threshold!==config.dem_threshold||e.rep_threshold!==config.rep_threshold||e.strong_tracts!==partisan.strong||e.tracts!==ids.length||!bounded(e.alpha,3,10)||Math.abs(e.alpha-partisan.alpha)>1e-12||e.baseline!=='unit-edge-weight'||e.boost!=='same-strong-lean'||!hash(e.shares_sha256)||e.scope!=='edge-weighting')throw new Error('Project partisan weighting evidence disagrees with inputs or settings.');}
  else if(m.weighting_evidence!=null)throw new Error('Unexpected project weighting evidence.');

  if(['ratio-optimal-area','ratio-optimal-vra'].includes(config.structure)&&Object.hasOwn(config,'metis_objective')&&k>1)verifyRatioRefinement(m.structure_evidence?.metis_refinement,config);
  if(config.structure==='ratio-optimal-vra')verifyVraSection(graph,state,assignments,config);
  if(config.search==='vra-recom')verifyVraPreservation(graph,state,assignments,config);
  if (Object.keys(assignments).length !== ids.length || m.units !== ids.length) throw new Error('Project assignment coverage mismatch.');
  const plan=ids.map(id=>{if(!Object.hasOwn(assignments,id))throw new Error('Project omits a tract.');return assignments[id];});
  const populations=Array(k).fill(0), units=Array(k).fill(0), components=Array(k).fill(0), counties=new Map();
  let total=0, boundary=0, weighted=0;
  for(let i=0;i<ids.length;i++) { const d=plan[i]-1; populations[d]+=graph.population[i]; units[d]++; total+=graph.population[i]; const county=ids[i].slice(0,5);if(!counties.has(county))counties.set(county,new Set());counties.get(county).add(plan[i]); }
  const visited=new Uint8Array(ids.length);
  for(let i=0;i<ids.length;i++)if(!visited[i]) { const d=plan[i], queue=[i];visited[i]=1;components[d-1]++;for(let j=0;j<queue.length;j++)for(const next of graph.adjacency[queue[j]])if(!visited[next]&&plan[next]===d){visited[next]=1;queue.push(next);} }
  for(const [u,v,length] of graph.edges)if(plan[u]!==plan[v]) { boundary+=length;weighted+=edgeWeight(u,v,length); }
  const close=(actual,expected)=>Number.isFinite(actual)&&Math.abs(actual-expected)<=Math.max(1e-8,Math.abs(expected)*1e-10);
  const deviation=populations.map(pop=>(pop/(total/k)-1)*100), max=Math.max(...deviation.map(Math.abs)), contiguous=components.every(c=>c===1), splits=[...counties.values()].filter(ds=>ds.size>1).length;
  if(m.population!==total||!close(m.max_deviation_percent,max)||!close(m.graph_boundary_m,boundary)||!close(m.weighted_boundary,weighted)||m.split_counties!==splits||m.contiguous!==contiguous||m.balance_passed!==(max<=config.balance_tolerance))throw new Error('Project metrics disagree with assignments.');
  for(let i=0;i<k;i++) { const d=m.districts[i];if(d.population!==populations[i]||d.units!==units[i]||d.components!==components[i]||!close(d.deviation_percent,deviation[i]))throw new Error('Project district metrics disagree with assignments.'); }
  if(config.search==='bisection-ensemble'&&k>1&&(m.structure_evidence!=null||Object.hasOwn(config,'metis_objective'))){
    const e=m.structure_evidence;
    if(Object.hasOwn(config,'metis_objective')){
      const r=e?.metis_initialization;
      if(!exact(r,['objective','internal_trials','iterations','scope','internal_trial_selection','post_refinement'])||r.objective!==config.metis_objective||r.internal_trials!==config.metis_trials||r.iterations!==config.iterations||r.scope!=='initial-partition-and-small-region-fallback'||r.internal_trial_selection!=='population-excess-then-edge-cut'||r.post_refinement!=='native-population-rebalance')throw new Error('Project local-bisection evidence disagrees with initialization controls.');
    }
    if(!exact(e,['method','rng','steps_per_split','percentile','selection','rank_rule','target_policy','small_region_fallback','scope','tree_sampler',...(Object.hasOwn(config,'metis_objective')?['metis_initialization']:[])])||e.method!=='local-bisection-ensemble'||e.rng!=='chacha12-u64-v1'||e.steps_per_split!==config.steps||e.percentile!==config.percentile||e.selection!=='initial-and-accepted-unweighted-cut-percentile'||e.rank_rule!=='floor(p*record-count), clamped'||e.target_policy!=='floor-ceil-seat-ratio'||e.small_region_fallback!=='at-most-four-tracts-single-metis'||e.scope!=='each-recursive-split'||e.tree_sampler!=='wilson')throw new Error('Project local-bisection evidence disagrees with requested options.');
  }
  if(config.search==='flip'&&k>1&&m.structure_evidence!=null){
    const e=m.structure_evidence;
    if(!exact(e,['method','rng','steps','percentile','selection','rank_rule','seed_domain','initial_refinement_iterations','population_allowance','nonempty_districts','record_count','selected_rank'])||e.method!=='boundary-flip'||e.rng!=='chacha12-u64-v1'||e.steps!==config.steps||e.percentile!==config.percentile||e.selection!=='initial-and-accepted-unweighted-cut-percentile'||e.rank_rule!=='floor(p*record-count), clamped'||e.seed_domain!=='FLIP_CHAIN_'||e.initial_refinement_iterations!==100||e.population_allowance!=='fraction-of-ideal-district'||e.nonempty_districts!==true||!integer(e.record_count,1,config.steps+1)||e.selected_rank!==Math.min(Math.floor(config.percentile*e.record_count),e.record_count-1)||units.some(n=>n===0))throw new Error('Project boundary-flip evidence disagrees with requested options.');
  }
  if(k===1&&(m.root_split!=null||m.structure_evidence!=null))throw new Error('One-district project must not claim split or search evidence.');
  if(config.structure==='simulated-annealing'&&k>1){
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','rng','objective','steps_per_tract','t0_factor','t_final','initial_refinement_iterations','population_targets','temperature_schedule'])||evidence.method!=='simulated-annealing'||evidence.rng!=='chacha12-u64-v1'||evidence.objective!=='unweighted-edge-cut'||evidence.steps_per_tract!==config.sa_steps_per_tract||evidence.t0_factor!==config.sa_t0_factor||evidence.t_final!==config.sa_t_final||evidence.initial_refinement_iterations!==100||evidence.population_targets!=='floor-ceil-seat-ratio'||evidence.temperature_schedule!=='geometric; T0=max(1,factor*initial-cut); final clamped to [1e-12,T0]'||!contiguous||max>config.balance_tolerance)throw new Error('Project annealing parameters or feasibility disagree with assignments.');
  }
  if(config.search==='smc-percentile'&&k>1){
    const e=m.structure_evidence,n=config.smc_particles,cut=graph.edges.filter(([u,v])=>plan[u]!==plan[v]).length;
    if(!exact(e,['method','rng','objective','base_seed','sampler_seed','particles','percentile','resample_threshold','sampler_tolerance','tolerance_basis','selected_particle','selected_edge_cut','resample_count','resample_rounds','ess_trace','ranked_particles'])||e.method!=='smc-percentile'||e.rng!=='chacha12-u64-v1'||e.objective!=='unweighted-edge-cut'||e.base_seed!==String(config.seed)||typeof e.sampler_seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(e.sampler_seed)||BigInt(e.sampler_seed)>((1n<<64n)-1n)||e.particles!==n||e.percentile!==config.percentile||e.resample_threshold!==config.smc_resample_threshold||e.sampler_tolerance!==0.005||e.tolerance_basis!=='remaining-component-total'||e.selected_edge_cut!==cut||!integer(e.selected_particle,0,n-1)||!integer(e.resample_count,0,k-1)||!Array.isArray(e.resample_rounds)||e.resample_rounds.length!==e.resample_count||!Array.isArray(e.ess_trace)||e.ess_trace.length!==k-1||e.ess_trace.some(v=>!bounded(v,0,n+1e-8))||!Array.isArray(e.ranked_particles)||e.ranked_particles.length!==n)throw new Error('Invalid SMC evidence.');
    const expectedRounds=e.ess_trace.flatMap((ess,i)=>ess<config.smc_resample_threshold*n?[i+1]:[]);
    if(JSON.stringify(expectedRounds)!==JSON.stringify(e.resample_rounds))throw new Error('SMC resampling evidence disagrees with ESS.');
    const seen=new Set();let totalWeight=0,cumulative=0,chosen,lastPositive;
    for(let i=0;i<n;i++){
      const p=e.ranked_particles[i],prev=e.ranked_particles[i-1];
      if(!exact(p,['particle','edge_cut','weight'])||!integer(p.particle,0,n-1)||seen.has(p.particle)||!integer(p.edge_cut,0,graph.edges.length)||!bounded(p.weight,0,1)||(prev&&(p.edge_cut<prev.edge_cut||(p.edge_cut===prev.edge_cut&&p.particle<=prev.particle))))throw new Error('Invalid SMC ranked weight ledger.');
      seen.add(p.particle);totalWeight+=p.weight;
      if(p.weight>0){lastPositive=p;cumulative+=p.weight;if(!chosen&&(config.percentile===0||cumulative>=config.percentile))chosen=p;}
    }
    chosen??=lastPositive;
    if(!close(totalWeight,1)||!chosen||chosen.particle!==e.selected_particle||chosen.edge_cut!==cut)throw new Error('SMC percentile selection disagrees with its weight ledger.');
  }
  if(['capacity-clustering','regionalization'].includes(config.structure)&&k>1){
    const regional=config.structure==='regionalization',evidence=m.structure_evidence,summary=regional?evidence?.summary:evidence;
    const common=['schema_version','method','repair_method','capacity_status','repair_status','population_deviation','edge_cut','parameter_hash'];
    const keys=regional?[...common,'merge_policy','merge_count','hierarchy_depth']:[...common,'seed_method'];
    const cut=graph.edges.filter(([u,v])=>plan[u]!==plan[v]).length;
    if(!exact(summary,keys)||summary.schema_version!==(regional?'bisect-regionalization-summary-v1':'bisect-clustering-summary-v1')||summary.method!==config.structure||summary.capacity_status!=='valid'||!['none','exhaustive-small'].includes(summary.repair_method)||!['not-needed','repaired'].includes(summary.repair_status)||(summary.repair_method==='exhaustive-small')!==(summary.repair_status==='repaired')||!close(summary.population_deviation,max/100)||summary.edge_cut!==cut||!/^sha256:[a-f0-9]{64}$/.test(summary.parameter_hash)||!contiguous||max>config.balance_tolerance||(!regional&&summary.seed_method!=='farthest'))throw new Error('Project clustering evidence disagrees with assignments or options.');
    if(regional){
      if(!exact(evidence,['method','summary','merge_log'])||evidence.method!=='regionalization'||summary.merge_policy!=='adjacent-balanced-agglomerative'||!Array.isArray(evidence.merge_log)||evidence.merge_log.length!==ids.length-k||summary.merge_count!==evidence.merge_log.length)throw new Error('Invalid regionalization merge history.');
      const regions=new Map(graph.population.map((population,id)=>[id,{population,depth:0}])),labels=ids.map((_,i)=>i);
      for(let step=0;step<evidence.merge_log.length;step++){
        const merge=evidence.merge_log[step],left=regions.get(merge?.left_region),right=regions.get(merge?.right_region);
        if(!exact(merge,['step','left_region','right_region','merged_region','merged_population','cut_edges_between'])||merge.step!==step||!left||!right||merge.left_region>=merge.right_region||merge.merged_region!==merge.left_region||merge.merged_population!==left.population+right.population)throw new Error('Invalid regionalization merge history.');
        const between=graph.edges.filter(([u,v])=>(labels[u]===merge.left_region&&labels[v]===merge.right_region)||(labels[v]===merge.left_region&&labels[u]===merge.right_region)).length;
        if(!between||merge.cut_edges_between!==between)throw new Error('Regionalization merge disagrees with graph boundaries.');
        left.population+=right.population;left.depth=Math.max(left.depth,right.depth)+1;regions.delete(merge.right_region);
        for(let i=0;i<labels.length;i++)if(labels[i]===merge.right_region)labels[i]=merge.left_region;
      }
      if(summary.hierarchy_depth!==Math.max(...[...regions.values()].map(region=>region.depth)))throw new Error('Regionalization hierarchy depth mismatch.');
      if(summary.repair_method==='none'){
        const district=new Map([...regions.keys()].sort((a,b)=>a-b).map((region,i)=>[region,i+1]));
        if(labels.some((region,i)=>plan[i]!==district.get(region)))throw new Error('Regionalization merges disagree with final assignments.');
      }
    }
  }
  if(config.structure==='flow-construction'&&k>1){
    const evidence=m.structure_evidence;
    const cut=graph.edges.filter(([u,v])=>plan[u]!==plan[v]).length;
    if(!exact(evidence,['schema_version','method','seed_method','cost_method','repair_method','status','population_deviation','edge_cut','seeds','infeasibility_witness','parameter_hash'])||evidence.schema_version!=='bisect-flow-summary-v1'||evidence.method!=='flow-construction'||evidence.seed_method!=='farthest'||evidence.cost_method!=='edge-cut'||evidence.repair_method!==(config.flow_repair??'bfs')||evidence.status!=='valid'||!close(evidence.population_deviation,max/100)||evidence.edge_cut!==cut||!Array.isArray(evidence.seeds)||evidence.seeds.length!==k||new Set(evidence.seeds).size!==k||evidence.seeds.some(seed=>!integer(seed,0,ids.length-1))||evidence.infeasibility_witness!==null||!/^sha256:[a-f0-9]{64}$/.test(evidence.parameter_hash)||!contiguous||max>config.balance_tolerance)throw new Error('Project flow evidence disagrees with assignments or options.');
  }
  if(config.search==='convergence'&&k>1){
    const evidence=m.structure_evidence;
    if(Object.hasOwn(config,'metis_objective')){
      const r=evidence?.metis_refinement;
      if(!exact(r,['objective','internal_trials_per_candidate','iterations','scope','internal_trial_selection','post_refinement'])||r.objective!==config.metis_objective||r.internal_trials_per_candidate!==config.metis_trials||r.iterations!==config.iterations||r.scope!=='each-floor-ceil-tree-node'||r.internal_trial_selection!=='population-excess-then-edge-cut'||r.post_refinement!=='native-population-rebalance')throw new Error('Project convergence refinement evidence disagrees with options.');
    }
    let score=0;
    for(const [u,v,length]of graph.edges)if(plan[u]!==plan[v]){
      let first=1,count=k;
      while(true){const left=Math.floor(count/2),middle=first+left;
        if((plan[u]<middle)!==(plan[v]<middle)){const weight=edgeWeight(u,v,length);score+=weight/Math.sqrt(Math.min(left,count-left));break;}
        if(plan[u]<middle)count=left;else{first=middle;count-=left;}
      }
    }
    if(config.structure!=='standard-bisect'||!exact(evidence,['schema_version','method','objective','base_seed','selected_seed','selected_index','attempted','rejected','threshold','consecutive_non_improving','seed_limit','halt','best_normalized_cut',...(Object.hasOwn(config,'metis_objective')?['metis_refinement']:[])])||evidence.schema_version!=='bisect-convergence-summary-v1'||evidence.method!=='convergence'||evidence.objective!=='recursive-normalized-weighted-cut'||evidence.base_seed!==String(config.seed)||!integer(evidence.attempted,1,config.steps)||!integer(evidence.selected_index,0,evidence.attempted-1)||evidence.selected_seed!==((BigInt(config.seed)+BigInt(evidence.selected_index))%(1n<<64n)).toString()||!integer(evidence.rejected,0,evidence.attempted-1)||evidence.threshold!==config.seeds||evidence.seed_limit!==config.steps||evidence.consecutive_non_improving!==evidence.attempted-1-evidence.selected_index||!close(evidence.best_normalized_cut,score)||!contiguous||max>config.balance_tolerance||(evidence.halt==='threshold'?evidence.consecutive_non_improving!==config.seeds:evidence.halt!=='seed-limit'||evidence.attempted!==config.steps||evidence.consecutive_non_improving>=config.seeds))throw new Error('Project convergence evidence disagrees with assignments, limits or options.');
  }
  if(config.structure==='spectral'&&k>1){
    const evidence=m.structure_evidence;
    if(!exact(evidence,['schema_version','method','max_iters','tolerance','k','edge_cut','nodes'])||evidence.schema_version!=='bisect-spectral-run-summary-v1'||evidence.method!=='spectral'||evidence.max_iters!==config.steps||evidence.tolerance!==config.balance_tolerance/100||evidence.k!==k||evidence.edge_cut!==graph.edges.filter(([u,v])=>plan[u]!==plan[v]).length||!Array.isArray(evidence.nodes)||evidence.nodes.length!==k-1)throw new Error('Project spectral evidence disagrees with assignments or options.');
    let index=0;
    function checkNode(first,count){
      if(count===1)return;
      const leftCount=Math.floor(count/2),middle=first+leftCount,last=first+count,node=evidence.nodes[index++];
      let leftPop=0,regionPop=0,cut=0;
      for(let i=0;i<plan.length;i++)if(plan[i]>=first&&plan[i]<last){regionPop+=graph.population[i];if(plan[i]<middle)leftPop+=graph.population[i];}
      for(const [u,v]of graph.edges)if(plan[u]>=first&&plan[u]<last&&plan[v]>=first&&plan[v]<last&&(plan[u]<middle)!==(plan[v]<middle))cut++;
      const fraction=leftCount/count,deviation=Math.max(Math.abs(leftPop-regionPop*fraction)/(regionPop*fraction),Math.abs(regionPop-leftPop-regionPop*(1-fraction))/(regionPop*(1-fraction)));
      if(!exact(node,['schema_version','method','sweep','iterations','converged','edge_cut','population_deviation','tolerance','target_fraction','parameter_hash'])||node.schema_version!=='bisect-spectral-summary-v1'||node.method!=='spectral'||node.sweep!=='population-balanced-min-cut'||!integer(node.iterations,1,config.steps)||typeof node.converged!=='boolean'||node.edge_cut!==cut||!close(node.population_deviation,deviation)||node.tolerance!==config.balance_tolerance/100||!close(node.target_fraction,fraction)||!/^sha256:[a-f0-9]{64}$/.test(node.parameter_hash)||deviation>node.tolerance+1e-10)throw new Error('Project spectral node measurements disagree with assignments.');
      checkNode(first,leftCount);checkNode(middle,count-leftCount);
    }
    checkNode(1,k);
  }
  if(config.structure==='prime-factor'&&k>1){
    const evidence=m.structure_evidence;
    const factors=[];let remaining=k;
    for(let prime=2;prime<=remaining;prime++)while(remaining%prime===0){factors.push(prime);remaining/=prime;}
    function depth(count){
      if(count<=1)return 0;if(count<=3)return 1;
      let largest=0,rest=count;
      for(let prime=2;prime<=rest;prime++)while(rest%prime===0){largest=prime;rest/=prime;}
      return largest===count?1+Math.max(depth(Math.floor(count/2)),depth(Math.ceil(count/2))):1+depth(count/largest);
    }
    const treeDepth=depth(k);let integerCut=0;
    for(const [u,v,length]of graph.edges)if(plan[u]!==plan[v]){
      const weight=edgeWeight(u,v,length);
      integerCut+=Math.min(2147483647,Math.max(1,Math.round(weight*100)));
    }
    if(!exact(evidence,['method','factor_sequence','tree_depth','per_level_tolerance','research_balance_limit_percent','total_edge_cut_integer','cache_hits','seed','split_prescription'])||evidence.method!=='apportion-regions'||JSON.stringify(evidence.factor_sequence)!==JSON.stringify(factors)||evidence.tree_depth!==treeDepth||!close(evidence.per_level_tolerance,Math.max(config.balance_tolerance/100/(treeDepth+1),0.001))||evidence.research_balance_limit_percent!==3||evidence.total_edge_cut_integer!==integerCut||!integer(evidence.cache_hits,0,1000000)||evidence.seed!==config.seed||evidence.split_prescription!=='largest-prime-first; prime > 3 uses floor/ceil binary'||max>3)throw new Error('Project ApportionRegions evidence disagrees with assignments or options.');
  }
  if(config.structure==='ratio-optimal'&&Object.hasOwn(config,'metis_objective')&&k>1){
    const e=m.structure_evidence;
    if(!exact(e,['method','refinement_objective','internal_trials_per_candidate','refinement_iterations','scope','internal_trial_selection','candidate_selection','ratio_selection','seeds_per_ratio','seed_schedule','post_refinement','partial_assignment_fallback'])||e.method!=='geosection-metis'||e.refinement_objective!==config.metis_objective||e.internal_trials_per_candidate!==config.metis_trials||e.refinement_iterations!==config.iterations||e.scope!=='each-recursive-ratio-search-and-two-seat-shortcut'||e.internal_trial_selection!=='population-excess-then-edge-cut'||e.candidate_selection!=='minimum-original-weighted-cut-per-ratio'||e.ratio_selection!=='weighted-cut-divided-by-sqrt-min-child-seats'||e.seeds_per_ratio!==(config.search==='multi'?config.seeds:1)||e.seed_schedule!=='base-plus-candidate-index-wrapping-u64-at-each-node'||e.post_refinement!=='native-population-rebalance'||e.partial_assignment_fallback!=='legacy-standard-cut-one-trial')throw new Error('Project GeoSection refinement evidence disagrees with options.');
  }
  if(config.structure==='standard-bisect'&&['single','multi'].includes(config.search)&&Object.hasOwn(config,'metis_objective')&&k>1){
    const e=m.structure_evidence;
    if(!exact(e,['method','refinement_objective','internal_trials_per_candidate','refinement_iterations','scope','internal_trial_selection','candidate_selection','candidates_per_node','seed_schedule','post_refinement'])||e.method!=='recursive-metis'||e.refinement_objective!==config.metis_objective||e.internal_trials_per_candidate!==config.metis_trials||e.refinement_iterations!==config.iterations||e.scope!=='each-floor-ceil-tree-node'||e.internal_trial_selection!=='population-excess-then-edge-cut'||e.candidate_selection!==(config.search==='multi'?'minimum-weighted-cut-among-balanced-contiguous':'single-candidate')||e.candidates_per_node!==(config.search==='multi'?config.seeds:1)||e.seed_schedule!=='base-plus-candidate-index-wrapping-u64'||e.post_refinement!=='native-population-rebalance')throw new Error('Project recursive METIS evidence disagrees with options.');
  }
  if(config.structure==='standard-bisect'&&config.search==='percentile'&&Object.hasOwn(config,'metis_objective')&&k>1){
    const e=m.structure_evidence,keys=['method','refinement_objective','internal_trials_per_candidate','refinement_iterations','seed_count','rank','selected_seed_index','selected_edge_cut','cuts_by_seed','seed_walk','selection','scope','internal_trial_selection'];
    if(!exact(e,keys)||e.method!=='percentile-metis'||e.refinement_objective!==config.metis_objective||e.internal_trials_per_candidate!==config.metis_trials||e.refinement_iterations!==config.iterations||e.seed_count!==config.seeds||e.rank!==Math.min(Math.floor(config.percentile*config.seeds),config.seeds-1)||!Array.isArray(e.cuts_by_seed)||e.cuts_by_seed.length!==config.seeds||e.cuts_by_seed.some(c=>!integer(c,0,graph.edges.length))||!integer(e.selected_seed_index,0,config.seeds-1)||e.seed_walk!=='PERCENTILE_SWEEP_V1-u64-le-sha256'||e.selection!=='unweighted-edge-cut-then-seed-index'||e.scope!=='full-plans-from-prescribed-floor-ceil-tree'||e.internal_trial_selection!=='population-excess-then-edge-cut')throw new Error('Project percentile METIS evidence disagrees with options.');
    const indices=e.cuts_by_seed.map((_,i)=>i).sort((a,b)=>e.cuts_by_seed[a]-e.cuts_by_seed[b]||a-b);
    const cut=graph.edges.filter(([u,v])=>plan[u]!==plan[v]).length;
    if(e.selected_seed_index!==indices[e.rank]||e.selected_edge_cut!==e.cuts_by_seed[e.selected_seed_index]||e.selected_edge_cut!==cut)throw new Error('Project percentile selection disagrees with assignment cut or candidate ranks.');
  }
  if(config.structure==='nway'&&k>1&&(m.structure_evidence!=null||Object.hasOwn(config,'metis_objective'))){
    const e=m.structure_evidence;
    if(!exact(e,['method','refinement_objective','internal_trials','refinement_iterations','trial_selection','edge_weight_scaling','contiguity','min_connectivity'])||e.method!=='nway-metis'||e.refinement_objective!==(config.metis_objective??'cut')||e.internal_trials!==(config.metis_trials??1)||e.refinement_iterations!==config.iterations||e.trial_selection!=='population-excess-then-edge-cut'||e.edge_weight_scaling!=='metres-times-100-truncated-minimum-1'||e.contiguity!==true||e.min_connectivity!==true)throw new Error('Project n-way METIS evidence disagrees with options.');
  }
  if(config.structure==='moving-knife' && k>1 && (m.structure_evidence!=null || Object.hasOwn(config,'mka_orientations'))) {
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','requested_metric','effective_metric','orientations_per_split','scoring_boundary'])||evidence.method!=='moving-knife'||evidence.requested_metric!==(config.mka_metric??'reock')||evidence.effective_metric!=='reock'||evidence.orientations_per_split!==Math.max(1,config.mka_orientations??36)||evidence.scoring_boundary!=='Native Polsby option currently falls back to Reock; no perimeter scoring is claimed.')throw new Error('Project moving-knife evidence disagrees with requested options.');
  }
  if(config.structure==='compact-polsby' && k>1 && (m.structure_evidence!=null || Object.hasOwn(config,'compact_epsilon'))) {
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','epsilon','seeds_per_split','seed_policy','selection'])||evidence.method!=='compact-polsby'||evidence.epsilon!==(config.compact_epsilon??0.05)||evidence.seeds_per_split!==(config.search==='multi'?config.seeds:1)||evidence.seed_policy!=='fixed-1-through-budget'||evidence.selection!=='geometric-mean-polsby-among-near-minimum-weighted-cut')throw new Error('Project CompactBisect evidence disagrees with requested options.');
  }
  if(config.structure==='ratio-optimal-area'&&(Object.hasOwn(config,'area_init')||Object.hasOwn(config,'metis_objective'))&&k>1) {
    const evidence=m.structure_evidence,moving=config.area_init==='moving-knife';
    if(!exact(evidence,['method','initialization','orientations','direction_radians','directional_lambda','scope',...(Object.hasOwn(config,'metis_objective')?['metis_refinement']:[])])||evidence.method!=='areasection-initialization'||evidence.initialization!==(config.area_init??'ratio-optimal')||evidence.orientations!==(moving?180:0)||evidence.directional_lambda!==(moving?1:0)||evidence.scope!=='root-only'||(moving?!bounded(evidence.direction_radians,0,Math.PI):evidence.direction_radians!==null))throw new Error('Project AreaSection initialization evidence disagrees with requested options.');
  }
  if(config.search.startsWith('short-burst')&&Object.hasOwn(config,'burst_length')&&k>1) {
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','chain','burst_length','n_bursts','total_proposals','selection','rng'])||evidence.method!=='short-burst'||evidence.chain!==config.search||evidence.burst_length!==config.burst_length||evidence.n_bursts!==config.n_bursts||evidence.total_proposals!==config.burst_length*config.n_bursts||evidence.selection!=='endpoint-unweighted-cut-percentile'||evidence.rng!=='chacha12-u64-v1')throw new Error('Project short-burst evidence disagrees with requested options.');
  }
  if(config.search==='parallel-tempering'&&Object.hasOwn(config,'pt_replicas')&&k>1) {
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','replicas','swap_interval','cold_tolerance','hot_tolerance','steps','selection','rng','swap_support'])||evidence.method!=='parallel-tempering'||evidence.replicas!==config.pt_replicas||evidence.swap_interval!==config.pt_swap_interval||evidence.cold_tolerance!==config.pt_cold_tol||evidence.hot_tolerance!==config.pt_hot_tol||evidence.steps!==config.seeds||evidence.selection!=='cold-record-unweighted-cut-percentile'||evidence.rng!=='chacha12-u64-v1'||evidence.swap_support!=='both-receiving-tolerances')throw new Error('Project parallel-tempering evidence disagrees with requested options.');
  }
  if(['forest-recom','merge-split'].includes(config.search)&&k>1&&m.structure_evidence!=null) {
    const evidence=m.structure_evidence,domains=config.search==='forest-recom'?['FR_FORWARD_','FR_REVERSE_']:['MS_STEP_','MS_REVERSE_'];
    if(!exact(evidence,['method','rng','steps','percentile','selection','rank_rule','seed_domains'])||evidence.method!==config.search||evidence.rng!=='chacha12-u64-v1'||evidence.steps!==config.steps||evidence.percentile!==config.percentile||evidence.selection!=='initial-and-accepted-unweighted-cut-percentile'||evidence.rank_rule!=='floor(p*record-count), clamped'||JSON.stringify(evidence.seed_domains)!==JSON.stringify(domains))throw new Error('Project standalone chain evidence disagrees with requested options.');
  }
  const root=m.root_split;
  if(config.structure==='centroidal-voronoi' && k>1 && (m.structure_evidence!=null || Object.hasOwn(config,'cvd_iters'))) {
    const evidence=m.structure_evidence;
    if(!exact(evidence,['method','distance_metric','iterations_per_split','rng','seed_index'])||evidence.method!=='centroidal-voronoi'||evidence.distance_metric!==(config.cvd_metric??'geographic')||evidence.iterations_per_split!==Math.max(1,config.cvd_iters??50)||evidence.rng!=='chacha12-u64-v1'||evidence.seed_index!=='full-u64-modulo-v1')throw new Error('Project CVD evidence disagrees with requested options.');
  }
  if(config.structure==='ratio-optimal-area'&&k>1&&!record(root))throw new Error('AreaSection project omits root constraint evidence.');
  if(root!==undefined&&root!==null){
    if(!record(root)||!integer(root.left_districts,1,k-1)||root.right_districts!==k-root.left_districts||root.constraint_scope!=='root-only'||root.base_seed!==config.seed||root.seeds_per_ratio!==(config.search==='multi'?config.seeds:1))throw new Error('Invalid project root split.');
    const areaTotal=graph.areas.reduce((sum,area)=>sum+area,0);
    let areaLeft=0,popLeft=0,rootCost=0;
    for(let i=0;i<ids.length;i++)if(plan[i]<=root.left_districts){areaLeft+=graph.areas[i];popLeft+=graph.population[i];}
    for(const [u,v,length]of [...graph.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1]))if((plan[u]<=root.left_districts)!==(plan[v]<=root.left_districts))rootCost+=edgeWeight(u,v,length);
    const active=config.structure==='ratio-optimal-area', passed=areaLeft<=areaTotal*0.5*config.area_swing+1e-6&&areaTotal-areaLeft<=areaTotal*0.5*config.area_swing+1e-6;
    if(root.population_total!==total||root.population_left!==popLeft||!close(root.population_target_left,root.left_districts/k)||!close(root.area_total_m2,areaTotal)||!close(root.area_left_m2,areaLeft)||!close(root.weighted_boundary,rootCost)||(areaTotal>0?!close(root.area_fraction_left,areaLeft/areaTotal):root.area_fraction_left!==null)||root.area_constraint_active!==active||root.area_swing!==(active?config.area_swing:null)||root.population_multiplier!==(active?1.001:null)||root.area_within_requested_swing!==(active?passed:null))throw new Error('Project root-split metrics disagree with assignments.');
    if(active&&(!passed||popLeft>total*(root.left_districts/k)*1.001+1e-6||total-popLeft>total*(root.right_districts/k)*1.001+1e-6))throw new Error('Project root split violates its dual constraints.');
  }
}

function verifyRatioRefinement(r,config){
  if(!exact(r,['objective','internal_trials','iterations','scope','internal_trial_selection','candidate_selection','seeds_per_ratio','seed_schedule','post_refinement','partial_assignment_fallback'])||r.objective!==config.metis_objective||r.internal_trials!==config.metis_trials||r.iterations!==config.iterations||r.scope!=='each-recursive-ratio-search-and-two-seat-shortcut'||r.internal_trial_selection!=='population-excess-then-edge-cut'||r.candidate_selection!=='minimum-original-weighted-cut-per-ratio'||r.seeds_per_ratio!==(config.search==='multi'?config.seeds:1)||r.seed_schedule!=='base-plus-candidate-index-wrapping-u64-at-each-node'||r.post_refinement!=='native-population-rebalance'||r.partial_assignment_fallback!=='legacy-standard-cut-one-trial')throw new Error('Project ratio refinement evidence disagrees with options.');
}

function verifyVraSection(graph,state,assignments,config){
  const input=validateDemographicInput(config.demographics[state.code??graph.state],graph),k=state.metrics.district_count,e=state.metrics.structure_evidence,root=state.metrics.root_split;
  if(input.basis!=='total-population')throw new Error('VRASection requires total-population fractions.');
  if(k===1){if(e!=null)throw new Error('One-district project must not claim VRASection execution.');return;}
  if(!record(root)||!integer(root.left_districts,1,Math.floor(k/2)))throw new Error('VRASection project omits root split evidence.');
  const mass=graph.geoids.map((id,i)=>input.minority_fractions[id]*graph.population[i]),total=mass.reduce((sum,v)=>sum+v,0);
  let left=0,cut=0;
  for(let i=0;i<mass.length;i++)if(assignments[graph.geoids[i]]<=root.left_districts)left+=mass[i];
  for(const [u,v,length]of [...graph.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1]))if((assignments[graph.geoids[u]]<=root.left_districts)!==(assignments[graph.geoids[v]]<=root.left_districts))cut+=config.weights==='unweighted'?1:config.weights==='county'&&graph.geoids[u].slice(0,5)===graph.geoids[v].slice(0,5)?length*Math.max(1,config.alpha_county):length;
  const alignment=total>0?Math.abs(left/total-0.5)*2:0,normalised=cut/Math.sqrt(Math.min(root.left_districts,k-root.left_districts)),score=normalised-config.w_vra*alignment*Math.max(normalised,1);
  const close=(a,b)=>Number.isFinite(a)&&Math.abs(a-b)<=1e-9*Math.max(1,Math.abs(b));
  if(!exact(e,['method','scope','basis','aggregation','w_vra','minority_mass_total','minority_mass_left','minority_share_left','alignment','normalised_cut','selection_score','selection_policy','ratio_count','root_shortcut','tie_policy','demographics_sha256',...(Object.hasOwn(config,'metis_objective')?['metis_refinement']:[])])||e.method!=='vra-section'||e.scope!=='root-only'||e.basis!==input.basis||e.aggregation!=='fraction-times-graph-population'||e.w_vra!==config.w_vra||!close(e.minority_mass_total,total)||!close(e.minority_mass_left,left)||(total>0?!close(e.minority_share_left,left/total):e.minority_share_left!==null)||!close(e.alignment,alignment)||!close(e.normalised_cut,normalised)||!close(e.selection_score,score)||e.selection_policy!=='minimum-weighted-cut-per-ratio-then-alignment-adjusted-ratio'||e.ratio_count!==Math.floor(k/2)||e.root_shortcut!==(k===2&&config.search==='single')||e.tie_policy!=='strict-first-minimum'||!hash(e.demographics_sha256))throw new Error('VRASection scoring evidence disagrees with demographic data, assignments or options.');
}

function verifyVraPreservation(graph,state,assignments,config){
  const input=validateDemographicInput(config.demographics[state.code??graph.state],graph),k=state.metrics.district_count,e=state.metrics.structure_evidence;
  if(k===1){if(e!=null)throw new Error('One-district project must not claim VRA sampler execution.');return;}
  const r=e?.run;
  if(!exact(e,['method','rng','steps','threshold','basis','minority_aggregation','population_tolerance_percent','selection','rank_rule','demographics_sha256','run'])||e.method!=='vra-recom'||e.rng!=='chacha12-u64-v1'||e.steps!==config.steps||e.threshold!==config.vra_threshold||e.basis!==input.basis||e.minority_aggregation!=='unweighted-tract-mean'||e.population_tolerance_percent!==0.5||e.selection!=='initial-and-accepted-unweighted-cut-percentile'||e.rank_rule!=='floor(p*record-count), clamped'||!hash(e.demographics_sha256)||!exact(r,['initial_assignment','protected_districts','proposals','accepted_moves','mh_rejections','minority_rejections','retained_records','selected_rank']))throw new Error('Invalid VRA preservation evidence.');
  if(!Array.isArray(r.initial_assignment)||r.initial_assignment.length!==graph.geoids.length||!Array.isArray(r.protected_districts)||r.proposals!==config.steps||!integer(r.accepted_moves,0,r.proposals)||!integer(r.mh_rejections,0,r.proposals)||!integer(r.minority_rejections,0,r.proposals)||r.accepted_moves+r.mh_rejections+r.minority_rejections!==r.proposals||r.retained_records!==r.accepted_moves+1||r.selected_rank!==Math.min(Math.floor(config.percentile*r.retained_records),r.retained_records-1))throw new Error('VRA proposal accounting disagrees with run settings.');
  const initial=Object.fromEntries(graph.geoids.map((id,i)=>[id,r.initial_assignment[i]])),initialMeans=tractMeanFractions(graph,initial,input,k),finalMeans=tractMeanFractions(graph,assignments,input,k);
  const protectedIds=initialMeans.flatMap((fraction,i)=>fraction>=config.vra_threshold?[i+1]:[]);
  if(JSON.stringify(protectedIds)!==JSON.stringify(r.protected_districts)||protectedIds.some(d=>finalMeans[d-1]<config.vra_threshold)||(config.steps===0&&graph.geoids.some(id=>assignments[id]!==initial[id])))throw new Error('VRA protected districts disagree with demographic fractions or assignments.');
}

export async function verifyDemographicEvidence(graph,state,config,assignments){
  if(['proportional-bisect','proportional-section'].includes(config.structure))await verifyElectionResult(graph,{...config,districts:state.metrics.district_count},config.elections[state.code??graph.state],{metrics:state.metrics,assignments});
  else if(state.metrics.election_input_evidence!=null)throw new Error('Unexpected election execution evidence.');
  if(config.weights==='partisan'&&state.metrics.weighting_evidence.shares_sha256!==await partisanIdentity(config.partisans[state.code??graph.state]))throw new Error('Project partisan input identity mismatch.');
  if(config.search!=='vra-recom'&&config.structure!=='ratio-optimal-vra')return;
  const input=validateDemographicInput(config.demographics[state.code??graph.state],graph);
  if(state.metrics.district_count>1&&state.metrics.structure_evidence.demographics_sha256!==await demographicIdentity(input))throw new Error('Project demographic identity mismatch.');
}

function storage(mode, action) {
  return new Promise((resolve,reject)=>{
    const open=indexedDB.open('bisect-laboratory',1);open.onupgradeneeded=()=>open.result.createObjectStore('projects');open.onerror=()=>reject(open.error);
    open.onsuccess=()=>{const db=open.result,tx=db.transaction('projects',mode),request=action(tx.objectStore('projects'));tx.oncomplete=()=>{db.close();resolve(request.result);};tx.onabort=tx.onerror=()=>{db.close();reject(tx.error||request.error);};};
  });
}
export function writeLabAutosave(project) { return storage('readwrite',store=>store.put(validateLabProject(project),'current')); }
export async function readLabAutosave() { const project=await storage('readonly',store=>store.get('current'));return project?validateLabProject(project):null; }

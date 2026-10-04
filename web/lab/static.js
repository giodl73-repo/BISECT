// Seeds above the JS integer range travel as canonical decimal strings.
export function parseSeed(value){
  if(typeof value==='number'){
    if(!Number.isSafeInteger(value)||value<0||Object.is(value,-0))throw new Error('Seed must be an exact unsigned integer.');
    return value;
  }
  if(typeof value!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(value))throw new Error('Seed must be a canonical decimal unsigned integer.');
  const n=BigInt(value);
  if(n>((1n<<64n)-1n))throw new Error('Seed exceeds 18446744073709551615.');
  return n<=BigInt(Number.MAX_SAFE_INTEGER)?Number(n):value;
}
export function isSeed(value){try{return parseSeed(value)===value;}catch{return false;}}
// Static adapter. All requests are reads of published, hashed assets.
export function annealingOptions(config){return config.structure==='simulated-annealing'?{sa_steps_per_tract:config.sa_steps_per_tract,sa_t0_factor:config.sa_t0_factor,sa_t_final:config.sa_t_final}:{};}
export function engineOptions(config){return {...(config.structure==='proportional-section'&&Object.hasOwn(config,'proportional_eta')?{proportional_eta:config.proportional_eta}:{}),...(config.weights==='partisan'?{dem_threshold:config.dem_threshold,rep_threshold:config.rep_threshold}:{}),...(['nway','standard-bisect','ratio-optimal','ratio-optimal-area','ratio-optimal-vra'].includes(config.structure)&&Object.hasOwn(config,'metis_objective')?{metis_objective:config.metis_objective,metis_trials:config.metis_trials}:{}),...(config.structure==='ratio-optimal-vra'?{w_vra:config.w_vra}:{}),...(config.search==='vra-recom'?{vra_threshold:config.vra_threshold}:{}),...(config.search==='parallel-tempering'&&Object.hasOwn(config,'pt_replicas')?{pt_replicas:config.pt_replicas,pt_swap_interval:config.pt_swap_interval,pt_cold_tol:config.pt_cold_tol,pt_hot_tol:config.pt_hot_tol}:{}),...(config.search?.startsWith('short-burst')&&Object.hasOwn(config,'burst_length')?{burst_length:config.burst_length,n_bursts:config.n_bursts}:{}),...(config.structure==='ratio-optimal-area'&&Object.hasOwn(config,'area_init')?{area_init:config.area_init}:{}),...(config.structure==='compact-polsby'&&Object.hasOwn(config,'compact_epsilon')?{compact_epsilon:config.compact_epsilon}:{}),...(config.structure==='moving-knife'&&Object.hasOwn(config,'mka_orientations')?{mka_orientations:config.mka_orientations,mka_metric:config.mka_metric}:{}),...(config.structure==='centroidal-voronoi'&&Object.hasOwn(config,'cvd_iters')?{cvd_iters:config.cvd_iters,cvd_metric:config.cvd_metric}:{}),...annealingOptions(config),...(config.search==='smc-percentile'?{smc_particles:config.smc_particles,smc_resample_threshold:config.smc_resample_threshold}:{}),...(config.structure==='flow-construction'&&Object.hasOwn(config,'flow_repair')?{flow_repair:config.flow_repair}:{})};}
export function effectiveConfig(config, districtCount) {
  const search = config.search;
  const budget = config.structure==='nway'&&Object.hasOwn(config,'metis_trials')?config.metis_trials:search.startsWith('short-burst')&&Object.hasOwn(config,'burst_length')?config.burst_length*config.n_bursts:config.structure==='compact-polsby'&&Object.hasOwn(config,'compact_epsilon')?(search==='multi'?config.seeds:1):config.structure==='moving-knife'&&Object.hasOwn(config,'mka_orientations')?Math.max(1,config.mka_orientations):config.structure==='centroidal-voronoi'&&Object.hasOwn(config,'cvd_iters')?Math.max(1,config.cvd_iters):search==='smc-percentile'?config.smc_particles:config.structure==='simulated-annealing'?config.sa_steps_per_tract:config.structure==='spectral'?config.steps:search === 'single' ? 0 : ['multi','percentile','parallel-tempering'].includes(search) ? config.seeds : search.startsWith('short-burst') ? Math.ceil(config.steps / 20) * 20 : config.steps;
  return {...engineOptions(config),...(search==='convergence'?{convergence_threshold:config.seeds}:{}),year:config.year,chamber:config.chamber,district_count:districtCount,structure:config.structure,weights:config.weights,search,seed:config.structure==='compact-polsby'&&Object.hasOwn(config,'compact_epsilon')?0:['proportional-section','spectral','flow-construction','capacity-clustering','regionalization'].includes(config.structure)?0:config.seed,budget,
    percentile:['single','multi'].includes(search) ? 0 : config.percentile,
    alpha_county:config.weights === 'county' ? Math.max(1,config.alpha_county) : 0,
    balance_tolerance:config.balance_tolerance,iterations:search==='flip'?100:config.structure==='moving-knife'&&Object.hasOwn(config,'mka_orientations')?0:config.structure==='centroidal-voronoi'&&Object.hasOwn(config,'cvd_iters')?0:search==='smc-percentile'?0:config.structure==='simulated-annealing'?100:['spectral','flow-construction','capacity-clustering','regionalization'].includes(config.structure)?0:config.iterations,area_swing:config.structure === 'ratio-optimal-area' ? config.area_swing : 0};
}
const same = (a,b) => Object.keys(a).length === Object.keys(b).length && Object.keys(a).every(key => a[key] === b[key]);
export class StaticCatalog {
  constructor(manifest, fetcher = path => fetch(path)) {
    if (manifest.schema_version !== 1) throw new Error('Unsupported saved catalog version.');
    this.manifest = manifest; this.fetcher = fetcher; this.assets = new Map(); this.jobs = new Map(manifest.runs.map(job => [job.id,job])); this.sequence = 0;
    for (const [path,hash] of Object.entries(manifest.assets)) {
      if (!/^assets\/[a-f0-9]{64}\.json$/.test(path) || path !== `assets/${hash}.json`) throw new Error('Invalid catalog asset reference.');
    }
  }
  districtCount(code, config) {
    return config.districts ?? this.manifest.catalog.states.find(state => state.code === code)?.years.find(year => year.year === config.year)?.districts[config.chamber];
  }
  find(code, config) {
    const expected = effectiveConfig(config,this.districtCount(code,config));
    return [...this.manifest.runs].sort((a,b) => b.created_unix - a.created_unix || b.id.localeCompare(a.id)).flatMap(job => job.states.map(state => ({job,state})))
      .find(({state}) => state.code === code && state.status === 'completed' && same(expected,state.effective_config));
  }
  coverage(config) { return config.states.filter(code => this.find(code,config)).length; }
  async asset(path) {
    if (!Object.hasOwn(this.manifest.assets,path)) throw new Error('Asset is not declared in the catalog.');
    if (!this.assets.has(path)) this.assets.set(path,(async () => {
      const response = await this.fetcher(path);
      if (!response.ok) throw new Error('Saved asset could not be loaded.');
      const bytes = await response.arrayBuffer();
      const hash = [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b => b.toString(16).padStart(2,'0')).join('');
      if (hash !== this.manifest.assets[path]) throw new Error('Saved asset failed its SHA-256 integrity check.');
      return JSON.parse(new TextDecoder().decode(bytes));
    })());
    return this.assets.get(path);
  }
  async map(state,year) {
    const geometry = structuredClone(await this.asset(state.geometry_ref));
    const assignments = await this.asset(state.assignments_ref);
    if (geometry.state !== state.code || geometry.year !== year) throw new Error('Saved geometry belongs to a different state or year.');
    const seen = new Set();
    for (const feature of geometry.features) {
      const geoid = feature.properties.geoid;
      if (!/^\d{11}$/.test(geoid) || seen.has(geoid)) throw new Error('Invalid or duplicate saved tract GEOID.');
      seen.add(geoid); const district = assignments[geoid] ?? null;
      if (district !== null && (!Number.isInteger(district) || district < 1 || district > state.metrics.district_count)) throw new Error('Invalid saved district assignment.');
      feature.properties.district = district;
    }
    if (Object.keys(assignments).some(geoid => !seen.has(geoid)) || Object.keys(assignments).length !== state.metrics.units) throw new Error('Saved map does not cover every assigned unit.');
    return geometry;
  }
  select(config) {
    const states = config.states.map(code => {
      const match = this.find(code,config);
      return match ? {...structuredClone(match.state),source_run_id:match.job.id} : {code,status:'unavailable',metrics:null,error:'This exact configuration has not been precomputed for this state. Choose a saved experiment or generate it in the local laboratory.',elapsed_seconds:0,command:[]};
    });
    const completed = states.filter(state => state.metrics).length;
    const job = {id:`saved-selection-${++this.sequence}`,config:structuredClone(config),states,created_unix:0,status:completed === states.length ? 'completed' : completed ? 'partial' : 'unavailable',logs:[`Loaded ${completed}/${states.length} saved state results. No engine was executed.`]};
    this.jobs.set(job.id,job); return job;
  }
  async api(path,options = {}) {
    if (options.method === 'POST') {
      if (path !== '/api/runs') throw new Error('Static catalogs cannot execute or cancel engine jobs.');
      return this.select(JSON.parse(options.body));
    }
    if (path === '/api/catalog') return this.manifest.catalog;
    if (path === '/api/runs') return [...this.jobs.values()].reverse().map(job => ({id:job.id,name:job.config.name,status:job.status,mode:job.config.mode,year:job.config.year,structure:job.config.structure,weights:job.config.weights,search:job.config.search,state_count:job.states.length,completed:job.states.filter(state => state.metrics).length,created_unix:job.created_unix}));
    if (path.startsWith('/api/geometry?')) {
      const parameters = new URLSearchParams(path.split('?')[1]), code = parameters.get('state'), year = parameters.get('year');
      const ref = this.manifest.geometries[`${code}:${year}`];
      if (!ref) throw new Error('This state and year have no published geometry.');
      return this.asset(ref);
    }
    const parts = path.split('/').filter(Boolean), job = this.jobs.get(parts[2]);
    if (!job || parts[0] !== 'api' || parts[1] !== 'runs') throw new Error('Saved experiment not found.');
    if (parts.length === 3) return structuredClone(job);
    const state = job.states.find(state => state.code === parts[3]);
    if (!state?.metrics) throw new Error('This state has no saved result for the selected configuration.');
    if (parts[4] === 'map') return this.map(state,job.config.year);
    if (parts[4] === 'assignments') return this.asset(state.assignments_ref);
    throw new Error('Unknown catalog route.');
  }
}

export async function loadStaticCatalog() {
  const response = await fetch('./catalog.json');
  if (!response.ok) throw new Error('Published catalog is unavailable.');
  return new StaticCatalog(await response.json());
}

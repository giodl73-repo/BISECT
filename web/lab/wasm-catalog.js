import {validateElectionInput} from './election-input.js';
import {validatePartisanInput} from './partisan-input.js';
import {validateCharacterInput,usesCharacterWeights} from './character-input.js';
import {StaticCatalog,isSeed,effectiveConfig,engineOptions} from './static.js';
import {validateLabConfig,validateLabProject} from './laboratory-project.js';
import {validateDemographicInput} from './demographic-input.js';
import {verifyAssignments} from './assignment-verification.js';
import {createProject} from './project.js';
export class WasmCatalog extends StaticCatalog {
  constructor(manifest,fetcher,workerFactory=()=>new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'})){
    super(manifest,fetcher);this.workerFactory=workerFactory;this.worker=null;this.pending=null;this.active=null;this.outputs=new Map();
    if(!/^[a-f0-9]{64}$/.test(manifest.wasm_sha256))throw new Error('Missing WASM module integrity hash.');
    for(const [key,entry]of Object.entries(manifest.graphs||{})){
      if(!/^[A-Z]{2}:(2000|2010|2020)$/.test(key)||!Object.hasOwn(manifest.assets,entry.graph_ref))throw new Error('Invalid prepared graph reference.');
    }
  }
  async initialize(){
    if(this.initializing)return this.initializing;
    if(this.worker)return;
    const worker=this.workerFactory();this.worker=worker;
    this.initializing=new Promise((resolve,reject)=>{
      this.initReject=reject;
      const timer=setTimeout(()=>{this.destroy(new Error('Browser engine initialization timed out.'));reject(new Error('Browser engine initialization timed out.'));},30000);
      this.initTimer=timer;
      worker.onmessage=({data})=>{
        if(data.type==='ready'){clearTimeout(timer);resolve();}
        if(data.type==='result'&&this.pending&&data.id===this.pending.id){const pending=this.pending;this.pending=null;clearTimeout(pending.timer);this.lastMemoryBytes=data.memoryBytes??null;pending.resolve(data.result);}
        if(data.type==='error'){const error=new Error(data.error);clearTimeout(timer);if(this.pending){const pending=this.pending;this.pending=null;clearTimeout(pending.timer);pending.reject(error);}else{this.destroy(error);reject(error);}}
      };
      worker.onerror=()=>{clearTimeout(timer);const error=new Error('Browser engine Worker failed.');this.destroy(error);reject(error);};
      worker.postMessage({type:'initialize',url:new URL('./bisect_wasm.wasm',import.meta.url).href,sha256:this.manifest.wasm_sha256});
    });
    try{await this.initializing;}finally{this.initializing=null;this.initReject=null;clearTimeout(this.initTimer);this.initTimer=null;}
  }
  destroy(error){
    this.worker?.terminate();this.worker=null;
    if(this.initReject){clearTimeout(this.initTimer);this.initReject(error);this.initReject=null;}
    if(this.pending){clearTimeout(this.pending.timer);this.pending.reject(error);this.pending=null;}
  }
  async execute(request,timeout){
    await this.initialize();
    if(this.pending)throw new Error('Browser engine is already processing an operation.');
    return new Promise((resolve,reject)=>{
      const id=++this.sequence;
      const timer=setTimeout(()=>this.destroy(new Error('Browser engine execution timed out.')),timeout*1000);
      this.pending={id,timer,resolve,reject};this.worker.postMessage({type:'run',id,request});
    });
  }
  validateConfig(config){
    if(this.exporting)throw new Error('Finish the practitioner export first.');
    if(this.active)throw new Error('Finish or cancel the active experiment first.');
    validateLabConfig(config);
    if(!config||!Array.isArray(config.states)||!config.states.length||config.states.length>50||new Set(config.states).size!==config.states.length)throw new Error('Invalid state selection.');
    if(!['state','national'].includes(config.mode)||!['congressional','house','senate'].includes(config.chamber))throw new Error('Invalid experiment mode or chamber.');
    if(!this.manifest.catalog.search_compatibility[config.structure]?.includes(config.search)||!['geographic','unweighted','county','partisan','economic-character','housing-character'].includes(config.weights))throw new Error('Unsupported engine settings.');
    if(!isSeed(config.seed)||!Number.isFinite(config.timeout_seconds)||config.timeout_seconds<1||config.timeout_seconds>86400)throw new Error('Invalid seed or timeout.');
    for(const code of config.states){
      if(!Object.hasOwn(this.manifest.graphs,`${code}:${config.year}`))throw new Error(`Prepared input unavailable for ${code} ${config.year}.`);
      if(!Number.isInteger(this.districtCount(code,config))||this.districtCount(code,config)<1)throw new Error('District allocation unavailable.');
    }
  }
  async restoreProject(project,{signal,isCurrent=()=>true}={}){
    if(this.exporting)throw new Error('Finish the practitioner export first.');
    validateLabProject(project);
    const available=()=>{if(signal?.aborted)throw new Error('Project import cancelled.');if(!isCurrent())throw new Error('Current project changed during import. Open the file again.');if(this.active)throw new Error('Finish or cancel the active experiment first.');};
    available();
    const saved=project.experiment;
    // Loading a historical configuration must not force today's compatibility
    // choices or replace its recorded engine identity.
    for(const state of saved.states){
      available();
      const key=`${state.code}:${saved.config.year}`,entry=this.manifest.graphs[key],input=project.inputs[state.code];
      if(!entry||input.prepared_graph_sha256!==this.manifest.assets[entry.graph_ref]||input.geometry_sha256!==this.manifest.assets[this.manifest.geometries[key]])throw new Error('Project needs different prepared inputs. Current work was kept.');
      if(state.metrics){
        if(state.metrics.graph_sha256!==entry.native_graph_sha256||state.metrics.district_count!==this.districtCount(state.code,saved.config))throw new Error('Project graph or district identity mismatch.');
        try{const graph=await this.asset(entry.graph_ref);available();if(graph.state!==state.code||graph.year!==saved.config.year)throw new Error('Prepared graph identity mismatch.');await verifyAssignments(graph,state,project.assignments[state.code],saved.config,{signal});available();}
        finally{this.assets.delete(entry.graph_ref);}
      }
    }
    available();
    const job=structuredClone(saved);job.id=`imported-${Date.now()}-${++this.sequence}`;job.restored_from_project=true;
    job.logs.push('Opened local project. Assignment metrics checked against hashed inputs; engine provenance is an unverified imported claim. No engine was executed.');
    // Commit only after every state's evidence passes. Never overwrite a run ID.
    for(const state of job.states)if(state.metrics)this.outputs.set(`${job.id}:${state.code}`,structuredClone(project.assignments[state.code]));
    this.jobs.set(job.id,job);return structuredClone(job);
  }
  submit(config){
    this.validateConfig(config);
    const job={id:`wasm-${Date.now()}-${++this.sequence}`,config:structuredClone(config),states:config.states.map(code=>({code,status:'queued',metrics:null,error:null,elapsed_seconds:0,command:[]})),created_unix:Math.floor(Date.now()/1000),status:'queued',logs:['Queued for local browser execution. No native server process.']};
    this.jobs.set(job.id,job);this.active=job;queueMicrotask(()=>this.run(job));return structuredClone(job);
  }
  async exportPractitionerProject(jobId,code){
    if(this.active||this.exporting)throw new Error('Finish or cancel the active operation first.');
    const job=this.jobs.get(jobId),state=job?.states.find(s=>s.code===code);
    if(!state?.metrics||state.status!=='completed')throw new Error('Select a completed state result.');
    const assignments=this.outputs.get(`${jobId}:${code}`),entry=this.manifest.graphs[`${code}:${job.config.year}`];
    if(!assignments||!entry)throw new Error('Prepared graph or assignments unavailable.');
    this.exporting=true;
    try{
      const graph=await this.asset(entry.graph_ref);
      if(graph.state!==code||graph.year!==job.config.year)throw new Error('Prepared graph identity mismatch.');
      // Recheck saved assignments against the hashed graph, including imported runs.
      await verifyAssignments(graph,state,assignments,job.config);
      const c=job.config,options=state.metrics.recorded_options;
      if(!options)throw new Error('Recorded native options are required for export.');
      const exported=await this.execute({operation:'export-engine-plan',request:{graph,options,...(usesCharacterWeights(c.weights)?{character:validateCharacterInput(c.characters[code],graph,c.weights.replace('-character',''))}:{}),...(['proportional-bisect','proportional-section'].includes(c.structure)?{elections:validateElectionInput(c.elections[code],graph)}:{}),...(c.weights==='partisan'?{partisan:validatePartisanInput(c.partisans[code],graph)}:{}),...(c.demographics?.[code]?{demographics:validateDemographicInput(c.demographics[code],graph)}:{})},assignments,label:`${c.name} · ${code}`,chamber:c.chamber,created_at:new Date(job.created_unix*1000).toISOString()},c.timeout_seconds);
      return {exported,project:createProject({name:`${c.name} · ${code}`,files:{plan:exported.document,context:exported.context},operation:'validate-rplan',constraints:['plan-shape','population','contiguity'],result:null,lastOperation:null})};
    }finally{this.assets.delete(entry.graph_ref);this.exporting=false;}
  }
  async run(job){
    if(job.status!=='cancelled')job.status='running';
    for(const state of job.states){
      if(job.status==='cancelled'){state.status='cancelled';continue;}
      state.status='running';const started=performance.now(),entry=this.manifest.graphs[`${state.code}:${job.config.year}`];
      try{
        const graph=await this.asset(entry.graph_ref);
        if(job.status==='cancelled'){state.status='cancelled';continue;}
        if(graph.state!==state.code||graph.year!==job.config.year)throw new Error('Prepared graph identity mismatch.');
        const c=job.config,options={...engineOptions(c),structure:c.structure,weights:c.weights,search:c.search,districts:this.districtCount(state.code,c),seed:c.seed,seeds:c.seeds,steps:c.steps,percentile:c.percentile,alpha_county:c.alpha_county,balance_tolerance:c.balance_tolerance,area_swing:c.area_swing,iterations:c.iterations};
        const result=await this.execute({graph,options,...(usesCharacterWeights(c.weights)?{character:validateCharacterInput(c.characters[state.code],graph,c.weights.replace('-character',''))}:{}),...(['proportional-bisect','proportional-section'].includes(c.structure)?{elections:validateElectionInput(c.elections[state.code],graph)}:{}),...(c.weights==='partisan'?{partisan:validatePartisanInput(c.partisans[state.code],graph)}:{}),...((c.search==='vra-recom'||c.structure==='ratio-optimal-vra')?{demographics:validateDemographicInput(c.demographics[state.code],graph)}:{})},c.timeout_seconds);
        if(job.status==='cancelled'){state.status='cancelled';continue;}
        const m=result.metrics;
        state.metrics={...m,district_count:m.districts,districts:m.district_metrics,balance_passed:m.within_requested_tolerance,balance_tolerance_percent:c.balance_tolerance,graph_sha256:entry.native_graph_sha256,prepared_graph_sha256:this.manifest.assets[entry.graph_ref],engine_provenance:{backend:result.backend,runtime:'wasm-browser',wasm_sha256:this.manifest.wasm_sha256},requested_options:options,recorded_options:result.options,effective_config:effectiveConfig(c,options.districts),wasm_memory_bytes:this.lastMemoryBytes};
        this.outputs.set(`${job.id}:${state.code}`,result.assignments);state.status='completed';job.logs.push(`${state.code}: completed in this browser.`);
      }catch(error){state.status=job.status==='cancelled'?'cancelled':'failed';state.error=error.message;job.logs.push(`${state.code}: ${error.message}`);}
      finally{state.elapsed_seconds=(performance.now()-started)/1000;this.assets.delete(entry.graph_ref);}
    }
    if(job.status!=='cancelled'){const completed=job.states.filter(s=>s.status==='completed').length;job.status=completed===job.states.length?'completed':completed?'partial':'failed';}
    this.active=null;
  }
  async api(path,options={}){
    if(options.method==='POST'){
      if(path==='/api/runs')return this.submit(JSON.parse(options.body));
      const match=path.match(/^\/api\/runs\/([^/]+)\/cancel$/);
      if(match){const job=this.jobs.get(match[1]);if(!job||job!==this.active)throw new Error('No active experiment to cancel.');job.status='cancelled';this.destroy(new Error('Cancelled by user.'));return structuredClone(job);}
      throw new Error('Unknown browser operation.');
    }
    const match=path.match(/^\/api\/runs\/([^/]+)\/([A-Z]{2})\/(map|assignments)$/);
    if(match&&this.outputs.has(`${match[1]}:${match[2]}`)){
      const assignments=this.outputs.get(`${match[1]}:${match[2]}`);
      if(match[3]==='assignments')return structuredClone(assignments);
      const job=this.jobs.get(match[1]),geometry=structuredClone(await this.asset(this.manifest.geometries[`${match[2]}:${job.config.year}`]));
      const ids=new Set();for(const feature of geometry.features){const id=feature.properties.geoid;if(ids.has(id)||!Object.hasOwn(assignments,id))throw new Error('Map/assignment join failed.');ids.add(id);feature.properties.district=assignments[id];}
      if(ids.size!==Object.keys(assignments).length)throw new Error('Map omits assigned units.');return geometry;
    }
    return super.api(path,options);
  }
}
export async function loadWasmCatalog(){
  const response=await fetch('./catalog.json');if(!response.ok)throw new Error('Browser input catalog unavailable.');
  const catalog=new WasmCatalog(await response.json());await catalog.initialize();return catalog;
}

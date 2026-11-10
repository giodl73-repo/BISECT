import {nativeStateDataEntries} from './native-state-files.js';
import {storedZipBytes} from './reproducibility-files.js';
import {validateNativeRoundProgress,isNativeRoundProgress} from './native-round-progress.js';
import {zoneRequest} from './zone-weighting.js';
import {validateWorkerFailure,verifyWorkerFailure} from './worker-failure.js';
import {verifyNativeCharacterFallback} from './character-fallback-source.js';
import {verifyCharacterPreparation} from './character-handoff.js';
import {acceptedBuildConfig} from './balance-build.js';
import {laboratoryCoiRequest} from './laboratory-coi.js';
import {electoralRequest,validateLaboratoryElectoral,verifyLaboratoryElectoral} from './laboratory-electoral.js';
import {verifyLaboratoryIlpReports} from './laboratory-ilp-reports.js';
import {verifyLaboratoryCapture} from './laboratory-round-capture.js';
import {planMapModel} from './plan-map.js';
import {encodeBytes} from './package-files.js';
import {isMultiscale,multiscaleRequest,adaptMultiscaleResult,bundleIdentity,fineGraph} from './multiscale-input.js?v=multiscale-build-v1';
import {validateElectionInput} from './election-input.js';
import {validatePartisanInput} from './partisan-input.js';
import {validateCharacterInput,usesCharacterWeights,characterRequest} from './character-input.js';
import {StaticCatalog,isSeed,isLaboratorySeed,effectiveConfig,engineOptions} from './static.js';
import {validateLabConfig,validateLabProject} from './laboratory-project.js?v=multiscale-build-v1';
import {validateDemographicInput} from './demographic-input.js';
import {verifyAssignments} from './assignment-verification.js';
import {parseGeometryJson} from './project.js';
import {preparePractitionerProject} from './laboratory-practitioner-project.js';
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
        if(this.worker!==worker)return;
        if(data.type==='ready'){clearTimeout(timer);resolve();}
        if(data.type==='progress'&&this.pending&&data.id===this.pending.id){try{this.pending.onProgress?.(data.progress);}catch(error){this.destroy(error);}return;}
        if(data.type==='result'&&this.pending&&data.id===this.pending.id){const pending=this.pending;this.pending=null;clearTimeout(pending.timer);this.lastMemoryBytes=data.memoryBytes??null;pending.resolve(data.result);}
        if(data.type==='error'){
          if(data.id!==undefined&&(!this.pending||data.id!==this.pending.id))return;
          const error=new Error(data.error);clearTimeout(timer);
          if(this.pending){const pending=this.pending;this.pending=null;clearTimeout(pending.timer);
            try{if(data.failure){const receipt=validateWorkerFailure(data.failure);if(receipt.error!==data.error&&!receipt.error_truncated)throw new Error('Failure error mismatch');if(receipt.runtime_artifact&&receipt.runtime_artifact.sha256!==this.manifest.wasm_sha256)throw new Error('Failure artifact mismatch');if(receipt.request_json!==null&&receipt.request_json!==JSON.stringify(pending.request))throw new Error('Failure request mismatch');error.failureReceipt=receipt;}}catch{}
            pending.reject(error);
          }else{this.destroy(error);reject(error);}
        }
      };
      worker.onerror=()=>{if(this.worker!==worker)return;clearTimeout(timer);const error=new Error('Browser engine Worker failed.');this.destroy(error);reject(error);};
      worker.postMessage({type:'initialize',url:new URL('./bisect_wasm.wasm',import.meta.url).href,sha256:this.manifest.wasm_sha256});
    });
    try{await this.initializing;}finally{this.initializing=null;this.initReject=null;clearTimeout(this.initTimer);this.initTimer=null;}
  }
  destroy(error){
    this.liveRoundProgress=null;this.onRoundProgress?.();
    this.worker?.terminate();this.worker=null;
    if(this.initReject){clearTimeout(this.initTimer);this.initReject(error);this.initReject=null;}
    if(this.pending){clearTimeout(this.pending.timer);this.pending.reject(error);this.pending=null;}
  }
  async execute(request,timeout,onProgress){
    await this.initialize();
    if(this.pending)throw new Error('Browser engine is already processing an operation.');
    return new Promise((resolve,reject)=>{
      const id=++this.sequence;
      const timer=setTimeout(()=>this.destroy(new Error('Browser engine execution timed out.')),timeout*1000);
      this.pending={id,timer,resolve,reject,request,onProgress};this.worker.postMessage({type:'run',id,request});
    });
  }
  validateConfig(config){
    if(this.exporting)throw new Error('Finish the practitioner export first.');
    if(this.active)throw new Error('Finish or cancel the active experiment first.');
    validateLabConfig(config);
    if(!config||!Array.isArray(config.states)||!config.states.length||config.states.length>50||new Set(config.states).size!==config.states.length)throw new Error('Invalid state selection.');
    if(!['state','national'].includes(config.mode)||!['congressional','house','senate'].includes(config.chamber))throw new Error('Invalid experiment mode or chamber.');
    if(!this.manifest.catalog.search_compatibility[config.structure]?.includes(config.search)||!['geographic','unweighted','county','partisan','vra-aligned','economic-character','housing-character','zone-membership'].includes(config.weights))throw new Error('Unsupported engine settings.');
    if(!isLaboratorySeed(config)||!Number.isFinite(config.timeout_seconds)||config.timeout_seconds<1||config.timeout_seconds>86400)throw new Error('Invalid seed or timeout.');
    for(const code of config.states){
      if(!Object.hasOwn(this.manifest.graphs,`${code}:${config.year}`))throw new Error(`Prepared input unavailable for ${code} ${config.year}.`);
      validateLaboratoryElectoral(config,this.districtCount(code,config));
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
      if(state.failure_receipt){
        const receipt=state.failure_receipt;
        try{const graph=await this.asset(entry.graph_ref);available();const expected=JSON.stringify(this.stateRequest(graph,saved.config,state.code).request);
          if(receipt.request_json!==null){if(!await verifyWorkerFailure(receipt)||receipt.request_json!==expected)throw new Error('Failure request differs from the saved configuration or prepared inputs.');}
          else if(receipt.request_sha256!==null){const bytes=new TextEncoder().encode(expected),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(bytes.length!==receipt.request_bytes||hash!==receipt.request_sha256)throw new Error('Failure request digest differs from saved inputs.');}
          available();
        }finally{this.assets.delete(entry.graph_ref);}
      }
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
  async exportNativeStateData(jobId,code){
    if(this.active||this.exporting)throw new Error('Finish or cancel the active operation first.');
    const job=this.jobs.get(jobId),state=job?.states.find(s=>s.code===code),assignments=this.outputs.get(`${jobId}:${code}`),entry=this.manifest.graphs[`${code}:${job?.config.year}`];
    if(state?.status!=='completed'||!state.metrics||!assignments||!entry)throw new Error('Select a completed state result.');
    this.exporting=true;
    try{
      const graph=await this.asset(entry.graph_ref);if(graph.state!==code||graph.year!==job.config.year)throw new Error('Prepared graph identity mismatch.');
      await verifyAssignments(graph,state,assignments,job.config);
      const options=state.metrics.recorded_options;if(!options)throw new Error('Recorded native options are required for State data export.');
      const {request}=this.stateRequest(graph,job.config,code),input=isMultiscale(job.config)?request.input:request.request??request,base=isMultiscale(job.config)?input.request:input,original={...base,options:{...base.options,seed:options.seed}};
      return storedZipBytes(nativeStateDataEntries(original,{options,assignments,metrics:state.metrics},isMultiscale(job.config)?{fine:input.fine_level==='bg'?input.block_groups:null,multiscale:input}:{}),{maxSourceBytes:128*1024*1024});
    }finally{this.assets.delete(entry.graph_ref);this.exporting=false;}
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
      const metadata={assignments,label:`${c.name} · ${code}`,chamber:c.chamber,created_at:new Date(job.created_unix*1000).toISOString()};
      const exported=await this.execute(isMultiscale(c)?{operation:'export-multiscale-plan',input:multiscaleRequest(graph,options,c).input,fine_edges:c.multiscale.fine_level==='bg'?c.multiscale.inputs[code].boundary_edges:null,...metadata}:{operation:'export-engine-plan',request:{graph,options,...electoralRequest(c),...zoneRequest(c,code,graph),...laboratoryCoiRequest(c),...characterRequest(c,graph),...(['proportional-bisect','proportional-section'].includes(c.structure)?{elections:validateElectionInput(c.elections[code],graph)}:{}),...(c.weights==='partisan'?{partisan:validatePartisanInput(c.partisans[code],graph)}:{}),...(c.demographics?.[code]?{demographics:validateDemographicInput(c.demographics[code],graph)}:{})},...metadata},c.timeout_seconds);
      const geometryRef=this.manifest.geometries[`${code}:${c.year}`];
      const geometry=isMultiscale(c)&&c.multiscale.fine_level==='bg'?c.multiscale.inputs[code].geometry:await this.asset(geometryRef);
      const geometryText=JSON.stringify(geometry);
      planMapModel(exported.document,parseGeometryJson(geometryText));
      return {exported,...await preparePractitionerProject({name:`${c.name} · ${code}`,files:{...(c.character_fallback_sources?.[code]?{characterFallbackSource:structuredClone(c.character_fallback_sources[code])}:{}),...(c.character_preparations?.[code]?{characterPreparation:structuredClone(c.character_preparations[code])}:{}),planGeometry:{name:`${code}-${c.year}-unit-geometry.geojson`,base64:encodeBytes(new TextEncoder().encode(geometryText))},plan:exported.document,...(exported.context_json?.length>3*1024*1024?{}:{context:exported.context}),...(typeof exported.document_json==='string'&&typeof exported.context_json==='string'?{auditPlanText:{name:'laboratory.rplan',base64:encodeBytes(new TextEncoder().encode(exported.document_json))},auditContextText:{name:'laboratory.rctx',base64:encodeBytes(new TextEncoder().encode(exported.context_json))}}:{})},operation:typeof exported.document_json==='string'?'validate-rplan-text':'validate-rplan',constraints:['plan-shape','population','contiguity'],result:null,lastOperation:null})};
    }finally{this.assets.delete(entry.graph_ref);this.exporting=false;}
  }
  stateRequest(graph,c,code){
    const options={...engineOptions(c),structure:c.structure,weights:c.weights,search:c.search,districts:this.districtCount(code,c),seed:c.seed,seeds:c.seeds,steps:c.steps,percentile:c.percentile,alpha_county:c.alpha_county,balance_tolerance:c.balance_tolerance,area_swing:c.area_swing,iterations:c.iterations};
    if(isMultiscale(c)){const request=multiscaleRequest(graph,options,c);if(c.native_build)request.operation='run-multiscale-build';return {request,options};}
    const request={graph,options,...electoralRequest(c),...zoneRequest(c,code,graph),...laboratoryCoiRequest(c),...characterRequest(c,graph),...(['proportional-bisect','proportional-section'].includes(c.structure)?{elections:validateElectionInput(c.elections[code],graph)}:{}),...(c.weights==='partisan'?{partisan:validatePartisanInput(c.partisans[code],graph)}:{}),...((c.vra_absent_file_fallback!==true&&(c.search==='vra-recom'||c.structure==='ratio-optimal-vra'||c.weights==='vra-aligned'))?{demographics:validateDemographicInput(c.demographics[code],graph)}:{})};return {request:c.native_build?{operation:'run-engine-build',request}:request,options};
  }
  async run(job){
    if(job.status!=='cancelled')job.status='running';
    for(const state of job.states){
      if(job.status==='cancelled'){state.status='cancelled';continue;}
      state.status='running';this.liveRoundProgress=null;this.onRoundProgress?.();const started=performance.now(),entry=this.manifest.graphs[`${state.code}:${job.config.year}`];
      try{
        const graph=await this.asset(entry.graph_ref);
        if(job.status==='cancelled'){state.status='cancelled';continue;}
        if(graph.state!==state.code||graph.year!==job.config.year)throw new Error('Prepared graph identity mismatch.');
        if(job.config.character_fallback_sources?.[state.code])await verifyNativeCharacterFallback(job.config.character_fallback_sources[state.code]);
        if(job.config.character_preparations?.[state.code])await verifyCharacterPreparation(job.config.character_preparations[state.code],job.config.characters[state.code]);
        const c=job.config,{request,options}=this.stateRequest(graph,c,state.code),raw=await this.execute(request,c.timeout_seconds,progress=>{
          if(!isNativeRoundProgress(progress))return;
          validateNativeRoundProgress(progress);this.liveRoundProgress={jobId:job.id,state:state.code,progress};this.onRoundProgress?.();
        });
        const result=isMultiscale(c)?adaptMultiscaleResult(raw,graph,options,c):raw;
        if(isMultiscale(c))result.metrics.fine_input_sha256=c.multiscale.fine_level==='bg'?await bundleIdentity(c.multiscale.inputs[state.code]):null;

        if(job.status==='cancelled'){state.status='cancelled';continue;}
        const m=result.metrics;
        if(c.capture_rounds===true)m.round_capture=result.round_capture;
        await verifyLaboratoryCapture(graph,{metrics:m},result.assignments,c);
        await verifyLaboratoryElectoral(graph,{metrics:m},c);
        if(result.build_report)m.balance_build=result.build_report;
        if(c.capture_ilp_reports===true)m.ilp_report_capture=result.ilp_report_capture;
        await verifyLaboratoryIlpReports(graph,{metrics:{...m,recorded_options:result.options}},result.assignments,c);
        const accepted=acceptedBuildConfig(c,{...m,balance_passed:m.within_requested_tolerance});
        state.metrics={...m,district_count:m.districts,districts:m.district_metrics,balance_passed:m.within_requested_tolerance,balance_tolerance_percent:c.balance_tolerance,graph_sha256:entry.native_graph_sha256,prepared_graph_sha256:this.manifest.assets[entry.graph_ref],engine_provenance:{backend:result.backend,runtime:'wasm-browser',wasm_sha256:this.manifest.wasm_sha256},requested_options:options,recorded_options:result.options,effective_config:effectiveConfig(accepted,options.districts,'wasm'),wasm_memory_bytes:this.lastMemoryBytes};
        this.outputs.set(`${job.id}:${state.code}`,result.assignments);state.status='completed';job.logs.push(`${state.code}: completed in this browser.${result.build_report?` Accepted seed ${result.options.seed} after ${result.build_report.attempts.length} population trials.`:''}`);
      }catch(error){state.status=job.status==='cancelled'?'cancelled':'failed';state.error=error.message;if(state.status==='failed'&&error.failureReceipt)state.failure_receipt=error.failureReceipt;job.logs.push(`${state.code}: ${error.message}`);}
      finally{state.elapsed_seconds=(performance.now()-started)/1000;this.assets.delete(entry.graph_ref);}
    }
    if(job.status!=='cancelled'){const completed=job.states.filter(s=>s.status==='completed').length;job.status=completed===job.states.length?'completed':completed?'partial':'failed';}
    this.active=null;this.liveRoundProgress=null;this.onRoundProgress?.();
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
      const job=this.jobs.get(match[1]),geometry=structuredClone(isMultiscale(job.config)&&job.config.multiscale.fine_level==='bg'?job.config.multiscale.inputs[match[2]].geometry:await this.asset(this.manifest.geometries[`${match[2]}:${job.config.year}`]));
      geometry.state=match[2];geometry.year=job.config.year;
      const ids=new Set();for(const feature of geometry.features){const id=feature.properties.geoid;if(ids.has(id)||!Object.hasOwn(assignments,id))throw new Error('Map/assignment join failed.');ids.add(id);feature.properties.state=match[2];feature.properties.county=id.slice(0,5);feature.properties.district=assignments[id];}
      if(ids.size!==Object.keys(assignments).length)throw new Error('Map omits assigned units.');return geometry;
    }
    return super.api(path,options);
  }
}
export async function loadWasmCatalog(){
  const response=await fetch('./catalog.json');if(!response.ok)throw new Error('Browser input catalog unavailable.');
  const catalog=new WasmCatalog(await response.json());await catalog.initialize();return catalog;
}

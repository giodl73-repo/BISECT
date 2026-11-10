import {validateCountyAugmentation} from './county-augmentation.js';
import {zoneDocument} from './zone-weighting.js';
import {validateElectionInput} from './election-input.js';
import {validatePartisanInput} from './partisan-input.js';
import {validateCharacterInput,usesCharacterWeights} from './character-input.js';
import {validateNativeCharacterWeightInputs} from './native-character-weight-files.js';
import {metisSidecarImport,isMetisSidecarRetired} from './metis-c-sidecar.js';
// Raw WASM ABI keeps the public site independent of a native process or bundler.
export async function instantiateEngine(bytes,{metisSidecar=null}={}) {
  // Compile and hash the same private snapshot, including sliced typed arrays.
  const snapshot=ArrayBuffer.isView(bytes)?new Uint8Array(bytes.buffer,bytes.byteOffset,bytes.byteLength).slice():new Uint8Array(bytes).slice();
  const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',snapshot)),b=>b.toString(16).padStart(2,'0')).join('');
  const artifactIdentity=Object.freeze({sha256,byte_length:snapshot.byteLength});
  const runtimeArtifacts=Object.freeze({schema_version:'bisect-engine-runtime-v2',engine:artifactIdentity,metis_sidecar:metisSidecar?.identity??null});
  const module = await WebAssembly.compile(snapshot);
  let instance,progressHandler=null,executing=false,progressError=null;
  const imports = { env: { bisect_monotonic_millis(){return performance.now();},bisect_random(pointer, length) {
    try {
      const memory = new Uint8Array(instance.exports.memory.buffer, pointer, length);
      for (let offset=0; offset<length; offset+=65536) crypto.getRandomValues(memory.subarray(offset, Math.min(length,offset+65536)));
      return 0;
    } catch { return 1; }
  },bisect_progress(pointer,length){
    try{if(!executing||!Number.isInteger(length)||length<0||length>64*1024*1024)throw new Error('Invalid WASM progress callback.');if(progressHandler){const memory=new Uint8Array(instance.exports.memory.buffer,pointer,length);progressHandler(JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(memory)));}return 0;}
    catch(error){progressError=error;return 1;}
  } } };
  let cCalls=0;
  const cImport=metisSidecar?metisSidecarImport(metisSidecar,()=>instance.exports.memory):()=>-4;
  imports.env.bisect_metis_partition=(...args)=>{cCalls++;return cImport(...args);};
  // Reject unknown imports rather than supplying dummy implementations.
  for (const entry of WebAssembly.Module.imports(module)) {
    if (entry.module!=='env' || !['bisect_random','bisect_progress','bisect_monotonic_millis','bisect_metis_partition'].includes(entry.name) || entry.kind!=='function') throw new Error(`Unsupported WASM import: ${entry.module}.${entry.name}`);
  }
  instance = await WebAssembly.instantiate(module, imports);
  const api=instance.exports;
  if (api.bisect_schema_version()!==1) throw new Error('Unsupported browser engine ABI.');
  let hashBusy=false;
  const host={
    get supportsCMetis(){return api.bisect_c_metis_bridge_version?.()===1;},
    get cMetisRetired(){return metisSidecar!==null&&isMetisSidecarRetired(metisSidecar);},
    get runtimeArtifacts(){return runtimeArtifacts;},
    get artifactIdentity(){return artifactIdentity;},
    execute(request,{onProgress}={}) {
      if(hashBusy)throw new Error('Finish the current file hash first.');
      if(executing)throw new Error('Finish the current WASM operation first.');
      if(onProgress!==undefined&&typeof onProgress!=='function')throw new Error('Progress listener must be a function.');
      // Recorded replay derives its backend from validated selected bytes in
      // Rust. A build that supports C is not authority to use an unloaded C
      // runtime. Preflight through the same typed boundary before dispatch.
      if(request?.operation==='run-native-recorded-replay'&&host.supportsCMetis&&
         (!metisSidecar||host.cMetisRetired)){
        const review=host.execute({...request,operation:'prepare-native-recorded-replay'});
        if(review.status!=='prepared-replay-not-run')return review;
        const saved=JSON.parse(review.prepared.prepared_request_json);
        const resolved=saved.operation==='run-engine-build'?saved.request:
          ['run-multiscale','run-multiscale-build'].includes(saved.operation)?saved.input?.request:saved;
        if(resolved?.options?.metis_engine==='c-ffi')return {...review,status:'preparation-failed',prepared:null,
          error:'Load the verified C METIS sidecar before explicit recorded C execution.'};
      }
      const multiscale=['run-multiscale','run-multiscale-build','export-multiscale-plan'].includes(request?.operation);
      const options=multiscale?request.input?.request?.options:['export-engine-plan','run-engine-build'].includes(request?.operation)?request.request?.options:request?.graph?request.options:null;
      if(options && typeof options.proportional_eta==='number' && !Number.isFinite(options.proportional_eta))throw new Error('ProportionalSection eta must be finite.');
      if(options){
        validateCountyAugmentation(options);
        for(const key of ['sa_steps_per_tract','sa_t0_factor','sa_t_final']){
          if(typeof options[key]==='number'&&!Number.isFinite(options[key]))throw new Error('Simulated annealing parameters must be finite.');
        }
        const limit=options.ilp_time_limit;
        if(limit!=null&&(typeof limit==='number'?(!Number.isSafeInteger(limit)||limit<0||Object.is(limit,-0)):(typeof limit!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(limit)||BigInt(limit)>((1n<<64n)-1n))))throw new Error('ILP time limit must be an exact unsigned integer or canonical decimal u64 string.');
        if(options.ilp_gap!=null&&(!Number.isFinite(options.ilp_gap)||options.ilp_gap<0))throw new Error('ILP gap must be finite and nonnegative.');
        if(options.metis_ufactor!=null&&(!Number.isSafeInteger(options.metis_ufactor)||options.metis_ufactor<0||options.metis_ufactor>4294967295||Object.is(options.metis_ufactor,-0)))throw new Error('METIS imbalance factor must be an exact unsigned 32-bit integer.');
        const seed=options.seed;
        if(seed!=null&&(typeof seed==='number'?(!Number.isSafeInteger(seed)||seed<0||Object.is(seed,-0)):(typeof seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(seed)||BigInt(seed)>((1n<<64n)-1n))))throw new Error('Seed must be an exact unsigned integer or canonical decimal u64 string.');
      }

      const engineRequest=multiscale?request.input?.request:['export-engine-plan','run-engine-build'].includes(request?.operation)?request.request:request;
      let backendRequest=engineRequest;
      if(['run-prepared-engine','run-prepared-engine-timed'].includes(request?.operation)&&request.input?.inspect_only===false){
        // Inspect only the backend string. The original JSON (including exact
        // integer text) is still sent unchanged to the typed Rust boundary.
        try{const saved=JSON.parse(request.input.request_json);backendRequest=['run-multiscale','run-multiscale-build'].includes(saved.operation)?saved.input?.request:saved.operation==='run-engine-build'?saved.request:saved;}catch{}
      }
      if(backendRequest?.options?.metis_engine==='c-ffi'&&!metisSidecar)throw new Error(api.bisect_c_metis_bridge_version?.()===1?'Load the verified C METIS sidecar before explicit C execution.':'This engine build has no C METIS sidecar bridge.');
      if(multiscale){
        const input=request.input;
        for(const value of [input?.alpha,input?.percentile,input?.adaptive?.target_accept,input?.adaptive?.gamma_0])if(value!==undefined&&(!Number.isFinite(value)||Object.is(value,-0)))throw new Error('Multiscale probabilities must be finite and nonnegative.');
      }
      if(engineRequest?.zone_membership)zoneDocument(engineRequest.zone_membership,engineRequest.graph);
      if(engineRequest?.elections)validateElectionInput(engineRequest.elections,engineRequest.graph);
      if(engineRequest?.partisan)validatePartisanInput(engineRequest.partisan,engineRequest.graph);
      if(engineRequest?.character)validateCharacterInput(engineRequest.character,engineRequest.graph,engineRequest.options?.weights?.replace('-character',''));
      if(options&&usesCharacterWeights(options.weights)&&(!Number.isFinite(options.character_alpha)||options.character_alpha<0||options.character_alpha>1||Object.is(options.character_alpha,-0)))throw new Error('Character blend alpha must be finite in [0,1].');
      if(request?.operation==='build-character-weights'){validateCharacterInput(request.input);if(!Number.isFinite(request.alpha)||request.alpha<0||request.alpha>1||Object.is(request.alpha,-0))throw new Error('Character blend alpha must be finite in [0,1].');}
      if(request?.operation==='evaluate-native-character-weights')validateNativeCharacterWeightInputs({target:request.input?.target,alpha:request.input?.alpha,allow_absent_fallback:request.input?.allow_absent_fallback});
      const data=new TextEncoder().encode(JSON.stringify(request));
      const previousCCalls=cCalls;
      const input=api.bisect_alloc(data.length);
      executing=true;progressHandler=onProgress||null;progressError=null;
      try {
        new Uint8Array(api.memory.buffer,input,data.length).set(data);
        const pointer=api.bisect_execute(input,data.length);
        if(progressError)throw progressError;
        const output=new Uint8Array(api.memory.buffer,pointer,api.bisect_response_length());
        const result=JSON.parse(new TextDecoder().decode(output));
        if (!result.ok) throw new Error(result.error);
        if(cCalls>previousCCalls&&result.result&&typeof result.result==='object')result.result.runtime_artifacts=runtimeArtifacts;
        return result.result;
      } finally { api.bisect_free(input,data.length);executing=false;progressHandler=null;progressError=null; }
    },
    async hashFile(file){
      if(hashBusy||executing)throw new Error('A WASM operation is already running.');
      if(!file||!Number.isSafeInteger(file.size)||file.size<0||typeof file.slice!=='function')throw new Error('Invalid local file size or source.');
      if(!api.bisect_hash_start||!api.bisect_hash_update||!api.bisect_hash_finish)throw new Error('This WASM build has no streaming file hash support.');
      hashBusy=true;const chunkSize=1024*1024,input=api.bisect_alloc(chunkSize),output=api.bisect_alloc(32);
      try{api.bisect_hash_start();for(let offset=0;offset<file.size;offset+=chunkSize){const expected=Math.min(chunkSize,file.size-offset),chunk=new Uint8Array(await file.slice(offset,offset+expected).arrayBuffer());if(chunk.length!==expected)throw new Error('File size changed while hashing.');new Uint8Array(api.memory.buffer,input,chunk.length).set(chunk);if(api.bisect_hash_update(input,chunk.length)!==0)throw new Error('WASM file hash update failed.');}
       if(api.bisect_hash_finish(output)!==0)throw new Error('WASM file hash finalization failed.');return Array.from(new Uint8Array(api.memory.buffer,output,32),b=>b.toString(16).padStart(2,'0')).join('');
      }finally{api.bisect_free(input,chunkSize);api.bisect_free(output,32);hashBusy=false;}
    },
    memoryBytes:()=>api.memory.buffer.byteLength,
  };
  return host;
}

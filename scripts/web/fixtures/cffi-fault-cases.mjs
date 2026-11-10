// Controlled faults around the real C module, never a project-supplied runtime.
// The tiny additional WASM export executes `unreachable` for a genuine trap.
export async function createFaultSidecarAssets({mode,failAt=1,javascript,binary,sha}){
  const trapBytes=[0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,7,8,1,4,116,114,97,112,0,0,10,5,1,3,0,0,11];
  const glue='data:text/javascript;base64,'+btoa(new TextDecoder().decode(javascript));
    const source=`import create from ${JSON.stringify(glue)};
      export default async options=>{
        const module=await create(options),trap=(await WebAssembly.instantiate(new Uint8Array(${JSON.stringify(trapBytes)}))).instance.exports.trap;
        const metrics=globalThis.__bisectCffiFaultMetrics={malloc:0,free:0,partition:0};
        const malloc=module._malloc,free=module._free,partition=module._bisect_metis_partition;
        module._malloc=size=>{metrics.malloc++;if(metrics.malloc===${failAt}){if(${JSON.stringify(mode)}==='malloc-trap')trap();if(${JSON.stringify(mode)}==='allocation-refusal')return 0;}return malloc(size);};
        module._free=address=>{metrics.free++;if(${JSON.stringify(mode)}==='free-trap'&&metrics.free===${failAt})trap();return free(address);};
        module._bisect_metis_partition=(...args)=>{metrics.partition++;if(${JSON.stringify(mode)}==='partition-trap'){module.HEAP32[args[11]>>>2]=777;module.HEAP32[args[12]>>>2]=777;trap();}return partition(...args);};
        return module;
      };`;
    const bytes=new TextEncoder().encode(source);
    const manifest={schema:'bisect-c-metis-sidecar-v1',runtime:'msvcrt-v1',abi:1,
      javascript:{sha256:await sha(bytes),byte_length:bytes.length},wasm:{sha256:await sha(binary),byte_length:binary.length}};
    return {javascript:bytes,wasm:binary,manifest};
}
export async function runFaultCases({loadMetisSidecar,metisSidecarImport,instantiateEngine,javascript,binary,engineBytes,cffiEngineCase,sha}){
  const check=(condition,message)=>{if(!condition)throw new Error(message);};
  const equal=(a,b,message)=>check(JSON.stringify(a)===JSON.stringify(b),message);
  const records=[];
  async function fresh(mode,failAt=1){
    const {javascript:bytes,manifest}=await createFaultSidecarAssets({mode,failAt,javascript,binary,sha});
    const authority=await loadMetisSidecar({javascript:bytes,wasm:binary,manifest});
    return {authority,metrics:globalThis.__bisectCffiFaultMetrics,fixture_sha256:manifest.javascript.sha256};
  }
  // Five bridge allocations: rows, neighbors, options, cut, labels.
  for(const mode of ['allocation-refusal','malloc-trap','partition-trap','free-trap'])for(const failAt of mode==='partition-trap'?[1]:[1,2,3,4,5]){
    const fixture=await fresh(mode,failAt),memory=new WebAssembly.Memory({initial:2});
    const ints=new Int32Array(memory.buffer);ints.set([0,1,2],1);ints.set([1,0],5);ints.fill(-1,25,65);ints.set([123,456,789],75);
    const original=new Uint8Array(memory.buffer).slice(),call=metisSidecarImport(fixture.authority,()=>memory);
    const invoke=()=>call(0,2,1,2,4,20,0,0,0,0,100,300,304);
    const code=invoke();check(code===(mode==='allocation-refusal'?-3:-4),mode+' error code');
    equal([...new Uint8Array(memory.buffer)],[...original],mode+' must leave all Rust memory unchanged');
    if(mode==='allocation-refusal'){
      check(fixture.metrics.free===failAt-1,'Free allocations preceding refusal');
      check(fixture.metrics.partition===0,'Do not partition after refusal');
      check(invoke()===1,'Allocation refusal must permit a successful retry');
    }else{
      check(fixture.metrics.free===(mode==='free-trap'?failAt:0),'Never free after a trapped instance');
      const before={...fixture.metrics};check(invoke()===-4,'Retired C instance must refuse retry');equal(fixture.metrics,before,'No retired module reentry');
    }
    records.push({mode,failAt,code,fixture_sha256:fixture.fixture_sha256,metrics:{...fixture.metrics}});
  }
  const request=cffiEngineCase(),baseline=(await instantiateEngine(engineBytes)).execute(request);
  const cRequest=structuredClone(request);Object.assign(cRequest.options,{metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
  for(const mode of ['allocation-refusal','malloc-trap','partition-trap','free-trap']){
    const fixture=await fresh(mode),engine=await instantiateEngine(engineBytes,{metisSidecar:fixture.authority});
    let failure=null;try{engine.execute(cRequest);}catch(error){failure=error.message;}check(failure!==null,mode+' must fail through the real Rust engine');
    check(fixture.metrics.malloc>0,mode+' must reach the C import rather than fail preflight');
    if(mode==='partition-trap'||mode==='free-trap')check(fixture.metrics.partition>0,mode+' must enter actual C dispatch');
    if(mode==='free-trap')check(fixture.metrics.free===1,'Engine cleanup trap must reach free exactly once');
    equal(engine.execute(request),baseline,mode+' must restore the Rust backend');
    if(mode==='allocation-refusal')check(engine.execute(cRequest).backend==='c-metis-msvcrt-v1','Retry C after allocation refusal');
    else{const before={...fixture.metrics};let retired=false;try{engine.execute(cRequest);}catch{retired=true;}check(retired,'Retired engine refuses C');equal(fixture.metrics,before,'Rust retry cannot reenter retired C');}
    records.push({mode,engine_boundary:true,failure,fixture_sha256:fixture.fixture_sha256,metrics:{...fixture.metrics}});
  }
  const recovered=await fresh('none'),engine=await instantiateEngine(engineBytes,{metisSidecar:recovered.authority});
  check(engine.execute(cRequest).backend==='c-metis-msvcrt-v1','A freshly loaded C instance executes successfully');
  equal(engine.execute(request),baseline,'Fresh C success restores Rust backend');
  delete globalThis.__bisectCffiFaultMetrics;
  return {records,fresh_instance_recovery:true,scope:'Controlled allocator refusal and real WASM trap injection around the pinned C module; not an exhaustion/capacity or native parity matrix.'};
}

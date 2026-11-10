// Same comparison runs in Node and a real browser module worker. No production
// dispatcher or stored project is involved in this feasibility fixture.
export function compareSidecar(module, expected) {
  const mismatches=[];
  const compare=(name,actual,wanted)=>{
    if(JSON.stringify(actual)!==JSON.stringify(wanted))mismatches.push({name,actual,expected:wanted});
  };
  const rng=expected.rng_seeds.map(seed=>{
    module._bisect_rng_seed(seed);
    return Array.from({length:64},()=>module._bisect_rng_next());
  });
  compare('native-crt-rng',rng,expected.rng);
  const cases=[];
  for(const test of expected.cases){
    const allocated=[];
    const array=(values,float=false)=>{
      if(!values.length)return 0;
      const address=module._malloc(values.length*4);
      if(!address)throw new Error('Sidecar allocation failed');
      allocated.push(address);
      new (float?Float32Array:Int32Array)(module.HEAPU8.buffer,address,values.length).set(values);
      return address;
    };
    try{
      const xadj=array(test.xadj),adjncy=array(test.adjncy),vwgt=array(test.vwgt),
        adjwgt=array(test.adjwgt),tpwgts=array(test.tpwgts,true),cut=array([0]),part=array(Array(test.n).fill(0));
      const code=module._bisect_metis_case(test.n,test.ncon,test.k,xadj,adjncy,vwgt,adjwgt,tpwgts,test.seed,cut,part);
      const actual=code===1?{code,cut:new Int32Array(module.HEAPU8.buffer,cut,1)[0],part:Array.from(new Int32Array(module.HEAPU8.buffer,part,test.n))}:{code};
      const wanted=test.code===1?{code:test.code,cut:test.cut,part:test.part}:{code:test.code};
      compare(test.id,actual,wanted);
      // Independent integer edge-cut witness; no METIS/shared analyzer call.
      if(code===1){
        if(actual.part.some(p=>p<0||p>=test.k))throw new Error('Out-of-range district label');
        let crossing=0;
        for(let i=0;i<test.n;i++)for(let j=test.xadj[i];j<test.xadj[i+1];j++){
          if(actual.part[i]!==actual.part[test.adjncy[j]])crossing+=test.adjwgt[j]??1;
        }
        if(crossing!==2*actual.cut)throw new Error('Independent weighted cut differs');
      }
      cases.push({id:test.id,...actual});
    }finally{for(const address of allocated)module._free(address);}
  }
  const values={cycle:module._bisect_metis_probe(),disconnected:module._bisect_metis_error_probe(),after_error:module._bisect_metis_probe()};
  values.invalid_option=module._bisect_metis_invalid_option_probe();
  values.after_invalid=module._bisect_metis_probe();
  for(const [name,value]of Object.entries(values))compare(name,value,expected[name]);
  return {rng,cases,values,mismatches,exact_match:mismatches.length===0};
}

// Importing this module in a browser has no Node/FS side effects.
if(typeof process!=='undefined'&&process.argv[1]&&import.meta.url.endsWith('/compare-sidecar.mjs')){
  const {pathToFileURL}=await import('node:url'),fs=await import('node:fs/promises');
  const create=(await import(pathToFileURL(process.argv[2]))).default;
  const bytes=await fs.readFile(process.argv[3]),expected=JSON.parse(await fs.readFile(process.argv[4],'utf8'));
  const result=compareSidecar(await create({wasmBinary:bytes,print:line=>console.error('C stdout: '+line)}),expected);
  console.log(JSON.stringify({...result,imports:WebAssembly.Module.imports(await WebAssembly.compile(bytes))}));
  if(!result.exact_match)process.exitCode=1;
}

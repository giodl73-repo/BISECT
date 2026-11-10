// Trusted package assets only. Project data cannot supply a module or authority.
const authorities=new WeakMap();
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export async function loadMetisSidecar({javascript,wasm,manifest}){
  if(manifest?.schema!=='bisect-c-metis-sidecar-v1'||manifest.runtime!=='msvcrt-v1'||manifest.abi!==1)throw new Error('Unsupported C METIS sidecar profile/ABI.');
  const js=new Uint8Array(javascript).slice(),binary=new Uint8Array(wasm).slice();
  for(const [bytes,ref]of [[js,manifest.javascript],[binary,manifest.wasm]]){
    if(bytes.byteLength!==ref?.byte_length||await sha(bytes)!==ref.sha256)throw new Error('C METIS asset identity mismatch.');
  }
  // Execute the verified private snapshot, not a second fetch of a mutable URL.
  const node=typeof process!=='undefined'&&process.versions?.node;
  const url=node?'data:text/javascript;base64,'+Buffer.from(js).toString('base64'):URL.createObjectURL(new Blob([js],{type:'application/javascript'}));
  let create;
  try{create=(await import(url)).default;}finally{if(!node)URL.revokeObjectURL(url);}
  const compiled=await WebAssembly.compile(binary);
  const module=await create({instantiateWasm(imports,receive){receive(new WebAssembly.Instance(compiled,imports));}});
  if(typeof module._bisect_metis_partition!=='function'||!module.HEAPU8)throw new Error('C METIS bridge export missing.');
  const identity=Object.freeze({backend:'c-ffi',runtime:'msvcrt-v1',abi:1,javascript:Object.freeze({...manifest.javascript}),wasm:Object.freeze({...manifest.wasm})});
  const authority=Object.freeze({identity});
  authorities.set(authority,{module,busy:false,failed:false});return authority;
}
export function isMetisSidecarRetired(authority){
  const state=authorities.get(authority);
  if(!state)throw new Error('Select a freshly loaded, verified C METIS module.');
  return state.failed;
}
export function metisSidecarImport(authority,getMemory){
  const state=authorities.get(authority);
  if(!state)throw new Error('Select a freshly loaded, verified C METIS module.');
  const {module}=state;
  return function(mode,n,ncon,k,xadj,adjncy,vwgt,adjwgt,tpwgts,ubvec,options,cut,part){
    const allocated=[];let enteredC=false,pendingOutput=null;
    if(state.busy||state.failed)return -4;
    state.busy=true;
    try{
      if(![0,1].includes(mode)||![n,ncon,k].every(Number.isSafeInteger)||n<1||ncon<1||k<1||n>10_000_000||ncon>32||k>n)return -2;
      const memory=getMemory();if(!(memory instanceof WebAssembly.Memory))return -2;
      let transferBytes=0;
      const view=(address,length,float=false)=>{
        address>>>=0;
        if(!Number.isSafeInteger(length)||length<0||length>0x1fffffff||address%4||address===0||address+length*4>memory.buffer.byteLength)throw new RangeError('Invalid Rust METIS array range.');
        transferBytes+=length*4;if(transferBytes>256*1024*1024)throw new RangeError('C METIS transfer exceeds 256 MiB.');
        return new (float?Float32Array:Int32Array)(memory.buffer,address,length);
      };
      const rows=view(xadj,n+1),edges=rows[n];
      if(rows[0]!==0||edges<0||rows.some((offset,i)=>offset<0||i>0&&offset<rows[i-1]))return -2;
      const neighbors=view(adjncy,edges);
      if(neighbors.some(v=>v<0||v>=n))return -2;
      const weights=vwgt?view(vwgt,n*ncon):null,edgeWeights=adjwgt?view(adjwgt,edges):null,
        targets=tpwgts?view(tpwgts,k*ncon,true):null,bounds=ubvec?view(ubvec,ncon,true):null,
        settings=view(options,40),outCut=view(cut,1),outPart=view(part,n);
      if(weights?.some(w=>w<0)||edgeWeights?.some(w=>w<=0)||targets?.some(w=>!Number.isFinite(w)||w<=0)||bounds?.some(w=>!Number.isFinite(w)||w<=1))return -2;
      if(![-1,0].includes(settings[18]))return -2; // zero-based CSR only
      if((cut>>>0)<(part>>>0)+n*4&&(part>>>0)<(cut>>>0)+4)return -2;
      const copy=values=>{
        if(values===null)return 0;
        const bytes=values.byteLength||4,address=module._malloc(bytes);
        if(!address)throw new Error('C METIS allocation failed');
        allocated.push(address);
        module.HEAPU8.set(new Uint8Array(values.buffer,values.byteOffset,values.byteLength),address);
        return address;
      };
      const cRows=copy(rows),cEdges=copy(neighbors),cWeights=copy(weights),cEdgeWeights=copy(edgeWeights),
        cTargets=copy(targets),cBounds=copy(bounds),cOptions=copy(settings),cCut=copy(new Int32Array(1)),cPart=copy(new Int32Array(n));
      enteredC=true;
      const code=module._bisect_metis_partition(mode,n,ncon,k,cRows,cEdges,cWeights,cEdgeWeights,cTargets,cBounds,cOptions,cCut,cPart);
      if(code!==1)return [-2,-3,-4].includes(code)?code:-4;
      const result=new Int32Array(module.HEAPU8.buffer,cPart,n),value=new Int32Array(module.HEAPU8.buffer,cCut,1)[0];
      if(value<0||result.some(p=>p<0||p>=k))return -4;
      // Cleanup can itself trap. Publish only after it has completed successfully.
      pendingOutput={outCut,outPart,value,result:new Int32Array(result)};return 1;
    }catch(error){if(enteredC||error instanceof WebAssembly.RuntimeError)state.failed=true;return error instanceof RangeError?-2:error.message==='C METIS allocation failed'?-3:-4;}
    finally{
      // A trapped C instance is never re-entered, including for allocator cleanup.
      // Its bounded memory is released when the worker is restarted/terminated.
      let cleanupFailed=false;
      if(!state.failed)for(const address of allocated){try{module._free(address);}catch{state.failed=true;cleanupFailed=true;break;}}
      state.busy=false;if(cleanupFailed)return -4;
      if(pendingOutput&&!state.failed){pendingOutput.outCut[0]=pendingOutput.value;pendingOutput.outPart.set(pendingOutput.result);}
    }
  };
}

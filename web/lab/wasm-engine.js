import {validateElectionInput} from './election-input.js';
import {validatePartisanInput} from './partisan-input.js';
import {validateCharacterInput,usesCharacterWeights} from './character-input.js';
// Raw WASM ABI keeps the public site independent of a native process or bundler.
export async function instantiateEngine(bytes) {
  const module = await WebAssembly.compile(bytes);
  let instance;
  const imports = { env: { bisect_random(pointer, length) {
    try {
      const memory = new Uint8Array(instance.exports.memory.buffer, pointer, length);
      for (let offset=0; offset<length; offset+=65536) crypto.getRandomValues(memory.subarray(offset, Math.min(length,offset+65536)));
      return 0;
    } catch { return 1; }
  } } };
  // Reject unknown imports rather than supplying dummy implementations.
  for (const entry of WebAssembly.Module.imports(module)) {
    if (entry.module!=='env' || entry.name!=='bisect_random' || entry.kind!=='function') throw new Error(`Unsupported WASM import: ${entry.module}.${entry.name}`);
  }
  instance = await WebAssembly.instantiate(module, imports);
  const api=instance.exports;
  if (api.bisect_schema_version()!==1) throw new Error('Unsupported browser engine ABI.');
  return {
    execute(request) {
      const multiscale=['run-multiscale','export-multiscale-plan'].includes(request?.operation);
      const options=multiscale?request.input?.request?.options:request?.operation==='export-engine-plan'?request.request?.options:request?.graph?request.options:null;
      if(options && typeof options.proportional_eta==='number' && !Number.isFinite(options.proportional_eta))throw new Error('ProportionalSection eta must be finite.');
      if(options){
        const seed=options.seed;
        if(typeof seed==='number'?(!Number.isSafeInteger(seed)||seed<0||Object.is(seed,-0)):(typeof seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(seed)||BigInt(seed)>((1n<<64n)-1n)))throw new Error('Seed must be an exact unsigned integer or canonical decimal u64 string.');
      }

      const engineRequest=multiscale?request.input?.request:request?.operation==='export-engine-plan'?request.request:request;
      if(multiscale){
        const input=request.input;
        for(const value of [input?.alpha,input?.percentile,input?.adaptive?.target_accept,input?.adaptive?.gamma_0])if(value!==undefined&&(!Number.isFinite(value)||Object.is(value,-0)))throw new Error('Multiscale probabilities must be finite and nonnegative.');
      }
      if(engineRequest?.elections)validateElectionInput(engineRequest.elections,engineRequest.graph);
      if(engineRequest?.partisan)validatePartisanInput(engineRequest.partisan,engineRequest.graph);
      if(engineRequest?.character)validateCharacterInput(engineRequest.character,engineRequest.graph,engineRequest.options?.weights?.replace('-character',''));
      if(options&&usesCharacterWeights(options.weights)&&(!Number.isFinite(options.character_alpha)||options.character_alpha<0||options.character_alpha>1||Object.is(options.character_alpha,-0)))throw new Error('Character blend alpha must be finite in [0,1].');
      if(request?.operation==='build-character-weights'){validateCharacterInput(request.input);if(!Number.isFinite(request.alpha)||request.alpha<0||request.alpha>1||Object.is(request.alpha,-0))throw new Error('Character blend alpha must be finite in [0,1].');}
      const data=new TextEncoder().encode(JSON.stringify(request));
      const input=api.bisect_alloc(data.length);
      try {
        new Uint8Array(api.memory.buffer,input,data.length).set(data);
        const pointer=api.bisect_execute(input,data.length);
        const output=new Uint8Array(api.memory.buffer,pointer,api.bisect_response_length());
        const result=JSON.parse(new TextDecoder().decode(output));
        if (!result.ok) throw new Error(result.error);
        return result.result;
      } finally { api.bisect_free(input,data.length); }
    },
    memoryBytes:()=>api.memory.buffer.byteLength,
  };
}

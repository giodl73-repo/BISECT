import {sha256} from './proof-solver-core.js';
import {storedZipBytes} from './reproducibility-files.js';
const encode=text=>new TextEncoder().encode(text);
export async function prepareNativeCertifiedModel({contextJson,districts,weightPolicy,wasmBinary,receipt}){
 if(typeof contextJson!=='string'||encode(contextJson).length>8*1024*1024||!Number.isInteger(districts)||districts<2||districts>1024||!['source-integer','unweighted-edge-count'].includes(weightPolicy))throw new Error('Select a context of at most 8 MiB, 2–1024 seats and an explicit model weight policy.');
 if(receipt?.schema!=='bisect-veripb-wasm-build-v1'||await sha256(wasmBinary)!==receipt.wasm_sha256||wasmBinary.byteLength!==receipt.wasm_bytes)throw new Error('Model preparation artifact identity mismatch.');
 const bytes=encode(JSON.stringify({operation:'prepare-native-certified-model',context_json:contextJson,districts,weight_policy:weightPolicy}));if(bytes.length>40*1024*1024)throw new Error('Encoded model preparation input exceeds 40 MiB.');
 const {instance}=await WebAssembly.instantiate(wasmBinary,{}),api=instance.exports,pointer=api.proof_alloc(bytes.length);if(!pointer)throw new Error('Model preparation allocation refused.');let response;
 try{new Uint8Array(api.memory.buffer,pointer,bytes.length).set(bytes);const output=api.proof_execute(pointer,bytes.length),length=api.proof_response_length();if(length>32*1024*1024)throw new Error('Model preparation output exceeds 32 MiB.');response=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(api.memory.buffer,output,length)));}finally{api.proof_free(pointer,bytes.length);}
 if(!response.ok)throw new Error(response.error||'Native model preparation refused.');const result=response.result;
 if(result?.schema!=='bisect-native-certified-model-preparation-v1'||result.status!=='model-prepared-unproved'||result.original_context_json!==contextJson||result.original_context_source_sha256!==await sha256(encode(contextJson))||result.weight_policy!==weightPolicy||result.districts!==districts||result.objective_changed!==(weightPolicy==='unweighted-edge-count'))throw new Error('Native prepared-model source or policy binding mismatch.');
 return result;
}
export async function nativeModelPreparationZip(result){
 if(result?.schema!=='bisect-native-certified-model-preparation-v1'||result.status!=='model-prepared-unproved')throw new Error('Prepare a native model first.');
 const files={'original.rctx':result.original_context_json,'model.rctx':result.model_context_json,'root-instance.json':result.root_instance_json,'land-adjacency.json':JSON.stringify(result.land_adjacency)+'\n'};
 if(Object.values(files).some(text=>typeof text!=='string'))throw new Error('Prepared model files are missing.');
 const hashes=Object.fromEntries(await Promise.all(Object.entries(files).map(async([name,text])=>[name,await sha256(encode(text))])));
 files['manifest.json']=JSON.stringify({schema:'bisect-native-model-preparation-package-v1',status:'unproved',weight_policy:result.weight_policy,objective_changed:result.objective_changed,original_context_source_sha256:result.original_context_source_sha256,original_context_hash:result.original_context_hash,model_context_hash:result.model_context_hash,root_instance_hash:result.root_instance_hash,files:hashes,scope:result.scope},null,2)+'\n';
 return storedZipBytes(Object.entries(files).map(([name,text])=>[name,encode(text)]),{maxSourceBytes:32*1024*1024});
}

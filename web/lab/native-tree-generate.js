import {encodeProofSession} from './native-proof-session.js';
import {sha256} from './proof-solver-core.js';
import {runNativeProofPipeline} from './native-proof-pipeline.js';
import {adoptNativeExternalTree} from './native-external-tree.js';
async function provedChildren(parentRequest, wasmBinary, receipt) {
  if (await sha256(wasmBinary) !== receipt.wasm_sha256 || wasmBinary.byteLength !== receipt.wasm_bytes) throw new Error('Child verifier artifact identity mismatch.');
  const bytes = new TextEncoder().encode(JSON.stringify({operation:'derive-native-proved-children',parent_request:parentRequest}));
  if (bytes.length > 192*1024*1024) throw new Error('Parent proof request exceeds 192 MiB.');
  const {instance}=await WebAssembly.instantiate(wasmBinary,{}),api=instance.exports,pointer=api.proof_alloc(bytes.length); if(!pointer)throw new Error('Parent proof input allocation refused.'); let response;
  try {new Uint8Array(api.memory.buffer,pointer,bytes.length).set(bytes);const output=api.proof_execute(pointer,bytes.length),length=api.proof_response_length();if(length>12*1024*1024)throw new Error('Derived child response exceeds 12 MiB.');response=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(api.memory.buffer,output,length)));}finally{api.proof_free(pointer,bytes.length);}
  if(!response.ok)throw new Error(response.error||'Proved child derivation refused.');if(response.result?.schema_version!=='bisect-native-proved-children-v1'||response.result.children_json?.length!==2)throw new Error('Unexpected proved child response.');return response.result;
}
export async function generateNativeProofTree({rootInstanceJson, seed='1', refinement='full', landAdjacency=null, onProgress=()=>{}, ...options}) {
  if(!Number.isInteger(options.seconds??30)||(options.seconds??30)<1||(options.seconds??30)>120)throw new Error('Solver seconds per decision must be 1–120.');
  if(typeof rootInstanceJson!=='string'||new TextEncoder().encode(rootInstanceJson).length>3*1024*1024||typeof seed!=='string'||!/^(0|[1-9]\d{0,19})$/.test(seed)||BigInt(seed)>(1n<<64n)-1n||!['metis','population','nrs-v0-1','nrs-v0-2','nrs-v0-3','fast','full'].includes(refinement))throw new Error('Invalid root source, exact u64 seed or native refinement profile.');
  const root=JSON.parse(rootInstanceJson);if(root.node_path!==''||root.parent_certificate_id!==null||!Number.isSafeInteger(root.k_parent)||root.k_parent<2||root.k_parent>1024)throw new Error('Automatic tree generation requires a canonical 2–1024-seat root.');
  const queue=[{text:rootInstanceJson,land:landAdjacency}],nodes=[],runs=[];let proofsBytes=0;
  while(queue.length){
    const {text,land}=queue.shift(),instance=JSON.parse(text),path=instance.node_path;
    onProgress({phase:'discovering',node_path:path,completed_nodes:nodes.length,total_nodes:root.k_parent-1,seed,refinement});
    const discovery=options.engine.execute({operation:'solve-certified-discovery-instance',instance_json:text,settings:{seed,refinement,land_adjacency:land}});
    if(discovery.schema_version!=='bisect-certified-discovery-instance-v1'||discovery.status!=='unproved-incumbent'||discovery.instance_json!==text||discovery.seed!==seed)throw new Error('Native discovery source/settings binding mismatch.');
    const run=await runNativeProofPipeline({...options,instanceJson:text,discoveryJson:discovery.discovery_json,onProgress:progress=>onProgress({...progress,node_path:path,completed_nodes:nodes.length,total_nodes:root.k_parent-1,seed,refinement})});
    runs.push({node_path:path,instance_hash:run.instance_hash,discovery_id:run.discovery_id,status:run.status});
    if(run.status!=='all-native-decisions-checked')return {schema:'bisect-native-tree-generation-v1',status:'tree-generation-incomplete',stopped_node_path:path,seed,refinement,solver_seconds_per_decision:options.seconds??30,completed_nodes:nodes.length,required_nodes:root.k_parent-1,runs,stopped_run:run,scope:'A native discovered split did not obtain all checked UNSAT proofs. No complete tree certificate is issued.'};
    const proofs=Object.fromEntries(run.required_decisions.map(stem=>[stem,run.files[stem+'.pbp']]));proofsBytes+=Object.values(proofs).reduce((sum,proof)=>sum+new TextEncoder().encode(proof).length,0);if(proofsBytes>128*1024*1024)throw new Error('Generated tree proofs exceed 128 MiB.');
    const parentRequest={operation:'adopt-native-external-split',instance_json:text,discovery_json:discovery.discovery_json,proofs};
    const derived=await provedChildren(parentRequest,options.checkerWasm,options.checkerReceipt);nodes.push({...parentRequest,expected_certificate:derived.parent_certificate});
    for(const childText of derived.children_json){const child=JSON.parse(childText);if(child.k_parent===1)continue;let childLand=null;
      if(land!==null){const parentIndex=new Map(instance.unit_ids.map((id,index)=>[id,index])),localIndex=new Map(child.unit_ids.map((id,index)=>[id,index]));childLand=child.unit_ids.map(id=>land[parentIndex.get(id)].map(to=>localIndex.get(instance.unit_ids[to])).filter(to=>to!==undefined).sort((a,b)=>a-b));}
      queue.push({text:childText,land:childLand});
    }
  }
  const bundle={operation:'adopt-native-external-tree',root_instance_json:rootInstanceJson,nodes,expected_tree:null};
  onProgress({phase:'verifying-complete-tree',completed_nodes:nodes.length,total_nodes:root.k_parent-1,seed,refinement});
  const tree=await adoptNativeExternalTree({rootInstanceJson,nodes,wasmBinary:options.checkerWasm,receipt:options.checkerReceipt});bundle.expected_tree=tree;
  return {schema:'bisect-native-tree-generation-v1',status:'tree-generated-and-checked',seed,refinement,solver_seconds_per_decision:options.seconds??30,completed_nodes:nodes.length,required_nodes:root.k_parent-1,runs,bundle,tree,scope:'Shared native discovery/refinement at each node, actual generated/checkable decision proofs, concrete verified-parent child derivation and complete native tree revalidation. No bounded oracle, hidden backtracking, global plan optimum or legal compliance is claimed.'};
}

// The existing isolated proof-session importer treats all reopened evidence as
// unverified. This exports only the stopped split, never a complete tree bundle.
export function encodeStoppedTreeProofSession(generation){
 if(generation?.schema!=='bisect-native-tree-generation-v1'||generation.status!=='tree-generation-incomplete'||!['checked-counterexample','proof-run-incomplete'].includes(generation.stopped_run?.status))throw new Error('A stopped tree split is required.');
 const run=generation.stopped_run;
 return encodeProofSession({schema:'bisect-native-proof-session-v1',instance_json:run.files?.['certified-split-instance.json'],discovery_json:run.files?.['certified-discovery.json'],seconds:generation.solver_seconds_per_decision,result:run});
}

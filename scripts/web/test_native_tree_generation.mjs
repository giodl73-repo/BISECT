import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {parseProofSession} from '../../web/lab/native-proof-session.js';
import {generateNativeProofTree,encodeStoppedTreeProofSession} from '../../web/lab/native-tree-generate.js';
const root='target/native-tree-generation';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base='target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';const {default:solverFactory}=await import(pathToFileURL(path.resolve('target/roundingsat-wasm/roundingsat.mjs')).href);
const options={engine,solverFactory,solverWasm:await fs.readFile('target/roundingsat-wasm/roundingsat.wasm'),solverReceipt:JSON.parse(await fs.readFile('target/roundingsat-wasm/build-receipt.json','utf8')),checkerWasm:await fs.readFile(base+'bisect_proof_checker.wasm'),checkerReceipt:JSON.parse(await fs.readFile(base+'proof-checker-build.json','utf8')),seconds:3,seed:'42',refinement:'full'};
const evidence=[];
for(const k of [2,4]) {
 const source=JSON.parse(await fs.readFile('target/native-external-tree/request-'+k+'.json','utf8')).root_instance_json;
 const progress=[],result=await generateNativeProofTree({...options,rootInstanceJson:source,onProgress:item=>progress.push(item)});assert.equal(result.status,'tree-generated-and-checked');assert.equal(result.completed_nodes,k-1);assert.equal(result.tree.leaves.length,k);assert.deepEqual(result.tree.assignments,JSON.parse(await fs.readFile('target/native-external-tree/tree-'+k+'.json','utf8')).assignments);
 assert.equal(result.bundle.nodes.length,k-1);assert.equal(progress.filter(item=>item.phase==='discovering').length,k-1);
 await fs.writeFile(root+'/generated-'+k+'.json',JSON.stringify(result));await fs.writeFile(root+'/bundle-'+k+'.json',JSON.stringify(result.bundle));evidence.push({k,complete:true,shared_native_discovery:true,verified_parent_child_derivation:true});
}
const landSource=JSON.parse(await fs.readFile('target/native-external-tree/request-4.json','utf8')).root_instance_json;
const landRoot=JSON.parse(landSource),land=landRoot.unit_ids.map(()=>[]);
for(const edge of landRoot.edges){land[edge.left].push(edge.right);land[edge.right].push(edge.left);}
const landResult=await generateNativeProofTree({...options,rootInstanceJson:landSource,landAdjacency:land});assert.equal(landResult.status,'tree-generated-and-checked');assert.deepEqual(landResult.tree,JSON.parse(await fs.readFile(root+'/generated-4.json','utf8')).tree);
await assert.rejects(()=>generateNativeProofTree({...options,rootInstanceJson:landSource,landAdjacency:land.map((row,index)=>index===0?[]:row)}),/symmetric/);
evidence.push({k:4,supplied_land_propagation:true,asymmetric_land_refused:true});
const unequal=JSON.parse(await fs.readFile('target/native-external-tree/request-3.json','utf8')).root_instance_json;
const counterexample=await generateNativeProofTree({...options,rootInstanceJson:unequal,refinement:'nrs-v0-3'});assert.equal(counterexample.status,'tree-generation-incomplete');assert.equal(counterexample.stopped_run.status,'checked-counterexample');assert.equal(counterexample.completed_nodes,0);assert.equal(counterexample.tree,undefined);evidence.push({k:3,complete:false,checked_counterexample_stops_tree:true});
const stoppedBytes=encodeStoppedTreeProofSession(counterexample),stopped=parseProofSession(stoppedBytes);assert.equal(stopped.seconds,3);assert.deepEqual(stopped.result,counterexample.stopped_run);await fs.writeFile(root+'/stopped-split.native-proof.json',stoppedBytes);assert.throws(()=>encodeStoppedTreeProofSession({schema:'bisect-native-tree-generation-v1',status:'tree-generated-and-checked'}),/stopped/);
await assert.rejects(()=>generateNativeProofTree({...options,rootInstanceJson:unequal,seed:'00'}),/Invalid/);
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence,production_bounded_oracle:false,hidden_backtracking:false,stopped_split_session_roundtrip:true,tree_ui:false},null,2));console.log('Automatic proof trees: two/four seats complete with shared native discovery and concrete proofs; three-seat canonical counterexample stops without a tree certificate.');

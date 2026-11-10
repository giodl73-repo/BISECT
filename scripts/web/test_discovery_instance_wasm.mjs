import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/discovery-instance-wasm'; await fs.mkdir(root,{recursive:true}); const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source=await fs.readFile('target/proof-models-wasm/2-full-42-instance.json','utf8'),child=JSON.parse(await fs.readFile('target/native-external-tree/request-4.json','utf8')).nodes[1].instance_json;
const profiles=['metis','population','nrs-v0-1','nrs-v0-2','nrs-v0-3','fast','full'];let matches=0;
const compare=async request=>{const file=path.resolve(root+'/request.json');await fs.writeFile(file,JSON.stringify(request));const expected=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024}));const actual=engine.execute(request);assert.deepEqual(actual,expected);matches++;return actual;};
for(const refinement of profiles)for(const seed of ['42','18446744073709551615']) {
 const request={operation:'solve-certified-discovery-instance',instance_json:source,settings:{seed,refinement,land_adjacency:null}};const result=await compare(request);assert.equal(result.status,'unproved-incumbent');assert.equal(result.seed,seed);await fs.writeFile(root+'/'+refinement+'-'+seed+'.json',JSON.stringify(result));
}
await compare({operation:'solve-certified-discovery-instance',instance_json:child,settings:{seed:'42',refinement:'full',land_adjacency:[[1],[0]]}});
for(const settings of [{seed:'00',refinement:'full',land_adjacency:null},{seed:'18446744073709551616',refinement:'full',land_adjacency:null},{seed:'42',refinement:'full',land_adjacency:[[1],[],[3],[2]]},{seed:'42',refinement:'full',land_adjacency:[[1,1],[0],[3],[2]]}]){
 const request={operation:'solve-certified-discovery-instance',instance_json:source,settings};assert.throws(()=>engine.execute(request));const file=path.resolve(root+'/bad.json');await fs.writeFile(file,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[file],{stdio:'pipe'}));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({whole_native_wasm_matches:matches,all_seven_profiles:true,exact_max_u64_seed:true,recursive_child_instance:true,symmetric_land_subset_validation:true,proof_status:'unproved-incumbent'},null,2));console.log('Direct native discovery: 15 whole native/WASM matches, seven refinements, max u64 seed, proved-child source and seed/land refusals passed.');

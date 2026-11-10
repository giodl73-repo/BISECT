import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(JSON.parse(await fs.readFile('target/native-engine-input-wasm/prepared.bisect','utf8')).result.graph_json);
let cases=0;
for(const weights of ['geographic','county','unweighted'])for(const seed of ['42','18446744073709551615']){
 const request={operation:'run-engine-build',request:{graph,options:{structure:'standard-bisect',weights,search:'single',districts:1,seed,seeds:1,steps:1,percentile:0,alpha_county:weights==='county'?1:0,balance_tolerance:0.5,area_swing:1,iterations:10}}};
 const file=path.resolve('target/zero-boundary-request.json');await fs.writeFile(file,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8'})),actual=engine.execute(request);assert.deepEqual(actual,native);
 for(const result of [actual,native])for(const key of ['graph_boundary_m','weighted_boundary']){assert.equal(result.metrics[key],0);assert.equal(Object.is(result.metrics[key],-0),false);}cases++;
}
console.log(`${cases} empty-cut native/WASM whole results preserve positive zero for geographic/county/unweighted and exact-u64 seeds.`);

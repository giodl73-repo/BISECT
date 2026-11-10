import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/native-tree-map'; await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const geometryJson=await fs.readFile('target/certified-context-wasm/synthetic-map.geojson','utf8');
let cases=0;
for(const k of [2,3,4]) {
 const plan=JSON.parse(await fs.readFile('target/native-tree-plan/plan-'+k+'.json','utf8'));
 const request={operation:'render-native-district-map',input:{document_json:plan.document_json,unit_geometry_json:geometryJson,dpi:72,map_type:'districts',round_key_basis:'unit-id',dissolve_method:'sequential'}};
 const file=path.resolve(root+'/request.json'); await fs.writeFile(file,JSON.stringify(request));
 const expected=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:64*1024*1024})); const actual=engine.execute(request); assert.deepEqual(actual,expected); assert.equal(actual.status,'rendered');
 await fs.writeFile(root+'/map-'+k+'.json',JSON.stringify(actual)); await fs.writeFile(root+'/map-'+k+'.svg',actual.svg_text); await fs.writeFile(root+'/map-'+k+'.png',Buffer.from(actual.png_base64,'base64')); cases++;
 const missing=JSON.parse(geometryJson); missing.features.pop(); assert.throws(()=>engine.execute({...request,input:{...request.input,unit_geometry_json:JSON.stringify(missing)}}));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({whole_native_wasm_matches:cases,missing_unit_geometry_refused:true,geometry_fixture:'Four separately supplied synthetic geographic cells; no real-state geometry or statewide proof capacity claim.'},null,2));
console.log('External tree plan maps: three whole native/WASM geographic renders and missing-unit geometry refusals passed.');

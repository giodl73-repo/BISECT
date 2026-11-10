// Explicit one-time declaration. Never revise expected outcomes after a test run.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gunzipSync} from 'node:zlib';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {digest,realDataMatrixRequest} from './fixtures/real-data-matrix-case.mjs';
const output='docs/specs/wasm-real-data-matrix.json';
try{await fs.access(output);throw new Error('Existing declaration must be reviewed/versioned, not regenerated over results.');}catch(error){if(error.code!=='ENOENT')throw error;}
const wasmFile='dist/wasm-workspace-continuation-validated/toolkit/bisect_wasm.wasm',wasm=await fs.readFile(wasmFile),engine=await instantiateEngine(wasm);
const identity=async path=>{const bytes=await fs.readFile(path);return {path,sha256:digest(bytes),byte_length:bytes.length};};
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8')),rows=[];
for(const [state,size,districts,units,population,expected]of [['RI','small',2,250,'1097379',{status:'completed'}],['NC','medium',14,2672,'10439388',{status:'failed',error:'apportion-regions balance 36.1% exceeds 3% research limit'}],['CA','large',52,9129,'39538223',{status:'failed',error:'apportion-regions balance 43.9% exceeds 3% research limit'}]]){
 const def=manifest.states[state],stem=`data/2020/tiger/tracts/tl_2020_${def.fips}_tract/tl_2020_${def.fips}_tract`,slug=def.name.toLowerCase().replaceAll(' ','_');
 const sources=[];for(const extension of ['shp','dbf','shx','prj'])sources.push(await identity(stem+'.'+extension));
 for(const tail of ['geo2020.pl','000012020.pl'])sources.push(await identity(`data/2020/redistricting/${slug}/${state.toLowerCase()}${tail}`));
 const graph=await identity(`target/original-all-state-wasm/${state}/engine-graph.json.gz`);graph.decoded_sha256=digest(gunzipSync(await fs.readFile(graph.path)));
 const row={id:state+'-2020-tract-apportion-regions-core-42',tier:'retained-integration',size,state,year:'2020',unit_kind:'tract',chamber:'congressional',districts,units,population,seed:'42',structure:'apportion-regions',search:'single',weights:'geographic',backend:'metis-core',balance_tolerance_percent:0.5,sources,graph,derivation_witness:await identity(`target/original-all-state-wasm/${state}/receipt.json`),expected_native:expected,expected_basis:'Historical retained original-state campaign; this declaration fixes the same template/graph/TUI recipe before new execution. Any changed result fails rather than updating expectations.',browser_requirements:['WebAssembly','Worker','crypto.subtle','portable file import/download'],required_checks:['native-adapter exact result or error','all graph units assigned on success','exact seed','population and graph-contiguity recomputation on success','native/WASM request hash','original raw source identity'],remaining_checks:['fresh raw-to-graph derivation','independent native CLI orchestration','actual browser workflow','map/report follow-ups','Save/Open and replay','recovery/install','other vintages/resolutions/methods','national capacity']};
 const request=await realDataMatrixRequest(engine,row);row.request_sha256=digest(JSON.stringify(request));rows.push(row);
}
const pins=[];for(const path of ['scripts/web/fixtures/real-data-matrix-case.mjs','web/lab/config-template.js','web/lab/tui-run-files.js','web/lab/config-run.js'])pins.push(await identity(path));
assert.equal(rows.length,3);
await fs.writeFile(output,JSON.stringify({schema_version:'bisect-real-data-matrix-v1',declared_at_utc:new Date().toISOString(),status:'predeclared-not-run',scope:'First real-data retained-graph/native-adapter tier. A matching native failure is not successful workload support; original raw hashes and historical derivation witnesses do not prove fresh derivation or source authenticity. Browser and broader lifecycle rows remain not-run.',declaration_engine:{path:wasmFile,sha256:digest(wasm),byte_length:wasm.length},recipe_sources:pins,comparison:{successful_results:'Exact complete native adapter/WASM reply. No normalizations.',failures:'Exact declared native and WASM error text; no success/capacity claim.',kernel_orchestration_limit:'Native host shares adapter kernels; this tier makes no independent CLI orchestration claim.'},rows},null,2)+'\n',{flag:'wx'});
console.log(output+' declared with original-source hashes, recipe/request identities and fixed expectations; execution not performed.');

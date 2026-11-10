// Prepared display boundaries are rejection probes, not analysis-quality inputs.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const directory=process.argv[2]??'dist/wasm-merge-validation';
const catalog=JSON.parse(await fs.readFile(directory+'/catalog.json','utf8'));
const root='target/state-geometry-wasm';await fs.mkdir(root,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const evidence=[];
async function asset(ref){const bytes=await fs.readFile(directory+'/'+ref);assert.equal(sha(bytes),catalog.assets[ref]);return JSON.parse(bytes);}
async function native(req,name){await fs.writeFile(root+'/request.json',JSON.stringify(req));const text=execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});if(name)await fs.writeFile(root+'/'+name+'-native.stdout',text);return JSON.parse(text);}
for(const [state,k] of [['RI',2],['IA',4]]){
 const key=state+':2020',graph=await asset(catalog.graphs[key].graph_ref);
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:k,seed:'42',seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:2,area_swing:1.1,iterations:10};
 const prepared=engine.execute({operation:'export-engine-plan',request:{graph,options},assignments:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%k+1])),label:'Geometry rejection fixture; not a proposed plan',chamber:'congressional',created_at:'2026-10-06T00:00:00Z'});
 const geometry=await asset(catalog.geometries[key]);
 const req={operation:'analyze-plan-compactness',document_json:prepared.document_json,unit_geometry_json:JSON.stringify(geometry),coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326'};
 // Keep original lowercase aliases intact; duplicating them is a separate refusal.
 const duplicate=structuredClone(geometry);assert.equal(typeof duplicate.features[0].properties.geoid,'string');duplicate.features[0].properties.GEOID=duplicate.features[0].properties.geoid;
 const ambiguous={...req,unit_geometry_json:JSON.stringify(duplicate)};assert.throws(()=>engine.execute(ambiguous),/duplicate field `GEOID`/);
 let aliasError;try{await native(ambiguous);}catch(e){aliasError=e;}assert(aliasError);assert.match(aliasError.stderr.toString(),/duplicate field `GEOID`/);assert.doesNotMatch(aliasError.stderr.toString(),/panicked|segment not found/);
 await fs.writeFile(root+'/'+state+'-ambiguous-request.json',JSON.stringify(ambiguous));await fs.writeFile(root+'/'+state+'-ambiguous-native.stderr',aliasError.stderr);evidence.push({state,status:'matching-duplicate-alias-rejection',geometry:'Both geoid and GEOID are deliberately supplied only in this negative case'});
 let error;const started=performance.now();try{engine.execute(req);}catch(e){error=e;}
 const wasm_ms=performance.now()-started;
 assert.ok(error,'Display geometry must fail this rejection fixture');
 assert.ok(!(error instanceof WebAssembly.RuntimeError),'Geometry rejection must not trap the WASM instance');
 assert.match(error.message,state==='RI'?/Ring self-intersects or touches itself/:/Assigned unit polygon interiors overlap/);
 let nativeError;try{await native(req);}catch(e){nativeError=e;}
 assert.ok(nativeError);assert.doesNotMatch(nativeError.stderr.toString(),/panicked|segment not found/);
 assert.match(nativeError.stderr.toString(),state==='RI'?/Ring self-intersects or touches itself/:/Assigned unit polygon interiors overlap/);
 await fs.writeFile(root+'/'+state+'-display-request.json',JSON.stringify(req));await fs.writeFile(root+'/'+state+'-display-native.stderr',nativeError.stderr);
 evidence.push({state,units:graph.geoids.length,display_geometry_bytes:Buffer.byteLength(req.unit_geometry_json),status:'structured-rejection',reason:error.message,wasm_ms});
 // A valid, explicitly synthetic partition exercises statewide unit capacity
 // and proves the same WASM instance remains usable after rejection.
 const synthetic={type:'FeatureCollection',features:graph.geoids.map((GEOID,i)=>{const x=(i%40)*2,y=Math.floor(i/40)*2;return {type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[[[x,y],[x+1,y],[x+1,y+1],[x,y+1],[x,y]]]}};})};
 const valid={...req,unit_geometry_json:JSON.stringify(synthetic),coordinate_basis:'projected-metres',crs_label:'Synthetic separated metre squares; real unit IDs only; not state boundaries'};
 const out=engine.execute(valid);assert.deepEqual(out,await native(valid,state+'-synthetic'));await fs.writeFile(root+'/'+state+'-synthetic-request.json',JSON.stringify(valid));await fs.writeFile(root+'/'+state+'-synthetic-wasm.json',JSON.stringify(out));
 execFileSync('target/release/examples/compactness_reference.exe',[root+'/request.json',root+'/native.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});
 assert.equal(out.report_json,await fs.readFile(root+'/native.json','utf8'));
 await fs.copyFile(root+'/native.json',root+'/'+state+'-independent-report.json');
 assert.equal(out.report.districts.reduce((sum,row)=>sum+row.component_count,0),graph.geoids.length);
 assert.ok(out.report.districts.every(row=>row.area_m2===1));
 evidence.push({state,units:graph.geoids.length,status:'synthetic-capacity-native-wasm-byte-parity',geometry:'Separated unit squares; cyclic assignments; not geographic boundaries or a proposed plan'});
}
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2));
console.log(JSON.stringify(evidence,null,2));

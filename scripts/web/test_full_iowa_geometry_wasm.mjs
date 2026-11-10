import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {compactnessRequest,readCompactnessGeometry,MAX_COMPACTNESS_BYTES} from '../../web/lab/compactness-files.js';
import {createProject,parseProject,MAX_PROJECT_BYTES} from '../../web/lab/project.js';
import {validateSourceFile} from '../../web/lab/package-files.js';
const root='target/full-iowa-geometry-wasm';await fs.mkdir(root,{recursive:true});
const geometryBytes=await fs.readFile('target/analysis-geometry-source-v1-IA/unit-geometry.json');
const provenance=JSON.parse(await fs.readFile('target/analysis-geometry-source-v1-IA/provenance.json','utf8'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
assert.equal(sha(geometryBytes),provenance.geometry_sha256);assert.equal(provenance.simplification,false);
assert.equal(provenance.units,896);assert.equal(provenance.positions,454150);
assert.ok(geometryBytes.length>8*1024*1024&&geometryBytes.length<MAX_COMPACTNESS_BYTES);
const geometry=await readCompactnessGeometry({name:'IA-unsimplified-EPSG5070.json',size:geometryBytes.length,arrayBuffer:async()=>geometryBytes});
assert.equal(geometry.base64,geometryBytes.toString('base64'));
assert.throws(()=>validateSourceFile(geometry),/encoding|8 MiB/); // Other source types keep their original ceiling.
const plan=await fs.readFile('target/smc-real-state-wasm/IA-selected.rplan','utf8');
const files={auditPlanText:{name:'IA-selected.rplan',base64:Buffer.from(plan).toString('base64')},compactnessInputs:{geometry,coordinate_basis:'projected-metres',crs_label:'EPSG:5070',dissolve_method:'balanced'}};
const request=compactnessRequest(files,files.compactnessInputs);assert.equal(request.unit_geometry_json,geometryBytes.toString('utf8'));
const project=createProject({name:'Iowa full-source geometry admission',files,operation:request.operation,constraints:[]});
const projectText=JSON.stringify(project);assert.ok(Buffer.byteLength(projectText)<MAX_PROJECT_BYTES);
assert.deepEqual(parseProject(projectText).files,files);await fs.writeFile(root+'/IA.bisect',projectText);
await fs.writeFile(root+'/request.json',JSON.stringify(request));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const start=performance.now();let wasmError;try{engine.execute(request);assert.fail('Expected current balanced-dissolve work bound.');}catch(e){wasmError=e.message;}
assert.match(wasmError,/^Balanced dissolve: Geometry exceeds the 50-million work budget/);
let nativeError;try{execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});assert.fail('Expected native work bound.');}catch(e){nativeError=e.stderr;}
assert.equal(nativeError.trim(),'Error: '+JSON.stringify(wasmError));
const spatialRequest={...request,dissolve_method:'spatial-balanced'};
await fs.writeFile(root+'/spatial-request.json',JSON.stringify(spatialRequest));
let spatialError;assert.throws(()=>engine.execute(spatialRequest),e=>{spatialError=e.message;return /^Spatial balanced dissolve: Geometry exceeds the 50-million work budget/.test(e.message);});
let spatialNativeError;try{execFileSync('target/release/examples/execute_tool_request.exe',[root+'/spatial-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});assert.fail('Expected spatial native work bound.');}catch(e){spatialNativeError=e.stderr;}
assert.equal(spatialNativeError.trim(),'Error: '+JSON.stringify(spatialError));
const spatialFiles={...files,compactnessInputs:{...files.compactnessInputs,dissolve_method:'spatial-balanced'}};
await fs.writeFile(root+'/IA-spatial.bisect',JSON.stringify(createProject({name:'Iowa spatial dissolve (bounded source)',files:spatialFiles,operation:request.operation,constraints:[]})));
const ri=JSON.parse(await fs.readFile('target/full-state-compactness-wasm/request.json','utf8'));
const recovered=engine.execute(ri);assert.equal(recovered.matched_units,250);assert.equal(recovered.report.districts.length,2);
const evidence={state:'IA',units:896,positions:454150,geometry_bytes:geometryBytes.length,project_bytes:Buffer.byteLength(projectText),geometry_sha256:sha(geometryBytes),status:'source-and-project-admitted-native-wasm-balanced-work-rejection',error:wasmError,spatial_error:spatialError,elapsed_with_native_and_recovery_ms:performance.now()-start,recovered_state:'RI',wasm_memory_bytes:engine.memoryBytes()};
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2));console.log(JSON.stringify(evidence,null,2));

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject,MAX_PROJECT_BYTES} from '../../web/lab/project.js';
const root='target/full-state-batch-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const sha=text=>createHash('sha256').update(text).digest('hex');
const sum=values=>{let total=0,correction=0;for(const value of values){const y=value-correction;const t=total+y;correction=(t-total)-y;total=t;}return total;};
const ringArea=ring=>{const [x0,y0]=ring[0];return Math.abs(sum(ring.slice(1).map(([x,y],i)=>(ring[i][0]-x0)*(y-y0)-(x-x0)*(ring[i][1]-y0)))/2);};
const polygonArea=rings=>ringArea(rings[0])-sum(rings.slice(1).map(ringArea));
const evidence=[];let recovery;
for(const [state,units,positions,districtCount] of [['RI',250,83797,2],['IA',896,454150,4]]){
 const sourceRoot=`target/analysis-geometry-source-v1-${state}`;
 const geometry=await fs.readFile(sourceRoot+'/unit-geometry.json','utf8');const provenance=JSON.parse(await fs.readFile(sourceRoot+'/provenance.json','utf8'));
 assert.equal(sha(geometry),provenance.geometry_sha256);assert.equal(provenance.simplification,false);assert.equal(provenance.positions,positions);assert.equal(provenance.units,units);
 const document_json=await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8');
 let executionGeometry=geometry;
 const request={operation:'analyze-plan-compactness',document_json,unit_geometry_json:geometry,coordinate_basis:'projected-metres',crs_label:'EPSG:5070',dissolve_method:'batch'};
 if(state==='IA'){
  // Default admission is separate from work capacity. Do not weaken either cap.
  const reason='Batch dissolve: Geometry exceeds the 50-million work budget; use a documented analysis-quality simplification.';
  assert.throws(()=>engine.execute(request),e=>!(e instanceof WebAssembly.RuntimeError)&&e.message===reason);
  const rejected=await fs.mkdtemp(root+'/IA-default-'),file=rejected+'/request.json';await fs.writeFile(file,JSON.stringify(request));
  for(const [name,executable,args]of [['native-tool','execute_tool_request.exe',[file]],['independent','compactness_reference.exe',[file,rejected+'/report.json']]]){
   let error;try{execFileSync('target/release/examples/'+executable,args,{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});}catch(e){error=e;}
   assert(error,'Default IA must retain the native work-budget refusal');assert(error.stderr.toString().includes(reason));assert.doesNotMatch(error.stderr.toString(),/panicked|segment not found/);await fs.writeFile(rejected+'/'+name+'.stderr',error.stderr);
  }
  await assert.rejects(fs.stat(rejected+'/report.json'),{code:'ENOENT'});assert.deepEqual(engine.execute(recovery.request),recovery.result);
  evidence.push({state,units,positions,status:'matching-default-native-wasm-work-budget-refusal',error:reason,recovered_state:'RI',default_request_directory:rejected,report_produced:false});
  assert.equal(JSON.parse(geometry).geometry_profile,undefined);
  executionGeometry=geometry.replace(/^(\s*\{)/,'$1"geometry_profile":"source-exact-large",');assert.equal(executionGeometry.replace('"geometry_profile":"source-exact-large",',''),geometry);
  request.unit_geometry_json=executionGeometry;await fs.writeFile(root+'/IA-original-geometry.json',geometry);
 }
 const started=performance.now();const result=engine.execute(request);const wasm_ms=performance.now()-started;
 await fs.writeFile(`${root}/${state}-request.json`,JSON.stringify(request));
 const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[`${root}/${state}-request.json`],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));assert.deepEqual(result,native);
 await fs.writeFile(`${root}/${state}-wasm-result.json`,JSON.stringify(result));await fs.writeFile(`${root}/${state}-native-result.json`,JSON.stringify(native));if(state==='RI')recovery={request:structuredClone(request),result:structuredClone(result)};
 execFileSync('target/release/examples/compactness_reference.exe',[`${root}/${state}-request.json`,`${root}/${state}-native.json`],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});
 assert.equal(result.report_json,await fs.readFile(`${root}/${state}-native.json`,'utf8'));assert.equal(result.report_sha256,sha(result.report_json));
 assert.equal(result.matched_units,units);assert.equal(result.unmatched_geometry_units,0);assert.equal(result.report.districts.length,districtCount);assert.equal(result.report.dissolve_method,'validated-partition-single-sweep-v1');
 const plan=JSON.parse(document_json).plan;const labels=new Map(plan.units.unit_ids.map((id,i)=>[id,plan.assignment[i]+1]));
 const areas=new Map();for(const f of JSON.parse(geometry).features){const label=labels.get(f.properties.GEOID);assert.notEqual(label,undefined);const rings=f.geometry.type==='Polygon'?[f.geometry.coordinates]:f.geometry.coordinates;const area=sum(rings.map(polygonArea));if(!areas.has(label))areas.set(label,[]);areas.get(label).push(area);}
 const areaChecks=[];for(const d of result.report.districts){assert.equal(d.status,'measured');const expected=sum(areas.get(d.district));const actual=d.area_m2/d.measured_area_fraction;const difference=Math.abs(actual-expected);assert.ok(difference<=Math.max(.05,expected*1e-10),`Area oracle ${state}/${d.district}: ${actual} vs ${expected}`);areaChecks.push({district:d.district,source_total_area_m2:expected,dissolved_total_area_m2:actual,absolute_difference_m2:difference});}
 const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
 const files={auditPlanText:source(`${state}-selected.rplan`,document_json),compactnessInputs:{geometry:source(`${state}-unsimplified-EPSG5070${state==='IA'?'-large':''}.json`,executionGeometry),coordinate_basis:'projected-metres',crs_label:'EPSG:5070',dissolve_method:'batch'}};
 const project=JSON.stringify(createProject({name:`${state} full-source batch compactness`,files,operation:request.operation,constraints:[],result,lastOperation:request.operation}));assert.ok(Buffer.byteLength(project)<=MAX_PROJECT_BYTES);assert.deepEqual(parseProject(project).files,files);await fs.writeFile(`${root}/${state}.bisect`,project);
 evidence.push({state,units,positions,geometry_bytes:Buffer.byteLength(geometry),project_bytes:Buffer.byteLength(project),wasm_ms,geometry_sha256:sha(geometry),execution_geometry_sha256:sha(executionGeometry),geometry_profile:state==='IA'?'source-exact-large':'default',original_coordinate_bytes_unchanged:true,report_sha256:result.report_sha256,status:'native-wasm-native-report-bytes-and-independent-area-match',areaChecks});
}
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2));console.log(JSON.stringify(evidence,null,2));

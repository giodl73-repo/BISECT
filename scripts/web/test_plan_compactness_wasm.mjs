import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/plan-compactness-wasm';await fs.mkdir(root,{recursive:true});
const plan=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
const ring=(x,y,w=1,h=1)=>[[x,y],[x+w,y],[x+w,y+h],[x,y+h],[x,y]];
const geometry={type:'FeatureCollection',features:plan.plan.units.unit_ids.map((GEOID,i)=>({type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[ring(i%3,Math.floor(i/3))]}}))};
const base={operation:'analyze-plan-compactness',document_json:JSON.stringify(plan),unit_geometry_json:JSON.stringify(geometry),coordinate_basis:'projected-metres',crs_label:'Synthetic metre grid (not geographic data)'};
let matches=0,reports=0,rejects=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matches++;execFileSync('target/release/examples/compactness_reference.exe',[root+'/request.json',root+'/native.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']});assert.equal(out.report_json,await fs.readFile(root+'/native.json','utf8'));assert.equal(out.report_sha256,createHash('sha256').update(out.report_json).digest('hex'));reports++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejects++;}
const change=(f)=>{const g=structuredClone(geometry);f(g);return {...base,unit_geometry_json:JSON.stringify(g)};};
const out=await exact(base),row=out.report.districts[0];assert.equal(row.area_m2,3);assert.equal(row.perimeter_m,8);assert.equal(row.length_width_ratio,3);assert.equal(row.convex_hull_ratio,1);assert.ok(Math.abs(row.polsby_popper-3*Math.PI/16)<1e-14);assert.ok(Math.abs(row.reock-3/(2.5*Math.PI))<1e-14);
const lowerRequest=change(g=>g.features.forEach(f=>{f.properties.geoid=f.properties.GEOID;delete f.properties.GEOID;}));
const lowerOut=engine.execute(lowerRequest);assert.deepEqual(lowerOut,await native(lowerRequest));assert.equal(lowerOut.report_json,out.report_json);matches++;
await reject(change(g=>g.features[0].properties.geoid=g.features[0].properties.GEOID));
await reject(change(g=>g.features[0].properties.geoid='different-identity'));
const degree=await exact({...base,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326'});assert.equal(degree.report.districts[0].area_m2,null);assert.equal(degree.report.districts[0].perimeter_m,null);assert.equal(degree.report.districts[0].area_coordinate_units_squared,3);
const holes=await exact(change(g=>g.features.forEach((f,i)=>{f.geometry.coordinates=[ring(i*4,0,3,3),ring(i*4+1,1,1,1).reverse()];})));assert.equal(holes.report.districts[0].component_count,3);assert.ok(Math.abs(holes.report.districts[0].measured_area_fraction-1/3)<1e-14);assert.equal(holes.report.districts[0].area_m2,8);assert.equal(holes.report.districts[0].perimeter_m,12);
const holeIsland=await exact(change(g=>g.features.forEach((f,i)=>{
 f.geometry.coordinates=i===0?[ring(0,0,3,3),ring(1,1).reverse()]:[i===1?ring(1.25,1.25,.5,.5):ring(i*10,0)];
})));assert.equal(holeIsland.report.districts[0].area_m2,8);assert.equal(holeIsland.report.districts[0].component_count,3);
const multi=await exact(change(g=>{g.features[0].geometry={type:'MultiPolygon',coordinates:[[ring(0,0)],[ring(20,0,.5,.5)]]};}));assert.equal(multi.report.districts[0].component_count,2);assert.equal(multi.report.districts[0].measured_area_fraction,3/3.25);
await exact(change(g=>g.features.reverse()));
const extras=await exact(change(g=>g.features.push({type:'Feature',properties:{GEOID:'unmatched'},geometry:{type:'Polygon',coordinates:[ring(40,40)]}})));assert.equal(extras.unmatched_geometry_units,1);assert.deepEqual(extras.report,out.report);
await exact(change(g=>g.features.forEach(f=>f.geometry.coordinates[0]=f.geometry.coordinates[0].map(([x,y])=>[x*.8-y*.6,x*.6+y*.8]))));
await exact(change(g=>g.features.forEach((f,i)=>f.geometry.coordinates=[[[i*4,0],[i*4+3,0],[i*4+3,1],[i*4+1,1],[i*4+1,3],[i*4,3],[i*4,0]]])));
for(const edit of [g=>g.features.pop(),g=>g.features.push(g.features[0]),g=>delete g.features[0].properties.GEOID,g=>g.features[0].properties.GEOID=5,g=>g.features[0].geometry.type='LineString',g=>g.features[0].geometry.coordinates[0].pop(),g=>g.features[0].geometry.coordinates=[ring(0,0,0,1)],g=>g.features[0].geometry.coordinates=[[[0,0],[2,2],[0,2],[2,0],[0,0]]],g=>g.features[0].geometry.coordinates.push(ring(20,20)),g=>g.features[0].geometry.coordinates.push(ring(0,0,.5,.5)),g=>g.features[0].geometry.coordinates[0][0].push(0),g=>g.features[0].geometry.coordinates[0][0][0]=null,g=>g.features[0].geometry.coordinates[0][0][0]=1e10,g=>g.features[0].geometry={type:'MultiPolygon',coordinates:[[ring(0,0)],[ring(.5,0)]]},g=>g.features[0].geometry.coordinates=[ring(0,0,10,10),ring(1,1,3,3),ring(2,2,3,3)]])await reject(change(edit));
await reject({...base,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:5070'});await reject({...base,crs_label:''});await reject({...base,unit_geometry_json:' '.repeat(3*1024*1024)});
await reject({...base,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326',unit_geometry_json:JSON.stringify({...geometry,features:[{...geometry.features[0],geometry:{type:'Polygon',coordinates:[ring(181,0)]}},...geometry.features.slice(1)]})});
const badPlan=structuredClone(plan);badPlan.plan.units.unit_universe_hash='sha256:wrong';await reject({...base,document_json:JSON.stringify(badPlan)});
const expensive=change(g=>g.features[0].geometry.coordinates=[Array.from({length:8000},(_,i)=>[Math.cos(i*2*Math.PI/7999),Math.sin(i*2*Math.PI/7999)])]);await reject(expensive);
await reject({...base,unit_geometry_json:base.unit_geometry_json.replace('"GEOID":','"GEOID":"duplicate","GEOID":')});
await reject({...base,unit_geometry_json:base.unit_geometry_json.replace('"coordinates":','"coordinates":[],"coordinates":')});

const exactBase={...base,include_exact_reock:true};const mbc=await exact(exactBase);assert.ok(Math.abs(mbc.report.districts[0].exact_reock-3/(2.5*Math.PI))<1e-14);assert.equal(mbc.report.districts[0].exact_reock_status,'measured');assert.equal(out.report.districts[0].exact_reock,undefined);
const triangle=change(g=>g.features.forEach((f,i)=>f.geometry.coordinates=[[[i*10,0],[i*10+4,0],[i*10,3],[i*10,0]]]));const triangleExact=await exact({...triangle,include_exact_reock:true});assert.ok(Math.abs(triangleExact.report.districts[0].exact_reock-6/(6.25*Math.PI))<1e-14);assert.ok(triangleExact.report.districts[0].exact_reock>triangleExact.report.districts[0].reock);for(const district of triangleExact.report.districts)assert.ok(Math.abs(district.length_width_ratio-4/3)<1e-12);assert.equal(triangleExact.report.length_width_method,'minimum-area-hull-edge-box-minimum-aspect-roundoff-ties-v1');
const translatedTriangle=structuredClone(triangle);const tg=JSON.parse(translatedTriangle.unit_geometry_json);for(const feature of tg.features)feature.geometry.coordinates=feature.geometry.coordinates.map(r=>r.map(([x,y])=>[x+1000000,y-1000000]));translatedTriangle.unit_geometry_json=JSON.stringify(tg);const translated=await exact(translatedTriangle);for(const district of translated.report.districts)assert.ok(Math.abs(district.length_width_ratio-4/3)<1e-12);
const holeExact=await exact({...change(g=>g.features.forEach((f,i)=>f.geometry.coordinates=[ring(i*4,0,3,3),ring(i*4+1,1,1,1).reverse()])),include_exact_reock:true});assert.ok(Math.abs(holeExact.report.districts[0].exact_reock-8/(4.5*Math.PI))<1e-14);
const degreeExact=await exact({...exactBase,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326'});assert.equal(degreeExact.report.districts[0].area_m2,null);
const exactWork=change(g=>g.features.forEach((f,i)=>{const points=Array.from({length:100},(_,j)=>[i*10+Math.cos(j*2*Math.PI/100),Math.sin(j*2*Math.PI/100)]);points.push(points[0]);f.geometry.coordinates=[points];}));const detailedCircle=await exact({...exactWork,include_exact_reock:true});assert.equal(detailedCircle.report.districts[0].exact_reock_status,'measured');assert.ok(Number(detailedCircle.report.exact_reock_work_charged)<50_000_000);await reject({...base,include_exact_reock:'true'});
const oddCircle=change(g=>g.features.forEach((f,i)=>{const points=Array.from({length:601},(_,j)=>[i*10+Math.cos(j*2*Math.PI/601),Math.sin(j*2*Math.PI/601)]);points.push(points[0]);f.geometry.coordinates=[points];}));await reject({...oddCircle,include_exact_reock:true});


const populationCsv=(pop='100')=>'GEOID,x,y,population\n'+plan.plan.units.unit_ids.map((id,i)=>`${id},${i%3+.5},${Math.floor(i/3)+.5},${typeof pop==='function'?pop(i):pop}\n`).join('');
const populationBase={...base,population_centroids_csv:populationCsv()};const populationOut=await exact(populationBase);for(const d of populationOut.report.districts){assert.ok(Math.abs(d.population_compactness.mean_squared_distance-2/3)<1e-14);assert.equal(d.population_compactness.total_population,'300');assert.equal(d.population_compactness.unit_count,3);}assert.equal(populationOut.report.population_compactness_source.coordinate_units_squared,'m2');assert.equal(populationOut.report.population_compactness_source.source_sha256,createHash('sha256').update(populationCsv()).digest('hex'));
const zeroPopulation=await exact({...populationBase,population_centroids_csv:populationCsv('0')});assert.ok(zeroPopulation.report.districts.every(d=>d.population_compactness.status==='zero-population-native-zero'&&d.population_compactness.mean_squared_distance===0));
const largePopulation=await exact({...populationBase,population_centroids_csv:populationCsv(i=>i===0?'18446744073709551615':'0')});assert.equal(largePopulation.report.districts[0].population_compactness.total_population,'18446744073709551615');
const populationExtra=await exact({...populationBase,population_centroids_csv:populationCsv()+'extra,1,1,1\n'});assert.equal(populationExtra.report.population_compactness_source.excluded_units,1);
const populationDegrees=await exact({...populationBase,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326'});assert.equal(populationDegrees.report.population_compactness_source.coordinate_units_squared,'degrees2');
const populationIslands=await exact({...populationBase,...change(g=>g.features[0].geometry={type:'MultiPolygon',coordinates:[[ring(0,0)],[ring(20,0,.5,.5)]]})});assert.equal(populationIslands.report.districts[0].population_compactness.unit_count,3);assert.notEqual(populationIslands.report.districts[0].population_compactness.district_area_centroid[0],populationOut.report.districts[0].population_compactness.district_area_centroid[0]);
await exact({...populationBase,include_exact_reock:true});
for(const csv of [populationCsv().split('\n').slice(0,3).join('\n'),populationCsv()+populationCsv().split('\n')[1]+'\n',populationCsv().replace('GEOID,x,y,population','GEOID,x,x,population'),populationCsv().replace(',100',',-1'),populationCsv().replace(',100',',1.5'),populationCsv().replace(',100',',18446744073709551616'),populationCsv().replace(',0.5,0.5,',',NaN,0.5,'),populationCsv('18446744073709551615'),populationCsv().replace('population','missing'),populationCsv()+'extra,Infinity,1,1\n',' '.repeat(3*1024*1024)])await reject({...populationBase,population_centroids_csv:csv});
await reject({...populationBase,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326',population_centroids_csv:populationCsv().replace(',0.5,0.5,',',181,0.5,')});
await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output:out}));await fs.writeFile(root+'/compactness.json',out.report_json);
await reject(change(g=>g.features[1].geometry.coordinates=[ring(.5,0)]));
const overPositionCap=change(g=>{g.features[0].geometry.coordinates=[Array.from({length:500001},(_,i)=>[i,0])];});
assert.throws(()=>engine.execute(overPositionCap),/500,000 positions/);await reject(overPositionCap);
const detailedBoundary=change(g=>g.features.forEach((f,i)=>{
 if(i){f.geometry.coordinates=[ring(10000+i*3,0)];return;}
 const side=3000,points=[];
 for(let j=0;j<side;j++)points.push([j,0]);
 for(let j=0;j<side;j++)points.push([side,j]);
 for(let j=0;j<side;j++)points.push([side-j,side]);
 for(let j=0;j<side;j++)points.push([0,side-j]);
 points.push(points[0]);f.geometry.coordinates=[points];
}));
const detailedBoundaryOut=await exact(detailedBoundary);
assert.equal(detailedBoundaryOut.report.districts[0].area_m2,9_000_000);
assert.equal(detailedBoundaryOut.report.districts[0].length_width_ratio,1);
const batchBase={...base,dissolve_method:'batch'};const batchOut=await exact(batchBase);
assert.equal(batchOut.report.dissolve_method,'validated-partition-single-sweep-v1');
await exact({...populationBase,dissolve_method:'batch',include_exact_reock:true});
await exact({...detailedBoundary,dissolve_method:'batch'});
await exact({...change(g=>g.features.forEach((f,i)=>{f.geometry.coordinates=[ring(i*4,0,3,3),ring(i*4+1,1,1,1).reverse()];})),dissolve_method:'batch'});
await exact({...change(g=>g.features[0].geometry={type:'MultiPolygon',coordinates:[[ring(0,0)],[ring(20,0,.5,.5)]]}),dissolve_method:'batch'});
const spatialBase={...base,dissolve_method:'spatial-balanced'};const spatialOut=await exact(spatialBase);
assert.equal(spatialOut.report.dissolve_method,'balanced-morton16-disjoint-envelopes-v1');
await exact({...populationBase,dissolve_method:'spatial-balanced',include_exact_reock:true});
await exact({...detailedBoundary,dissolve_method:'spatial-balanced'});
await exact({...change(g=>g.features.forEach((f,i)=>{f.geometry.coordinates=[ring(i*4,0,3,3),ring(i*4+1,1,1,1).reverse()];})),dissolve_method:'spatial-balanced'});
const balancedBase={...base,dissolve_method:'balanced'};const balancedOut=await exact(balancedBase);
const largeSource={...balancedBase,unit_geometry_json:base.unit_geometry_json+' '.repeat(3*1024*1024)};
const largeSourceOut=await exact(largeSource);assert.deepEqual(largeSourceOut.report,balancedOut.report);
const overCombined={...base,unit_geometry_json:base.unit_geometry_json+' '.repeat(18*1024*1024-Buffer.byteLength(base.document_json)-Buffer.byteLength(base.unit_geometry_json)+1)};
assert.throws(()=>engine.execute(overCombined),/sources must fit their 18\/64 MiB source profile/);await reject(overCombined);
assert.equal(balancedOut.report.dissolve_method,'balanced-input-order-pairs-indexed-work-v1');
const balancedMetrics=structuredClone(balancedOut.report);delete balancedMetrics.dissolve_method;assert.deepEqual(balancedMetrics,out.report);
await exact({...populationBase,dissolve_method:'balanced',include_exact_reock:true});
await exact({...detailedBoundary,dissolve_method:'balanced'});
await reject({...base,dissolve_method:'parallel'});await reject({...base,dissolve_method:null});
const capacityCount=2048,capacityIds=Array.from({length:capacityCount},(_,i)=>String(44001000000+i));
const capacityGraph={schema_version:1,state:'RI',year:'2020',geoids:capacityIds,adjacency:capacityIds.map((_,i)=>[i-1,i+1].filter(j=>j>=0&&j<capacityCount)),population:Array(capacityCount).fill(1),edges:capacityIds.slice(1).map((_,i)=>[i,i+1,1]),areas:Array(capacityCount).fill(1),exterior_perimeters:Array(capacityCount).fill(4),centroids:capacityIds.map((_,i)=>[-71+i*.000001,41])};
const capacityOptions={structure:'standard-bisect',weights:'geographic',search:'single',districts:1,seed:'42',seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:2,area_swing:1.1,iterations:10};
const capacityPlan=engine.execute({operation:'export-engine-plan',request:{graph:capacityGraph,options:capacityOptions},assignments:Object.fromEntries(capacityIds.map(id=>[id,1])),label:'Synthetic separated-square capacity fixture, not a proposed plan',chamber:'congressional',created_at:'2026-10-06T00:00:00Z'});
const capacityGeometry={type:'FeatureCollection',features:capacityIds.map((GEOID,i)=>({type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[ring(i*3,0)]}}))};
const capacityRequest={...balancedBase,document_json:capacityPlan.document_json,unit_geometry_json:JSON.stringify(capacityGeometry)};
const capacityOut=await exact(capacityRequest);assert.equal(capacityOut.report.districts[0].component_count,capacityCount);assert.equal(capacityOut.report.districts[0].area_m2,1);assert.equal(capacityOut.report.districts[0].measured_area_fraction,1/capacityCount);
console.log(`Compactness: ${matches} native/WASM matches, ${reports} native report byte matches and ${rejects} rejections. Analytic rectangle, holes, islands, rotation, concavity, exact joins, coordinate units and population-weighted full-centroid distances/counts verified.`);

const {createProject}=await import('../../web/lab/project.js');const {compactnessRequest}=await import('../../web/lab/compactness-files.js');const {countResultTables}=await import('../../web/lab/count-results.js');const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const files={auditPlanText:source('grid.rplan',base.document_json),compactnessInputs:{geometry:source('grid-metres.json',base.unit_geometry_json),coordinate_basis:base.coordinate_basis,crs_label:base.crs_label}};
assert.deepEqual(compactnessRequest(files,files.compactnessInputs),base);assert.equal(countResultTables(out)[0].rows[0][9],'3');assert.match(countResultTables(degree)[0].headings[9],/deg/);
await fs.writeFile(root+'/browser.bisect',JSON.stringify(createProject({name:'Native compactness fixture (synthetic metre grid)',files,operation:base.operation,constraints:[],result:out,lastOperation:base.operation})));
console.log('Compactness request, unit-labelled tables and inert project fixture verified.');

const exactFiles={...files,compactnessInputs:{...files.compactnessInputs,include_exact_reock:true}};assert.deepEqual(compactnessRequest(exactFiles,exactFiles.compactnessInputs),exactBase);await fs.writeFile(root+'/exact.bisect',JSON.stringify(createProject({name:'Exact minimum-circle Reock fixture (synthetic)',files:exactFiles,operation:base.operation,constraints:[],result:mbc,lastOperation:base.operation})));

assert.equal(countResultTables(triangleExact)[1].title,'Exact and proxy Reock');assert.notEqual(countResultTables(triangleExact)[1].rows[0][1],countResultTables(triangleExact)[1].rows[0][2]);
const triangleFiles={...exactFiles,compactnessInputs:{...exactFiles.compactnessInputs,geometry:source('right-triangles-metres.json',triangle.unit_geometry_json)}};await fs.writeFile(root+'/triangle.bisect',JSON.stringify(createProject({name:'Exact versus proxy Reock (synthetic right triangles)',files:triangleFiles,operation:base.operation,constraints:[],result:triangleExact,lastOperation:base.operation})));

const populationFiles={...files,compactnessInputs:{...files.compactnessInputs,population:source('unit-centroids-population.csv',populationCsv())}};assert.deepEqual(compactnessRequest(populationFiles,populationFiles.compactnessInputs),populationBase);assert.equal(countResultTables(populationOut)[1].title,'Population-weighted compactness');await fs.writeFile(root+'/population.bisect',JSON.stringify(createProject({name:'Population-weighted compactness (synthetic grid)',files:populationFiles,operation:base.operation,constraints:[],result:populationOut,lastOperation:base.operation})));

const detailedFiles={...exactFiles,compactnessInputs:{...exactFiles.compactnessInputs,geometry:source('hundred-vertex-circles.json',exactWork.unit_geometry_json)}};await fs.writeFile(root+'/detailed-circle.bisect',JSON.stringify(createProject({name:'Hull-reduced exact Reock (synthetic 100-vertex circles)',files:detailedFiles,operation:base.operation,constraints:[],result:detailedCircle,lastOperation:base.operation})));
const boundaryFiles={...files,compactnessInputs:{...files.compactnessInputs,geometry:source('twelve-thousand-edge-boundary.json',detailedBoundary.unit_geometry_json)}};
await fs.writeFile(root+'/detailed-boundary.bisect',JSON.stringify(createProject({name:'Indexed ring validation (synthetic 12,000-edge square)',files:boundaryFiles,operation:base.operation,constraints:[],result:detailedBoundaryOut,lastOperation:base.operation})));
const balancedFiles={...files,compactnessInputs:{...files.compactnessInputs,dissolve_method:'balanced'}};
assert.deepEqual(compactnessRequest(balancedFiles,balancedFiles.compactnessInputs),balancedBase);
await fs.writeFile(root+'/balanced.bisect',JSON.stringify(createProject({name:'Balanced dissolve (synthetic metre grid)',files:balancedFiles,operation:base.operation,constraints:[],result:balancedOut,lastOperation:base.operation})));


const spatialFiles={...files,compactnessInputs:{...files.compactnessInputs,dissolve_method:'spatial-balanced'}};
assert.deepEqual(compactnessRequest(spatialFiles,spatialFiles.compactnessInputs),spatialBase);
await fs.writeFile(root+'/spatial.bisect',JSON.stringify(createProject({name:'Spatial balanced dissolve (synthetic metre grid)',files:spatialFiles,operation:base.operation,constraints:[],result:spatialOut,lastOperation:base.operation})));

const batchFiles={...files,compactnessInputs:{...files.compactnessInputs,dissolve_method:'batch'}};
assert.deepEqual(compactnessRequest(batchFiles,batchFiles.compactnessInputs),batchBase);
await fs.writeFile(root+'/batch.bisect',JSON.stringify(createProject({name:'Batch dissolve (synthetic metre grid)',files:batchFiles,operation:base.operation,constraints:[],result:batchOut,lastOperation:base.operation})));

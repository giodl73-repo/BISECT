// Actual native Map and browser-ready WASM reference from a restored fixture.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {nativeMapRequest} from '../../web/lab/map-export-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],`Supply ${name}`);return process.argv[i+1];};
const fixture=await fs.realpath(arg('--fixture')),cli=await fs.realpath(arg('--cli')),root=path.resolve(arg('--output'));
const desktopOnly=process.argv.includes('--desktop-only');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const declarationBytes=await fs.readFile(new URL('./fixtures/original-tiger-state-map-v1.json',import.meta.url));
const declaration=JSON.parse(declarationBytes), states=Object.keys(declaration.cases);
assert.equal(declaration.schema_version,'bisect-original-tiger-state-map-fixture-v1');
assert.deepEqual(states.sort(),['CA','TX']);
const reference=JSON.parse(await fs.readFile(arg('--expected-readback'),'utf8')).profiles.core;
assert.deepEqual(Object.keys(reference).sort(),states);
const wasm=desktopOnly?null:await fs.readFile(arg('--wasm'));
const nativeIdentity={path:cli,sha256:hash(await fs.readFile(cli))};
const source=(filename,bytes)=>({name:path.basename(filename),base64:bytes.toString('base64')});
await fs.mkdir(root);

for(const state of states){
 const d=declaration.cases[state],expected=reference[state],out=path.join(root,state),files={};
 assert.deepEqual([d.units,d.source_positions,d.districts],[expected.units,expected.source_positions,expected.districts]);
 for(const [kind,r] of Object.entries(d.files)){
  const filename=await fs.realpath(path.join(fixture,r.path));
  assert(filename.startsWith(fixture+path.sep));
  const bytes=await fs.readFile(filename);assert.equal(bytes.length,r.bytes);assert.equal(hash(bytes),r.sha256);
  assert.equal(r.sha256,kind==='assignments'?expected.assignment_sha256:expected.tiger_sha256[kind]);
  files[kind]=bytes;
 }
 const nativeRoot=path.join(out,'outputs'),assignment=path.join(out,d.files.assignments.path);
 await fs.mkdir(path.dirname(assignment),{recursive:true});await fs.writeFile(assignment,files.assignments,{flag:'wx'});
 const args=['map','--state',state,'--year',String(declaration.year),'--version','v1','--types','districts',
  '--dpi',String(declaration.dpi),'--output-base',nativeRoot];
 const native=spawnSync(cli,args,{cwd:fixture,encoding:'utf8',windowsHide:true,timeout:1200000,maxBuffer:8*1024*1024});
 await fs.writeFile(path.join(out,'native.stdout'),native.stdout??'');await fs.writeFile(path.join(out,'native.stderr'),native.stderr??'');
 assert.ifError(native.error);assert.equal(native.status,0,native.stderr);
 const png=await fs.readFile(path.join(path.dirname(path.dirname(assignment)),'maps/districts.png'));
 assert.equal(hash(png),expected.desktop_png_sha256);
 const receipt={status:'passed-original-tiger-fixture-desktop',state,cli:nativeIdentity,arguments:args,
  fixture_declaration_sha256:hash(declarationBytes),desktop_png_sha256:hash(png),partition_execution_performed:false};
 if(wasm){
  const engine=await instantiateEngine(wasm),tiger={kind:'tracts',year:declaration.year};
  for(const kind of ['shp','dbf','shx'])tiger[kind]=source(d.files[kind].path,files[kind]);
  const inputs={plan:null,geometry:null,geometry_source:'tiger',tiger,source_mode:'native-assignments',raw_state:state,
   raw_year:declaration.year,assignment_key_basis:'unit-id',assignment_index:null,
   assignments:source(d.files.assignments.path,files.assignments),analysis:null,map_type:'districts',dpi:declaration.dpi,dissolve_method:'sequential'};
  const result=engine.execute(nativeMapRequest(inputs));
  assert.equal(result.execution_performed,false);assert.equal(result.geometry_materialization,'original-wkb-direct');
  assert.deepEqual([result.units,result.source_positions,result.districts],[d.units,d.source_positions,d.districts]);
  assert.deepEqual(Buffer.from(result.png_base64,'base64'),png);assert.equal(result.svg_sha256,expected.browser_svg_sha256);
  const operation='render-native-district-map',project={name:`Original ${state} state TIGER map`,operation,lastOperation:operation,
   files:{nativeMapInputs:inputs},constraints:[],result};
  const archive=createWorkbenchArchive(project),reopened=parseWorkbenchArchive(archive);
  assert.deepEqual(reopened.files.nativeMapInputs,inputs);assert.deepEqual(reopened.result,result);
  await fs.writeFile(path.join(out,'state.bisect-workbench'),archive);await fs.writeFile(path.join(out,'result.json'),JSON.stringify(result));
  Object.assign(receipt,{status:'passed-original-tiger-fixture-desktop-and-wasm',wasm_sha256:hash(wasm),run_directory:out,
   units:d.units,source_positions:d.source_positions,districts:d.districts});
 }
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(receipt,null,2)+'\n');
 console.log(`${state}: ${receipt.status}`);
}

// Verify the imported-plan contiguity workflow with retained real state graphs.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const arg=name=>{
  const i=process.argv.indexOf(name);assert(i>0&&process.argv[i+1],name);
  return path.resolve(process.argv[i+1]);
};
const root=arg('--output'),cli=await fs.realpath(arg('--cli')),wasm=await fs.realpath(arg('--wasm')),manifest=await fs.realpath(arg('--inputs'));
const target=await fs.realpath('target');
assert.equal(path.dirname(root),target);
assert(manifest.startsWith(target+path.sep));
await fs.mkdir(root);
const jsRoot=process.argv.includes('--js-root')?await fs.realpath(arg('--js-root')):await fs.realpath('web/lab');
const module=name=>import(pathToFileURL(path.join(jsRoot,name)).href);
const {labelImportPlanRequest,labelImportRequest}=await module('label-import-files.js');
const {labelAnalysisFromExternalPlan,labelAnalysisRequest}=await module('label-analysis-files.js');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputRoster=JSON.parse(await fs.readFile(manifest,'utf8'));
const cases={
  'ri-tract':{state:'rhode_island',code:'RI',unit:'tract',units:250,districts:2},
  'nc-tract':{state:'north_carolina',code:'NC',unit:'tract',units:2672,districts:14},
  'ca-tract':{state:'california',code:'CA',unit:'tract',units:9129,districts:52},
  'ri-block':{state:'rhode_island',code:'RI',unit:'block',units:25649,districts:2},
};
assert.deepEqual(Object.keys(inputRoster).sort(),Object.keys(cases).sort());
const engine=await instantiateEngine(await fs.readFile(wasm)),outcomes=[];
for(const [id,fixture] of Object.entries(cases)){
  const pins=inputRoster[id],dir=path.join(root,id),label=id.replace('-','_'),year='2020';
  await fs.mkdir(dir);
  const records=[];
  const consume=async pin=>{
    const file=await fs.realpath(path.resolve(path.dirname(manifest),pin.path));
    assert(file.startsWith(target+path.sep));
    const bytes=await fs.readFile(file);
    assert.equal(bytes.length,pin.bytes);assert.equal(sha(bytes),pin.sha256);
    return bytes.toString('utf8');
  };
  const document=await consume(pins.plan),context=await consume(pins.context);
  const plan=JSON.parse(document).plan,ctx=JSON.parse(context);
  assert.equal(plan.units.state,fixture.code);assert.equal(plan.units.unit_kind,fixture.unit);
  assert.equal(plan.units.unit_ids.length,fixture.units);assert.equal(plan.k,fixture.districts);
  assert.deepEqual(plan.units.unit_ids,ctx.units.unit_ids);
  assert.equal(ctx.units.year,2020);assert.equal(ctx.graph.adjacency.length,fixture.units);
  const csv='GEOID,district\n'+plan.units.unit_ids.map((g,i)=>`${g},${plan.assignment[i]+1}\n`).join('');
  await fs.writeFile(path.join(dir,'assigned.csv'),csv);
  await fs.writeFile(path.join(dir,'source.rctx'),context);
  const desktop=args=>{
    const result=spawnSync(cli,args,{cwd:dir,encoding:'utf8',windowsHide:true,timeout:90000});
    assert.ifError(result.error);assert.equal(result.status,0,result.stderr);
    records.push({args,status:result.status,stdout:result.stdout,stderr:result.stderr});
  };
  desktop(['label-import',label,'--from','assigned.csv','--year',year]);
  const indexPath=`runs/${label}/${year}/index.json`,index=await fs.readFile(path.join(dir,indexPath),'utf8');
  const inputs={source:source('assigned.csv',csv),dbf:null,shx:null,registry:null,
    settings:{label,year,format:'auto',created:JSON.parse(index).created}};
  const imported=engine.execute(labelImportRequest(inputs));
  assert.equal(imported.index_json,index);
  const assignmentPath=`runs/${label}/${year}/${fixture.state}/assignments.json`;
  assert.equal(imported.files[assignmentPath],await fs.readFile(path.join(dir,assignmentPath),'utf8'));
  const selection={state_name:fixture.state,index_sha256:sha(index),district_basis:'one-based',context:source('source.rctx',context)};
  const aligned=engine.execute(labelImportPlanRequest(inputs,selection));
  assert.deepEqual(JSON.parse(aligned.document_json).plan.assignment,plan.assignment);
  desktop(['label-analyze',label,'--year',year,'--states',fixture.state,'--types','contiguity',
    '--context','source.rctx','--district-basis','one-based']);
  const analysisPath=`analysis/${label}/${year}/index.json`,reportPath=`analysis/${label}/${year}/${fixture.state}/contiguity.json`;
  const nativeIndex=await fs.readFile(path.join(dir,analysisPath),'utf8'),report=await fs.readFile(path.join(dir,reportPath),'utf8');
  const batch=labelAnalysisFromExternalPlan(aligned,inputs,selection);
  batch.created=JSON.parse(nativeIndex).created;batch.types=['contiguity'];
  const computed=engine.execute(labelAnalysisRequest(batch));
  assert.equal(computed.index_json,nativeIndex);
  assert.equal(computed.files[reportPath],report);
  assert.deepEqual(JSON.parse(computed.registry_json),JSON.parse(await fs.readFile(path.join(dir,'.bisect'),'utf8')));
  const result=JSON.parse(report);
  assert.equal(result.districts.length,fixture.districts);
  assert.equal(result.districts.reduce((n,d)=>n+d.tract_count,0),fixture.units);
  assert.equal(result.all_contiguous,result.districts.every(d=>d.contiguous));
  await fs.writeFile(path.join(dir,'commands.json'),JSON.stringify(records,null,2));
  outcomes.push({id,units:fixture.units,districts:fixture.districts,unit_kind:fixture.unit,
    context_bytes:Buffer.byteLength(context),context_sha256:sha(context),source_plan_sha256:sha(document),
    report_sha256:sha(report),index_sha256:sha(nativeIndex),contiguous:result.all_contiguous,
    components:result.districts.map(d=>d.component_count)});
}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({
  status:'passed-real-retained-state-label-contiguity',outcomes,native_sha256:sha(await fs.readFile(cli)),
  wasm_sha256:sha(await fs.readFile(wasm)),inputs_sha256:sha(await fs.readFile(manifest)),
  partition_performed:false,browser_exercised:false,full_parity:false,
},null,2)+'\n');
console.log(JSON.stringify({root,outcomes:outcomes.map(({id,units,contiguous})=>({id,units,contiguous}))}));

// Actual desktop LabelAnalyze and the corresponding browser operation.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelImportPlanRequest,labelImportRequest} from '../../web/lab/label-import-files.js';
import {labelAnalysisFromExternalPlan,labelAnalysisRequest} from '../../web/lab/label-analysis-files.js';

const arg=name=>{
  const i=process.argv.indexOf(name);
  assert(i>0&&process.argv[i+1],name);
  return path.resolve(process.argv[i+1]);
};
const cli=await fs.realpath(arg('--cli')),wasm=await fs.realpath(arg('--wasm')),root=arg('--output');
assert.equal(path.dirname(root),await fs.realpath('target'));
await fs.mkdir(root);
const sha=data=>createHash('sha256').update(data).digest('hex');
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const label='context_fixture',state='washington',year='2020';
const context=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const ids=JSON.parse(context).units.unit_ids;
assert.equal(ids.length,9);
const indexPath=`runs/${label}/${year}/index.json`,analysisIndex=`analysis/${label}/${year}/index.json`;
const file=`analysis/${label}/${year}/${state}/contiguity.json`;
const engine=await instantiateEngine(await fs.readFile(wasm));
const legacy=process.argv.includes('--expect-legacy');
const cases=[
  {id:'connected-one-based',basis:'one-based',values:ids.map((_,i)=>Math.floor(i/3)+1),connected:true},
  {id:'connected-zero-based',basis:'zero-based',values:ids.map((_,i)=>Math.floor(i/3)),connected:true},
  {id:'disconnected-one-based',basis:'one-based',values:[1,2,2,2,3,2,2,2,1],connected:false},
  {id:'wrong-basis',basis:'one-based',values:ids.map((_,i)=>Math.floor(i/3)),refusal:/district labels/i},
  {id:'missing-context-unit',basis:'one-based',values:[1,1,1,2,2,2,3,3],refusal:/cover exactly/i},
  {id:'wide-district',basis:'one-based',values:['18446744073709551615',1,1,2,2,2,3,3,3],refusal:/500 districts/i},
];
const outcomes=[];
for(const test of legacy?cases.slice(0,1):cases){
  const dir=path.join(root,test.id);
  await fs.mkdir(dir);
  const csv='GEOID,district\n'+test.values.map((value,i)=>`${ids[i]},${value}\n`).join('');
  const sourceName='supplied.csv';
  const text=p=>fs.readFile(path.join(dir,p),'utf8');
  const save=(p,v)=>fs.writeFile(path.join(dir,p),JSON.stringify(v,null,2)+'\n',{flag:'wx'});
  await fs.writeFile(path.join(dir,sourceName),csv);
  await fs.writeFile(path.join(dir,'context.rctx'),context);
  async function run(name,args){
    const r=spawnSync(cli,args,{cwd:dir,encoding:'utf8',timeout:25000,windowsHide:true});
    assert.ifError(r.error);
    await save(name+'.json',{args,status:r.status,stdout:r.stdout,stderr:r.stderr});
    return r;
  }
  const imported=await run('desktop-import',['label-import',label,'--from',sourceName,'--year',year]);
  assert.equal(imported.status,0,imported.stderr);
  const index=await text(indexPath),registry=await text('.bisect');
  const inputs={source:source(sourceName,csv),dbf:null,shx:null,registry:null,
    settings:{label,year,format:'auto',created:JSON.parse(index).created}};
  assert.equal(engine.execute(labelImportRequest(inputs)).index_json,index);
  const selection={state_name:state,index_sha256:sha(index),district_basis:test.basis,context:source('context.rctx',context)};
  const flags=['label-analyze',label,'--year',year,'--states',state,'--types','contiguity'];
  const native=await run('desktop-analysis',legacy?flags:[...flags,'--context','context.rctx','--district-basis',test.basis]);
  const request=labelImportPlanRequest(inputs,selection);
  if(test.refusal){
    assert.equal(native.status,1,native.stderr);assert.match(native.stderr,test.refusal);
    assert.throws(()=>engine.execute(request),test.refusal);
    await assert.rejects(fs.stat(path.join(dir,'analysis')),{code:'ENOENT'});
    assert.equal(await text('.bisect'),registry);
    outcomes.push({id:test.id,status:'matching-refusal-no-analysis-effects'});
    continue;
  }
  assert.equal(native.status,0,native.stderr);
  const nativeIndex=await text(analysisIndex),nativeReport=await text(file);
  const aligned=engine.execute(request),browserInputs=labelAnalysisFromExternalPlan(aligned,inputs,selection);
  browserInputs.types=['contiguity'];
  browserInputs.created=JSON.parse(nativeIndex).created;
  const browserRequest=labelAnalysisRequest(browserInputs),result=engine.execute(browserRequest);
  await save('wasm-request.json',browserRequest);
  await save('wasm-result.json',result);
  assert.equal(result.external_import_verified,true);
  assert.equal(JSON.parse(result.files[file]).all_contiguous,test.connected);
  assert.equal(JSON.parse(result.index_json).run_index_sha256,sha(index));
  assert.deepEqual(JSON.parse(await text('.bisect'))[label].analyzed,[year]);
  if(legacy){
    assert.equal(JSON.parse(nativeReport).analyzer,'contiguity');
    assert(!Object.hasOwn(JSON.parse(nativeReport),'all_contiguous'));
  } else {
    assert.equal(nativeReport,result.files[file]);
    assert.equal(nativeIndex,result.index_json);
    assert.deepEqual(JSON.parse(await text('.bisect')),JSON.parse(result.registry_json));
    if(!test.connected)assert(JSON.parse(nativeReport).districts.some(d=>d.component_count>1));
  }
  outcomes.push({id:test.id,status:legacy?'reproduced-desktop-placeholder':'exact-analysis-index-and-contiguity-report',
    contiguous:test.connected,report_sha256:sha(result.files[file])});
}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({
  status:legacy?'reproduced-native-contiguity-placeholder':'passed-native-context-label-contiguity',
  outcomes,native:{path:cli,sha256:sha(await fs.readFile(cli))},wasm:{path:wasm,sha256:sha(await fs.readFile(wasm))},
  full_parity:false,
},null,2)+'\n');
console.log(JSON.stringify({root,outcomes}));

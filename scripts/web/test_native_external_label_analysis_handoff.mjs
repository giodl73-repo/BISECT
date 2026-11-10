// Reproduce the desktop import/analyze handoff against the current WASM path.
// All commands run in disposable, explicitly named fixture workspaces.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelImportPlanRequest,labelImportRequest} from '../../web/lab/label-import-files.js';
import {labelAnalysisFromExternalPlan,labelAnalysisRequest,labelAnalysisZip} from '../../web/lab/label-analysis-files.js';

function arg(key) {
  const i=process.argv.indexOf(key);
  assert(i>0&&process.argv[i+1],`Missing ${key}`);
  return path.resolve(process.argv[i+1]);
}
const cli=await fs.realpath(arg('--cli')),wasm=await fs.realpath(arg('--wasm'));
const root=arg('--output'),target=await fs.realpath('target');
assert(path.dirname(root)===target,'Use a fresh direct child of target');
await fs.mkdir(root);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const source=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')});
const json=async (dir,name)=>JSON.parse(await fs.readFile(path.join(dir,name),'utf8'));
const save=async (dir,name,data)=>fs.writeFile(path.join(dir,name),JSON.stringify(data,null,2)+'\n',{flag:'wx'});
const expectLegacy=process.argv.includes('--expect-legacy');
const context=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const ids=JSON.parse(context).units.unit_ids;
assert.equal(ids.length,9);
const csv='GEOID,district\n'+ids.map((id,i)=>`${id},${Math.floor(i/3)+1}\n`).join('');
const sourceName='supplied.csv',label='external_handoff',year='2020',state='washington';
const run=`runs/${label}/${year}`,analysis=`analysis/${label}/${year}`,reports=`reports/${label}/${year}`;
const importedPath=`${run}/${state}/assignments.json`;
const generatedPath=`${run}/${state}/final_assignments.json`;
const engine=await instantiateEngine(await fs.readFile(wasm));
const scenarios={};

async function command(dir,name,args) {
  const res=spawnSync(cli,args,{cwd:dir,encoding:'utf8',timeout:25000,windowsHide:true});
  assert.ifError(res.error);
  await save(dir,name+'.json',{args,exit_code:res.status,stdout:res.stdout,stderr:res.stderr});
  assert.equal(res.status,0,res.stderr);
  return res;
}
for(const scenario of ['fresh-import','with-older-build-assignments']) {
  const dir=path.join(root,scenario);
  await fs.mkdir(dir);
  await fs.writeFile(path.join(dir,sourceName),csv);
  await command(dir,'desktop-import',['label-import',label,'--from',sourceName,'--year',year]);
  const index=await fs.readFile(path.join(dir,run,'index.json'),'utf8');
  const original=await fs.readFile(path.join(dir,importedPath),'utf8');
  if(scenario==='with-older-build-assignments') {
    await fs.writeFile(path.join(dir,generatedPath),JSON.stringify({[ids[0]]:99}),{flag:'wx'});
    if(!expectLegacy) {
      const orphan=path.join(dir,run,'vermont');
      await fs.mkdir(orphan);
      await fs.writeFile(path.join(orphan,'final_assignments.json'),'{"50001000100":1}');
      const stale=path.join(dir,analysis,state);
      await fs.mkdir(stale,{recursive:true});
      await fs.writeFile(path.join(stale,'summary.json'),JSON.stringify({source:generatedPath}));
    }
  } else {
    await assert.rejects(fs.stat(path.join(dir,generatedPath)),{code:'ENOENT'});
  }
  const registry=await fs.readFile(path.join(dir,'.bisect'),'utf8');
  const stateFlags=expectLegacy?['--states',state]:[];
  const res=await command(dir,'desktop-analyze',['label-analyze',label,'--year',year,...stateFlags,'--types','summary']);
  const analysisBytes=await fs.readFile(path.join(dir,analysis,'index.json'),'utf8');
  const nativeState=JSON.parse(analysisBytes).states[state];
  assert.deepEqual(Object.keys(JSON.parse(analysisBytes).states),[state]);
  assert.equal(JSON.parse(analysisBytes).run_index_sha256,sha(index));
  assert.deepEqual((await json(dir,'.bisect'))[label].analyzed,[year]);
  const expectedSource=expectLegacy?generatedPath:importedPath;
  if(expectLegacy&&scenario==='fresh-import') {
    assert.equal(nativeState.status,'failed');
    assert.match(nativeState.error,/final_assignments\.json not found/);
    assert.match(res.stderr, /FAILED/);
    await assert.rejects(fs.stat(path.join(dir,analysis,state,'summary.json')),{code:'ENOENT'});
  } else {
    assert.equal(nativeState.status,'ok');
    assert.equal((await json(dir,`${analysis}/${state}/summary.json`)).source,expectedSource);
  }
  assert.equal(await fs.readFile(path.join(dir,importedPath),'utf8'),original);
  await command(dir,'desktop-report',['label-report',label,'--year',year,'--format','json']);
  const report=await json(dir,`${reports}/index.json`);
  assert.equal(report.analysis_index_sha256,sha(analysisBytes));
  assert.deepEqual((await json(dir,'.bisect'))[label].reported,[year]);
  scenarios[scenario]={native_state:nativeState,report_files:report.files};
  if(scenario!=='fresh-import')continue;

  const inputs={source:source(sourceName,csv),dbf:null,shx:null,registry:null,
    settings:{label,year,format:'auto',created:JSON.parse(index).created}};
  const parsed=engine.execute(labelImportRequest(inputs));
  assert.equal(parsed.index_json,index);
  assert.equal(parsed.files[importedPath],original);
  assert.deepEqual(JSON.parse(parsed.registry_json),JSON.parse(registry));
  const selection={state_name:state,index_sha256:sha(index),district_basis:'one-based',
    context:source('grid3x3.rctx',context)};
  const aligned=engine.execute(labelImportPlanRequest(inputs,selection));
  const browserInputs=labelAnalysisFromExternalPlan(aligned,inputs,selection);
  browserInputs.created=JSON.parse(analysisBytes).created;
  browserInputs.types=['manifest','contiguity','statistics'];
  const request=labelAnalysisRequest(browserInputs),result=engine.execute(request);
  assert.equal(result.external_import_verified,true);
  assert.equal(JSON.parse(result.index_json).run_index_sha256,sha(index));
  assert.equal(JSON.parse(result.index_json).states[state].status,'ok');
  assert.deepEqual(result.states[0].steps.map(step=>step.status),['computed','computed','computed']);
  await save(dir,'browser-analysis-request.json',request);
  await save(dir,'browser-analysis-result.json',result);
  await fs.writeFile(path.join(dir,'browser-analysis.zip'),await labelAnalysisZip(result));
  scenarios[scenario].browser_state=JSON.parse(result.index_json).states[state];
}
if(!expectLegacy) {
  const dir=path.join(root,'wide-metric-guard');
  await fs.mkdir(dir);
  await fs.writeFile(path.join(dir,sourceName),csv.replace(`${ids[0]},1`,`${ids[0]},18446744073709551615`));
  await command(dir,'desktop-import',['label-import',label,'--from',sourceName,'--year',year]);
  const bytes=await fs.readFile(path.join(dir,importedPath),'utf8');
  assert.match(bytes,/18446744073709551615/);
  await fs.writeFile(path.join(dir,generatedPath),JSON.stringify({[ids[0]]:1}));
  await command(dir,'desktop-analyze',['label-analyze',label,'--year',year,'--types','compactness']);
  const result=(await json(dir,`${analysis}/index.json`)).states[state];
  assert.equal(result.status,'failed');
  assert.match(result.error,/500-district analysis bound/);
  await assert.rejects(fs.stat(path.join(dir,analysis,state,'compactness.json')),{code:'ENOENT'});
  assert.equal(await fs.readFile(path.join(dir,importedPath),'utf8'),bytes);
  scenarios['wide-metric-guard']={native_state:result};
}
await save(root,'evidence.json',{
  status:expectLegacy?'reproduced-desktop-external-label-analysis-handoff-gap':'passed-desktop-external-label-analysis-handoff',
  native:{path:cli,sha256:sha(await fs.readFile(cli))},wasm:{path:wasm,sha256:sha(await fs.readFile(wasm))},
  scenarios,partition_performed:false,full_parity:false,
});
console.log(JSON.stringify({root,scenarios}));

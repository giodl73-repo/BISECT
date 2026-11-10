// Compare unlabeled custom State target actions to the public native CLI.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {nativeWorkspaceCase} from './fixtures/native-workspace-case.mjs';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';

const [toolkitArg,cliArg,manifestArg,rootArg]=process.argv.slice(2);
const [toolkit,cli,manifest,root]=[toolkitArg,cliArg,manifestArg,rootArg].map(p=>path.resolve(p));
const load=module=>import(pathToFileURL(path.join(toolkit,module)));
const {instantiateEngine}=await load('wasm-engine.js');
const {prepareNativeStateTargetRun,executeReviewedNativeSelectedStateSource}=await load('native-state-target-run.js');
const {nativeSelectedStateSourceRequest,nativeCommandOutputChoices}=await load('native-selected-state-source.js');
const {reviewNativeStateTargetActions,performNativeStateTargetActions}=await load('native-state-install.js');
const {nativeStateOutputSource,executeNativeStateOutput}=await load('native-state-output.js');
const {createProject}=await load('project.js');
const engine=await instantiateEngine(await fs.readFile(path.join(toolkit,'bisect_wasm.wasm')));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async file=>JSON.parse(await fs.readFile(file));
const source=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')});
await fs.mkdir(root);
const {directory}=nativeOutputDirectoryHarness(root),fixture=nativeWorkspaceCase();
const records=[];
const cases=[
 {name:'registered',state:'RI',year:'2020',kind:'imported',slug:'rhode_island',reset:true},
 {name:'fallback',state:'ZZ',year:'2024',kind:'precinct',slug:'zz',reset:true},
 {name:'explicit-name',state:'ZZ',year:'2024',kind:'imported',slug:'Research Area',override:'Research Area',reset:true},
 {name:'force-only',state:'RI',year:'2020',kind:'imported',slug:'rhode_island',reset:false},
];
for(const c of cases){
  const args=['--state',c.state,'--year',c.year,'--chamber','congressional','--version','vCustom',
    '--adjacency','unopened/custom.adj.bin','--districts','2','--metis-engine','metis-core',
    '--structure','nway','--search','single','--seed','42','--balance-tolerance','5','--manifest',
    ...(c.override?['--state-name',c.override]:[]),...(c.reset?['--reset']:['--force'])];
  const files={nativeSelectedStateSourceSettings:{kind:'native'},nativeStateSourceSettings:{enabled:true,command_kind:'state'},
    nativeAdjacencySettings:{state:c.state,year:c.year,unit_kind:c.kind},
    nativeAdjacencyInputs:{adjacency:source('custom.adj.bin',fixture.files.get(fixture.stem+'.adj.bin')),
      geoids:source('custom_geoids.json',fixture.files.get(fixture.stem+'_geoids.json')),
      centroids:source('custom_centroids.json',fixture.files.get(fixture.stem+'_centroids.json')),geometry:null,
      county_ids:source('custom-counties.json',JSON.stringify({schema_version:1,state:c.state,year:c.year,
        unit_ids:fixture.ids,county_ids:['North','North','South','South']}))},
    stateCommandInputs:{arguments_json:JSON.stringify(args)},
    stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'BrowserInputLabel',chamber:'congressional',
      created_at:'2026-11-06T00:00:00Z',inspect_only:true}}};
  const request=await nativeSelectedStateSourceRequest(files),ns=`vCustom/${c.year}/states/${c.slug}`;
  const expectedLabel=`${c.slug.toLowerCase().replaceAll(' ','_')}_congressional_${c.year}`;
  let actions=0,partitions=0;
  const observed={artifactIdentity:engine.artifactIdentity,runtimeArtifacts:engine.runtimeArtifacts,
    execute(r,o){if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)partitions++;return engine.execute(r,o);}};
  const prepared=await prepareNativeStateTargetRun(observed,request.input);
  assert.equal(partitions,0);assert.equal(prepared.intent.target_namespace,ns);assert.equal(prepared.intent.label,null);
  assert.equal(prepared.binding.settings.layout,'state');assert.equal(prepared.binding.name_effects.preview.identity.effective_label,expectedLabel);
  const target=await directory(c.name+'-browser'),work=path.join(target.dir,ns);
  const seedTarget=async dest=>{
    await fs.mkdir(path.join(dest,'data'),{recursive:true});
    await fs.writeFile(path.join(dest,'data/old.txt'),'old data');await fs.writeFile(path.join(dest,'keep.txt'),'keep');
  };
  await seedTarget(work);
  const perform=async intent=>{
    assert.equal(partitions,0);actions++;assert.equal(intent.target_namespace,ns);
    const review=await reviewNativeStateTargetActions(r=>observed.execute(r),target,intent);
    assert.equal(await fs.readFile(path.join(work,'data/old.txt'),'utf8'),'old data');
    return performNativeStateTargetActions(r=>observed.execute(r),review);
  };
  await assert.rejects(executeReviewedNativeSelectedStateSource(observed,request.input),/current explicit action callback/);
  assert.equal(await fs.readFile(path.join(work,'data/old.txt'),'utf8'),'old data');
  const run=await executeReviewedNativeSelectedStateSource(observed,request.input,{performTargetAction:perform});
  assert.equal(actions,1);assert.equal(partitions,1);
  assert.equal(await fs.readFile(path.join(work,'keep.txt'),'utf8'),'keep');
  if(c.reset)await assert.rejects(fs.stat(path.join(work,'data/old.txt')),e=>e.code==='ENOENT');
  else assert.equal(await fs.readFile(path.join(work,'data/old.txt'),'utf8'),'old data');
  assert.equal(run.prepared.export.document.metadata.label,expectedLabel);
  assert.equal(nativeCommandOutputChoices(run).layout,'state');
  const output=await executeNativeStateOutput(engine,{source:nativeStateOutputSource(run),version:'vCustom',layout:'state'});
  assert.equal(output.target_namespace,ns);assert.equal(output.partition_execution_performed,false);
  const desktop=path.join(root,c.name+'-native'),inputs=path.join(desktop,'inputs');
  await fs.mkdir(inputs,{recursive:true});
  for(const k of ['adjacency','geoids','centroids']){
    const s=files.nativeAdjacencyInputs[k];await fs.writeFile(path.join(inputs,s.name),Buffer.from(s.base64,'base64'));
  }
  const nativeTarget=path.join(desktop,ns);await seedTarget(nativeTarget);
  const cliArgs=['state',...args,'--output-dir',path.join(desktop,'vCustom')];
  cliArgs[cliArgs.indexOf('--adjacency')+1]=path.join(inputs,'custom.adj.bin');
  const result=spawnSync(cli,cliArgs,{cwd:desktop,env:{...process.env,BISECT_MANIFEST:manifest},windowsHide:true,timeout:120000});
  assert.ifError(result.error);
  await fs.writeFile(path.join(desktop,'stdout.log'),result.stdout);await fs.writeFile(path.join(desktop,'stderr.log'),result.stderr);
  assert.equal(result.status,0,result.stderr.toString());
  const nativePlan=await read(path.join(nativeTarget,'plan.rplan'));
  assert.equal(nativePlan.metadata.label,expectedLabel);
  assert.deepEqual(nativePlan.plan.units.unit_ids,fixture.ids);
  assert.deepEqual(Object.fromEntries(fixture.ids.map((id,i)=>[id,nativePlan.plan.assignment[i]+1])),run.prepared.result.assignments);
  assert.equal(await fs.readFile(path.join(nativeTarget,'keep.txt'),'utf8'),'keep');
  if(c.reset)await assert.rejects(fs.stat(path.join(nativeTarget,'data/old.txt')),e=>e.code==='ENOENT');
  else assert.equal(await fs.readFile(path.join(nativeTarget,'data/old.txt'),'utf8'),'old data');
  const packet=createProject({name:c.name,files,operation:'run-native-selected-state-source',constraints:[],result:null,lastOperation:null});
  await fs.writeFile(path.join(root,c.name+'.bisect'),JSON.stringify(packet),{flag:'wx'});
  await fs.writeFile(path.join(root,c.name+'-run.json'),JSON.stringify(run),{flag:'wx'});
  records.push({case:c.name,target:ns,expectedLabel,reset:c.reset,native_cli_args:cliArgs,assignments:run.prepared.result.assignments});
}
const packet=await read(path.join(root,'explicit-name.bisect'));
const args=JSON.parse(packet.files.stateCommandInputs.arguments_json);args[args.indexOf('--state-name')+1]='../unsafe';
packet.files.stateCommandInputs.arguments_json=JSON.stringify(args);
let unsafeCallback=false;
await assert.rejects(executeReviewedNativeSelectedStateSource(engine,(await nativeSelectedStateSourceRequest(packet.files)).input,
  {performTargetAction:()=>{unsafeCallback=true;}}),/portable native name|portable single output component/);
assert.equal(unsafeCallback,false);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-native-custom-target-reset',
  native_sha256:hash(await fs.readFile(cli)),wasm:engine.artifactIdentity,cases:records,unsafe_name_refused_before_authority:true,
  full_parity:false},null,2)+'\n',{flag:'wx'});

// Native Export reads literal legacy IDs and metadata; none is a plan size here.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const arg = key => {
  const i = process.argv.indexOf(key);
  assert(i >= 0 && process.argv[i + 1], `Supply ${key}`);
  return process.argv[i + 1];
};
const cli = await fs.realpath(arg('--cli'));
const wasm = await fs.readFile(arg('--wasm'));
const output = path.resolve(arg('--output'));
const legacy = process.argv.includes('--expect-legacy-width-failure');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const engine = await instantiateEngine(wasm);
const label = 'ExportWidth';
const max = '18446744073709551615', over = '18446744073709551616';
const formats = ['csv', 'geojson', 'gerrychain', 'reproducibility-package'];
const manifest = {label:'SourceLabel',state_code:'WA',year:'2020',chamber:'house',num_districts:2,
  population_source:'fixture',partition_mode:'fixture',seed:42,binary_version:'fixture',binary_sha256:'',
  binary_download_url:'',adjacency_file:'fixture.adj.bin',adjacency_sha256:'',adjacency_build_command:'',
  adjacency_build_version:'',tiger_source_url:'',created_at:'2026-11-06T00:00:00Z',
  balance_tolerance_pct:0.5,population_balance_valid:true};
const fields = ['num_districts','seats_per_district','total_seats','split_seeds','spectral_iters','ilp_solve_report_count',
  'flip_steps','flip_visited_count','flip_selected_plan_rank','burst_length','n_bursts','selected_burst_idx','n_units'];
const token = (value, field, number) => JSON.stringify({...value,[field]:'__COUNT__'}).replace('"__COUNT__"',number);
const assignments = id => `{"44001000100":${id},"44001000200":2}`;
const rplan = (n, id) => `{"rplan_version":"0.1","metadata":${token({label:'OriginalLabel',state_fips:'44',state_code:'RI',year:'2020',chamber:'house',population_source:'fixture',balance_tolerance_pct:0.5,created_at:'fixture',created_by:'fixture'},'num_districts',n)},"assignments":${assignments(id)},"geometry":null}`;
const cases = [
  ['baseline',{'final_assignments.json':assignments('1'),'manifest.json':JSON.stringify(manifest)},'final_assignments.json'],
  ...fields.map((field,i)=>['manifest-'+field,{[i%2===0?'final_assignments.json':'data/final_assignments.json']:assignments('1'),
    'manifest.json':token(manifest,field,max)},i%2===0?'final_assignments.json':'data/final_assignments.json','wide']),
  ['literal-assignments',{'final_assignments.json':assignments(max)},'final_assignments.json','wide'],
  ['legacy-metadata',{[label+'.rplan']:rplan(max,'1'),'final_assignments.json':'invalid'},label+'.rplan','wide'],
  ['legacy-assignments',{[label+'.rplan']:rplan('2',max)},label+'.rplan','wide'],
  ['legacy-both',{[label+'.rplan']:rplan(max,max)},label+'.rplan','wide'],
  ['unsafe-js-count',{[label+'.rplan']:rplan('9007199254740993','1')},label+'.rplan','wide'],
  ['safe-js-count',{[label+'.rplan']:rplan('9007199254740991','1')},label+'.rplan','wide'],
  ['overflow-manifest',{'final_assignments.json':assignments('1'),'manifest.json':token(manifest,'num_districts',over)},'final_assignments.json','invalid'],
  ['overflow-assignments',{'final_assignments.json':assignments(over),'data/final_assignments.json':assignments('1')},'final_assignments.json','invalid'],
  ['overflow-legacy',{[label+'.rplan']:rplan(over,'1'),'final_assignments.json':assignments('1')},label+'.rplan','invalid'],
  ['negative-id',{'final_assignments.json':assignments('-1')},'final_assignments.json','invalid'],
];
if(process.argv.includes('--original-ri-block')){
  let assignments;
  if(process.argv.includes('--original-ri-assignments')){
    assignments=await fs.readFile(arg('--original-ri-assignments'),'utf8');
  }else{
    const recorded=await fs.readFile('target/prepared-census-real/block-output.json');
    assert.equal(sha(recorded),'074a0ab06add8f2f6ee686ed80e8de64de39faa0f934f832a80303bbc0656d57');
    assignments=JSON.stringify(JSON.parse(recorded).result.prepared.result.assignments);
  }
  assert.equal(sha(assignments),'e488ae02ffd4a18b4fa063397be8240fc3d24de0c51922ead05edb630c578863');
  const literal=JSON.parse(assignments);
  assert.equal(Object.keys(literal).length,25649);
  assert(Object.entries(literal).every(([id,district])=>/^44\d{13}$/.test(id)&&Number.isSafeInteger(district)&&district>0));
  cases.push(['original-ri-block',{'data/final_assignments.json':assignments},'data/final_assignments.json']);
}

async function tree(dir,prefix='') {
  const files={};
  for(const entry of await fs.readdir(dir,{withFileTypes:true})){
    const file=path.join(dir,entry.name),name=prefix+entry.name;
    if(entry.isDirectory()) Object.assign(files,await tree(file,name+'/'));
    else files[name]=await fs.readFile(file,'utf8');
  }
  return files;
}

await fs.mkdir(output);
const records=[];
for(const [id,sources,selected,kind] of cases){
  const root=path.join(output,id),base=path.join(root,'inputs'),out=path.join(root,'output');
  const plan=path.join(base,'v1/2020/plans',label);
  await fs.mkdir(plan,{recursive:true});
  for(const [name,source] of Object.entries(sources)){
    const file=path.join(plan,name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,source);
  }
  const argv=['export','--label',label,'--output-base',base,'--out',out,'--format',...formats];
  const native=spawnSync(cli,argv,{cwd:root,encoding:'utf8',timeout:25000,windowsHide:true});
  assert.ifError(native.error);assert.equal(native.status,kind==='invalid'?1:0,native.stderr);
  await fs.writeFile(path.join(root,'native.stdout'),native.stdout);await fs.writeFile(path.join(root,'native.stderr'),native.stderr);
  const request={operation:'export-native-plan-folder',input:{label,formats,files:Object.fromEntries(
    Object.entries(sources).map(([name,text])=>[name,Buffer.from(text).toString('base64')]))}};
  let result,error;
  try{result=engine.execute(request);}catch(e){error=e.message;}
  await fs.writeFile(path.join(root,'requests.json'),JSON.stringify({folder:request}));
  await fs.writeFile(path.join(root,'result.json'),JSON.stringify({folder:result?{result}:{error}}));
  const refused=kind==='invalid'||legacy&&kind==='wide';
  if(refused){
    assert.match(error,legacy?/expected usize/:/expected u64/);
    assert.equal(result,undefined);
    if(kind==='invalid') await assert.rejects(fs.stat(out),{code:'ENOENT'});
  }else{
    assert.ok(result,error);assert.equal(result.status,'completed');assert.equal(result.selected_source,selected);
    assert.deepEqual(result.stages.map(r=>[r.format,r.status]),formats.map(format=>[format,'completed']));
    assert.deepEqual(result.source_sha256,Object.fromEntries(Object.entries(sources).map(([name,text])=>[name,sha(text)])));
    assert.deepEqual(result.files,await tree(out));
    assert.deepEqual(result.file_sha256,Object.fromEntries(Object.entries(result.files).map(([name,text])=>[name,sha(text)])));
    assert.equal(result.document_json,result.files[label+'_reproducibility/'+label+'.rplan']);
    if(id==='original-ri-block')assert.equal(result.units,25649);
    const exact=/(?:"num_districts": )(\d+)/.exec(result.document_json)[1];
    assert.equal(result.declared_districts,BigInt(exact)<=BigInt(Number.MAX_SAFE_INTEGER)?Number(exact):exact);
  }
  records.push({id,argv,native_exit:native.status,selected,refused});
}
const status=legacy?'reproduced-native-export-source-width-gap':'passed-native-export-source-width';
await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify({status,records,native:{path:cli,sha256:sha(await fs.readFile(cli))},
  wasm_sha256:sha(wasm),full_parity:false},null,2));
console.log(`${status}: ${records.length} actual CLI folder exports.`);

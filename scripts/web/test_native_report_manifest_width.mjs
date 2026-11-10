// Actual desktop Report reference and exact selected-file WASM comparisons.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const fields=['num_districts','seats_per_district','total_seats','split_seeds','spectral_iters',
 'ilp_solve_report_count','flip_steps','flip_visited_count','flip_selected_plan_rank','burst_length',
 'n_bursts','selected_burst_idx','n_units'];
const max='18446744073709551615',label='WidthOutput';
const fixture={label:'WidthRecorded',state_code:'RI',year:'2020',chamber:'congressional',num_districts:2,
 population_source:'synthetic fixture',partition_mode:'fixture',seed:42,binary_version:'report-width-fixture',
 binary_sha256:'',binary_download_url:'',adjacency_file:'fixture.adj.bin',adjacency_sha256:'',adjacency_build_command:'',
 adjacency_build_version:'',tiger_source_url:'',created_at:'2026-11-06T00:00:00Z',balance_tolerance_pct:0.5,
 population_balance_valid:true};
const analysis={'analysis/summary.json':'{}','analysis/contiguity.json':'{}','analysis/compactness.json':'{}'};
const arg=name=>{const i=process.argv.indexOf(name);assert(i>=0&&process.argv[i+1],`Supply ${name}`);return process.argv[i+1];};
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const save=async(p,v)=>fs.writeFile(p,JSON.stringify(v,null,2)+'\n',{flag:'wx'});
const output=path.resolve(arg('--output'));
const replaceCount=(field,token)=>JSON.stringify({...fixture,[field]:'__COUNT__'}).replace('"__COUNT__"',token);
const cases=[['baseline',JSON.stringify(fixture),false],...fields.map(field=>['wide-'+field,replaceCount(field,max),false]),
 ...[['negative','-1'],['fraction','1.5'],['overflow','18446744073709551616'],['string','"2"']]
  .map(([name,token])=>['invalid-'+name,replaceCount('num_districts',token),true])];

async function labelReportCountBoundary(engine,out,red){
 const plan=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'),native=JSON.parse(plan).plan;
 assert.equal(native.units.state,'WA');assert.equal(native.units.year,2020);
 const label='WidthCount',year='2020',state='washington',prefix=`analysis/${label}/${year}/`,statePrefix=prefix+state+'/';
 const index=JSON.stringify({label,year,run:label,run_index_sha256:'0'.repeat(64),types:['manifest','contiguity','statistics','compactness'],
  created:'2026-11-06T00:00:00Z',states:{[state]:{status:'ok'}}});
 const count=String((1n<<32n)+BigInt(native.k));
 for(const [name,token] of [['baseline',String(native.k)],['wide-alias',count]]){
  const files={[prefix+'index.json']:index,[statePrefix+'plan.rplan']:plan,
   [statePrefix+'manifest.json']:JSON.stringify({...fixture,label,state_code:'WA',rplan_path:'plan.rplan',num_districts:'__COUNT__'}).replace('"__COUNT__"',token),
   ...Object.fromEntries(Object.entries(analysis).map(([key,v])=>[statePrefix+path.basename(key),v]))};
  const request={operation:'run-label-report',input:{analysis_index_json:index,files,file_sha256:Object.fromEntries(Object.entries(files).map(([k,v])=>[k,hash(v)])),generated_at:'2026-11-06T00:00:00Z',format:'json',audit_with_report:true}};
  const response=engine.execute(request),row=response.states.find(v=>v.state_name===state);
  await save(path.join(out,'label-report-'+name+'.json'),{request,response});
  assert.equal(row.status,name==='baseline'?'generated':'failed',row.error);
  if(name==='wide-alias'){
   assert.match(row.error,red?/expected usize/:/district count does not match the manifest/);
   assert(!Object.keys(response.files).some(key=>key.endsWith('_report.json')));
  }
 }
}

async function prepare(){
 const cli=await fs.realpath(arg('--cli')),native={path:cli,sha256:hash(await fs.readFile(cli))},records=[];
 await fs.mkdir(output);
 for(const [id,text,invalid] of cases){
  const root=path.join(output,id),base=path.join(root,'outputs'),plan=path.join(base,'v1/2020/plans',label);
  await fs.mkdir(plan,{recursive:true});await fs.mkdir(path.join(plan,'analysis'));
  const files={'manifest.json':text,...analysis};
  for(const [name,raw] of Object.entries(files))await fs.writeFile(path.join(plan,name),raw,{flag:'wx'});
  const commands={};
  for(const [mode,extra] of [['audit',['--audit-only']],['report',['--format','html','json','--audit-with-report']]]){
   const out=path.join(root,mode),argv=['report','--label',label,'--year','2020','--version','v1','--output-base',base,'--out',out,...extra];
   const p=spawnSync(cli,argv,{cwd:root,encoding:'utf8',windowsHide:true,timeout:45000,maxBuffer:8*1024*1024});
   assert.ifError(p.error);await fs.writeFile(path.join(root,mode+'.stdout'),p.stdout);await fs.writeFile(path.join(root,mode+'.stderr'),p.stderr);
   assert.equal(p.status,0,p.stderr);commands[mode]={argv,exit:0,stdout_sha256:hash(p.stdout),stderr_sha256:hash(p.stderr)};
  }
  const audit=await fs.readFile(path.join(root,'audit/audit.json'),'utf8'),companion=await fs.readFile(path.join(root,'report/audit.json'),'utf8');
  assert.equal(audit,companion);assert.equal(JSON.parse(audit).label,invalid?label:fixture.label);
  const json=await fs.readFile(path.join(root,'report',label+'_report.json'),'utf8'),html=await fs.readFile(path.join(root,'report',label+'_report.html'),'utf8');
  assert.equal(JSON.parse(json).label,invalid?label:fixture.label);
  if(id==='wide-num_districts'){assert.match(json,/"num_districts": 18446744073709551615/);assert(audit.includes('--districts '+max));}
  records.push({id,invalid,manifest_sha256:hash(text),audit_sha256:hash(audit),json_sha256:hash(json),html_sha256:hash(html),commands});
 }
 await save(path.join(output,'evidence.json'),{status:'passed-actual-native-report-width-reference',native,records,full_parity:false});
 console.log(`Native Report: ${records.length} audit-only and complete report cases.`);
}

async function verify(){
 const reference=await fs.realpath(arg('--cases')),expected=await read(path.join(reference,'evidence.json'));
 assert.equal(expected.status,'passed-actual-native-report-width-reference');assert.equal(expected.records.length,cases.length);
 const wasm=await fs.readFile(arg('--wasm')),engine=await instantiateEngine(wasm),outcomes=[],red=process.argv.includes('--expect-legacy-width-failure');
 await fs.mkdir(output);
 for(const [id,text,invalid] of cases){
  const root=path.join(reference,id),out=path.join(output,id);await fs.mkdir(out);
  const rec=expected.records.find(r=>r.id===id);assert.equal(rec.manifest_sha256,hash(text));
  const audit=await fs.readFile(path.join(root,'audit/audit.json'),'utf8'),json=await fs.readFile(path.join(root,'report',label+'_report.json'),'utf8'),html=await fs.readFile(path.join(root,'report',label+'_report.html'),'utf8');
  assert.deepEqual([hash(audit),hash(json),hash(html)],[rec.audit_sha256,rec.json_sha256,rec.html_sha256]);
  const files=Object.fromEntries(Object.entries({'manifest.json':text,...analysis}).map(([name,raw])=>[name,Array.from(Buffer.from(raw))]));
  const auditRequest={operation:'assemble-commission-audit',manifest_json:text};
  const reportRequest={operation:'assemble-commission-report',files,generated_at:JSON.parse(json).generated_at,
   delivery:{label,formats:['html','json'],audit_with_report:true}};
  if(red&&id.startsWith('wide-')){
   const failures={};
   for(const [kind,request] of [['audit',auditRequest],['report',reportRequest]]){
    assert.throws(()=>engine.execute(request),error=>{failures[kind]=error.message;return /expected usize/.test(error.message);});
   }
   await save(path.join(out,'requests.json'),{audit:auditRequest,report:reportRequest});
   await save(path.join(out,'legacy-failures.json'),failures);
   outcomes.push({id,legacy_width_rejected_audit:true,legacy_width_rejected_report:true});continue;
  }
  if(invalid){
   const diagnostic=red?/expected usize/:/expected u64/;
   assert.throws(()=>engine.execute(auditRequest),diagnostic);assert.throws(()=>engine.execute(reportRequest),diagnostic);
   const fallback={label,year:'2020'};auditRequest.fallback=fallback;reportRequest.fallback=fallback;
  }
  const a=engine.execute(auditRequest),r=engine.execute(reportRequest);
  assert.equal(a.audit_json,audit);assert.equal(a.manifest_sha256,hash(text));assert.equal(a.manifest_resolution.fallback_used,invalid);
  assert.equal(r.audit_json,audit);assert.equal(r.report_json,json);assert.equal(r.report_html,html);
  assert.deepEqual(r.report_files,{'audit.json':audit,[label+'_report.json']:json,[label+'_report.html']:html});
  assert.equal(r.source_sha256['manifest.json'],hash(text));assert.equal(r.manifest_resolution.fallback_used,invalid);
  await save(path.join(out,'requests.json'),{audit:auditRequest,report:reportRequest});
  await save(path.join(out,'result.json'),{audit:a,report:r});
  outcomes.push({id,invalid,exact_audit:true,exact_full_json_html:true,exact_delivery:true});
 }
 await labelReportCountBoundary(engine,output,red);
 const status=red?'reproduced-legacy-width-audit-and-report-refusals':'passed-report-width-native-and-wasm';
 await save(path.join(output,'evidence.json'),{status,native:expected.native,wasm_sha256:hash(wasm),outcomes,label_report_count_alias_refused:true,full_parity:false});
 console.log(`Report width: ${status} (${outcomes.length} cases).`);
}

assert.notEqual(process.argv.includes('--prepare'),process.argv.includes('--verify'));
await (process.argv.includes('--prepare')?prepare():verify());

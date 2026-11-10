import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';

import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {explicitCountySplitsCase} from './fixtures/explicit-county-splits-case.mjs';
const arg=name=>{const i=process.argv.indexOf(name);assert(i>0&&process.argv[i+1],name);return path.resolve(process.argv[i+1]);};
const wasm=arg('--wasm'), cli=arg('--cli'), fixtureWasm=arg('--fixture-wasm'), rewrite=arg('--rewriter');
const base=path.resolve('target/explicit-municipal-splits');await fs.mkdir(base,{recursive:true});
const output=await fs.mkdtemp(path.join(base,'runtime-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const pin=async p=>{const b=await fs.readFile(p);return {path:p,sha256:sha(b),bytes:b.length};};
const fixture=await explicitCountySplitsCase(await instantiateEngine(await fs.readFile(fixtureWasm)));
const engine=await instantiateEngine(await fs.readFile(wasm));
const {plan,context}=fixture;
let legacyReport=null;
const document=JSON.parse(plan).plan, assignments=document.assignment, units=document.units.unit_ids;
const inputs=['plan.rplan','context.rctx','manifest.json','data/final_assignments.json'];
const run=(exe,args)=>spawnSync(exe,args,{cwd:output,encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024});
const raw=async(name,text)=>fs.writeFile(path.join(output,name),text,{flag:'wx'});
const rows=[];

async function install(name){
 const root=path.join(output,name), target=path.join(root,'vExplicit/2020/plans/ExplicitCounties');
 await fs.mkdir(path.join(target,'data'),{recursive:true});
 for(const input of inputs){
  const source=fixture.output.files_base64[input];
  assert.equal(typeof source,'string',input);
  await fs.writeFile(path.join(target,input),Buffer.from(source,'base64'),{flag:'wx'});
 }
 return {root,target};
}
function oracle(ids){
 const groups=new Map();
 units.forEach((unit,i)=>{if(ids[i]===null)return;const current=groups.get(ids[i])??{units:0,districts:new Set()};
  current.units++;current.districts.add(assignments[i]+1);groups.set(ids[i],current);});
 const ordered=units.map((u,i)=>[u,ids[i]]).sort(([a],[b])=>Buffer.compare(Buffer.from(a),Buffer.from(b)));
 const digest=sha('bisect-explicit-municipal-identity-v1\0{'+ordered.map(([u,id])=>JSON.stringify(u)+':'+JSON.stringify(id)).join(',')+'}');
 const split=[...groups].sort(([a],[b])=>Buffer.compare(Buffer.from(a),Buffer.from(b))).filter(([,g])=>g.districts.size>1)
  .map(([id,g])=>({municipality_id:id,municipality_name:null,districts_containing:[...g.districts].sort((a,b)=>a-b),unit_count:g.units,split_severity:g.districts.size}));
 return {digest,known:ids.filter(id=>id!==null).length,total:groups.size,split};
}

try{
 for(const [name,ids] of Object.entries({legacy:null,complete:['M-β','M-A','M-β','M-A'],distinctUnicode:['Å','A\u030a','Å','A\u030a'],zero:assignments.map(d=>`M-${d}`),partial:['M-β',null,'M-β','M-A'],allUnknown:units.map(()=>null),wrongOrder:['M-A','M-B','M-A','M-B'],badLength:['M-A'],invalid:['M-β',' ', 'M-β','M-A']})){
  const {root,target}=await install(name), contextPath=path.join(target,'municipal.rctx');
  let contextText=context;
  if(ids){const item=JSON.parse(context);item.subdivisions.municipal_ids=ids;
   if(name==='wrongOrder')item.units.unit_ids.reverse();
   const text=JSON.stringify(item);await raw(name+'-context-input.json',text);
   if(name==='badLength')contextText=text;
   else{const result=run(rewrite,[path.join(output,name+'-context-input.json')]);
    assert.equal(result.status,0,result.stderr);contextText=result.stdout;assert.deepEqual(JSON.parse(contextText).subdivisions.municipal_ids,ids);}
  }
  await fs.writeFile(contextPath,contextText,{flag:'wx'});
  const request={operation:'analyze-plan-splits',document_json:plan,context_json:contextText};
  await raw(name+'-request.json',JSON.stringify(request));
  const native=run(cli,['analyze','--state','RI','--label','ExplicitCounties','--year','2020','--version','vExplicit','--output-dir',root,'--types','splits','--splits-context',contextPath,'--stdout','--force']);
  await raw(name+'-native.stdout',native.stdout);await raw(name+'-native.stderr',native.stderr);
  const row={name,context:await pin(contextPath),request:await pin(path.join(output,name+'-request.json')),
   native_status:native.status,native_stdout:await pin(path.join(output,name+'-native.stdout'))};rows.push(row);
  const refusal={invalid:/invalid explicit municipal identity/i,wrongOrder:/plan\/context ordered unit identity mismatch/i,badLength:/subdivision|context hash/i}[name];
  if(refusal){assert.notEqual(native.status,0);assert.match(native.stderr,refusal);
   assert.throws(()=>engine.execute(request),refusal);continue;}
  assert.equal(native.status,0,native.stderr);
  const result=engine.execute(request);await raw(name+'-wasm.json',JSON.stringify(result));
  row.wasm_result=await pin(path.join(output,name+'-wasm.json'));
  assert.deepEqual(JSON.parse(native.stdout),result.report);assert.equal(native.stdout.replace(/\r\n/g,'\n'),result.report_json+'\n');
  assert.equal(result.context_source_sha256,sha(contextText));assert.equal(result.report_sha256,sha(result.report_json));
  if(!ids){assert.equal(result.schema_version,'bisect-explicit-plan-splits-v1');assert.equal(result.report.municipal_identity_schema,undefined);assert.equal(result.report.municipal_identity_sha256,undefined);assert.equal(result.report.municipalities.available,false);legacyReport=result.report;continue;}
  assert.deepEqual(result.report.counties,legacyReport.counties);
  const expected=oracle(ids),m=result.report.municipalities;
  assert.equal(result.report.municipal_identity_sha256,expected.digest);
  assert.equal(result.schema_version,'bisect-explicit-plan-splits-v2');
  assert.equal(result.report.municipal_identity_schema,'bisect-explicit-municipal-membership-v1');
  assert.equal(result.report.municipal_identity_source,'matching-rctx-explicit-municipalities');
  assert.equal(result.municipal_identity_source,'matching-rctx-explicit-municipalities');
  assert.equal(m.known_units,expected.known);assert.equal(m.missing_units,units.length-expected.known);
  if(expected.known===units.length){assert.equal(m.available,true);assert.equal(m.total,expected.total);assert.equal(m.split,expected.split.length);assert.deepEqual(m.split_list,expected.split);}
  else{assert.equal(m.available,false);assert.equal(m.total,null);assert.equal(m.split,null);assert.equal(m.preservation_score,null);assert.deepEqual(m.split_list,[]);}
  if(name==='complete')assert(expected.split.length>0);if(name==='distinctUnicode'){assert.equal(m.total,2);assert.equal(m.split,2);}if(name==='zero')assert.equal(m.split,0);
  assert.throws(()=>engine.execute({...request,geography_json:JSON.stringify({place_to_tracts:{'other':[units[0]]}})}),/single municipal membership source/);
  const pkg=engine.execute({operation:'analyze-plan-package',input:{document_json:plan,context_json:contextText,geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,tolerance_percent:25,election_settings:{bootstrap_seed:'42',bootstrap_replicates:5},compactness:null}});
  assert.equal(pkg.steps.find(step=>step.name==='splits').status,'computed');assert.equal(pkg.files['splits.json'],result.report_json);
 }
 await raw('evidence.json',JSON.stringify({status:'passed-explicit-municipal-cli-wasm-and-package',rows,cli:await pin(cli),wasm:await pin(wasm),rewriter:await pin(rewrite),fixture_wasm:await pin(fixtureWasm),full_parity:false},null,2));
 console.log(JSON.stringify({root:output,receipt:path.join(output,'evidence.json'),cases:rows.length}));
}catch(error){await raw('failure.txt',error.stack);throw error;}

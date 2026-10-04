import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
await fs.mkdir('target/character-csv-fixtures',{recursive:true});
let sequence=0,exact=0,rejected=0;
const economicHeader='geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11';
const housingHeader='geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage';
const economicRow='01001020100,1000,10,20,30,400,40,100,50,50';
const housingRow='01001020100,.5,.4,.75,.5';
const request=(kind,text,change={})=>({operation:'import-character-csv',kind,state:'AL',year:'2020',data_year:'2022',source_label:'SYNTHETIC character fixture',source_base64:Buffer.from(text).toString('base64'),...change});
async function native(req){const path=`target/character-csv-fixtures/request-${sequence++}.json`;await fs.writeFile(path,JSON.stringify(req));return spawnSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024});}
async function success(req){const ref=await native(req);assert.equal(ref.status,0,ref.stderr);const actual=engine.execute(req);assert.deepEqual(actual,JSON.parse(ref.stdout));exact++;return actual;}
async function failure(req){const ref=await native(req);assert.notEqual(ref.status,0);assert.throws(()=>engine.execute(req));rejected++;}
for(const kind of ['economic','housing'])for(const ending of ['\n','\r\n'])for(const upper of [false,true]){
 const header=kind==='economic'?economicHeader:housingHeader,row=kind==='economic'?economicRow:housingRow;
 const input=await success(request(kind,'\ufeff'+(upper?header.toUpperCase():header)+ending+row.replace('01001020100','1001020100')+ending));
 assert.equal(input.schema_version,'bisect-character-v1');assert.equal(input.year,'2020');assert.equal(input.data_year,'2022');
 assert.deepEqual(input.data.characters['01001020100'],kind==='economic'?{commercial_intensity:.6,industrial_fraction:.1,jobs_per_resident:.1}:{pct_single_family:.5,pct_multifamily:.4,pct_owner:.75,housing_vintage:.5});
 if(kind==='economic')assert.equal(input.data.raw_counts['01001020100'].c000,1000);
}
// Reordered/extra columns, quoted numeric cells, aliases, and zero-job policy.
await success(request('housing','extra,housing_vintage,pct_owner,pct_mf,pct_sf,geoid\n"note,quoted",.5,.75,.4,.5,01001020100'));
const zero=await success(request('economic',economicHeader+'\n01001020100,0,0,0,0,0,0,0,0,0'));assert.deepEqual(zero.data.characters['01001020100'],{commercial_intensity:0,industrial_fraction:0,jobs_per_resident:0});
const clamped=await success(request('economic',economicHeader+'\n01001020100,20000,10000,10000,10000,10000,10000,10000,10000,10000'));assert.deepEqual(clamped.data.characters['01001020100'],{commercial_intensity:1,industrial_fraction:1,jobs_per_resident:1});
await success(request('economic',economicHeader+'\n'+economicRow.replace(',1000,',',1000.5,')));
for(const kind of ['economic','housing']){
 const header=kind==='economic'?economicHeader:housingHeader,row=kind==='economic'?economicRow:housingRow;
 const valid=header+'\n'+row;
 for(const req of [request(kind,''),request(kind,header),request(kind,'bad\n'+row),request(kind,header+',GEOID\n'+row+',01001020100'),request(kind,header+',\n'+row+',x'),request(kind,valid+'\n'+row.replace('01001020100','1001020100')),request(kind,valid.replace('01001020100','44001020100')),request(kind,valid.replace('01001020100','abc')),request(kind,valid+'\n01001020200,1'),request(kind,valid,{state:'XX'}),request(kind,valid,{year:'2024'}),request(kind,valid,{data_year:'20x2'}),request(kind,valid,{source_label:' '}),request(kind,valid,{source_label:'é'.repeat(101)}),request(kind,valid,{source_base64:'???'}),request(kind,valid,{source_base64:Buffer.from([255]).toString('base64')}),request(kind,valid,{unexpected:true}),request(kind,valid,{kind:'zone-membership'}),...['-0','-1','NaN','Infinity','9007199254740992',''].map(value=>request(kind,header+'\n'+row.replace(/,([^,]+)/,','+value)))])await failure(req);
 await failure(request(kind,'x'.repeat(8*1024*1024+1)));
 const rows=Array.from({length:100001},(_,i)=>row.replace('01001020100',`01001${String(i).padStart(6,'0')}`));await failure(request(kind,header+'\n'+rows.join('\n')));
}
await failure(request('housing',housingHeader+'\n'+housingRow.replace(',.5,',',1.01,')));
await failure(request('housing',housingHeader+',pct_sf\n'+housingRow+',.5'));
let weighted=0,weightRejections=0;
for(const kind of ['economic','housing']){
 const ids=['01001020100','01001020200','01001020300'];
 const text=kind==='economic'?economicHeader+'\n01001020100,0,0,0,0,0,0,0,0,0\n01001020200,1000,10,20,30,400,40,100,50,50\n01001020300,1000,10,20,30,400,40,100,50,50':housingHeader+'\n01001020100,0,0,0,0\n01001020200,1,0,0,0\n01001020300,0,1,0,0';
 const input=await success(request(kind,text));
 const base={operation:'build-character-weights',input,geoids:ids,edges:[[0,1,100],[0,2,80],[1,2,60]],alpha:.5};
 for(const alpha of [0,.5,1]){
  const req={...base,alpha},result=await success(req);weighted++;
  const expected=kind==='economic'?[[0,1,100*(alpha+(1-alpha)*.5)],[0,2,80*(alpha+(1-alpha)*.5)],[1,2,60]]:[[0,1,100],[0,2,80],[1,2,60*alpha]];
  assert.deepEqual(result.edges,expected);assert.match(result.character_hash,/^[0-9a-f]{64}$/);
  const hash=createHash('sha256').update(['BISECT_CHARACTER_V1','AL','2020','2022',kind,''].join('\0'));
  const fields=kind==='economic'?['commercial_intensity','industrial_fraction','jobs_per_resident']:['pct_single_family','pct_multifamily','pct_owner','housing_vintage'];
  for(const id of ids){hash.update(id);for(const field of fields){const bytes=Buffer.alloc(8);bytes.writeDoubleLE(input.data.characters[id][field]);hash.update(bytes);}}
  assert.equal(result.character_hash,hash.digest('hex'));
  const label=structuredClone(req);label.input.source_label='Different description';assert.equal(engine.execute(label).character_hash,result.character_hash);
  const order=structuredClone(req);order.input.data.characters=Object.fromEntries(Object.entries(order.input.data.characters).reverse());assert.equal(engine.execute(order).character_hash,result.character_hash);
  const scope=structuredClone(req);scope.input.data_year='2021';assert.notEqual(engine.execute(scope).character_hash,result.character_hash);
 }
 const bad=[];
 for(const alpha of [-1,1.01])bad.push({...base,alpha});
 for(const edges of [[[0,0,1]],[[1,0,1]],[[0,3,1]],[[0,1,0]],[[0,1,-1]],[[0,1,1],[0,1,2]]])bad.push({...base,edges});
 bad.push({...base,geoids:ids.slice(0,2)}, {...base,geoids:[ids[0],ids[0],ids[2]]}, {...base,geoids:['44001020100',...ids.slice(1)]});
 for(const change of ['schema','missing','unexpected','range','unknown']){
  const req=structuredClone(base);
  if(change==='schema')req.input.schema_version='unknown';
  if(change==='missing')delete req.input.data.characters[ids[0]];
  if(change==='unexpected')req.input.data.characters['01001020400']=req.input.data.characters[ids[0]];
  if(change==='range')req.input.data.characters[ids[0]][kind==='economic'?'commercial_intensity':'pct_owner']=1.01;
  if(change==='unknown')req.input.data.characters[ids[0]].extra=true;
  bad.push(req);
 }
 if(kind==='economic'){
  const req=structuredClone(base);req.input.data.raw_counts[ids[1]].c000=1001;bad.push(req);
  const missing=structuredClone(base);delete missing.input.data.raw_counts[ids[0]];bad.push(missing);
 }
 for(const req of bad){await failure(req);weightRejections++;}
 // Non-axis vectors exercise sqrt and floating-point portability, with an
 // independent cosine oracle rather than replaying the Rust implementation.
 const varied=await success(request(kind,kind==='economic'?economicHeader+'\n'+economicRow+'\n01001020200,1700,40,60,90,150,70,220,330,80':housingHeader+'\n'+housingRow+'\n01001020200,.1,.7,.6,.2'));
 const vectors=Object.values(varied.data.characters).map(character=>Object.values(character));
 const similarity=vectors[0].reduce((sum,value,i)=>sum+value*vectors[1][i],0)/(Math.hypot(...vectors[0])*Math.hypot(...vectors[1]));
 for(const alpha of [.25,.75]){
  const result=await success({operation:'build-character-weights',input:varied,geoids:ids.slice(0,2),edges:[[0,1,123.45]],alpha});weighted++;
  assert.ok(Math.abs(result.edges[0][2]-123.45*(alpha+(1-alpha)*similarity))<1e-10);
 }
}
await fs.writeFile('target/character-csv-verification.json',JSON.stringify({exact_native_wasm_operations:exact,weighted_cases:weighted,matching_rejections:rejected,weight_rejections:weightRejections},null,2)+'\n');
console.log(`Character CSV/weights: ${exact} exact native/WASM operations (${weighted} analytic weight cases) and ${rejected} matching rejections passed.`);

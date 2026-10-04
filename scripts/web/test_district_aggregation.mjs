import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {encodeBytes,decodePackageFiles} from '../../web/lab/package-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture='docs/examples/rcount-golden-packages/district-aggregation-rplan',root='target/district-aggregation-fixtures';await fs.mkdir(root,{recursive:true});
async function filesAt(dir,prefix=''){const files={};for(const entry of await fs.readdir(dir,{withFileTypes:true})){const key=prefix+entry.name,full=path.join(dir,entry.name);if(entry.isDirectory())Object.assign(files,await filesAt(full,key+'/'));else files[key]=[...await fs.readFile(full)];}return files;}
const files=await filesAt(fixture+'/package'),document=JSON.parse(await fs.readFile(fixture+'/plan.rplan.json','utf8')),context=JSON.parse(await fs.readFile(fixture+'/context.rctx','utf8'));
const crosswalk=await fs.readFile(fixture+'/crosswalks.ndjson'),rows=crosswalk.toString().trim().split('\n').map(JSON.parse);
const explicitRows=rows.map(({source_refs,...row})=>row),explicitCrosswalk=Buffer.from(explicitRows.map(JSON.stringify).join('\n')+'\n');
const explicitFiles=await filesAt('docs/examples/rcount-golden-packages/summary-basic');
const base={operation:'aggregate-districts',files,document,context:null,crosswalk_base64:null,contest_id:'syn-2024-mayor',status:'canvassed',output_format:'pretty-json'};
let sequence=0;
async function native(request){const file=path.join(root,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));return spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});}
async function check(name,request){const reference=await native(request);assert.equal(reference.status,0,reference.stderr);const result=engine.execute(request);assert.deepEqual(result,JSON.parse(reference.stdout));const encoded=Object.fromEntries(Object.entries(request.files).map(([key,value])=>[key,encodeBytes(Uint8Array.from(value))]));const projectFiles={plan:request.document,packageFiles:encoded,aggregationSettings:{contest_id:request.contest_id,status:request.status,output_format:request.output_format}};if(request.context)projectFiles.context=request.context;if(request.crosswalk_base64)projectFiles.crosswalkFile={name:'crosswalks.ndjson',base64:request.crosswalk_base64};const project=createProject({name,files:projectFiles,operation:'aggregate-districts',constraints:[],result,lastOperation:'aggregate-districts'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));await fs.writeFile(path.join(root,name+'-native.json'),Buffer.from(result.transcript_base64,'base64'));assert.ok(countResultTables(result)[0].rows.every(row=>typeof row[2]==='string'));return result;}
for(const format of ['json','pretty-json'])for(const scope of ['direct','context','crosswalk']){
  const result=await check(scope+'-'+format,{...base,files:scope==='crosswalk'?explicitFiles:files,output_format:format,context:scope==='direct'?null:context,crosswalk_base64:scope==='crosswalk'?explicitCrosswalk.toString('base64'):null});assert.deepEqual(result.district_totals.map(d=>d.counted_ballots),['80','60']);
  if(scope==='context')assert.deepEqual(JSON.parse(Buffer.from(result.transcript_base64,'base64').toString()),JSON.parse(await fs.readFile(fixture+'/district-aggregation-transcript.json','utf8')));
}
const large='9007199254740993',total=(BigInt(large)+2n).toString();
const csv='contest_id,contest_title,vote_for,selection_id,selection_label,selection_kind,reporting_unit_id,reporting_unit_kind,parent_jurisdiction,status,votes,undervotes,overvotes,blank_contests,counted_ballots\n'+[['syn:precinct:P-001','precinct',large],['syn:precinct:P-002','precinct','2'],['syn:jurisdiction:SYN','jurisdiction-total',total]].map(([id,kind,count])=>`syn-2024-mayor,Mayor,1,cand-a,Candidate A,candidate,${id},${kind},county,canvassed,${count},0,0,0,${count}\n`).join('');
const imported=engine.execute({operation:'import-statement-csv',source_base64:Buffer.from(csv).toString('base64'),metadata:{country:'US',state:'SYN',county:'Synthetic County',date:'2024-11-05',election_type:'general',scope:'county',status:'canvassed'}});
const largeBase={...base,files:decodePackageFiles(imported.package_files)};
const largeResult=await check('large-direct',largeBase);assert.equal(largeResult.district_totals[0].counted_ballots,large);assert.equal(largeResult.district_totals[0].totals[0].votes,large);assert.ok(Buffer.from(largeResult.transcript_base64,'base64').toString().includes('"votes": '+large));
const projected=explicitRows.map(row=>({...row,to_unit_id:'syn:precinct:P-001'}));
const projectedBytes=Buffer.from(projected.map(JSON.stringify).join('\n')+'\n');
const projectedResult=await check('large-projection',{...largeBase,context,crosswalk_base64:projectedBytes.toString('base64')});assert.deepEqual(projectedResult.district_totals.map(d=>d.counted_ballots),[total,'0']);
const half=explicitRows.flatMap(row=>document.plan.units.unit_ids.map(id=>({...row,to_unit_id:id,weight:{num:1,den:2}})));
for(const request of [{...base,contest_id:'missing'},{...base,status:'unofficial'},{...base,document:{...document,plan:{...document.plan,assignment:[99,1]}}},{...base,context:{...context,context_hash:'sha256:'+'0'.repeat(64)}},{...base,crosswalk_base64:crosswalk.toString('base64')},{...base,context,crosswalk_base64:projectedBytes.toString('base64')},{...base,context,crosswalk_base64:crosswalk.toString('base64')},{...largeBase,context,crosswalk_base64:Buffer.from(half.map(JSON.stringify).join('\n')).toString('base64')},{...base,output_format:'execute-code'}]){assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);}
const damaged=structuredClone(base);const source=Object.keys(damaged.files).find(key=>key.startsWith('sources/')&&key!=='sources/source-index.json');damaged.files[source][0]^=1;const diagnostic=engine.execute(damaged);assert.equal(diagnostic.status,'fail');assert.equal(diagnostic.district_totals[0].counted_ballots,'80');
const bounded=countResultTables({schema_version:'bisect-district-aggregation-v1',district_totals:Array.from({length:501},()=>({district_label:'<script>alert(1)</script>',counted_ballots:large,totals:Array(501).fill({selection_id:'a',votes:large})}))});assert.equal(bounded[0].rows.length,500);assert.equal(bounded[1].rows.length,500);assert.equal(bounded[0].rows[0][0],'<script>alert(1)</script>');
console.log('District aggregation: eight exact native/WASM results, native golden transcript parity, direct/context/crosswalk paths and both output formats, exact large counts, projected conservation, Save/Open and nine matching rejections passed. Modified source stays failed; count tables remain bounded inert text.');

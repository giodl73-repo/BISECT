import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {CIVIC_DEFAULTS,civicRequest,civicZip,readCivicSources} from '../../web/lab/civic-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/civic-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let parity=0,cli=0,rejections=0;
const header='geoid,comment_id,label,source,source_url,confidence,submitted_at,extra\n';
const a=header+'01001000100,c1,Harbor,public,https://example.org,high,2026-01-01T00:00:00Z,"a,b"\n';
const b=header+'01001000100,c2,Harbor,public,,low,recorded,=1+1\n01001000200,c1,Hill,public,,bogus,not-a-date,extra\n';
const input=(label,text)=>({label,source_name:label+'.csv',csv_base64:Buffer.from(text).toString('base64')});
const req=(inputs,validate_mode='strict')=>({operation:'inspect-civic-coi',inputs,state:'AL',year:'2020',validate_mode});
async function native(r){await fs.writeFile(root+'/request.json',JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r){const out=engine.execute(r);assert.deepEqual(out,await native(r));parity++;
 for(const [i,source]of r.inputs.entries()){
  const file=`${root}/source-${parity}-${i}.csv`,base=`${root}/cli-${parity}`;await fs.writeFile(file,Buffer.from(source.csv_base64,'base64'));
  const ref=spawnSync('target/debug/examples/civic_reference.exe',['ingest',file,source.label,r.state,r.year,r.validate_mode,base],{encoding:'utf8'});const report=out.inputs[i];assert.equal(ref.status,report.native_exit_code,ref.stderr);if(report.error)assert.ok(ref.stderr.includes(report.error),ref.stderr);else{const folder=`${base}/v1/civic_inputs/${source.label}`;assert.equal(await fs.readFile(folder+'/normalized.csv','utf8'),report.normalized_csv);const manifest=JSON.parse(await fs.readFile(folder+'/manifest.json','utf8'));assert.equal(manifest.validation.errors,report.errors);assert.deepEqual(manifest.validation.warnings_detail,report.warnings);assert.equal(manifest.normalized.row_count,report.row_count);assert.equal(manifest.input.original_encoding_detected,report.encoding);assert.equal(manifest.input.original_line_endings,report.line_endings);assert.equal('sha256:'+manifest.normalized.sha256,report.normalized_sha256);}cli++;
 }
 if(out.conflicts_available&&r.inputs.length>1){const ref=spawnSync('target/debug/examples/civic_reference.exe',['conflicts',`${root}/cli-${parity}`,...r.inputs.map(i=>i.label)],{encoding:'utf8'});assert.equal(ref.status,0,ref.stderr);assert.equal(ref.stdout.trimEnd(),out.conflicts_json);cli++;}return out;
}
const result=await exact(req([input('alpha',a),input('beta',b)]));assert.equal(result.conflicts.coi_overlap.length,1);assert.equal(result.conflicts.coi_label_mismatch.length,1);assert.equal(result.inputs[1].errors,0);assert.ok(result.inputs[1].normalized_csv.includes('=1+1'));
for(const text of [a,'\ufeff'+a.replaceAll('\n','\r\n'),'# supplied schema\n'+a,header,a.replace('01001000100','1001000100'),a.replace('https://example.org','http://127.0.0.1'),a.replace('https://example.org','javascript:alert(1)'),a.replace('01001000100','bad'),a.replace('https://example.org','http://[::ffff:127.0.0.1]'),a.replace('extra','duplicate,extra'),'',Buffer.from([0xff,0xfe,65,0]),'wrong,header\n1,2\n']){
 if(text==='')continue;for(const mode of ['strict','lenient','advisory'])await exact(req([input('one',text)],mode));
}
await exact(req([input('within',a+a.split('\n')[1]+'\n')]));
const many=header+'01001000100,c,L,s,,h,d,x\n'.repeat(50_000);await exact(req([input('rows',many)]));
const padded=a.replace('"a,b"','"'+ 'x'.repeat(3*1024*1024-Buffer.byteLength(a)+3)+'"');assert.equal(Buffer.byteLength(padded),3*1024*1024);await exact(req([input('boundary',padded)]));
for(const r of [req([]),req(Array.from({length:21},(_,i)=>input('n'+i,a))),req([input('same',a),input('same',b)]),{...req([input('a',a)]),extra:true},req([input('a',a)],'unknown'),req([input('',a)]),req([input('a',padded+' ')]),req([{...input('a',a),csv_base64:'?'}]),req([input('a',a)],null),{...req([input('a',a)]),state:''},req([input('rows',many+'01001000100,c,L,s,,h,d,x\n')])]){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
const sources=[{name:'alpha.csv',base64:Buffer.from(a).toString('base64')},{name:'beta.csv',base64:Buffer.from(b).toString('base64')}],settings={...CIVIC_DEFAULTS,state:'AL',sources};const project=createProject({name:'Civic submissions',files:{civicInputs:settings},operation:'inspect-civic-coi',constraints:[],result,lastOperation:'inspect-civic-coi'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(civicRequest(settings).inputs[0].csv_base64,sources[0].base64);assert.equal(countResultTables(result)[0].rows[0][1],'assessed');assert.throws(()=>parseProject(JSON.stringify({...project,files:{civicInputs:{...settings,extra:true}}})));
let read=false;await assert.rejects(()=>readCivicSources([{name:'large.csv',size:3*1024*1024+1,arrayBuffer:async()=>{read=true;}}]));assert.equal(read,false);
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/assessment.zip',civicZip(result));await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,cli,rejections,scope:result.scope},null,2));console.log(`Civic assessment: ${parity} native/WASM results, ${cli} actual CLI checks and ${rejections} matching rejections passed.`);

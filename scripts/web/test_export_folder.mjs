import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {readExportFolder,exportFolderRequest,exportFolderZip} from '../../web/lab/export-folder-files.js';
const root=path.resolve('target/export-folder');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const b=s=>Buffer.from(typeof s==='string'?s:JSON.stringify(s)),sha=s=>createHash('sha256').update(s).digest('hex');
const assignments=b({'44001000100':1,'44001000200':2}),other=b({'44001000900':9});
const fixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8')),manifest=Buffer.from(fixture.files['manifest.json']);
const legacy={rplan_version:'0.1',metadata:{label:'recorded-label',state_fips:'44',state_code:'RI',year:'2020',chamber:'congressional',num_districts:2,population_source:'total',balance_tolerance_pct:.5,created_at:'recorded',created_by:'source'},assignments:JSON.parse(assignments),geometry:null};
const formats=['csv','gerrychain','geojson','reproducibility-package'];
const cases=[
 ['root-defaults',{'final_assignments.json':assignments}],
 ['root-manifest',{'final_assignments.json':assignments,'manifest.json':manifest,'analysis/summary.json':b('{"original":9007199254740993}\n')}],
 ['data-manifest',{'data/final_assignments.json':assignments,'manifest.json':manifest}],
 ['root-before-data',{'final_assignments.json':assignments,'data/final_assignments.json':other}],
 ['modern-rplan',{'sample.rplan':await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan')}],
 ['legacy-unrecorded-district-count',{'sample.rplan':b({...legacy,metadata:{...legacy.metadata,num_districts:0}})}],
 ['rplan-before-assignments',{'sample.rplan':b(legacy),'final_assignments.json':other,'data/final_assignments.json':b('bad')}],
 ['rplan-ignores-manifest',{'sample.rplan':b(legacy),'manifest.json':b('{bad')},'partial'],
 ['rplan-ignores-invalid-utf8',{'sample.rplan':b(legacy),'manifest.json':Buffer.from([255])},'partial'],
 ['ordered-partial',{'sample.rplan':b(legacy),'manifest.json':b('{bad')},'partial',['geojson','csv','geojson','reproducibility-package','gerrychain']],
 ['raw-analysis-retained',{'sample.rplan':b(legacy),'analysis/raw.json':b('this is raw source, not parsed JSON')}],
 ['geometry',{'sample.rplan':b({...legacy,geometry:{type:'FeatureCollection',features:[{type:'Feature',properties:{district_id:1},geometry:{type:'Polygon',coordinates:[[[0,0],[1,0],[1,1],[0,0]]]}}]}})}],
 ['native-unvalidated-assignments',{'final_assignments.json':b({'arbitrary-id':0})}],
 ['bad-preferred-rplan',{'sample.rplan':b('{bad'),'final_assignments.json':assignments},'error'],
 ['bad-root-no-fallback',{'final_assignments.json':b('bad'),'data/final_assignments.json':assignments},'error'],
 ['bad-manifest-with-assignments',{'final_assignments.json':assignments,'manifest.json':b('bad')},'error'],
 ['empty-preferred-rplan',{'sample.rplan':b(''),'final_assignments.json':assignments},'error'],
 ['missing-source',{'manifest.json':manifest},'error'],
];
let originalUnits=null;
if(process.argv.includes('--original-ri-block')){
 const original=JSON.parse(await fs.readFile('target/prepared-census-real/block-output.json','utf8'));
 const accepted=original.result.prepared.result.assignments;originalUnits=Object.keys(accepted).length;assert.equal(originalUnits,25649);
 cases.push(['original-ri-block-assignments',{'data/final_assignments.json':b(accepted)}]);
}
async function tree(dir,prefix=''){const out={};for(const item of await fs.readdir(dir,{withFileTypes:true})){const name=prefix+item.name;if(item.isDirectory())Object.assign(out,await tree(path.join(dir,item.name),name+'/'));else out[name]=await fs.readFile(path.join(dir,item.name));}return out;}
let cliMatches=0,nativeMatches=0,refusals=0;const evidence=[];
for(const [name,sources,expected='completed',requestedFormats=formats]of cases){
 const dir=path.join(root,name),plan=path.join(dir,'v1/2020/plans/sample'),out=path.join(dir,'out');await fs.mkdir(plan,{recursive:true});for(const [p,bytes]of Object.entries(sources)){await fs.mkdir(path.dirname(path.join(plan,p)),{recursive:true});await fs.writeFile(path.join(plan,p),bytes);}
 const input={label:'sample',formats:requestedFormats,files:Object.fromEntries(Object.entries(sources).map(([p,bytes])=>[p,bytes.toString('base64')]))},request=exportFolderRequest(input),requestPath=path.join(dir,'request.json');await fs.writeFile(requestPath,JSON.stringify(request));
 const cli=spawnSync(path.resolve('target/debug/bisect.exe'),['export','--label','sample','--output-base',dir,'--out',out,'--format',...requestedFormats],{encoding:'utf8'}),reference=spawnSync(path.resolve('target/debug/examples/export_source_reference.exe'),[plan,'sample'],{encoding:'utf8'});
 const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[requestPath],{encoding:'utf8',maxBuffer:32*1024*1024});
 if(expected==='error'){assert.notEqual(cli.status,0);assert.notEqual(reference.status,0);assert.notEqual(native.status,0);assert.throws(()=>engine.execute(request));refusals++;continue;}
 const result=engine.execute(request);assert.equal(native.status,0,native.stderr);assert.deepEqual(result,JSON.parse(native.stdout));nativeMatches++;
 assert.equal(reference.status,0,reference.stderr);assert.deepEqual(JSON.parse(result.document_json),JSON.parse(reference.stdout));assert.equal(result.status,expected);assert.equal(cli.status,expected==='partial'?1:0,cli.stderr);
 const cliFiles=await tree(out);assert.deepEqual(Object.keys(result.files).sort(),Object.keys(cliFiles).sort());for(const [p,text]of Object.entries(result.files)){assert.equal(text,cliFiles[p].toString('utf8'));assert.equal(result.file_sha256[p],sha(cliFiles[p]));}cliMatches++;
 for(const [p,bytes]of Object.entries(sources))assert.equal(result.source_sha256[p],sha(bytes));assert.equal(result.document_sha256,sha(result.document_json));
 await fs.writeFile(path.join(dir,'files.json'),JSON.stringify(result.files));await fs.writeFile(path.join(dir,'export.zip'),await exportFolderZip(result));await fs.writeFile(path.join(dir,'result.json'),JSON.stringify(result));
 const project=createProject({name:'Native folder export '+name,files:{exportFolderInputs:input},operation:'export-native-plan-folder',constraints:[],result,lastOperation:'export-native-plan-folder'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));evidence.push({case:name,source:result.selected_source,status:result.status,files:Object.keys(result.files)});
}
// Folder selection reads supported sources only, preserving opaque invalid bytes.
let reads=[];const file=(p,bytes)=>({name:p.split('/').at(-1),webkitRelativePath:'folder/'+p,size:bytes.length,arrayBuffer:async()=>{reads.push(p);return new Uint8Array(bytes).buffer;}});
const selected=await readExportFolder([file('final_assignments.json',assignments),file('manifest.json',Buffer.from([255])),file('unrelated.exe',b('never read'))],{label:'sample',formats});assert.deepEqual(reads,['final_assignments.json','manifest.json']);assert.equal(selected.files['manifest.json'],'/w==');
const prior=structuredClone(selected),bad=file('sample.rplan',b('x'));bad.size=3*1024*1024+1;reads=[];await assert.rejects(()=>readExportFolder([bad],selected));assert.deepEqual(reads,[]);const controller=new AbortController();controller.abort();await assert.rejects(()=>readExportFolder([file('final_assignments.json',assignments)],selected,{signal:controller.signal}));assert.deepEqual(selected,prior);
await assert.rejects(()=>readExportFolder([{...file('final_assignments.json',assignments),size:1}],selected));
const tampered=JSON.parse(await fs.readFile(path.join(root,'root-manifest/result.json'),'utf8'));tampered.files[Object.keys(tampered.files)[0]]+='changed';await assert.rejects(()=>exportFolderZip(tampered));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,actual_cli_file_sets_and_bytes:cliMatches,native_host_results:nativeMatches,preferred_source_refusals:refusals,original_ri_block_units:originalUnits,shared_cli_loader_document_matches:true,original_sources_hashed:true,source_only_reader_and_pre_read_bounds:true,partial_formats_retained:true,zip_checks_pending:true,scoped_native_semantics_no_plan_validation:true,cases:evidence},null,2));
console.log(`Native folder export: ${cliMatches} actual CLI file-set/byte/loader matches, ${nativeMatches} Native/WASM full results, ${refusals} matching preferred-source refusals, source/project/hash and folder read guards passed.`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {reproducibilityRequest,reproducibilityZip,readReproducibilityAnalysis} from '../../web/lab/reproducibility-files.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/reproducibility-wasm';await fs.mkdir(root,{recursive:true});let matched=0,rejected=0;
const document_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8');
const manifest=JSON.parse(await fs.readFile('target/plan-import-wasm/manifest.json','utf8'));manifest.seed=42;
const base={operation:'export-reproducibility-package',document_json,manifest_json:JSON.stringify(manifest,null,2)+'\n',analysis_files:{'population.json':'{ "supplied": true }\n'},label:'fixture'};
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;for(const [name,text] of Object.entries(out.files))assert.equal(out.file_sha256[name],createHash('sha256').update(text).digest('hex'));return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const output=await exact(base);assert.equal(output.files['manifest.json'],base.manifest_json);assert.equal(output.files['analysis/population.json'],base.analysis_files['population.json']);assert.equal(JSON.parse(output.files['audit.json']).audit.seed,42);assert.ok(output.files['verify.sh'].includes('--seed 42'));assert.equal(JSON.parse(output.files['fixture.rplan']).rplan_version,'0.1');
const absent=await exact({...base,manifest_json:null});assert.ok(!('manifest.json' in absent.files));assert.ok(absent.files['verify.sh'].startsWith('#'));assert.match(JSON.parse(absent.files['audit.json']).audit.note,/unavailable/);
const noSeed=await exact({...base,manifest_json:JSON.stringify({...manifest,seed:null})});assert.ok(noSeed.files['verify.sh'].startsWith('#'));assert.ok(!noSeed.files['verify.sh'].includes('bisect state'));
const dangerous="x'; $(touch bad)\n# embedded";
const quoted=await exact({...base,manifest_json:JSON.stringify({...manifest,label:dangerous})});
const quote=s=>"'"+s.replaceAll("'","'\"'\"'")+"'";
assert.ok(quoted.files['verify.sh'].includes('--label '+quote(dangerous)+' --seed 42'));
// Independently parse generated instructions as shell tokens without executing them.
await fs.writeFile(root+'/verify.sh',quoted.files['verify.sh']);
const tokens=JSON.parse(execFileSync('python',['-c','import shlex,json,sys;print(json.dumps(shlex.split(open(sys.argv[1],encoding="utf-8").read())))',root+'/verify.sh'],{encoding:'utf8'}));
assert.equal(tokens[tokens.indexOf('--label')+1],dangerous);assert.equal(tokens.filter(t=>t==='bisect').length,1);
await fs.writeFile(root+'/verification-command.sh',JSON.parse(quoted.files['audit.json']).audit.verification_command);
const commandTokens=JSON.parse(execFileSync('python',['-c','import shlex,json,sys;print(json.dumps(shlex.split(open(sys.argv[1],encoding="utf-8").read(),comments=True)))',root+'/verification-command.sh'],{encoding:'utf8'}));assert.deepEqual(commandTokens,tokens);
const legacy=await fs.readFile('target/plan-import-wasm/legacy.rplan','utf8');assert.equal((await exact({...base,document_json:legacy})).files['fixture.rplan'],legacy);
let real=0;for(const state of ['RI','IA','NC']){const source=await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8');const result=await exact({...base,document_json:source,manifest_json:null});const modern=JSON.parse(source),converted=JSON.parse(result.files['fixture.rplan']);for(let i=0;i<modern.plan.assignment.length;i++)assert.equal(converted.assignments[modern.plan.units.unit_ids[i]],modern.plan.assignment[i]+1);real++;}
for(const label of ['../x','x/a','x;command',''])await reject({...base,label});
for(const analysis_files of [{'../x.json':'{}'},{'.hidden.json':'{}'},{'x.txt':'{}'},{'x.json':'bad'},Object.fromEntries(Array.from({length:65},(_,i)=>[i+'.json','{}']))])await reject({...base,analysis_files});
await reject({...base,manifest_json:'bad'});await reject({...base,manifest_json:base.manifest_json+' '.repeat(3*1024*1024)});
for(const [name,text] of Object.entries(output.files)){await fs.mkdir(root+'/'+name.split('/').slice(0,-1).join('/'),{recursive:true});await fs.writeFile(root+'/'+name,text);}
await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output}));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={label:base.label,manifest:source('manifest.json',base.manifest_json),analysis:Object.fromEntries(Object.entries(base.analysis_files).map(([name,text])=>[name,source(name,text)]))};
const files={auditPlanText:source('source.rplan',base.document_json),reproducibilityInputs:inputs};
assert.deepEqual(reproducibilityRequest(files,inputs),base);
const project=createProject({name:'Reproducibility fixture',files,operation:base.operation,constraints:[],result:output,lastOperation:base.operation});assert.deepEqual(parseProject(JSON.stringify(project)).files,files);
await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
const archive=reproducibilityZip(output);await fs.writeFile(root+'/package.zip',archive);
const zipContents=JSON.parse(execFileSync('python',['-c','import zipfile,json,sys;z=zipfile.ZipFile(sys.argv[1]);assert z.testzip() is None;assert z.getinfo("verify.sh").external_attr>>16==0o100755;print(json.dumps({n:z.read(n).decode("utf-8") for n in z.namelist()}))',root+'/package.zip'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(zipContents,output.files);assert.deepEqual(reproducibilityZip(output),archive);
assert.throws(()=>reproducibilityZip({...output,files:{...output.files,'../escape.json':'{}'}}));
let oversizedRead=false;await assert.rejects(()=>readReproducibilityAnalysis([{name:'huge.json',size:3*1024*1024+1,arrayBuffer:async()=>{oversizedRead=true;}}]));assert.equal(oversizedRead,false);
await assert.rejects(()=>readReproducibilityAnalysis([{name:'duplicate.json',size:2},{name:'duplicate.json',size:2}]));
console.log(`Reproducibility packages: ${matched} native/WASM matches (${real} prepared states), ${rejected} rejections; exact source files, native declarations, missing manifest/seed, safe shell argument parsing and one-based legacy assignments.`);

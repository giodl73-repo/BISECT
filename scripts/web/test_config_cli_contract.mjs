import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {configValidationRequest} from '../../web/lab/config-files.js';
import {configRunRequest} from '../../web/lab/config-run.js';
import {tuiRunRequest,TUI_RUN_FORM_DEFAULTS} from '../../web/lab/tui-run-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),hash=b=>createHash('sha256').update(b).digest('hex');
await fs.mkdir('target/config-cli-contract',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/config-cli-contract/run-'));
const native=path.resolve(option('native')||'target/release/bisect.exe'),wasm=path.resolve(option('wasm')||process.env.BISECT_TEST_WASM||'target/cffi-legacy-gates/default.wasm');
const pin=async file=>({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256:hash(await fs.readFile(file))}),nativePin=await pin(native),wasmPin=await pin(wasm),engine=await instantiateEngine(await fs.readFile(wasm));
const run=args=>{const out=spawnSync(native,args,{encoding:'utf8',windowsHide:true});if(out.error)throw out.error;return {arguments:args,exit_code:out.status,stdout:out.stdout,stderr:out.stderr};};
const base={...CONFIG_TEMPLATE_DEFAULTS,name:'native_config',engine:null,seed:null,balance_tolerance:null},records=[],generated=[];
for(const [name,changes]of [['defaults',{}],['nway',{structure:'nway'}],['spectral',{structure:'spectral'}],['county-convergence',{weights:'county',alpha_county:2.5,search:'convergence',convergence_threshold:37}],['zero-workers',{workers:0,seeds:0,search:'multi',years:['2010']}],['literal-name',{name:'true'}]]){
 const settings={...base,...changes},dir=path.join(root,name);await fs.mkdir(dir);const file=path.join(dir,settings.name+'.yml');
 const args=['config','new','--name',settings.name,'--structure',settings.structure,'--weights',settings.weights,'--search',settings.search,'--workers',String(settings.workers),'--out',file,'--years',...settings.years];
 for(const [key,flag]of [['alpha_county','alpha-county'],['convergence_threshold','convergence-threshold'],['seeds','seeds'],['balance_tolerance','balance-tolerance']])if(settings[key]!==null)args.push('--'+flag+'='+settings[key]);
 const outcome=run(args);assert.equal(outcome.exit_code,0,outcome.stderr);const bytes=await fs.readFile(file),request=configTemplateRequest(settings),result=engine.execute(request);assert.equal(result.yaml_text,bytes.toString('utf8'),name+' native writer bytes');
 const collision=run(args);assert.equal(collision.exit_code,1);assert.deepEqual(await fs.readFile(file),bytes,'Collision preserves existing bytes');const forceArgs=args.slice();forceArgs[forceArgs.indexOf('--workers')+1]=String(settings.workers+1);const forced=run([...forceArgs,'--force']);assert.equal(forced.exit_code,0);assert.equal(await fs.readFile(file,'utf8'),engine.execute(configTemplateRequest({...settings,workers:settings.workers+1})).yaml_text);const forceRestore=run([...args,'--force']);assert.equal(forceRestore.exit_code,0);assert.deepEqual(await fs.readFile(file),bytes);
 const dryFile=path.join(dir,'dry-only.yml'),dryArgs=args.slice();dryArgs[dryArgs.indexOf('--out')+1]=dryFile;const dry=run([...dryArgs,'--dry-run']);assert.equal(dry.exit_code,0);assert.equal(dry.stdout,result.yaml_text+'\n','CLI println adds one newline after the native YAML');await assert.rejects(fs.stat(dryFile),e=>e.code==='ENOENT');
 const project=createProject({name,operation:request.operation,files:{configTemplate:settings},constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));records.push({name,kind:'create',native:outcome,collision,forced,forceRestore,dry,request,result});generated.push(bytes.toString('utf8'));
}
const nestedFile=path.join(root,'native-parent-case','nested','new.yml'),nestedArgs=records[0].native.arguments.slice();nestedArgs[nestedArgs.indexOf('--out')+1]=nestedFile;
const dryMissingParent=run([...nestedArgs,'--dry-run']);assert.equal(dryMissingParent.exit_code,0);await assert.rejects(fs.stat(path.join(root,'native-parent-case')),e=>e.code==='ENOENT');
const createMissingParent=run(nestedArgs);assert.equal(createMissingParent.exit_code,0);assert.equal(await fs.readFile(nestedFile,'utf8'),generated[0]);
const publication_reference={dry_run_missing_parent:dryMissingParent,create_missing_parent:createMissingParent,source_sha256:hash(Buffer.from(generated[0]))};
await fs.writeFile(path.join(root,'publication-reference.json'),JSON.stringify(publication_reference,null,2)+'\n');
const yaml=generated[0],validation=[['valid',yaml],['crlf',yaml.replaceAll('\n','\r\n')],['bom','\ufeff'+yaml],['bom-name','\ufeffname: probe\nalgorithm:\n  structure: nway\n  engine: metis-core\n'],['invalid','name: broken\nalgorithm: [\n'],['unknown-engine','name: wrong\nalgorithm:\n  engine: nope\n'],['exact-seed','name: exact\nalgorithm:\n  structure: nway\n  engine: metis-core\n  seed: 18446744073709551615\n']];
for(const [label,text]of validation){
 const name='validate-'+label,dir=path.join(root,name);await fs.mkdir(dir);const file=path.join(dir,'source.yml');await fs.writeFile(file,text);const nativeResult=run(['config','validate',file]),source={name:'source.yml',base64:Buffer.from(text).toString('base64')},request=configValidationRequest(source);let result,error;try{result=engine.execute(request);}catch(e){error=e.message;}
 const row={name,kind:'validate',native:nativeResult,request,result,error};records.push(row);await fs.writeFile(path.join(root,'progress.json'),JSON.stringify(records,null,2)+'\n');
 assert.equal(request.yaml_text,text,name+' original YAML must reach the native parser');
 if(nativeResult.exit_code===0){assert(!error,error);assert.equal(result.source_sha256,hash(Buffer.from(text)));assert.equal(nativeResult.stdout.replace(/^\[OK\] [^\n]*\n/,'[OK] source.yml\n'),result.native_summary+'\n');if(label==='exact-seed')assert.equal(result.seed,'18446744073709551615');}
 else{assert.equal(nativeResult.exit_code,1);assert(error,name+' native failure must be retained');assert(nativeResult.stderr.includes(error));}
 const project=createProject({name,operation:request.operation,files:{configSource:source,configSettings:{profile:'open'}},constraints:[],result:result??null,lastOperation:result?request.operation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));
}
const graph={schema_version:1,state:'RI',year:'2020',geoids:['44001000000','44001000001'],adjacency:[[1],[0]],population:[1,1],edges:[[0,1,1]],areas:[1,1],exterior_perimeters:[1,1],centroids:[[0,0],[1,0]]};
const graphSource={name:'graph.json',base64:Buffer.from(JSON.stringify(graph)).toString('base64')},runInputs={graph:graphSource},runSettings={chamber:'congressional',districts:2};
for(const row of records.filter(r=>r.name==='validate-bom'||r.name==='validate-bom-name')){
 const source={name:'source.yml',base64:Buffer.from(row.request.yaml_text).toString('base64')},configured=configRunRequest(source,{profile:'open'},runInputs,runSettings),tui=tuiRunRequest({form:{...TUI_RUN_FORM_DEFAULTS,location:'RI'},created_at:'2026-10-27T00:00:00Z'},source,{profile:'open'},runInputs,runSettings);
 assert.equal(configured.input.yaml_text,row.request.yaml_text);assert.equal(tui.input.run.yaml_text,row.request.yaml_text);
 if(row.error){assert.throws(()=>engine.execute(configured),/missing field `algorithm`/);assert.throws(()=>engine.execute(tui),/missing field `algorithm`/);}
}
assert.deepEqual(await pin(native),nativePin);assert.deepEqual(await pin(wasm),wasmPin);const sources=[];
for(const file of ['crates/bisect-cli/src/main.rs','crates/bisect-cli/src/args.rs','crates/bisect-cli/src/algo_config.rs','crates/bisect-runner/src/config.rs','crates/bisect-wasm/src/config.rs','web/lab/config-template.js','web/lab/config-files.js','web/lab/package-files.js','web/lab/config-run.js','web/lab/tui-run-files.js','scripts/web/test_config_cli_contract.mjs'])sources.push(await pin(file));
const evidence={status:'passed-actual-CLI-config-contract',sources,native:nativePin,wasm:wasmPin,records,publication_reference,scope:'Actual CLI config new and open config validate; exact generated files, dry-run/collision/force outcomes, raw YAML parser input/hash/summary, max u64 and original-source projects. No native automatic configs-directory discovery or browser filesystem publication; engine/seed generation extensions and court filter are separate.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root}));

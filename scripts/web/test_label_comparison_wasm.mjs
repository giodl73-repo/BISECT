import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelCompareRequest,validateLabelCompareInputs,readLabelInventoryRegistry} from '../../web/lab/label-inventory-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/label-comparison-wasm');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const entry={built:['2010','2020','2020'],analyzed:['2020'],reported:['unverified']};
const registry=JSON.stringify({proposal_a:entry,proposal_b:entry});
const request=(registry_json=registry,label_a='proposal_a',label_b='proposal_b',year='2020')=>({operation:'inspect-label-comparison',registry_json,label_a,label_b,year});
let cliCases=0,hostMatches=0,rejections=0;
async function check(r,expectError=false){
 const dir=await fs.mkdtemp(path.join(root,'native-'));if(r.registry_json!==null)await fs.writeFile(path.join(dir,'.bisect'),r.registry_json);
 let result,error;try{result=engine.execute(r);}catch(e){error=e.message;}
 assert.equal(typeof error==='string',expectError,'Expected native command outcome');
 for(const format of ['text','json']){
  const native=spawnSync(path.resolve('target/debug/examples/label_inventory_reference.exe'),['compare',format,r.label_a,r.label_b,r.year],{cwd:dir,encoding:'utf8'});cliCases++;
  if(error){assert.equal(native.status,1);assert.equal(JSON.parse(native.stderr.trim().slice('Error: '.length)),error);}
  else{assert.equal(native.status,0,native.stderr);assert.equal(native.stdout.replaceAll('\r\n','\n').trimEnd(),result['native_'+format]);}
 }
 if(result){
  const file=path.join(dir,'request.json');await fs.writeFile(file,JSON.stringify(r));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),result);hostMatches++;
  assert.equal(result.registry_sha256,r.registry_json===null?null:createHash('sha256').update(r.registry_json).digest('hex'));
  assert.equal(result.native_summary.status,'delegated_to_compare');assert.equal(Object.hasOwn(result,'measurements'),false);
  if(r.registry_json!==null)assert.equal(await fs.readFile(path.join(dir,'.bisect'),'utf8'),r.registry_json);
 }
 return result;
}
const result=await check(request());assert.deepEqual(result.native_summary,{label_a:'proposal_a',label_b:'proposal_b',year:'2020',analysis_a:'analysis/proposal_a/2020',analysis_b:'analysis/proposal_b/2020',status:'delegated_to_compare',next_step:'bisect compare --plan-a proposal_a --plan-b proposal_b --year 2020 --format json'});assert.match(result.native_text,/Both plans are built and analyzed for year 2020/);await check(request(registry,'proposal_a','proposal_a'));await check(request(registry+' \r\n'));await check(request(JSON.stringify({proposal_a:{...entry,reported:[]},proposal_b:entry})));
for(const r of [request(null),request(''),request('{}'),request(registry,'missing'),request(registry,'proposal_a','missing'),request(registry,undefined,undefined,'2010'),request(JSON.stringify({proposal_a:{...entry,analyzed:[]},proposal_b:entry})),request(JSON.stringify({proposal_a:entry,proposal_b:{...entry,analyzed:[]}})),request(JSON.stringify({proposal_a:{...entry,built:[]},proposal_b:{...entry,analyzed:[]}})),request(registry,'runs'),request(registry,'proposal_a','../bad'),request('{broken')])await check(r,true);
for(const r of [{...request(),extra:true},request(registry,undefined,undefined,'../2020'),request(registry+' '.repeat(1024*1024+1)),request(registry,'a'.repeat(201)),request(JSON.stringify(Object.fromEntries(Array.from({length:501},(_,i)=>['label'+i,entry])))),request(JSON.stringify({proposal_a:{...entry,built:Array(101).fill('2020')},proposal_b:entry}))]){assert.throws(()=>engine.execute(r));rejections++;}
assert.throws(()=>engine.execute(request(null)),{message:"[CONFIG] analyze: 'proposal_a' has not been built for year 2020.\nRun: bisect build proposal_a --year 2020\n  (label-compare requires both plans to be built for year 2020)"});
assert.throws(()=>engine.execute(request(JSON.stringify({proposal_a:entry,proposal_b:{...entry,analyzed:[]}}))),{message:"[CONFIG] report: 'proposal_b' has not been analyzed for year 2020.\nRun: bisect analyze proposal_b --year 2020\n  (label-compare requires both plans to be analyzed for year 2020)"});
const inputs={registry:{name:'.bisect',base64:Buffer.from(registry).toString('base64')},label_a:'proposal_a',label_b:'proposal_b',year:'2020'};assert.deepEqual(labelCompareRequest(inputs),request());
const project=createProject({name:'Label comparison readiness',operation:'inspect-label-comparison',constraints:[],files:{labelCompareInputs:inputs},result,lastOperation:'inspect-label-comparison'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,'project.bisect'),JSON.stringify(project));
for(const input of [{...inputs,extra:true},{...inputs,year:'bad'},{...inputs,label_a:'../bad'},{...inputs,registry:{name:'.bisect',base64:'AP8='}}])assert.throws(()=>validateLabelCompareInputs(input));
let read=false;await assert.rejects(()=>readLabelInventoryRegistry({name:'.bisect',size:1024*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({cliCases,hostMatches,rejections,registry_unmodified:true,instructions_only:true,scope:result.scope},null,2));
console.log(`Label comparison readiness: ${cliCases} actual CLI text/JSON/error comparisons, ${hostMatches} full native-host matches, ${rejections} boundary refusals; original registry and inert projects retained.`);

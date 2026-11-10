// Actual CLI orchestration, separate from the shared ToolRequest host adapter.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
await fs.mkdir('target/ceremony-native-contract',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/ceremony-native-contract/run-'));
const option=name=>process.argv.find(arg=>arg.startsWith('--'+name+'='))?.slice(name.length+3);
const nativeSource=option('native')??process.env.BISECT_CEREMONY_NATIVE??'target/release/bisect.exe';
const wasmSource=option('wasm')??process.env.BISECT_TEST_WASM??'target/wasm32-unknown-unknown/release/bisect_wasm.wasm';
const native=path.join(root,'native-reference.exe'),wasmPath=path.join(root,'engine.wasm');
await fs.copyFile(nativeSource,native);await fs.copyFile(wasmSource,wasmPath);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const engine=await instantiateEngine(await fs.readFile(wasmPath));
const run=args=>{const result=spawnSync(native,args,{encoding:'utf8',windowsHide:true});if(result.error)throw result.error;return {arguments:args,exit_code:result.status,stdout:result.stdout,stderr:result.stderr};};
const ordered=value=>Array.isArray(value)?value.map(ordered):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,ordered(value[key])])):value;
const unit_ids=Array.from({length:8},(_,i)=>'u'+i),instance={schema_version:'certified-recursive-bisection-split-instance-v1',model_id:'certified-standard-bisect-split-v1',node_path:'',parent_certificate_id:null,unit_universe_hash:'sha256:'+hash(JSON.stringify(ordered({unit_ids}))),unit_ids,populations:Array(8).fill(10),edges:unit_ids.slice(1).map((_,i)=>({left:i,right:i+1,weight:1})),k_parent:4,k_left:2,k_right:2,orientation_rule:'equal-seats-unit-zero-left'};
const tree=engine.execute({operation:'solve-certified-tree',instance_json:JSON.stringify(instance)}).tree_json;
await fs.writeFile(path.join(root,'tree.json'),tree);await fs.writeFile(path.join(root,'profile.txt'),'Synthetic native CLI parity fixture\n');
const generation=[];
const start=run(['ceremony','start','--tree',path.join(root,'tree.json'),'--rule-profile',path.join(root,'profile.txt'),'--genesis-unix-seconds','1000','--review-seconds','10','--minimum-witnesses','2','--witness','fixture-A','--witness','fixture-B','--out',path.join(root,'open.json')]);
assert.equal(start.exit_code,0,start.stderr);generation.push(start);
const open=await fs.readFile(path.join(root,'open.json'),'utf8');let previous=path.join(root,'open.json');
const config={rule_profile_hash:'sha256:'+hash(await fs.readFile(path.join(root,'profile.txt'))),root_instance_hash:JSON.parse(tree).nodes[0].certificate.instance_hash,district_count:4,genesis_unix_seconds:1000,minimum_review_seconds:10,minimum_witness_receipts:2,genesis_witness_receipts:['fixture-A','fixture-B']};
const browserStart=engine.execute({operation:'start-ceremony',tree_json:tree,config_json:JSON.stringify(config)});
assert.deepEqual(JSON.parse(browserStart.transcript_json),JSON.parse(open),'Actual CLI start transcript, compared as JSON values (native pretty output versus compact browser text)');
const browserGenerations=[{operation:'start-ceremony',result:browserStart}];
const depths=[...new Set(JSON.parse(tree).nodes.map(node=>node.instance.node_path.length))].sort((a,b)=>a-b);
for(let i=0;i<depths.length;i++){
 const next=path.join(root,'round-'+i+'.json'),outcome=run(['ceremony','announce','--tree',path.join(root,'tree.json'),'--transcript',previous,'--published-unix-seconds',String(1010+10*i),'--witness','round-A','--witness','round-B','--out',next]);
 assert.equal(outcome.exit_code,0,outcome.stderr);generation.push(outcome);previous=next;
 const previousText=i===0?open:await fs.readFile(path.join(root,'round-'+(i-1)+'.json'),'utf8');
 const browserRound=engine.execute({operation:'announce-ceremony',tree_json:tree,transcript_json:previousText,published_unix_seconds:String(1010+10*i),witness_receipts:['round-A','round-B']});
 assert.deepEqual(JSON.parse(browserRound.transcript_json),JSON.parse(await fs.readFile(next,'utf8')),'Actual CLI announcement transcript JSON values');browserGenerations.push({operation:'announce-ceremony',round:i,result:browserRound});
}
const completed=await fs.readFile(previous,'utf8');assert.equal(JSON.parse(completed).status,'completed');await fs.writeFile(path.join(root,'completed.json'),completed);
const halt=run(['ceremony','halt','--tree',path.join(root,'tree.json'),'--transcript',path.join(root,'open.json'),'--reason','Native parity fixture','--out',path.join(root,'halted.json')]);
assert.equal(halt.exit_code,0,halt.stderr);const halted=await fs.readFile(path.join(root,'halted.json'),'utf8');
const browserHalt=engine.execute({operation:'halt-ceremony',tree_json:tree,transcript_json:open,reason:'Native parity fixture'});assert.deepEqual(JSON.parse(browserHalt.transcript_json),JSON.parse(halted),'Actual CLI halt transcript JSON values');
const records=[];
for(const [status,transcript]of [['open',open],['halted',halted],['completed',completed]])for(const includeTree of [false,true]){
  const nativeResult=run(['ceremony','verify','--transcript',path.join(root,status+'.json'),...(includeTree?['--tree',path.join(root,'tree.json')]:[])]);
  const request={operation:'verify-ceremony',transcript_json:transcript,...(includeTree?{tree_json:tree}:{})};
  let browser;try{browser={ok:true,result:engine.execute(request)};}catch(error){browser={ok:false,error:error.message};}
  // Persist the observed mismatch before asserting, including raw native output.
  const record={status,include_tree:includeTree,native:nativeResult,browser};records.push(record);
  await fs.writeFile(path.join(root,'progress.json'),JSON.stringify({status:'running',records},null,2)+'\n');
  if(status==='completed'&&!includeTree){
    assert.equal(nativeResult.exit_code,1);assert.match(nativeResult.stderr,/completed ceremony verification requires --tree for final binding/);
    assert.equal(browser.ok,false,'Desktop-compatible verification must refuse completed transcripts without their complete tree');
    assert.equal(browser.error,'completed ceremony verification requires --tree for final binding');
  }else{
    assert.equal(nativeResult.exit_code,0,nativeResult.stderr);assert.equal(browser.ok,true,browser.error);
    assert.equal(browser.result.verification_scope,includeTree?'supplied-complete-tree':'released-prefix-only');
  }
}
// Preserve the useful browser adaptation under an explicit inspection operation.
const inspection=engine.execute({operation:'inspect-ceremony-prefix',transcript_json:completed});
assert.equal(inspection.verified,true);assert.equal(inspection.verification_scope,'released-prefix-only');
assert.equal(inspection.status,'completed');
const sources=await Promise.all([native,wasmPath,'scripts/web/test_ceremony_cli_contract.mjs','crates/bisect-cli/src/ceremony_cmd.rs','crates/bisect-wasm/src/ceremony.rs','crates/bisect-wasm/src/toolkit.rs'].map(async file=>({path:file,sha256:hash(await fs.readFile(file))})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-actual-CLI-ceremony-verification',sources,records,generation,halt,browser_generations:browserGenerations,browser_halt:browserHalt,comparison:{transcripts:'Exact parsed JSON values; raw native pretty JSON and browser compact text retained separately.',completed_tree_refusal:'Exact error message, native exit 1.'},explicit_completed_prefix_inspection:inspection,scope:'Synthetic k=4 tree; actual CLI generates open/halted/completed transcripts and supplies verification error/success boundaries. Prefix inspection is a separate browser adaptation. Not external publication or receipt authentication.'},null,2)+'\n');
console.log(JSON.stringify({status:'passed-actual-CLI-ceremony-verification',root,cases:records.length,explicit_completed_prefix_inspection:true}));

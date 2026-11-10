import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createProject,parseProject} from '../../web/lab/project.js';
import {ceremonyRequest} from '../../web/lab/package-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM??'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/ceremony-wasm';await fs.mkdir(root,{recursive:true});
const hash=v=>{const sort=x=>Array.isArray(x)?x.map(sort):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,sort(x[k])])):x;return 'sha256:'+createHash('sha256').update(JSON.stringify(sort(v))).digest('hex');};
async function native(request){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
let operations=0,rejections=0;
async function exact(request){const result=engine.execute(request);assert.deepEqual(result,await native(request));operations++;return result;}
async function reject(request){assert.throws(()=>engine.execute(request));await assert.rejects(()=>native(request));rejections++;}
function instance(k){const n=8,ids=Array.from({length:n},(_,i)=>'u'+i);return {schema_version:'certified-recursive-bisection-split-instance-v1',model_id:'certified-standard-bisect-split-v1',node_path:'',parent_certificate_id:null,unit_universe_hash:hash({unit_ids:ids}),unit_ids:ids,populations:Array(n).fill(10),edges:ids.slice(1).map((_,i)=>({left:i,right:i+1,weight:1})),k_parent:k,k_left:Math.floor(k/2),k_right:Math.ceil(k/2),orientation_rule:k%2?'seat-ordered-floor-left-ceil-right':'equal-seats-unit-zero-left'};}
function rehash(transcript){for(const round of transcript.rounds){const projection={...round};delete projection.round_id;round.round_id=hash(projection);}const projection={...transcript};delete projection.transcript_id;transcript.transcript_id=hash(projection);return JSON.stringify(transcript);}
for(const k of [4,5]){
 const solved=await exact({operation:'solve-certified-tree',instance_json:JSON.stringify(instance(k))}),tree=JSON.parse(solved.tree_json);
 const settings={rule_profile_hash:'sha256:'+'a'.repeat(64),root_instance_hash:tree.nodes[0].certificate.instance_hash,district_count:k,genesis_unix_seconds:1000,minimum_review_seconds:10,minimum_witness_receipts:2,genesis_witness_receipts:['synthetic-genesis-A','synthetic-genesis-B']};
 const startRequest={operation:'start-ceremony',tree_json:solved.tree_json,config_json:JSON.stringify(settings)};
 const initial=await exact(startRequest);let current=initial;
 assert.equal(initial.status,'open');assert.equal(initial.released_rounds,0);assert.equal(JSON.parse(initial.transcript_json).rounds.length,0);assert.ok(!initial.transcript_json.includes('canonical_assignment'));
 const haltRequest={operation:'halt-ceremony',tree_json:solved.tree_json,transcript_json:initial.transcript_json,reason:'Synthetic review stop'};
 const halted=await exact(haltRequest);assert.equal(halted.status,'halted');assert.equal(halted.released_rounds,0);await exact({operation:'verify-ceremony',transcript_json:halted.transcript_json});
 await reject({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:halted.transcript_json,published_unix_seconds:'1010',witness_receipts:['A','B']});
 await reject({...haltRequest,reason:''});
 await reject({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:initial.transcript_json,published_unix_seconds:'1009',witness_receipts:['A','B']});
 await reject({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:initial.transcript_json,published_unix_seconds:'1010',witness_receipts:['same','same']});
 const depths=[...new Set(tree.nodes.map(node=>node.instance.node_path.length))].sort((a,b)=>a-b);
 for(const [index,depth] of depths.entries()){
  const previous=current;
  current=await exact({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:current.transcript_json,published_unix_seconds:String(1010+index*10),witness_receipts:['synthetic-round-'+index+'-A','synthetic-round-'+index+'-B']});
  const transcript=JSON.parse(current.transcript_json),round=transcript.rounds[index];assert.equal(current.last_recorded_unix_seconds,String(1010+index*10));assert.equal(transcript.ceremony_id,hash(settings));const projection={...transcript};delete projection.transcript_id;assert.equal(transcript.transcript_id,hash(projection));
  assert.deepEqual(round.cuts.map(cut=>cut.node_path),tree.nodes.filter(node=>node.instance.node_path.length===depth).map(node=>node.instance.node_path));assert.equal(round.previous_round_id,index?JSON.parse(previous.transcript_json).rounds[index-1].round_id:null);
  assert.equal(current.released_rounds,index+1);assert.equal(current.status,index===depths.length-1?'completed':'open');
  if(current.status==='completed')await reject({operation:'verify-ceremony',transcript_json:current.transcript_json});
  const prefix=await exact({operation:current.status==='completed'?'inspect-ceremony-prefix':'verify-ceremony',transcript_json:current.transcript_json});assert.equal(prefix.verification_scope,'released-prefix-only');const full=await exact({operation:'verify-ceremony',transcript_json:current.transcript_json,tree_json:solved.tree_json});assert.equal(full.verification_scope,'supplied-complete-tree');
  const bad=structuredClone(transcript);bad.rounds[index].published_unix_seconds=1000;await reject({operation:'verify-ceremony',transcript_json:rehash(bad)});
 }
 await reject({...haltRequest,transcript_json:current.transcript_json});
 await reject({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:current.transcript_json,published_unix_seconds:'2000',witness_receipts:['A','B']});
 const invalid=JSON.parse(current.transcript_json);invalid.rounds[0].cuts[0].artifacts.proof.candidate_count++;await reject({operation:'verify-ceremony',transcript_json:rehash(invalid)});
 await reject({...startRequest,config_json:JSON.stringify({...settings,root_instance_hash:'sha256:'+'0'.repeat(64)})});
 await reject({...startRequest,config_json:JSON.stringify({...settings,district_count:100000000})});
 for(const time of ['+1010','01010','-0','9223372036854775808'])await reject({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:initial.transcript_json,published_unix_seconds:time,witness_receipts:['A','B']});
 const largeSettings=JSON.stringify(settings).replace('"genesis_unix_seconds":1000','"genesis_unix_seconds":9007199254740993');const large=await exact({...startRequest,config_json:largeSettings});assert.ok(large.transcript_json.includes('9007199254740993'));const advanced=await exact({operation:'announce-ceremony',tree_json:solved.tree_json,transcript_json:large.transcript_json,published_unix_seconds:'9007199254741003',witness_receipts:['A','B']});assert.equal(advanced.last_recorded_unix_seconds,'9007199254741003');await exact({operation:'verify-ceremony',transcript_json:advanced.transcript_json});
 if(k===4){const files={ceremonyTree:{name:'tree.json',base64:Buffer.from(solved.tree_json).toString('base64')},ceremonyConfig:{name:'config.json',base64:Buffer.from(JSON.stringify(settings)).toString('base64')},ceremonyTranscript:{name:'transcript.json',base64:Buffer.from(initial.transcript_json).toString('base64')},ceremonySettings:{published_unix_seconds:'1010',witness_receipts:['synthetic-round-0-A','synthetic-round-0-B'],reason:'',include_tree:false}};const request=ceremonyRequest('announce-ceremony',files,files.ceremonySettings);const stage=engine.execute(request);const project=createProject({name:'Synthetic staged ceremony',files,operation:'announce-ceremony',constraints:[],result:stage,lastOperation:'announce-ceremony'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(countResultTables(stage)[0].rows.find(row=>row[0]==='Released rounds')[1],'1');await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/browser-transcript.json',stage.transcript_json);await fs.writeFile(root+'/tree.json',solved.tree_json);await fs.writeFile(root+'/config.json',JSON.stringify(settings));await fs.writeFile(root+'/open-transcript.json',initial.transcript_json);await fs.writeFile(root+'/completed-transcript.json',current.transcript_json);}
}
await reject({operation:'verify-ceremony',transcript_json:'x'.repeat(3*1024*1024+1)});
console.log(`Ceremony WASM: ${operations} exact native/WASM operations, ${rejections} matching rejections; independent depth-prefix, hash-chain, status, review-time and lossless i64 timestamp checks passed.`);

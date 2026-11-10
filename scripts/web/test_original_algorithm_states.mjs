// Explicit algorithm campaign over retained original Census inputs. This does
// not alter or silently retry the default-seed ApportionRegions campaign.
import fs from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {readNativeCampaignRuntime} from './native-campaign-runtime.mjs';
const args=process.argv.slice(2);
if(args.some(a=>!a.startsWith('--states=')&&a!=='--resume'&&!a.startsWith('--campaign-name=')))throw new Error('Use --states=AL,CA and optional --resume.');
const campaignName=args.find(a=>a.startsWith('--campaign-name='))?.slice(16)||'campaign';assert.match(campaignName,/^[A-Za-z0-9_-]{1,80}$/);
const stateArg=args.find(a=>a.startsWith('--states='));
if(!stateArg)throw new Error('Select original states explicitly.');
const states=stateArg.slice(9).split(',');
assert(states.length&&states.length<=50&&new Set(states).size===states.length);
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
assert(states.every(s=>s!=='DC'&&manifest.states[s]));
const root='target/original-state-algorithm/standard-bisect';await fs.mkdir(root,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const pinned=await readNativeCampaignRuntime('target/original-all-state-wasm');
const identity={seed_oracle:sha(await fs.readFile(pinned.seed)),independent:sha(await fs.readFile('scripts/web/test_original_state_geometry_outputs.py')),wasm:sha(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),native:sha(await fs.readFile(process.env.BISECT_NATIVE_REFERENCE||'target/release/examples/execute_tool_request.exe')),harness:sha(await fs.readFile('scripts/web/test_original_alaska_workflow_wasm.mjs'))};
const run=(exe,argv)=>new Promise((resolve,reject)=>{const child=spawn(exe,argv,{stdio:'inherit',windowsHide:true});child.once('error',reject);child.once('exit',(code,signal)=>resolve({code,signal}));});
const campaign={structure:'standard-bisect',requested_seed:'42',requested:states,identity,completed:0,verified:0,failed:0,results:[],scope:'Explicit StandardBisect choice on complete original retained tract inputs; full native/WASM workflows and native CLI accepted-attempt assignment oracle. Separate from the unchanged default algorithm campaign. Does not prove all options or browser capacity.'};
const save=async()=>{const temporary=root+'/'+campaignName+'.json.tmp';await fs.writeFile(temporary,JSON.stringify(campaign,null,2));await fs.rename(temporary,root+'/'+campaignName+'.json');};
for(const state of states){
 const receipt=root+'/'+state+'/algorithm-receipt.json';let prior;
 if(args.includes('--resume')){try{prior=JSON.parse(await fs.readFile(receipt,'utf8'));}catch{}}
 let row;
 let proofMatches=false;try{proofMatches=prior?.evidence_sha256===sha(await fs.readFile(root+'/'+state+'/evidence.json'));}catch{}
 if(prior?.status==='verified'&&proofMatches&&JSON.stringify(prior.identity)===JSON.stringify(identity)){
  const checked=await run('python',['scripts/web/test_original_state_geometry_outputs.py','--states='+state,'--structure=standard-bisect']);
  if(checked.code===0)row=prior;
 }
 if(!row){
  console.log('Explicit StandardBisect: '+state);
  const workflow=await run(process.execPath,['scripts/web/test_original_alaska_workflow_wasm.mjs','--state='+state,'--structure=standard-bisect']);
  const independent=workflow.code===0?await run('python',['scripts/web/test_original_state_geometry_outputs.py','--states='+state,'--structure=standard-bisect']):null;
  row={state,identity,status:workflow.code===0&&independent?.code===0?'verified':'failed',workflow,independent};
  if(row.status==='verified')row.evidence_sha256=sha(await fs.readFile(root+'/'+state+'/evidence.json'));
  await fs.mkdir(root+'/'+state,{recursive:true});await fs.writeFile(receipt,JSON.stringify(row,null,2));
 }
 campaign.results.push(row);campaign.completed++;campaign[row.status==='verified'?'verified':'failed']++;await save();
 console.log(JSON.stringify({completed:campaign.completed,verified:campaign.verified,failed:campaign.failed,state,status:row.status}));
}
if(campaign.failed)process.exitCode=1;

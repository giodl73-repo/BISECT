await (await import('node:fs/promises')).mkdir('target/native-aggregate-effects',{recursive:true});
import {spawnSync} from 'node:child_process';


import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import {createHash} from 'node:crypto';


import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';import {instantiateEngine} from '../../web/lab/wasm-engine.js';import {reviewNativeAggregateEffects,reviewNativeAggregateEffectRecovery,applyNativeAggregateEffects,retireNativeAggregateEffectReview} from '../../web/lab/native-aggregate-effect-application.js';import {readNativeEffectJournal,listNativeEffectJournals,effectIdentity,effectHash} from '../../web/lab/native-aggregate-effect-journal.js';


const audit=path.resolve(process.argv[2]||process.env.BISECT_AGGREGATE_ORACLE),input=JSON.parse(await fs.readFile(path.join(audit,'evidence.json'),'utf8')),out=await fs.mkdtemp(path.resolve('target/native-aggregate-effects/effect-recovery-')),h=nativeOutputDirectoryHarness(out),wasm=await fs.readFile((process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),engine=await instantiateEngine(wasm),sha=b=>createHash('sha256').update(b).digest('hex'),cases=[];


async function snapshot(root){const tree={};async function walk(dir,prefix=''){for(const e of await fs.readdir(dir,{withFileTypes:true})){const relative=prefix?prefix+'/'+e.name:e.name,p=path.join(dir,e.name);if(e.isDirectory()){tree[relative]=null;await walk(p,relative);}else{const bytes=await fs.readFile(p);tree[relative]={sha256:sha(bytes),byte_length:bytes.length,base64:bytes.toString('base64')};}}}await walk(root.dir);return Object.fromEntries(Object.entries(tree).sort());}


function adapter(root){const at=name=>{const p=path.resolve(root.dir,name);assert(p===root.dir||p.startsWith(root.dir+path.sep));return p;};return {async mkdir(name){await fs.mkdir(at(name),{recursive:true});},async stat(name){try{const s=await fs.lstat(at(name));if(s.isSymbolicLink())throw Error('Prototype refuses symlink input.');return {kind:s.isDirectory()?'directory':'file'};}catch(e){if(['ENOENT','ENOTDIR'].includes(e.code))return null;throw e;}},async readdir(name){return (await fs.readdir(at(name),{withFileTypes:true})).map(e=>({name:e.name,kind:e.isDirectory()?'directory':'file'}));},async read(name,maxBytes){const file=await fs.stat(at(name));if(file.isFile()&&file.size>maxBytes)throw Object.assign(Error('Source exceeds remaining byte allowance'),{code:'ResourceLimit'});return new Uint8Array(await fs.readFile(at(name)));},async write(name,bytes){await fs.writeFile(at(name),bytes);},async rename(from,to){await fs.rename(at(from),at(to));}};}


const runtimeIdentity={sha256:sha(wasm),byte_length:wasm.length},settings={year:'2020',types:['summary'],format:'json',csv:true},flags=new WeakMap();


async function seed(root,before){for(const [name,value]of Object.entries(before)){const p=path.join(root.dir,name);if(value===null)await fs.mkdir(p,{recursive:true});else{await fs.mkdir(path.dirname(p),{recursive:true});await fs.writeFile(p,Buffer.from(value.base64,'base64'));}}}


function live(root,options={}){const io=adapter(root),original=io.write;io.checkAccess=async()=>!options.denied;io.supports=async capability=>capability==='same-parent-file-move'&&!options.noMove;io.write=async(p,bytes)=>{if(options.beforeWrite)await options.beforeWrite(p,bytes);await original(p,bytes);await options.afterWrite?.(p,bytes);};const rename=io.rename;io.rename=async(a,b)=>{await options.beforeRename?.(a,b);await rename(a,b);await options.afterRename?.(a,b);};flags.set(io,options);return io;}


function workingTree(tree,selection,expected){const journal=selection.namespace+'/'+selection.id;return Object.fromEntries(Object.entries(tree).filter(([p])=>p!==journal&&!p.startsWith(journal+'/')&&(p!==selection.namespace||Object.hasOwn(expected,p))));}


async function review(io,extra={}){return reviewNativeAggregateEffects(r=>engine.execute(r),io,{settings,runtimeIdentity,...extra});}


const reference=input.cases.find(r=>r.name==='reference'),referenceBefore=JSON.parse(await fs.readFile(path.join(path.dirname(reference.native_root),'before.json'),'utf8')),expectedMerged=JSON.parse(await fs.readFile(path.join(path.dirname(reference.native_root),'after.json'),'utf8'))['v1/2020/national/us_summary.json'],positives=[],negatives=[];


async function oracle(name,before,version='v1'){const base=await h.directory(name+'-native'),root=await base.getDirectoryHandle('outputs',{create:true});await seed(root,before);const args=['aggregate','--version',version,'--year','2020','--types','summary','--format','json','--csv'],result=spawnSync(path.join(audit,'native.exe'),args,{cwd:base.dir,maxBuffer:16*1024*1024});await fs.writeFile(base.dir+'.arguments.json',JSON.stringify(args));await fs.writeFile(base.dir+'.stdout.bin',result.stdout);await fs.writeFile(base.dir+'.stderr.bin',result.stderr);return {root,exit:result.status,after:await snapshot(root)};}


for(const mode of ['before-scratch-write','after-scratch-write','after-rename','empty-ack','created-empty-temp','partial-directories','baseline-empty-prepared','terminal-completion-empty','terminal-native-failure-empty','after-ack','before-intent','before-rename','after-csv-close','created-empty-csv']){


 const root=await h.directory(mode),version=mode==='baseline-empty-prepared'?'.bisect-aggregate-action-journals':'v1',fixture=['partial-directories','baseline-empty-prepared'].includes(mode)?{}:mode==='terminal-native-failure-empty'?JSON.parse(await fs.readFile(path.join(path.dirname(input.cases.find(r=>r.name==='merged-temp-is-directory').native_root),'before.json'),'utf8')):referenceBefore;await seed(root,fixture);const expected=await oracle(mode,fixture,version),options={},io=live(root,options),grant=await review(io,{version}),controller=new AbortController();let injected=false;


 options.beforeWrite=async(p,bytes)=>{let record=null;if(/\/\d{5}\.json$/.test(p))record=JSON.parse(new TextDecoder().decode(bytes));if(mode==='baseline-empty-prepared'&&p.endsWith('/00000.json')||mode==='empty-ack'&&p.endsWith('/00004.json')||mode==='terminal-completion-empty'&&record?.phase==='completed'||mode==='terminal-native-failure-empty'&&record?.phase==='native-failed'){await fs.writeFile(path.join(root.dir,p),Buffer.alloc(0));injected=true;throw Error('Synthetic empty checkpoint');}if(mode==='before-intent'&&record?.phase==='intent'&&record.cursor===0){injected=true;throw Error('Synthetic stop before first intent');}if(mode==='created-empty-csv'&&p.endsWith('/us_summary.csv')){await fs.writeFile(path.join(root.dir,p),Buffer.alloc(0));injected=true;throw Error('Synthetic empty direct CSV');}if(p.endsWith('/us_summary.tmp.json')&&['before-scratch-write','created-empty-temp'].includes(mode)){if(mode==='created-empty-temp')await fs.writeFile(path.join(root.dir,p),Buffer.alloc(0));injected=true;throw Error('Synthetic pending write');}};


 options.beforeRename=async()=>{if(mode==='before-rename'){injected=true;throw Error('Synthetic stop before rename');}};options.afterWrite=async p=>{if(mode==='after-csv-close'&&p.endsWith('/us_summary.csv')){injected=true;throw Error('Synthetic stop after direct CSV close');}if(mode==='after-scratch-write'&&p.endsWith('/us_summary.tmp.json')){injected=true;throw Error('Synthetic loss after temporary close');}};options.afterRename=async()=>{if(mode==='after-rename'){injected=true;throw Error('Synthetic loss after rename');}};


 if(mode==='partial-directories'){const mkdir=io.mkdir;io.mkdir=async p=>{if(p==='v1/2020/national'){await fs.mkdir(path.join(root.dir,'v1'),{recursive:true});injected=true;throw Error('Synthetic partial parent creation');}return mkdir(p);};}


 let fault;await assert.rejects(applyNativeAggregateEffects(grant,{confirmed:true,signal:controller.signal,onProgress:r=>{if(mode==='after-ack'&&r.completed_actions.length===2){injected=true;controller.abort();}}}).catch(e=>{fault=e.nativeEffectReceipt;throw e;}));assert(injected,mode);const afterFault=await snapshot(root),fresh=live(root,mode==='terminal-completion-empty'||mode==='baseline-empty-prepared'?{noMove:true}:{}),beforeReview=await snapshot(root),recovered=await reviewNativeAggregateEffectRecovery(r=>engine.execute(r),fresh,grant.journal,{runtimeIdentity});assert.deepEqual(await snapshot(root),beforeReview,mode+' recovery review wrote');await assert.rejects(applyNativeAggregateEffects(structuredClone(recovered),{confirmed:true}),/no live authority/);await assert.rejects(applyNativeAggregateEffects(recovered),/Confirm/);assert.deepEqual(await snapshot(root),beforeReview);


 const receipt=await applyNativeAggregateEffects(recovered,{confirmed:true}),after=await snapshot(root),history=await readNativeEffectJournal(fresh,grant.journal);assert.equal(receipt.status,expected.exit?'native-failed':'completed',mode);assert.deepEqual(workingTree(after,grant.journal,expected.after),expected.after,mode);const stem=grant.journal.namespace+'/'+grant.journal.id;for(const [name,value]of Object.entries(afterFault))if(name.startsWith(stem+'/'))assert.deepEqual(after[name],value,mode+' overwrote journal '+name);if(mode==='terminal-completion-empty'){assert.equal(receipt.effects_may_have_started,false);assert.equal(receipt.journal_effects_may_have_started,true);assert.equal(receipt.completed_actions.length,0);assert.deepEqual(recovered.required_capabilities,[]);}if(['after-scratch-write','after-rename','empty-ack','after-csv-close'].includes(mode))assert.equal(receipt.observed_actions_acknowledged.length,1);if(['created-empty-temp','created-empty-csv'].includes(mode))assert.equal(recovered.recovery.disposition,'created-empty-file');if(mode==='partial-directories')assert.equal(recovered.recovery.disposition,'partially-created-directories');await assert.rejects(applyNativeAggregateEffects(recovered,{confirmed:true}),/no live authority/);const finalBefore=await snapshot(root);let terminalCalls=0;await assert.rejects(reviewNativeAggregateEffectRecovery(r=>{terminalCalls++;return engine.execute(r);},fresh,grant.journal,{runtimeIdentity}),/already terminal/);assert.equal(terminalCalls,0);assert.deepEqual(await snapshot(root),finalBefore);


 await fs.writeFile(root.dir+'.recovery.json',JSON.stringify({mode,version,fault,after_fault:afterFault,review:recovered,receipt,after,history,native_root:expected.root.dir,native_exit:expected.exit},null,2));positives.push({mode,root:root.dir,disposition:recovered.recovery.disposition,status:receipt.status,native_root:expected.root.dir,native_exit:expected.exit,old_journal_unchanged:true});


}


async function rehashJournal(root,selection,mutate){const folder=path.join(root.dir,selection.namespace,selection.id),baselinePath=path.join(folder,'baseline.json'),baseline=JSON.parse(await fs.readFile(baselinePath,'utf8'));mutate(baseline.plan);const clean={...baseline.plan};delete clean.plan_sha256;baseline.plan.plan_sha256=await effectIdentity(clean);let bytes=new TextEncoder().encode(JSON.stringify(baseline));await fs.writeFile(baselinePath,bytes);let previous=await effectHash(bytes);for(const name of (await fs.readdir(folder)).filter(n=>/^\d{5}\.json$/.test(n)).sort()){const raw=await fs.readFile(path.join(folder,name));if(!raw.length)continue;const record=JSON.parse(raw);record.previous_sha256=previous;bytes=new TextEncoder().encode(JSON.stringify(record));await fs.writeFile(path.join(folder,name),bytes);previous=await effectHash(bytes);}}


for(const mode of ['ambiguous-mkdir','ambiguous-write','unknown-partial','changed-source','changed-roster','changed-chain','wrong-runtime','forged-action-self-hashed','forged-blob-self-hashed','move-unavailable','stale-context','unsafe-selection']){


 const root=await h.directory(mode),fixture=structuredClone(referenceBefore);if(mode==='ambiguous-mkdir')fixture['v1/2020/national']=null;if(mode==='ambiguous-write')fixture['v1/2020/national/us_summary.tmp.json']=expectedMerged;await seed(root,fixture);const options={},io=live(root,options),grant=await review(io);const mkdir=io.mkdir;if(mode==='ambiguous-mkdir')io.mkdir=async p=>{await mkdir(p);if(p==='v1/2020/national')throw Error('Synthetic unacknowledged idempotent mkdir');};options.beforeWrite=async p=>{if(mode==='unknown-partial'&&p.endsWith('/us_summary.tmp.json')){await fs.writeFile(path.join(root.dir,p),'UNKNOWN PARTIAL');throw Error('Synthetic unknown bytes');}};options.afterWrite=async p=>{if(mode!=='ambiguous-mkdir'&&p.endsWith('/us_summary.tmp.json'))throw Error('Synthetic closed temporary intent');};await assert.rejects(applyNativeAggregateEffects(grant,{confirmed:true}));


 if(mode==='changed-source')await fs.appendFile(path.join(root.dir,'v1/2020/states/alpha/analysis/summary.json'),' ');if(mode==='changed-roster')await fs.mkdir(path.join(root.dir,'v1/2020/states/new'));if(mode==='changed-chain')await fs.writeFile(path.join(root.dir,grant.journal.namespace,grant.journal.id,'00001.json'),'MALFORMED');if(mode==='forged-action-self-hashed')await rehashJournal(root,grant.journal,plan=>{plan.actions.find(a=>a.kind==='write').path='v1/2020/national/UNEXPECTED.txt';});if(mode==='forged-blob-self-hashed')await rehashJournal(root,grant.journal,plan=>{const key=Object.keys(plan.blobs)[0];plan.blobs[key].base64=Buffer.from('ALTERED').toString('base64');plan.blobs[key].byte_length=7;});


 const fresh=live(root,mode==='move-unavailable'?{noMove:true}:{});if(mode.startsWith('forged-'))await readNativeEffectJournal(fresh,grant.journal);const before=await snapshot(root);let calls=0,error;try{await reviewNativeAggregateEffectRecovery(r=>{calls++;return engine.execute(r);},fresh,mode==='unsafe-selection'?{namespace:'../escape',id:grant.journal.id}:grant.journal,{runtimeIdentity:mode==='wrong-runtime'?{...runtimeIdentity,sha256:'0'.repeat(64)}:runtimeIdentity,...(mode==='stale-context'?{assertCurrent:()=>{throw Error('Synthetic changed context');}}:{})});}catch(e){error=e.message;}assert(error,mode);if(mode.startsWith('ambiguous'))assert.match(error,/ambiguous/);if(mode.startsWith('forged'))assert.match(error,/recomputation/);if(['wrong-runtime','unsafe-selection','stale-context','changed-chain'].includes(mode))assert.equal(calls,0);assert.deepEqual(await snapshot(root),before,mode+' refusal wrote');await fs.writeFile(root.dir+'.refusal.json',JSON.stringify({mode,error,kernel_calls:calls,before,after:await snapshot(root)},null,2));negatives.push({mode,root:root.dir,refused:true,no_effects:true,kernel_calls:calls});


}


const postReviewGuards=[];

for(const mode of ['source-after-review','journal-after-review','context-after-review','move-after-review']){const root=await h.directory(mode);await seed(root,referenceBefore);const options={},io=live(root,options),original=await review(io);options.afterWrite=async p=>{if(p.endsWith('/us_summary.tmp.json'))throw Error('Synthetic pending temporary close');};await assert.rejects(applyNativeAggregateEffects(original,{confirmed:true}));const freshOptions={},fresh=live(root,freshOptions);let revision=0;const recovery=await reviewNativeAggregateEffectRecovery(r=>engine.execute(r),fresh,original.journal,{runtimeIdentity,assertCurrent:()=>{if(revision)throw Error('Synthetic context changed after recovery review');}});if(mode==='source-after-review')await fs.appendFile(path.join(root.dir,'v1/2020/states/alpha/analysis/summary.json'),' ');if(mode==='journal-after-review')await fs.writeFile(path.join(root.dir,original.journal.namespace,original.journal.id,'00004.json'),Buffer.alloc(0));if(mode==='context-after-review')revision++;if(mode==='move-after-review')freshOptions.noMove=true;const before=await snapshot(root);let receipt;await assert.rejects(applyNativeAggregateEffects(recovery,{confirmed:true}).catch(e=>{receipt=e.nativeEffectReceipt;throw e;}));assert.equal(receipt.effects_may_have_started,false);assert.equal(receipt.journal_effects_may_have_started,false);assert.deepEqual(await snapshot(root),before);await fs.writeFile(root.dir+'.post-review-guard.json',JSON.stringify({mode,receipt,before,after:await snapshot(root)},null,2));postReviewGuards.push({mode,root:root.dir,no_additional_effects:true});}

await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-private-native-action-recovery',native_audit:audit,wasm_sha256:sha(wasm),positive_cases:positives,refusal_cases:negatives,post_review_guards:postReviewGuards,scope:'Fresh private IO review and confirmed append-only recovery using actual WASM recomputation, fourteen fresh native CLI counterpart trees, twelve no-effect refusals including self-rehashed forged history, ambiguity, source/roster/runtime/capability changes. Original journal bytes retained. No public UI, selected OS folder or full host/capacity acceptance.'},null,2));console.log('PASS 14 native recoveries, 12 refusals and 4 post-review guards: '+out);



console.log(JSON.stringify({native_aggregate_evidence_root:out}));

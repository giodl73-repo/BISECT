import {reviewLabelBuildApplication,executeLabelBuildApplication,retireLabelBuildApplicationReview,readBuildRegistry} from './label-build-application.js';
import {nativeToolSession} from './native-state-output-ui.js';
import {encodeBytes} from './package-files.js';

export function installLabelBuildApplicationControls({document,getContext,setBaseline,setResult,setRegistry,setBusy,onError,createSession=nativeToolSession}){
 const get=id=>document.getElementById('label-build-live-'+id);
 let selected=null,working=false,controller=null,tool=null,receipt=null,context=null;
 const available=c=>c.ready&&c.operation==='run-label-build'&&c.inputs?.config;
 function current(){const c=getContext();if(!context||c.revision!==context.revision||c.inputs!==context.inputs||c.result!==context.result||c.operation!==context.operation||c.cMetis!==context.cMetis)throw Error('Build sources changed. Select and review the folder again.');}
 function retire(){if(selected)retireLabelBuildApplicationReview(selected.review);selected=null;tool?.close();tool=null;controller=null;get('confirm').checked=false;}
 function download(value,name){const url=URL.createObjectURL(new Blob([JSON.stringify(value,null,2)+'\n'],{type:'application/json'})),a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
 async function select(continuation=false,registryOnly=false){
  const c=getContext();try{
   if(c.busy||working||!available(c))throw Error('Choose build YAML and finish the current operation first.');
   if(continuation&&!c.baseline)throw Error('Open the original build sources and passive baseline first.');
   if(typeof globalThis.showDirectoryPicker!=='function')throw Error('Folder access is unavailable. Use the portable build and archive downloads.');
   const picker=globalThis.showDirectoryPicker({mode:registryOnly||get('dry-run').checked?'read':'readwrite'});retire();context=c;working=true;controller=new AbortController();setBusy(true);update();const root=await picker;controller.signal.throwIfAborted();current();
   if(registryOnly){const bytes=await readBuildRegistry(root,{signal:controller.signal,assertCurrent:current});current();setRegistry(bytes===null?null:{name:'.bisect',base64:encodeBytes(bytes)});context=getContext();get('summary').textContent='Read the current registry'+(bytes===null?' absence':' bytes')+'. No files changed. Review the folder to build.';return;}
   tool=createSession(c.artifact,controller.signal);const engine={execute:(request,options={})=>tool.execute(request,{...options,timeoutMs:request.operation==='run-label-build'?null:180000,cMetis:c.cMetis})};
   const value=await reviewLabelBuildApplication(engine,c.inputs,root,{dryRun:get('dry-run').checked,continuation,baseline:continuation?c.baseline:null,result:continuation?c.result:null,signal:controller.signal,assertCurrent:current});current();setBaseline(value.baseline);context=getContext();selected=value;
   const r=value.review;get('summary').textContent=[r.mode+' review · '+r.root_name+' · '+r.label,...r.years.map(y=>y.year+' · '+y.status+' · '+y.jobs.length+' jobs'),...r.removals.flatMap(x=>['REMOVE '+x.path+' ('+x.files.length+' files)',...x.files.map(f=>'  '+f.path+' · SHA-256 '+f.sha256)]),...r.potential_outputs.map(x=>'OUTPUT '+x.path+(x.before?' · replaces SHA-256 '+x.before.sha256:' · absent')),'Verified retained states: '+r.verified_states,'Config SHA-256 '+r.config_sha256,'Preparation precedes computation. Completed years may contain failed states. No atomic transaction or rollback.'].join('\n');
  }catch(e){retire();if(e.name!=='AbortError')onError(e.message);}finally{working=false;if(!selected){tool?.close();tool=null;controller=null;}setBusy(false);update();}
 }
 get('registry').onclick=()=>select(false,true);get('review').onclick=()=>select();get('continue').onclick=()=>select(true);
 get('dry-run').onchange=()=>{if(working)return;retire();get('summary').textContent='Mode changed. Review the selected folder again.';update();};
 get('confirm').onchange=update;
 get('cancel').onclick=()=>controller?.abort(Error('Build cancelled. Completed accepted outputs remain available; review continuation explicitly.'));
 get('apply').onclick=async()=>{
  try{const c=getContext();if(c.busy||working||!selected)throw Error('Review the selected build folder first.');current();const value=selected;if(value.review.mode!=='dry-run'&&!get('confirm').checked)throw Error('Confirm the listed preparation and output paths.');selected=null;get('confirm').checked=false;working=true;setBusy(true);update();
   const done=await executeLabelBuildApplication(value.review,{confirmed:true,signal:controller.signal,assertCurrent:current,onProgress:p=>{receipt=p.receipt;if(p.result){current();setResult(p.result);context=getContext();}get('summary').textContent='Build '+p.receipt.label+' · '+p.receipt.status+'\nVerified transitions: '+p.receipt.cursor+'\nAccepted states: '+(p.result?.states.length??p.receipt.partitioned_jobs.length);}});
   receipt=done.receipt;if(done.result){setResult(done.result);context=getContext();}get('summary').textContent=done.receipt.status==='dry-run'?'Dry-run complete. No directories, writes or partition execution.':'Build application completed and read back. '+done.result.states.filter(s=>s.status==='ok').length+' states succeeded; '+done.result.states.filter(s=>s.status==='failed').length+' failed. A built year records completion, not algorithm success.';
  }catch(e){if(e.buildReceipt)receipt=e.buildReceipt;onError(e.message);}finally{working=false;retire();setBusy(false);update();}
 };
 get('receipt').onclick=()=>{if(receipt&&!working)download(receipt,'label-build-application.json');};
 get('clear-baseline').onclick=()=>{if(working||getContext().busy)return;retire();setBaseline(null);get('summary').textContent='Passive build baseline cleared. No disk changes.';update();};
 function update(){const c=getContext();if(selected){try{current();}catch{retire();get('summary').textContent='Build review retired after project changes. Select and review the folder again.';}}
  const idle=!c.busy&&!working,api=typeof globalThis.showDirectoryPicker==='function';for(const id of ['registry','review'])get(id).disabled=!idle||!available(c)||!api;
  get('continue').disabled=!idle||!available(c)||!api||!c.baseline||get('dry-run').checked;get('dry-run').disabled=!idle;get('confirm').disabled=!idle||!selected||selected.review.mode==='dry-run';get('apply').disabled=!idle||!selected||selected.review.mode!=='dry-run'&&!get('confirm').checked;get('cancel').disabled=!working;get('receipt').disabled=working||!receipt;get('clear-baseline').disabled=!idle||!c.baseline;get('baseline').textContent=c.baseline?'Passive build baseline retained. Open restores no folder access or execution authority.':'No build continuation baseline retained.';
 }
 return {update};
}

import {reviewCandidateWorkspace,applyCandidateWorkspace,retireCandidateWorkspaceReview} from './candidate-workspace.js';
import {nativeToolSession} from './native-state-output-ui.js';

export function installCandidateWorkspaceControls({document,getContext,setBusy,onError}){
 const control=(tag,id,value='')=>{const e=document.createElement(tag);e.id='candidate-workspace-'+id;if(value)e.textContent=value;return e;};
 const panel=control('fieldset','controls'),version=control('input','version'),replace=control('input','replace'),confirm=control('input','confirm');
 const summary=control('pre','summary'),reviewBtn=control('button','review','Choose output folder and review'),applyBtn=control('button','apply','Write reviewed package');
 const cancel=control('button','cancel','Cancel workspace operation'),receiptBtn=control('button','receipt','Download write receipt');
 const label=(message,input)=>{const el=document.createElement('label');el.append(message,input);return el;};
 version.value='v1';version.maxLength=100;replace.type=confirm.type='checkbox';summary.setAttribute('role','status');
 for(const b of [reviewBtn,applyBtn,cancel,receiptBtn])b.type='button';
 panel.append(control('legend','heading','Write to an output folder'),label('Version ',version),label(' Allow replacement of the listed package files ',replace),
  reviewBtn,summary,label(' Confirm the listed writes ',confirm),applyBtn,cancel,receiptBtn);
 document.getElementById('download-candidate').after(panel);
 let grant=null,job=null,outcome=null,outcomeContext=null;
 const valid=c=>c.operation==='ingest-candidate-race'&&c.fresh&&c.result?.schema_version==='bisect-candidate-ingest-v1'&&c.inputs&&c.artifact;
 const matches=(a,b)=>valid(b)&&a.revision===b.revision&&a.result===b.result&&a.inputs===b.inputs&&a.artifact.sha256===b.artifact.sha256;
 const retire=()=>{retireCandidateWorkspaceReview(grant?.review);grant=null;confirm.checked=false;};
 function update(){
  const c=getContext();panel.hidden=c.operation!=='ingest-candidate-race';
  if(grant&&!matches(grant.context,c)){retire();summary.textContent='Review retired after the project or result changed.';}
  if(outcomeContext&&!matches(outcomeContext,c)){outcome=null;outcomeContext=null;}
  const idle=!job&&!c.busy,available=idle&&valid(c);
  reviewBtn.disabled=!available||typeof globalThis.showDirectoryPicker!=='function'||!globalThis.navigator?.locks?.request;
  version.disabled=replace.disabled=!idle;confirm.disabled=!available||!grant;applyBtn.disabled=!available||!grant||!confirm.checked;
  cancel.disabled=!job;receiptBtn.disabled=!!job||!outcome;
 }
 function begin(){job=new AbortController();setBusy(true);update();return job.signal;}
 function end(){job=null;setBusy(false);update();}
 function assertCurrent(c){if(!matches(c,getContext()))throw new Error('Candidate package changed. Run and review again.');}
 version.onchange=replace.onchange=()=>{retire();summary.textContent='Review the output folder with these settings.';update();};
 confirm.onchange=update;
 cancel.onclick=()=>job?.abort(new Error('Workspace operation cancelled. Started writes may remain.'));
 reviewBtn.onclick=async()=>{
  const c=getContext();if(job||c.busy||!valid(c))return;
  retire();outcome=null;outcomeContext=null;const signal=begin();let tool;
  try{
   const folder=await globalThis.showDirectoryPicker({mode:'readwrite'});signal.throwIfAborted();assertCurrent(c);
   tool=nativeToolSession(c.artifact,signal);
   const review=await reviewCandidateWorkspace(tool,c.result,c.inputs,folder,{version:version.value,replace:replace.checked,signal,assertCurrent:()=>assertCurrent(c)});
   signal.throwIfAborted();assertCurrent(c);grant={review,context:c};
   summary.textContent=[review.root_name+' / '+review.path,...review.actions.map(r=>r.action+' '+r.name+' ('+r.byte_length+' bytes)'),
    'Unrelated files are preserved. Each file is checked after writing. An interruption can leave a partial package.'].join('\n');
  }catch(e){retire();if(e.name!=='AbortError'){summary.textContent=e.message;onError(e.message);}}
  finally{tool?.close();end();}
 };
 applyBtn.onclick=async()=>{
  const c=getContext();if(job||c.busy||!grant||!confirm.checked)return;
  const g=grant;grant=null;confirm.checked=false;outcomeContext=g.context;const signal=begin();
  try{
   assertCurrent(g.context);
   outcome=await applyCandidateWorkspace(g.review,{confirmed:true,signal,assertCurrent:()=>assertCurrent(g.context),onProgress:r=>{outcome=r;summary.textContent='Writing and checking '+r.path;}});
   summary.textContent='Wrote and checked '+outcome.entries.length+' files in '+outcome.path+'.';
  }catch(e){retireCandidateWorkspaceReview(g.review);outcome=e.candidateWorkspaceReceipt??outcome;summary.textContent=e.message;onError(e.message);}
  finally{end();}
 };
 receiptBtn.onclick=()=>{
  if(!outcome||job)return;const url=URL.createObjectURL(new Blob([JSON.stringify(outcome,null,2)+'\n'],{type:'application/json'})),a=document.createElement('a');
  a.href=url;a.download='candidate-workspace-write.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
 };
 return {update};
}

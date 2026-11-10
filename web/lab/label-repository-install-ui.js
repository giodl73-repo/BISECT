import {reviewLabelRepositoryInstallation,applyLabelRepositoryInstallation,retireLabelRepositoryReview} from './label-repository-install.js';
import {nativeToolSession} from './native-state-output-ui.js';
export function installLabelRepositoryControls({document,getContext,setBusy,onError}){
 const get=id=>document.getElementById('label-repository-'+id);let selected=null,receipt=null,working=false,controller=null,tool=null;
 const valid=c=>c.fresh&&c.operation==='reconcile-label-package'&&c.result?.schema_version==='bisect-label-package-merge-v1';
 function current(c){const next=getContext();if(next.revision!==c.revision||next.result!==c.result||next.operation!==c.operation||!valid(next))throw new Error('Repository inputs changed. Prepare and review the package again.');}
 function retire(){retireLabelRepositoryReview(selected?.review);selected=null;get('confirm').checked=false;}
 async function review(continuation){let ownController;try{
  const c=getContext();if(c.busy||working||!valid(c))throw new Error('Run fresh repository package preparation first.');
  if(typeof globalThis.showDirectoryPicker!=='function')throw new Error('Directory writing is unavailable. Download the prepared repository ZIP.');
  const picker=globalThis.showDirectoryPicker({mode:'readwrite'});retire();working=true;ownController=controller=new AbortController();setBusy(true);update();
  const root=await picker;ownController.signal.throwIfAborted();current(c);tool=nativeToolSession(c.artifact,ownController.signal);
  const reviewed=await reviewLabelRepositoryInstallation(tool,c.result,c.inputs,root,{continuation,signal:ownController.signal,assertCurrent:()=>current(c)});current(c);selected={review:reviewed,context:c};
  get('summary').textContent=[(continuation?'Continuation':'Fresh installation')+' · '+reviewed.root_name+' · '+reviewed.label+' / '+reviewed.year+' / '+reviewed.stage,'Write '+reviewed.pending_paths.length+' paths; retain '+reviewed.retained_paths.length+' verified paths. Stage index precedes registry; registry is last.','Already written files may remain after interruption. Unlisted files are preserved.','Write paths:',...reviewed.pending_paths,'Retained paths:',...reviewed.retained_paths].join('\n');
 }catch(error){retire();if(error.name!=='AbortError')onError(error.message);}finally{tool?.close();tool=null;working=false;controller=null;setBusy(false);update();}}
 get('review').onclick=()=>review(false);get('continue').onclick=()=>review(true);
 get('confirm').onchange=update;
 get('cancel').onclick=()=>controller?.abort(new Error('Repository operation cancelled. Effects already started may remain; review explicit continuation.'));
 get('apply').onclick=async()=>{try{
  const c=getContext();if(c.busy||working||!selected||!get('confirm').checked)throw new Error('Review and confirm the listed repository changes first.');current(selected.context);const reviewed=selected.review;selected=null;get('confirm').checked=false;
  working=true;controller=new AbortController();setBusy(true);update();
  receipt=await applyLabelRepositoryInstallation(reviewed,{confirmed:true,signal:controller.signal,assertCurrent:()=>current(c),onProgress:value=>{receipt=value;get('summary').textContent='Repository '+value.root_name+' · '+value.label+' / '+value.year+'\n'+value.completed_paths.length+' files written and verified; '+value.retained_paths.length+' retained.'+(value.attempted_path?'\nWriting '+value.attempted_path:'')+(value.status==='cancelled'||value.status==='interrupted'?'\n'+value.status+': review explicit continuation before writing again.':'');}});
  get('summary').textContent='Repository installation completed and read back. Registry verified last.\n'+receipt.completed_paths.length+' files written; '+receipt.retained_paths.length+' retained. Download the receipt to document this operation.';
 }catch(error){if(error.installationReceipt)receipt=error.installationReceipt;retire();onError(error.message);}finally{working=false;controller=null;setBusy(false);update();}};
 get('receipt').onclick=()=>{if(!receipt||working)return;const url=URL.createObjectURL(new Blob([JSON.stringify(receipt,null,2)+'\n'],{type:'application/json'})),a=document.createElement('a');a.href=url;a.download='label-repository-installation.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
 function update(){const c=getContext();if(selected&&(selected.context.revision!==c.revision||selected.context.result!==c.result||!valid(c))){retire();get('summary').textContent='Review retired after inputs changed. Prepare and review again. Historical receipts grant no write authority.';}
  const available=valid(c)&&!c.busy&&!working;for(const id of ['review','continue'])get(id).disabled=!available||typeof globalThis.showDirectoryPicker!=='function';get('confirm').disabled=!available||!selected;get('apply').disabled=!available||!selected||!get('confirm').checked;get('cancel').disabled=!working;get('receipt').disabled=!receipt||working;
 }
 return {update};
}

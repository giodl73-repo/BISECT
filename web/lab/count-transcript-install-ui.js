import {reviewCountTranscriptInstallation,applyCountTranscriptInstallation,retireCountTranscriptReview} from './count-transcript-install.js';
import {nativeToolSession} from './native-state-output-ui.js';
export function installCountTranscriptControls({document,getContext,setBusy,onError}){
 const get=id=>document.getElementById('count-transcript-'+id);let selected=null,receipt=null,working=false,controller=null,tool=null;
 const valid=c=>c.fresh&&c.operation==='verify-count-files'&&c.result?.transcript_version==='rcount-audit-transcript-v1';
 function current(c){const next=getContext();if(!valid(next)||next.revision!==c.revision||next.result!==c.result||next.packageFiles!==c.packageFiles)throw new Error('Count package changed. Run verification and review again.');}
 function retire(){retireCountTranscriptReview(selected?.review);selected=null;get('confirm').checked=false;}
 get('review').onclick=async()=>{try{
  const c=getContext();if(c.busy||working||!valid(c))throw new Error('Run count package verification first.');if(typeof globalThis.showDirectoryPicker!=='function')throw new Error('Directory writing is unavailable. Download the verification result instead.');
  const picker=globalThis.showDirectoryPicker({mode:'readwrite'});retire();working=true;controller=new AbortController();setBusy(true);update();
  const root=await picker;controller.signal.throwIfAborted();current(c);tool=nativeToolSession(c.artifact,controller.signal);
  const review=await reviewCountTranscriptInstallation(tool,c.result,c.packageFiles,root,{signal:controller.signal,assertCurrent:()=>current(c)});current(c);selected={review,context:c};
  get('summary').textContent=[review.root_name+' · verification '+review.status,review.action+' '+review.path,review.byte_length+' exact native transcript bytes', 'SHA-256: '+review.transcript_sha256,'Existing destination: '+(review.previous_sha256||'absent'),'All other selected package bytes must match the verified inputs. Existing transcript is replaced only after confirmation. Verification does not certify an election.'].join('\n');
 }catch(error){retire();if(error.name!=='AbortError')onError(error.message);}finally{tool?.close();tool=null;working=false;controller=null;setBusy(false);update();}};
 get('confirm').onchange=update;get('cancel').onclick=()=>controller?.abort(new Error('Transcript installation cancelled. Started effects may remain; review the package again.'));
 get('apply').onclick=async()=>{try{
  const c=getContext();if(c.busy||working||!selected||!get('confirm').checked)throw new Error('Review and confirm the transcript destination.');current(selected.context);const review=selected.review;selected=null;get('confirm').checked=false;working=true;controller=new AbortController();setBusy(true);update();
  receipt=await applyCountTranscriptInstallation(review,{confirmed:true,signal:controller.signal,assertCurrent:()=>current(c),onProgress:value=>{receipt=value;get('summary').textContent='Writing '+value.path+'. Started effects may remain after interruption.';}});
  get('summary').textContent='Transcript written and read back: '+receipt.path+'\nVerification '+receipt.verification_status+'. This records software verification, not official certification. Download the receipt separately.';
 }catch(error){if(error.installationReceipt)receipt=error.installationReceipt;retire();onError(error.message);}finally{working=false;controller=null;setBusy(false);update();}};
 get('receipt').onclick=()=>{if(!receipt||working)return;const url=URL.createObjectURL(new Blob([JSON.stringify(receipt,null,2)+'\n'],{type:'application/json'})),a=document.createElement('a');a.href=url;a.download='rcount-transcript-installation.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
 function update(){const c=getContext();get('controls').hidden=c.operation!=='verify-count-files';if(selected&&(!valid(c)||selected.context.revision!==c.revision||selected.context.result!==c.result||selected.context.packageFiles!==c.packageFiles)){retire();get('summary').textContent='Review retired after package changes. Run verification and review again.';}
  const available=valid(c)&&!working&&!c.busy;get('review').disabled=!available||typeof globalThis.showDirectoryPicker!=='function';get('confirm').disabled=!available||!selected;get('apply').disabled=!available||!selected||!get('confirm').checked;get('cancel').disabled=!working;get('receipt').disabled=!receipt||working;
 }
 return {update};
}

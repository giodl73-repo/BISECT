import {CONFIG_PUBLICATION_DEFAULTS,validateConfigPublicationSettings,reviewConfigPublication,applyConfigPublication,retireConfigPublicationReview} from './config-publication.js';
import {nativeToolSession} from './native-state-output-ui.js';

export function installConfigPublicationControls({document,getContext,getSettings,onChange,setBusy,onError}){
 const panel=document.createElement('fieldset');panel.id='config-publication-controls';
 const make=(tag,id,text)=>{const e=document.createElement(tag);if(id)e.id=id;if(text)e.textContent=text;return e;};
 panel.append(make('legend',null,'Save generated YAML to a folder'));
 const field=(id,text,type)=>{const label=make('label',null,text),e=make('input','config-publication-'+id);e.type=type;label.append(e);panel.append(label);return e;};
 const path=field('path','Relative output path (blank uses configs/<name>.yml)','text'),force=field('force','Force replacement of an existing file','checkbox'),dry=field('dry-run','Dry-run: recompute and review YAML without reading or writing a folder','checkbox');path.maxLength=4096;
 const reviewButton=make('button','config-publication-review','Review destination'),summary=make('pre','config-publication-summary','Generate YAML on this device before reviewing a destination.');reviewButton.type='button';summary.setAttribute('role','status');panel.append(reviewButton,summary);
 const confirm=field('confirm','Confirm this exact reviewed destination and action','checkbox'),apply=make('button','config-publication-apply','Write reviewed YAML'),cancel=make('button','config-publication-cancel','Cancel config save'),download=make('button','config-publication-receipt','Download last save receipt');for(const button of [apply,cancel,download])button.type='button';panel.append(apply,cancel,download);
 panel.append(make('p',null,'Folder saving uses only a newly selected root and a fresh native YAML computation. Open never restores a folder or a write review. Cancellation or an external write can leave created folders/an empty or changed file; this is not an OS-atomic transaction. Download YAML remains available. The last receipt is historical data only.'));
 document.getElementById('config-template-fields').append(panel);
 let selected=null,receipt=null,working=false,controller=null,tool=null,draftError=null,editing=false;
 const settings=()=>getSettings()??CONFIG_PUBLICATION_DEFAULTS;
 const valid=c=>c.fresh&&c.operation==='create-algorithm-config'&&c.result?.schema_version==='bisect-algorithm-config-template-v1'&&c.template;
 const signature=c=>JSON.stringify({revision:c.revision,template:c.template,yaml:c.result?.yaml_text,filename:c.result?.generated_filename,settings:settings(),artifact:c.artifact?.sha256});
 function current(c,stamp){const next=getContext();if(!valid(next)||next.result!==c.result||signature(next)!==stamp||draftError)throw Error('Config, project or destination settings changed. Generate/review again.');}
 function retire(){retireConfigPublicationReview(selected?.review);selected=null;confirm.checked=false;}
 function change(){
  retire();editing=true;
  try{const value=validateConfigPublicationSettings({path:path.value,force:force.checked,dry_run:dry.checked});draftError=null;onChange(value);summary.textContent='Destination settings changed. Review again before any write.';}
  catch(error){draftError={revision:getContext().revision,message:error.message};onError(error.message);summary.textContent='Correct this draft before review. The project retains the earlier valid settings. '+error.message;}
  finally{editing=false;update();}
 }
 path.oninput=change;force.onchange=dry.onchange=change;confirm.onchange=update;
 reviewButton.onclick=async()=>{if(working||getContext().busy)return;try{
  const c=getContext();if(c.busy||working||!valid(c)||draftError)throw Error('Generate YAML and correct destination settings before review.');const recipe=structuredClone(settings()),stamp=signature(c);
  if(!recipe.dry_run&&typeof globalThis.showDirectoryPicker!=='function')throw Error('Folder saving is unavailable here. Download the YAML instead.');
  // Keep the real picker directly within the click's user activation.
  const picking=recipe.dry_run?null:globalThis.showDirectoryPicker({mode:'readwrite'});retire();working=true;controller=new AbortController();setBusy(true);update();
  const root=recipe.dry_run?null:await picking;controller.signal.throwIfAborted();current(c,stamp);tool=nativeToolSession(c.artifact,controller.signal);
  const review=await reviewConfigPublication(tool,c.result,c.template,recipe,root,{signal:controller.signal,assertCurrent:()=>current(c,stamp)});current(c,stamp);selected={review,context:c,stamp};
  summary.textContent=[review.dry_run?'Dry-run: no folder was read or written.':review.root_name+' · '+review.action,review.dry_run?'Ignored output intent: '+review.path:review.path,review.byte_length+' exact native YAML bytes','SHA-256: '+review.yaml_sha256,'Previous file: '+(review.previous_sha256||'absent or not read'),'Parents to create: '+(review.missing_parents.join(', ')||'none'),'Confirm only this reviewed action.'].join('\n');
 }catch(error){if(error.configPublicationReceipt)receipt=error.configPublicationReceipt;retire();summary.textContent=error.message;if(error.name!=='AbortError')onError(error.message);}
 finally{tool?.close();tool=null;working=false;controller=null;setBusy(false);update();}};
 apply.onclick=async()=>{if(working||getContext().busy)return;try{
  const c=getContext();if(c.busy||working||!selected||!confirm.checked||draftError)throw Error('Review and confirm the config destination.');current(selected.context,selected.stamp);const held=selected;selected=null;confirm.checked=false;working=true;controller=new AbortController();setBusy(true);update();
  receipt=await applyConfigPublication(held.review,{confirmed:true,signal:controller.signal,assertCurrent:()=>current(held.context,held.stamp),onProgress:value=>{receipt=value;summary.textContent='Config save: '+value.status+'\n'+value.path+'\nStarted effects may remain after interruption.';}});
  summary.textContent=receipt.dry_run?'Dry-run completed. No folder was read or written.':'YAML written and exact bytes read back: '+receipt.path+'\nSHA-256: '+receipt.yaml_sha256;
 }catch(error){if(error.configPublicationReceipt)receipt=error.configPublicationReceipt;retire();summary.textContent=error.message;onError(error.message);}
 finally{working=false;controller=null;setBusy(false);update();}};
 cancel.onclick=()=>controller?.abort(Error('Config save cancelled. Started effects may remain; review current destination bytes before retrying.'));
 download.onclick=()=>{if(!receipt||working)return;const url=URL.createObjectURL(new Blob([JSON.stringify(receipt,null,2)+'\n'],{type:'application/json'})),link=document.createElement('a');link.href=url;link.download='config-save-'+receipt.status+'.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
 function update(){
  const c=getContext(),value=settings();panel.hidden=c.operation!=='create-algorithm-config';
  if(draftError&&draftError.revision!==c.revision&&!editing)draftError=null;
  if(selected&&(!valid(c)||selected.context.result!==c.result||signature(c)!==selected.stamp)){retire();summary.textContent='Save review retired after project/config changes. Review the current destination again.';}
  if(!draftError&&!editing){if(path.value!==value.path)path.value=value.path;force.checked=value.force;dry.checked=value.dry_run;}
  const locked=c.busy||working;for(const input of [path,force,dry])input.disabled=locked;
  const available=!!valid(c)&&!locked&&!draftError;reviewButton.disabled=!available||(!value.dry_run&&typeof globalThis.showDirectoryPicker!=='function');confirm.disabled=!available||!selected;apply.disabled=!available||!selected||!confirm.checked;apply.textContent=selected?.review.dry_run?'Finish reviewed dry-run':'Write reviewed YAML';cancel.disabled=!working;download.disabled=!receipt||working;
 }
 return {update};
}

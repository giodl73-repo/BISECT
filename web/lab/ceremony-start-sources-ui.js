import {CEREMONY_START_DEFAULTS,ceremonyStartSourceRequest,validateCeremonyStartInputs,readCeremonyRuleProfile} from './ceremony-start-sources.js';
export function installCeremonyStartSourcesControls({document,getContext,getInputs,onChange,onInvalidate,onError}){
 const panel=document.createElement('fieldset');panel.id='ceremony-start-source-fields';
 const element=(tag,id,text)=>{const e=document.createElement(tag);if(id)e.id=id;if(text)e.textContent=text;return e;};
 panel.append(element('legend',null,'Start from original rule profile'));
 const field=(id,caption,tag='input')=>{const label=element('label',null,caption),input=element(tag,id);label.append(input);panel.append(label);return input;};
 const profile=field('ceremony-start-profile','Original rule-profile file (including binary or empty files)'),name=element('p','ceremony-start-profile-name');profile.type='file';panel.append(name);
 const genesis=field('ceremony-start-genesis','Genesis Unix timestamp (exact signed integer)'),review=field('ceremony-start-review','Minimum review interval in seconds (exact integer)'),minimum=field('ceremony-start-minimum','Minimum distinct witness receipts'),receipts=field('ceremony-start-witnesses','Genesis witness receipt IDs (one per line)','textarea');
 genesis.maxLength=20;review.maxLength=20;minimum.type='number';minimum.min='1';minimum.max='100';minimum.step='1';receipts.rows=3;
 const clear=element('button','ceremony-start-clear-profile','Clear original profile'),cancel=element('button','ceremony-start-cancel-read','Cancel profile read');clear.type=cancel.type='button';panel.append(clear,cancel);
 const status=element('p','ceremony-start-source-status');status.setAttribute('role','status');panel.append(status,element('p',null,'The selected bytes stay in the project. Run hashes the original profile and derives the configuration from the complete tree and these values. Native checks still validate the tree and timing/witness rules. Nothing is published or externally authenticated.'));
 document.getElementById('ceremony-fields').append(panel);
 let active=null,draft=null,editing=false;
 const inputs=()=>getInputs()??CEREMONY_START_DEFAULTS;
 const same=flow=>{const c=getContext();return active===flow&&!c.busy&&c.visible&&c.revision===flow.revision;};
 function update(){
  const c=getContext(),v=inputs();panel.hidden=!c.visible;
  if(draft&&draft.revision!==c.revision&&!editing)draft=null;
  if(active&&!same(active)){active=null;status.textContent='Project or operation changed. Pending profile selection was discarded.';}
  const locked=c.busy||!!active;for(const e of [profile,genesis,review,minimum,receipts])e.disabled=locked;
  clear.disabled=locked||!v.rule_profile;cancel.disabled=!active;
  name.textContent=v.rule_profile?v.rule_profile.name+(v.rule_profile.base64===''?' · empty file':''):'No original profile selected';
  const values=draft??{genesis:v.settings.genesis_unix_seconds,review:v.settings.review_seconds,minimum:String(v.settings.minimum_witnesses),receipts:v.settings.witness_receipts.join('\n')};
  for(const [e,value]of [[genesis,values.genesis],[review,values.review],[minimum,values.minimum],[receipts,values.receipts]])if(e.value!==value)e.value=value;
 }
 function invalidate(){editing=true;try{onInvalidate();if(draft)draft.revision=getContext().revision;}finally{editing=false;}}
 function change(next){editing=true;try{onChange(next);if(draft)draft.revision=getContext().revision;}finally{editing=false;}}
 const edit=event=>{
  const c=getContext();if(c.busy||active||!c.visible)return;
  const raw={genesis:genesis.value,review:review.value,minimum:minimum.value,receipts:receipts.value};
  try{if(raw.minimum==='')throw Error('Supply the minimum distinct witness count.');const old=inputs().settings.witness_receipts,editedReceipts=event?.currentTarget===receipts||event?.target===receipts,next={rule_profile:inputs().rule_profile,settings:{genesis_unix_seconds:raw.genesis,review_seconds:raw.review,minimum_witnesses:Number(raw.minimum),witness_receipts:editedReceipts?(raw.receipts===''?[]:raw.receipts.split(/\r?\n/)):[...old]}};validateCeremonyStartInputs(next);draft=null;change(next);status.textContent='Start inputs changed. Run explicitly to create a transcript.';}
  catch(error){draft={...raw,revision:getContext().revision,error:error.message};invalidate();status.textContent='Earlier valid settings retained in the project. '+error.message;onError(error.message);update();}
 };
 for(const e of [genesis,review,minimum,receipts])e.oninput=edit;
 profile.onchange=async()=>{
  const file=profile.files?.[0];profile.value='';if(!file||getContext().busy||!getContext().visible||active)return;
  invalidate();const flow={revision:getContext().revision};active=flow;status.textContent='Reading original rule-profile bytes…';update();
  try{const source=await readCeremonyRuleProfile(file);if(!same(flow))return;active=null;change({...inputs(),rule_profile:source});status.textContent='Original profile loaded. No ceremony has started.';}
  catch(error){if(same(flow)){active=null;status.textContent='Earlier profile retained. '+error.message;onError(error.message);}}
  finally{if(active===flow)active=null;update();}
 };
 clear.onclick=()=>{if(getContext().busy||active)return;invalidate();change({...inputs(),rule_profile:null});status.textContent='Original profile cleared.';update();};
 cancel.onclick=()=>{active=null;status.textContent='Profile read cancelled. Earlier inputs retained.';update();};
 update();return {update,request(tree){if(active)throw Error('Wait for the original profile read or cancel it.');if(draft)throw Error(draft.error);return ceremonyStartSourceRequest(tree,inputs());}};
}

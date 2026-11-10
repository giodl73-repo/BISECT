import {readPolicyFallbackSource,validatePolicyFallback} from './policy-fallback.js';
export function installPolicyFallbackControls({document,getContext,onBegin,onChange,setBusy,onError}){
 const make=(tag,id,text)=>{const e=document.createElement(tag);if(id)e.id=id;if(text)e.textContent=text;return e;};
 const panel=make('fieldset','policy-fallback-controls'),label=make('label',null,'Use native override fallback'),mode=make('input','policy-fallback-mode');mode.type='checkbox';label.append(mode);panel.append(make('legend',null,'Override source selection'),label);
 const controls={};
 for(const role of ['primary','alias']){
  const label=make('label',null,role==='primary'?'Primary override':'Alias override'),file=make('input','policy-fallback-'+role);file.type='file';const name=make('span','policy-fallback-'+role+'-name'),clear=make('button','policy-fallback-'+role+'-clear','Clear '+role);clear.type='button';label.append(file,name);panel.append(label,clear);controls[role]={file,name,clear};
 }
 const cancel=make('button','policy-fallback-cancel','Cancel source read'),status=make('p','policy-fallback-status');cancel.type='button';status.setAttribute('role','status');panel.append(cancel,status,make('p',null,'On Run, try primary JSON, then alias, then the embedded database. A valid JSON parse stops fallback even if location lookup fails. Empty files and original bytes are retained. Open restores data only. Unavailable entries are recorded input observations, not authenticated live read failures. Turning this mode off selects the embedded database.'));document.getElementById('policy-fields').append(panel);
 let active=null;
 const input=()=>getContext().inputs??{query:getContext().query,source:null};
 const fail=e=>{status.textContent=e.message;onError(e.message);};
 function change(value){if(active||getContext().busy)return;try{onBegin();onChange(value);status.textContent='Source recipe changed. Run to select and inspect a database.';}catch(e){fail(e);}update();}
 mode.onchange=()=>{if(active||getContext().busy){update();return;}const old=input();document.getElementById('policy-source').value='';change(mode.checked?{query:getContext().query,source:null,fallback:{primary:old.source?{status:'file',...old.source}:null,alias:null}}:{query:getContext().query,source:null});};
 for(const role of ['primary','alias']){
  controls[role].clear.onclick=()=>{const old=input();if(!old.fallback)return;change({...old,fallback:{...old.fallback,[role]:null}});controls[role].file.value='';};
  controls[role].file.onchange=async event=>{
   if(active||getContext().busy)return;
   const old=input();if(!old.fallback)return;let token;
   try{
    const expected=onBegin();token={expected};active=token;setBusy(true);update();
    const candidate=await readPolicyFallbackSource(event.target.files[0]);
    if(active!==token)return;if(getContext().revision!==expected)throw Error('Project changed while reading policy source. Select it again.');
    const fallback=validatePolicyFallback({...old.fallback,[role]:candidate});onChange({...old,query:getContext().query,fallback});status.textContent=candidate.status==='unavailable'?'Read failed. An unavailable-source observation was recorded; Run will continue to the next source.':'Original source bytes retained. Run to select and inspect a database.';
   }catch(e){if(!token||active===token)fail(e);}
   finally{if(active===token){active=null;setBusy(false);update();}}
  };
 }
 cancel.onclick=()=>{if(!active)return;active=null;setBusy(false);onBegin();status.textContent='Source read cancelled. Earlier recipe retained; late data will be ignored.';update();};
 function update(){const c=getContext(),value=input(),enabled=value.fallback!=null,locked=c.busy||!!active;mode.checked=enabled;mode.disabled=locked;for(const role of ['primary','alias']){const x=controls[role],candidate=value.fallback?.[role];x.file.disabled=!enabled||locked;x.clear.disabled=!enabled||locked||!candidate;x.name.textContent=candidate?candidate.name+(candidate.status==='unavailable'?' · recorded unavailable':''):'Not configured';}cancel.disabled=!active;if(enabled){document.getElementById('policy-source').disabled=true;document.getElementById('policy-embedded').disabled=true;}}
 return {update};
}

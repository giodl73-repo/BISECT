import {DOCTOR_COMMAND_DEFAULTS,validateDoctorCommand,doctorCommandReport} from './doctor-command.js';
export function installDoctorCommandControls({document,getContext,onChange,onInvalid,onError}){
 const make=(tag,id,text)=>{const e=document.createElement(tag);if(id)e.id=id;if(text)e.textContent=text;return e;};
 const panel=make('fieldset','doctor-command-controls'),enable=make('input','doctor-command-enabled'),label=make('label',null,'Include a full Doctor command report');enable.type='checkbox';label.append(enable);panel.append(make('legend',null,'Doctor command report'),label);
 const controls={};
 for(const [key,title]of [['mode','Report mode'],['label','Plan label'],['output_base','Reported output prefix (text only)'],['version','Reported version'],['year','Reported year']]){
  const label=make('label',null,title),input=make(key==='mode'?'select':'input','doctor-command-'+key);if(key==='mode')for(const mode of ['all','label']){const o=make('option',null,mode==='all'?'All supplied plans':'One plan label');o.value=mode;input.append(o);}else{input.type='text';input.maxLength=key==='output_base'?4096:200;}label.append(input);panel.append(label);controls[key]=input;
 }
 const status=make('p','doctor-command-status'),download=make('button','download-doctor-command','Download Doctor diagnosis text');download.type='button';status.setAttribute('role','status');panel.append(status,download,make('p',null,'The output prefix, version and year reproduce console headings only; no path is opened. Reports use the supplied inventory. Native exit 0 can include missing analysis, maps or audit files. Presence does not validate file contents or legal readiness. Open restores data only; Run again before exporting a command report.'));document.getElementById('plan-doctor-fields').append(panel);
 let draft=null;
 function change(){const c=getContext();if(c.busy)return;try{
  let input={...(c.input??{plans:[]})};if(enable.checked){const command=validateDoctorCommand(Object.fromEntries(Object.entries(controls).map(([k,e])=>[k,e.value])));input.command=command;}else delete input.command;
  draft=null;onChange(input);
 }catch(e){draft={input:c.input,message:e.message};status.textContent=e.message;onInvalid?.();onError(e.message);}update();}
 enable.onchange=()=>{if(enable.checked&&!getContext().input?.command)for(const [k,v]of Object.entries(DOCTOR_COMMAND_DEFAULTS))controls[k].value=v;change();};for(const e of Object.values(controls)){e.oninput=change;e.onchange=change;}
 download.onclick=()=>{const c=getContext();if(c.busy||!c.fresh||!c.input?.command||draft)return;try{const report=doctorCommandReport(c.result,c.input),url=URL.createObjectURL(new Blob([report.console_text],{type:'text/plain'})),a=document.createElement('a');a.href=url;a.download='doctor-'+report.command.mode+'.txt';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}catch(e){onError(e.message);}};
 function update(){const c=getContext(),command=c.input?.command;if(draft&&draft.input!==c.input)draft=null;if(!draft){enable.checked=!!command;for(const [k,e]of Object.entries(controls))e.value=(command??DOCTOR_COMMAND_DEFAULTS)[k];}enable.disabled=c.busy;for(const [k,e]of Object.entries(controls))e.disabled=c.busy||!enable.checked||(k==='label'&&controls.mode.value!=='label');download.disabled=c.busy||!c.fresh||!command||!!draft||c.result?.schema_version!=='bisect-plan-doctor-v1';if(!draft)status.textContent=command?(c.fresh?'Fresh supplied-data report available. A completed diagnostic is not a passing plan.':'Run the supplied inventory to create this report.'):'Native table export remains available without a command report.';}
 return {update,assertReady(){if(draft)throw Error('Correct the Doctor report draft: '+draft.message);}};
}

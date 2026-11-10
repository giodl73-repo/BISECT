// A guide over existing workbench actions. Routing never prepares or runs inputs.
import {practitionerInputChecklist} from './practitioner-inputs.js';
const routes={state:'run-tui-configured',selected:'run-native-selected-state-source',national:'run-state-batch',review:'validate-rplan-text'};
const contexts={
 'run-tui-configured':['One state','Choose a state graph and algorithm configuration below. Review the district count, dimensions and optional seed.'],
 'run-native-selected-state-source':['Selected source files','Choose native binary, Census or tract sources below. State flags must match their scope.'],
 'run-state-batch':['Several states','Choose prepared state jobs below and review the state filters and worker slots. Acquire source data separately if needed.'],
 'plan-fetch-sources':['Acquire state sources','Review states and years below. Source acquisition and partitioning are separate explicit actions.'],
 'validate-rplan-text':['Review an existing plan','Open the exact RPLAN below and optional matching map geometry. Run checks the selected plan.'],
 'render-native-district-map':['Map an accepted plan','Review the selected plan and matching geometry below, then render the map. This does not run another partition.']
};
const accepted=result=>['bisect-prepared-state-run-v1','bisect-tui-run-v1','bisect-state-run-batch-v1','bisect-state-run-years-v1'].includes(result?.schema_version);
export function installPractitionerFlow({document,getState,selectOperation}){
 const get=id=>document.getElementById(id),root=get('practitioner-flow');
 const available=id=>{const el=get(id);return el&&!el.disabled&&!el.hidden&&!el.closest('[hidden]');};
 const first=ids=>ids.find(available);
 const scroll=el=>{el?.scrollIntoView({block:'start',behavior:'auto'});if(el){if(!el.matches('input,button,select,textarea,a[href]'))el.tabIndex=-1;el.focus({preventScroll:true});}};
 const busy=()=>getState().busy;
 const missing=()=>practitionerInputChecklist(getState().operation,getState().files).filter(row=>!row.present);
 function focusInput(id){const el=get(id);if(!el)return scroll(inputTarget());for(let parent=el.parentElement;parent;parent=parent.parentElement)if(parent.tagName==='DETAILS')parent.open=true;scroll(el);}
 const inputTarget=()=>{const operation=getState().operation,ids={'run-tui-configured':'tui-run-fields','run-native-selected-state-source':get('single-state-source-kind').value==='native'?'native-adjacency-fields':'native-state-source-controls','run-state-batch':'state-run-batch-fields','plan-fetch-sources':'fetch-plan-fields','validate-rplan-text':'text-audit-fields','render-native-district-map':'native-map-fields'},target=get(ids[operation]);return target&&!target.hidden?target:get('practitioner-inputs');};
 function route(operation){if(busy())return;selectOperation(operation);
  if(operation===routes.selected){for(const [id,value]of [['native-state-source-command','state'],['native-state-source-enabled',true]]){const control=get(id),key=typeof value==='boolean'?'checked':'value';if(control[key]!==value){control[key]=value;control.dispatchEvent(new document.defaultView.Event('change',{bubbles:true}));}}}
  refresh();scroll(inputTarget());
 }
 for(const [key,operation]of Object.entries(routes))get('flow-route-'+key).onclick=()=>route(operation);
 get('flow-acquire').onclick=()=>route('plan-fetch-sources');
 get('flow-open').onclick=()=>{if(!busy()&&!get('open-project').disabled)get('open-project').click();};
 get('flow-open-archive').onclick=()=>{if(!busy()&&!get('open-workbench-archive').disabled)get('open-workbench-archive').click();};
 get('flow-inputs').onclick=()=>{if(!busy())scroll(inputTarget());};
 get('flow-run').onclick=()=>{if(!busy()&&!get('execute').disabled&&!missing().length)get('execute').click();};
 get('flow-fix-inputs').onclick=()=>{if(!busy())focusInput(missing()[0]?.target);};
 get('flow-error-inputs').onclick=()=>{if(!busy())focusInput(missing()[0]?.target);};
 get('flow-error-command').onclick=()=>{if(!busy()&&getState().operation==='run-native-selected-state-source')focusInput('state-command-arguments');};
 get('flow-failure-receipt').onclick=()=>{if(!busy()&&getState().failureFresh&&available('download-worker-failure'))get('download-worker-failure').click();};
 get('flow-review').onclick=()=>{if(!busy())scroll(get('practitioner-results'));};
 get('flow-save').onclick=()=>{if(!busy()&&!get('save-workbench-archive').disabled)get('save-workbench-archive').click();};
 const maps=['native-states-map','tui-run-map','map-configured-plan','state-run-batch-show-map'];
 const downloads=['download-native-states-run','download-tui-run','download-state-run-batch','download-native-state-output','download-native-map-png'];
 get('flow-map').onclick=()=>{if(busy()||!getState().result)return;const id=first(maps);if(id){get(id).click();scroll(get(id==='state-run-batch-show-map'?'practitioner-results':'practitioner-inputs'));}};
 get('flow-download').onclick=()=>{if(busy()||!getState().result)return;const id=first(downloads);if(id)get(id).click();};
 get('flow-workflow-download').onclick=()=>{if(!busy()&&getState().result?.workflow?.schema_version==='bisect-state-workflow-v1'&&available('download-tui-workflow'))get('download-tui-workflow').click();};
 function refresh(){
  const state=getState(),context=contexts[state.operation],result=state.result;
  const followup=result?.workflow_error?'failed':result?.workflow?.status,followupIncomplete=!!followup&&followup!=='completed';
  const checklist=practitionerInputChecklist(state.operation,state.files),absent=checklist.filter(row=>!row.present),signature=JSON.stringify(checklist);
  const list=get('flow-input-checklist');
  if(list.dataset.signature!==signature){list.replaceChildren();for(const row of checklist){const item=document.createElement('li');item.textContent=(row.present?'Selected: ':'Missing: ')+row.label+(row.source_name?' ('+row.source_name+')':'');list.append(item);}list.dataset.signature=signature;}
  get('flow-input-readiness').hidden=!checklist.length;
  get('flow-input-note').textContent=absent.length?absent.length+' required selection'+(absent.length===1?' is':'s are')+' missing. File contents and settings will be checked on Run.':'Required selections are present. File contents and settings still need native validation on Run.';
  get('flow-context').textContent=context?.[0]??'Specialist tool';
  get('flow-next').textContent=context?.[1]??'Review the selected tool’s inputs below. Choose another workflow or use Find a tool for the complete toolkit.';
  get('flow-source-status').textContent=state.hasSources?'Inputs retained in this project.':'Open a saved project or select source files below.';
  get('flow-run-status').textContent=state.busy?'An operation is in progress.':state.error?'The last action failed. Review the error and adjust inputs before retrying.':!state.ready?'Loading the browser engine…':absent.length?'Select the missing inputs before using guided Run.':result?accepted(result)?'Run completed on this device.'+(followupIncomplete?' Follow-up '+followup+'; review its stages.':''):'Check or preparation completed; review its result.':'Review inputs, then Run explicitly.';
  get('flow-result-status').textContent=result?followupIncomplete?'Accepted plan retained; follow-up is '+followup+'. Review stages below.':'Current result is available below.':state.retainedResult?'A retained result is unverified for these inputs. Run again to check.':'No current result. A map requires matching geometry.';
  get('flow-save-status').textContent='Save keeps the project’s inputs and retained results. Open never runs it.';
  for(const button of root.querySelectorAll('button'))button.disabled=state.busy;
  get('flow-run').disabled=state.busy||!state.ready||get('execute').disabled||!!absent.length;
  get('flow-fix-inputs').hidden=!absent.length;get('flow-fix-inputs').disabled=state.busy||!absent.length;
  get('flow-error').hidden=!state.error;get('flow-error-message').textContent=state.error?String(state.error).slice(0,2000):'';
  get('flow-error-inputs').disabled=state.busy;get('flow-failure-receipt').hidden=!state.error||!state.failureFresh||!available('download-worker-failure');get('flow-failure-receipt').disabled=state.busy;
  get('flow-error-command').hidden=!state.error||state.operation!=='run-native-selected-state-source';get('flow-error-command').disabled=state.busy;
  get('flow-run').textContent=state.operation==='render-native-district-map'?'Render map':state.operation==='validate-rplan-text'?'Check plan':'Run on this device';
  const native=(result?.native_state??result?.native_states??result?.native_run)?.compilation?.native_command,seed=native?.fields?.find(f=>f.id==='seed')?.values?.[0];
  get('flow-settings-summary').textContent=seed!==undefined&&seed!==null?'Recorded seed: '+seed+'. Keep it with your run.':'Review the state, year, districts, dimensions and seed.';
  get('flow-save').disabled=state.busy||get('save-workbench-archive').disabled;
  get('flow-open').disabled=state.busy||get('open-project').disabled;get('flow-open-archive').disabled=state.busy||get('open-workbench-archive').disabled;
  get('flow-review').disabled=state.busy||!result;
  const map=!!result&&!!first(maps),download=!!result&&!!first(downloads);get('flow-map').hidden=!map;get('flow-map').disabled=state.busy||!map;get('flow-download').hidden=!download;get('flow-download').disabled=state.busy||!download;
  const workflowDownload=result?.workflow?.schema_version==='bisect-state-workflow-v1'&&available('download-tui-workflow');get('flow-workflow-download').hidden=!workflowDownload;get('flow-workflow-download').disabled=state.busy||!workflowDownload;
  for(const [key,operation]of Object.entries(routes)){const button=get('flow-route-'+key);button.setAttribute('aria-pressed',String(operation===state.operation));}
  for(const item of root.querySelectorAll('[data-flow-step]'))item.removeAttribute('aria-current');
  get('flow-step-'+(state.busy?'run':result?'review':absent.length?'sources':state.hasSources?'settings':'sources')).setAttribute('aria-current','step');
 }
 refresh();return {refresh};
}

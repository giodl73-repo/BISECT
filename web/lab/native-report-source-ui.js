import {nativeToolSession} from './native-state-output-ui.js';
import {planNativeReportSources,readNativeReportSources,nativeReportSourcesFromFiles} from './native-report-source-snapshot.js';
import {validateNativeReportSnapshot} from './native-report-source-inputs.js';

/** Local handles belong only to an explicit read and never enter project data. */
export function installNativeReportSourceControls({document,getContext,onSnapshot,onClear,onError,toolSession=nativeToolSession}){
 const panel=document.createElement('fieldset');panel.id='native-report-sources';
 const element=(tag,id,text)=>{const e=document.createElement(tag);if(id)e.id=id;if(text)e.textContent=text;return e;};
 panel.append(element('legend',null,'State report population sources'));
 panel.append(element('p',null,'Review the native candidate paths, then read a working folder containing data/ or select the listed CSV files. Report counts do not replace the graph populations used for balancing. Loading does not run a partition.'));
 const button=(id,text)=>{const b=element('button',id,text);b.type='button';panel.append(b);return b;};
 const review=button('native-report-review','Review report source paths'),folder=button('native-report-folder','Read selected working folder');
 const label=element('label',null,'Report CSV files');const input=element('input','native-report-files');input.type='file';input.multiple=true;input.accept='.csv,text/csv';label.append(input);panel.append(label);
 const absenceLabel=element('label'),absence=element('input','native-report-accept-missing');absence.type='checkbox';absenceLabel.append(absence,document.createTextNode('Treat unselected candidate paths as absent in this file bundle. This says nothing about files elsewhere on my computer.'));panel.append(absenceLabel);
 const empty=button('native-report-empty','Use a bundle with no report files'),cancel=button('native-report-cancel','Cancel loading'),clear=button('native-report-clear','Detach report-source selection');
 const status=element('p','native-report-status');status.setAttribute('role','status');const paths=element('pre','native-report-paths'),summary=element('pre','native-report-summary'),outcome=element('p','native-report-outcome');outcome.setAttribute('role','status');panel.append(status,paths,summary,outcome);
 document.getElementById('state-workflow-history').after(panel);
 let active=null,plan=null,planRevision=null,consentRevision;
 const same=flow=>{const c=getContext();return active===flow&&!c.busy&&c.visible&&c.revision===flow.revision&&c.artifact?.sha256===flow.artifact.sha256&&c.command?.arguments_json===flow.command.arguments_json;};
 const current=flow=>{flow.controller.signal.throwIfAborted();if(!same(flow))throw Error('State flags, project or runtime changed. Load the report sources again.');};
 function retire(){const flow=active;active=null;flow?.controller.abort();flow?.tool?.close();}
 function showPlan(p){paths.textContent='Requested source: '+p.requested_population_source+'\nCVAP availability path: '+p.availability_path+'\nReport search order (first existing wins):\n'+p.report_candidates.map((path,i)=>(i+1)+'. '+path).join('\n');}
 async function load(action){
  const c=getContext();if(c.busy||!c.ready||!c.visible||active)return;
  const flow={revision:c.revision,command:structuredClone(c.command),artifact:structuredClone(c.artifact),controller:new AbortController(),tool:null};active=flow;status.textContent='Reading the explicitly selected report sources…';update();
  try{
   flow.tool=toolSession(flow.artifact,flow.controller.signal);const options={signal:flow.controller.signal,isCurrent:()=>same(flow)};
   const p=await planNativeReportSources(flow.tool,flow.command,options);current(flow);plan=p;planRevision=flow.revision;showPlan(p);
   const snapshot=await action(flow,p,options);current(flow);
   // Retire the worker before the host changes project revision.
   active=null;flow.tool.close();flow.tool=null;
   if(snapshot){validateNativeReportSnapshot(snapshot);onSnapshot(snapshot);status.textContent='Report-source snapshot loaded. Inspect the original flags, then Apply before Run. No folder permission is saved.';}
   else status.textContent='Candidate paths reviewed. Select a working folder or an explicit file bundle to load a snapshot.';
  }catch(error){if(active===flow){status.textContent=flow.controller.signal.aborted?'Report-source loading cancelled. Earlier project inputs remain unchanged.':'No report-source snapshot applied. '+error.message;if(!flow.controller.signal.aborted&&error.name!=='AbortError')onError(error.message);}}
  finally{if(active===flow)active=null;flow.tool?.close();update();}
 }
 review.onclick=()=>load(async()=>null);
 folder.onclick=()=>{
  if(active||getContext().busy||!getContext().ready)return;
  if(typeof globalThis.showDirectoryPicker!=='function'){onError('Folder selection is unavailable here. Select the listed CSV files as a portable bundle.');return;}
  // Open within the click's user activation, before starting worker requests.
  let selected;try{selected=globalThis.showDirectoryPicker({mode:'read'});}catch(error){onError(error.message);return;}
  // Attach immediately so a quick picker rejection cannot become unhandled.
  const selectedResult=Promise.resolve(selected).then(root=>({root}),error=>({error}));
  return load(async(flow,p,options)=>{const selection=await selectedResult;current(flow);if(selection.error)throw selection.error;return readNativeReportSources(flow.tool,flow.command,selection.root,options);});
 };
 async function filesSelected(selected){
  const accept=absence.checked;
  return load(async(flow,p,options)=>{
   if(!accept)throw Error('Choose how this portable bundle treats unselected candidates before loading files.');
   if(selected.length>5)throw Error('Select at most five candidate CSV files.');
   const files=[];let total=0;const seen=new Set();
   for(const file of selected){
    current(flow);const matches=p.required_observation_paths.filter(path=>path.split('/').at(-1)===file.name);
    if(matches.length!==1||seen.has(matches[0]))throw Error('Select each exact candidate filename once: '+file.name);seen.add(matches[0]);
    if(!Number.isSafeInteger(file.size)||file.size<0||file.size>8*1024*1024||(total+=file.size)>16*1024*1024)throw Error('Report CSVs are limited to 8 MiB each and 16 MiB total.');
    const bytes=new Uint8Array(await file.arrayBuffer());current(flow);if(bytes.length!==file.size)throw Error('A selected CSV changed while loading.');files.push({relative_path:matches[0],bytes});
   }
   return nativeReportSourcesFromFiles(flow.tool,flow.command,files,{...options,acceptUnselectedAsMissing:accept});
  });
 }
 input.onchange=()=>{const selected=Array.from(input.files??[]);input.value='';if(selected.length)return filesSelected(selected);};
 empty.onclick=()=>filesSelected([]);
 cancel.onclick=()=>{retire();status.textContent='Report-source loading cancelled. Earlier project inputs remain unchanged.';update();};
 clear.onclick=()=>{if(active||getContext().busy)return;onClear();status.textContent='Report-source selection detached. Explicit population-source flags require a new snapshot before Run.';update();};
 function update(){
  const c=getContext();panel.hidden=!c.visible;
  if(consentRevision!==c.revision){absence.checked=false;input.value='';consentRevision=c.revision;}
  if(active&&!same(active)){retire();status.textContent='Project, flags or runtime changed. The pending selection was discarded.';}
  if(planRevision!==c.revision){plan=null;planRevision=null;paths.textContent='Review paths for the current original State flags.';}
  const locked=c.busy||!c.ready||!!active;
  for(const e of [review,folder,input,absence,empty])e.disabled=locked;
  cancel.disabled=!active;clear.disabled=locked||!c.snapshot;
  const s=c.snapshot;
  const result=c.fresh?c.result?.native_report_sources:null;
  outcome.textContent=result?.status==='failed'?'Plan generation is retained, but report export failed: '+result.error+' Save the project or download the retained generation. Completed output and audit export are unavailable. Load corrected sources, inspect and Apply, then explicitly Run again.':result?.status==='bound'?'Report sources applied to the accepted plan. Balancing used the selected graph populations. Requested source: '+result.requested_population_source+(result.cvap_missing_warning?'. CVAP was missing in this snapshot; the native availability check warned.':''):'';
  summary.textContent=s?'Retained '+s.request.observation_origin+'\nSource bundle SHA-256: '+s.receipt.source_bundle_sha256+'\nRequested source: '+s.receipt.requested_population_source+'\nMissing-CVAP warning: '+s.receipt.cvap_missing_warning+'\n'+s.request.observations.map(o=>o.kind+': '+o.relative_path).join('\n')+'\nOpen grants no folder authority and performs no discovery or computation.':'No report-source snapshot selected. Open and path review never read a folder.';
 }
 update();return {update,cancel:()=>{retire();update();}};
}

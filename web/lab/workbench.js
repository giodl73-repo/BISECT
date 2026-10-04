import {renderCountResult} from './count-results.js';
import {resultPreview} from './result-preview.js';
import {readSelectedPackage, decodePackageFiles, validatePackageFiles, readElectionSource, validateSourceFile, validateImportSettings, validatePackageArchive,validateRiSources,validateAggregationSettings,validateDemographicSettings} from './package-files.js';
import {createProject, readJsonFile, readProjectFile, readAutosave, writeAutosave} from './project.js';
const $=id=>document.getElementById(id);
const files=Object.create(null);let revision=0, importSequence=0, importController;let worker,ready=false,busy=false,result,id=0,lastOperation;
const metadataKeys=['country','state','county','date','election_type','scope','status'];
function metadata(){return Object.fromEntries(metadataKeys.map(key=>[key,$('import-'+key).value]));}
function importedPackage(){if(result?.schema_version!=='bisect-election-import-v1')throw new Error('Run an election import first.');return validatePackageFiles(result.package_files);}
function updateImportControls(){let valid=false;try{importedPackage();valid=true;}catch{}$('use-imported-package').disabled=!valid||busy;$('download-package').disabled=!valid||busy;$('download-aggregation').disabled=busy||result?.schema_version!=='bisect-district-aggregation-v1';$('use-demographic-context').disabled=busy||result?.schema_version!=='bisect-demographic-context-v1';$('download-demographic-context').disabled=busy||result?.schema_version!=='bisect-demographic-context-v1';$('download-audit-profile').disabled=busy||!files.profile;for(const id of ['demographic-source','demographic-basis','demographic-label','vra-group','vra-report-threshold','apply-vra-policy'])$(id).disabled=busy;}
function updateInputPanels(){const operation=$('operation').value;$('demographic-context-fields').hidden=operation!=='attach-demographic-csv';$('vra-policy-fields').hidden=operation!=='audit-plan';$('demographic-source-name').textContent=files.demographicSource?.name||'No source selected';$('history-fields').hidden=operation!=='verify-history-files';$('aggregation-fields').hidden=operation!=='aggregate-districts';$('audit-check-fields').hidden=operation!=='audit-plan';$('election-import-fields').hidden=!['import-statement-csv','import-nist-cdf-json'].includes(operation);$('ri-import-fields').hidden=operation!=='import-ri2024-rep28-rla';for(const key of ['auditReport','ballotManifest','ballotRetrieval'])$('ri-'+key+'-name').textContent=files.riSources?.[key]?.name||'No source selected';$('crosswalk-name').textContent=files.crosswalkFile?.name||'No explicit crosswalk selected';}
function demographicSettings(){return {basis:$('demographic-basis').value,source_label:$('demographic-label').value.trim()||files.demographicSource?.name||''};}
function renderVraPolicy(){const policy=files.profile?.vra_policy;$('vra-group').value=policy?.type==='report-opportunity-districts'?policy.minority_group:'';$('vra-report-threshold').value=policy?.type==='report-opportunity-districts'?policy.vap_threshold*100:50;}
function preview(value){return resultPreview(value?.schema_version==='bisect-election-import-v1'?{...value,package_files:`${Object.keys(value.package_files).length} exact files. Download the package archive to retain their contents.`}:value?.schema_version==='bisect-district-aggregation-v1'?{...value,transcript_base64:'Download the native aggregation transcript for exact integer JSON.'}:value);}
function aggregationSettings(){return {contest_id:$('aggregation-contest').value,status:$('aggregation-status').value,output_format:$('aggregation-format').value};}
function start(){
  ready=false;$('execute').disabled=true;$('engine-status').textContent='Loading browser engine…';
  worker=new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'});
  worker.onmessage=({data})=>{
    if(data.type==='ready'){ready=true;$('engine-status').textContent='Rust WASM · on this device';$('execute').disabled=busy;}
    if(data.type==='result'){result=data.result;renderCountResult($('result-tables'),result);$('result').textContent=preview(result);$('result-status').textContent=`Operation finished · ${result.result||result.status||'see recorded checks'}`;$('download').disabled=false;$('verify-result').disabled=result.schema_version!=='audit-certificate-v1';finish();}
    if(data.type==='error'){showError(data.error);finish();}
  };
  worker.onerror=event=>{showError(event.message||'Worker failed.');finish();ready=false;$('execute').disabled=true;};
  worker.postMessage({type:'initialize',url:new URL('./bisect_wasm.wasm',import.meta.url).href});
}
function finish(){busy=false;updateImportControls();autosave();$('execute').disabled=!ready;$('cancel').disabled=true;}
function showError(message){$('result-status').textContent=message;$('result-status').className='error';}
for(const key of ['plan','context','profile','certificate','package']) $(key+'-file').addEventListener('change',async event=>{
  const expected=++revision;
  try{
    if(busy)throw new Error('Finish or cancel the current run first.');
    const file=event.target.files[0];const value=file?await readJsonFile(file):undefined;
    if(revision!==expected || busy)throw new Error('Project changed while reading the input. Select it again.');
    if(file)files[key]=value;else delete files[key];
    if(key==='profile')renderVraPolicy();
    $('input-status').textContent=`${key}: ${file?.name||'cleared'}`;autosave();
  }catch(error){showError(error.message);}
});
$('execute').onclick=()=>{
  try{
    const operation=$('operation').value;let request={operation};
    if(operation==='validate-rplan'){if(!files.plan)throw new Error('Select a plan.');request.document=files.plan;}
    if(operation==='audit-plan'){
      if(!files.plan||!files.profile)throw new Error('Select a plan and an explicit legal profile.');
      if(!files.plan.plan)throw new Error('For auditing, validate and convert a legacy RPLAN first.');
      Object.assign(request,{plan:files.plan.plan,context:files.context||null,profile:files.profile,constraints:[...document.querySelectorAll('[name=constraint]:checked')].map(input=>input.value),generated_at_utc:new Date().toISOString(),lineage:null});
    }
    if(operation==='verify-certificate'){if(!files.certificate)throw new Error('Select a certificate.');Object.assign(request,{certificate:files.certificate,plan:files.plan?.plan||null,context:files.context||null});}
    if(operation==='verify-count'){if(!files.package)throw new Error('Select a RCOUNT package object.');request.package=files.package;}
    if(['verify-history-files','verify-count-files','replay-count-audits'].includes(operation)){if(!files.packageFiles)throw new Error('Select a complete package folder or open a project containing it.');request.files=decodePackageFiles(files.packageFiles);}
    if(['import-statement-csv','import-nist-cdf-json'].includes(operation)){request.source_base64=validateSourceFile(files.sourceFile).base64;request.metadata=validateImportSettings(metadata());files.importSettings=request.metadata;}
    if(operation==='import-ri2024-rep28-rla'){const sources=validateRiSources(files.riSources);Object.assign(request,{audit_report_base64:sources.auditReport.base64,ballot_manifest_base64:sources.ballotManifest.base64,ballot_retrieval_base64:sources.ballotRetrieval.base64});}
    if(operation==='aggregate-districts'){if(!files.plan||!files.packageFiles)throw new Error('Select a plan and a complete RCOUNT package.');const settings=validateAggregationSettings(aggregationSettings());files.aggregationSettings=settings;Object.assign(request,{...settings,document:files.plan,context:files.context||null,files:decodePackageFiles(files.packageFiles),crosswalk_base64:files.crosswalkFile?validateSourceFile(files.crosswalkFile).base64:null});}
    if(operation==='attach-demographic-csv'){if(!files.plan||!files.context)throw new Error('Select a tract plan and its matching context.');const source=validateSourceFile(files.demographicSource),settings=validateDemographicSettings(demographicSettings());files.demographicSettings=settings;Object.assign(request,{document:files.plan,context:files.context,source_base64:source.base64,...settings});}
    revision++;lastOperation=operation;result=undefined;renderCountResult($('result-tables'),undefined);$('result').textContent='';$('download').disabled=true;$('verify-result').disabled=true;$('result-status').className='';$('result-status').textContent='Running…';busy=true;updateImportControls();$('execute').disabled=true;$('cancel').disabled=false;worker.postMessage({type:'run',id:++id,request});
  }catch(error){showError(error.message);}
};
$('cancel').onclick=()=>{worker.terminate();finish();$('result-status').textContent='Cancelled. No completed result was recorded.';start();};
$('download').onclick=()=>{const url=URL.createObjectURL(new Blob([JSON.stringify(result,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`bisect-${lastOperation}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
$('verify-result').onclick=()=>{files.certificate=result;$('operation').value='verify-certificate';autosave();$('execute').click();};
$('demographic-source').onchange=async event=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');const file=event.target.files[0],source=file?await readElectionSource(file):undefined;if(revision!==expected||busy)throw new Error('Project changed while reading demographics. Select the CSV again.');if(source)files.demographicSource=source;else delete files.demographicSource;files.demographicSettings=validateDemographicSettings(demographicSettings(),{requireComplete:false});$('input-status').textContent='Demographic CSV: '+(source?.name||'cleared');autosave();}catch(error){showError(error.message);}
};
for(const key of ['demographic-basis','demographic-label'])$(key).addEventListener('input',()=>{try{files.demographicSettings=validateDemographicSettings(demographicSettings(),{requireComplete:false});autosave();}catch(error){showError(error.message);}});
$('use-demographic-context').onclick=()=>{
  try{if(busy)throw new Error('Finish or cancel the current run first.');if(result?.schema_version!=='bisect-demographic-context-v1'||!result.context||result.context.context_hash!==result.context_hash)throw new Error('Prepare a demographic context first.');files.context=structuredClone(result.context);$('operation').value='audit-plan';$('input-status').textContent=`Selected ${result.basis} context with ${result.unit_count} aligned tracts. Apply an explicit VRA reporting policy and select the VRA check before auditing. Imported result provenance remains unverified.`;autosave();}catch(error){showError(error.message);}
};
$('apply-vra-policy').onclick=()=>{
  try{if(busy)throw new Error('Finish or cancel the current run first.');if(files.profile?.schema_version!=='legal-profile-v1')throw new Error('Select an explicit legal profile first.');const group=$('vra-group').value.trim(),raw=$('vra-report-threshold').value,threshold=Number(raw)/100;if(!group||new TextEncoder().encode(group).length>200||!raw.trim()||!Number.isFinite(threshold)||threshold<0||threshold>1)throw new Error('Supply a minority group/source definition and threshold from 0 through 100%.');files.profile={...structuredClone(files.profile),vra_policy:{type:'report-opportunity-districts',minority_group:group,vap_threshold:threshold}};$('input-status').textContent='Applied count-based VRA reporting policy to the supplied profile. Select the VRA check and Run to generate its witnesses.';autosave();}catch(error){showError(error.message);}
};
$('download-demographic-context').onclick=()=>{try{if(result?.schema_version!=='bisect-demographic-context-v1'||!result.context)throw new Error('Prepare a demographic context first.');downloadJson(result.context,'bisect-demographic-context.rctx');}catch(error){showError(error.message);}};
$('download-audit-profile').onclick=()=>{try{if(!files.profile)throw new Error('Select an explicit legal profile first.');downloadJson(files.profile,'bisect-audit-profile.json');}catch(error){showError(error.message);}};
function downloadJson(value,name){const url=URL.createObjectURL(new Blob([JSON.stringify(value,null,2)],{type:'application/json'}));const link=document.createElement('a');link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
$('election-source').onchange=async event=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');const file=event.target.files[0],source=file?await readElectionSource(file):undefined;
    if(revision!==expected||busy)throw new Error('Project changed while reading the source. Select it again.');
    if(source)files.sourceFile=source;else delete files.sourceFile;$('input-status').textContent='Election source: '+(source?.name||'cleared');autosave();
  }catch(error){showError(error.message);}
};
for(const key of metadataKeys)$('import-'+key).oninput=()=>{try{files.importSettings=validateImportSettings(metadata(),{requireComplete:false});autosave();}catch(error){showError(error.message);}};
for(const key of ['auditReport','ballotManifest','ballotRetrieval'])$('ri-'+key).onchange=async event=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');const file=event.target.files[0],source=file?await readElectionSource(file):undefined;
    if(revision!==expected||busy)throw new Error('Project changed while reading the RI source. Select it again.');
    const next={...files.riSources};if(source)next[key]=source;else delete next[key];validateRiSources(next,{requireComplete:false});files.riSources=next;$('input-status').textContent=`RI ${key}: ${source?.name||'cleared'}`;autosave();
  }catch(error){showError(error.message);}
};
$('package-archive').onchange=async event=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');const file=event.target.files[0];if(!file)return;const packageFiles=validatePackageArchive(await readJsonFile(file));
    if(revision!==expected||busy)throw new Error('Project changed while reading the archive. Select it again.');
    files.packageFiles=packageFiles;$('operation').value='verify-count-files';$('input-status').textContent=`Loaded ${Object.keys(packageFiles).length} exact package files from archive.`;autosave();
  }catch(error){showError(error.message);}
};
$('use-imported-package').onclick=()=>{try{if(busy)throw new Error('Finish or cancel the current run first.');files.packageFiles=structuredClone(importedPackage());$('operation').value='verify-count-files';$('input-status').textContent='Imported package selected. Run to verify its original sources and recorded equations.';autosave();}catch(error){showError(error.message);}};
$('download-package').onclick=()=>{try{const packageFiles=importedPackage();const url=URL.createObjectURL(new Blob([JSON.stringify({schema_version:'bisect-rcount-files-v1',files:packageFiles})],{type:'application/json'}));const link=document.createElement('a');link.href=url;link.download='bisect-election-package.rcount-files.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}catch(error){showError(error.message);}};
$('crosswalk-file').onchange=async event=>{const expected=++revision;try{if(busy)throw new Error('Finish or cancel the current run first.');const file=event.target.files[0],source=file?await readElectionSource(file):undefined;if(revision!==expected||busy)throw new Error('Project changed while reading the crosswalk. Select it again.');if(source)files.crosswalkFile=source;else delete files.crosswalkFile;autosave();}catch(error){showError(error.message);}};
for(const id of ['aggregation-contest','aggregation-status','aggregation-format'])$(id).addEventListener('input',()=>{try{files.aggregationSettings=validateAggregationSettings(aggregationSettings(),{requireComplete:false});autosave();}catch(error){showError(error.message);}});
$('download-aggregation').onclick=()=>{try{if(result?.schema_version!=='bisect-district-aggregation-v1')throw new Error('Run aggregation first.');validateSourceFile({name:'aggregation.json',base64:result.transcript_base64});const bytes=Uint8Array.from(atob(result.transcript_base64),char=>char.charCodeAt(0));const url=URL.createObjectURL(new Blob([bytes],{type:'application/json'}));const link=document.createElement('a');link.href=url;link.download='bisect-district-aggregation.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}catch(error){showError(error.message);}};
$('demo').onclick=async()=>{
  const expected=++revision, example=Object.create(null);
  try{
    if(busy)throw new Error('Finish or cancel the current run first.');
    for(const [key,name]of Object.entries({plan:'plan.rplan',context:'context.rctx',profile:'profile.json',certificate:'certificate.json'})){const response=await fetch(`./fixtures/${name}`);if(!response.ok)throw new Error('Example unavailable.');example[key]=await response.json();}
    if(revision!==expected || busy)throw new Error('Project changed while loading the example.');
    Object.assign(files,example);renderVraPolicy();$('operation').value='audit-plan';$('input-status').textContent='Loaded public 3×3 grid example: plan, context, profile and certificate.';autosave();
  }catch(error){showError(error.message);}
};
$('package-folder').onchange=async event=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');
    const packageFiles=await readSelectedPackage(event.target.files);
    if(revision!==expected || busy)throw new Error('Project changed while reading the package. Select it again.');
    files.packageFiles=packageFiles;$('input-status').textContent=`Loaded ${Object.keys(packageFiles).length} exact package files.`;autosave();
  }catch(error){showError(error.message);}
};
$('count-demo').onclick=async()=>{
  const expected=++revision;
  try{if(busy)throw new Error('Finish or cancel the current run first.');
    const response=await fetch('./fixtures/count-package.json');if(!response.ok)throw new Error('Election example unavailable.');
    const packageFiles=validatePackageFiles(await response.json());
    if(revision!==expected || busy)throw new Error('Project changed while loading the example.');
    files.packageFiles=packageFiles;$('operation').value='verify-count-files';$('input-status').textContent='Loaded public synthetic election package with exact source bytes.';autosave();
  }catch(error){showError(error.message);}
};
let autosaveTimer, autosaveQueue=Promise.resolve();
function snapshot(){return createProject({name:$('project-name').value,files,operation:$('operation').value,
  constraints:[...document.querySelectorAll('[name=constraint]:checked')].map(input=>input.value),result,lastOperation});}
function autosave(){
  updateInputPanels();
  updateImportControls();
  revision++;
  clearTimeout(autosaveTimer);
  autosaveTimer=setTimeout(()=>{
    let project;
    try { project=snapshot(); } catch(error) { $('project-status').textContent='Autosave failed: '+error.message;return; }
    autosaveQueue=autosaveQueue.catch(()=>{}).then(()=>writeAutosave(project)).then(()=>{
      $('restore-project').disabled=false;$('project-status').textContent='Autosaved on this browser. Save project for a portable backup.';
    }).catch(()=>{$('project-status').textContent='Autosave unavailable. Use Save project to keep your work.';});
  },300);
}
function restore(project){
  if(busy)throw new Error('Finish or cancel the current operation before opening a project.');
  for(const key of Object.keys(files))delete files[key];
  Object.assign(files,project.files);$('project-name').value=project.name;$('operation').value=project.operation;
  for(const input of document.querySelectorAll('[name=constraint]'))input.checked=project.constraints.includes(input.value);
  for(const key of ['plan','context','profile','certificate','package'])$(key+'-file').value='';
  $('package-folder').value='';
  $('election-source').value='';$('package-archive').value='';for(const key of metadataKeys)$('import-'+key).value=files.importSettings?.[key]||'';
  for(const key of ['auditReport','ballotManifest','ballotRetrieval'])$('ri-'+key).value='';
  $('crosswalk-file').value='';$('aggregation-contest').value=files.aggregationSettings?.contest_id||'';$('aggregation-status').value=files.aggregationSettings?.status||'canvassed';$('aggregation-format').value=files.aggregationSettings?.output_format||'pretty-json';
  $('demographic-source').value='';$('demographic-basis').value=files.demographicSettings?.basis||'voting-age-population';$('demographic-label').value=files.demographicSettings?.source_label||'';renderVraPolicy();
  result=project.result??undefined;lastOperation=project.lastOperation??undefined;
  renderCountResult($('result-tables'),result);
  $('result').textContent=result?preview(result):'';
  $('result-status').className='';$('result-status').textContent=result?'Imported result is unverified. Run again or verify its certificate.':'Project loaded. Ready to run.';
  $('download').disabled=!result;$('verify-result').disabled=result?.schema_version!=='audit-certificate-v1';
  updateImportControls();
  $('input-status').textContent='Project inputs: '+(Object.keys(files).join(', ')||'none');autosave();
}
$('save-project').onclick=()=>{
  try{
    const project=snapshot();const url=URL.createObjectURL(new Blob([JSON.stringify(project,null,2)],{type:'application/json'}));
    const link=document.createElement('a');link.href=url;link.download=(project.name.replace(/[^a-z0-9_-]+/gi,'-')||'project')+'.bisect';
    link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
    $('project-status').textContent='Project download requested. Keep this file to reopen or share your work.';
  }catch(error){$('project-status').textContent=error.message;}
};
$('open-project').onchange=async event=>{
  importController?.abort();importController=new AbortController();
  const sequence=++importSequence, expectedRevision=revision;
  try{
    const file=event.target.files[0];if(!file)return;
    if(busy)throw new Error('Finish or cancel the current operation before opening a project.');
    $('project-status').textContent='Validating project…';
    const project=await readProjectFile(file,{signal:importController.signal});
    if(sequence!==importSequence)return;
    if(revision!==expectedRevision || busy)throw new Error('Current project changed during import. Open the file again to replace it.');
    restore(project);$('project-status').textContent='Opened '+file.name;
  }catch(error){if(sequence===importSequence)$('project-status').textContent=error.message;}
  finally{if(sequence===importSequence)event.target.value='';}
};
$('restore-project').onclick=async()=>{
  importController?.abort();
  const sequence=++importSequence, expectedRevision=revision;
  try{
    if(busy)throw new Error('Finish or cancel the current operation before opening a project.');
    const project=await readAutosave();
    if(sequence!==importSequence)return;
    if(revision!==expectedRevision || busy)throw new Error('Current project changed during restore. Restore again to replace it.');
    if(!project)throw new Error('No autosaved project found.');restore(project);
  }catch(error){if(sequence===importSequence)$('project-status').textContent=error.message;}
};
$('project-name').oninput=autosave;$('operation').addEventListener('change',autosave);
for(const input of document.querySelectorAll('[name=constraint]'))input.addEventListener('change',autosave);
readAutosave().then(project=>{if(project){$('restore-project').disabled=false;$('project-status').textContent='An autosaved project is available. Restore autosave to reopen it.';}}).catch(()=>{$('project-status').textContent='Autosave unavailable. Download a project file to keep your work.';});
updateInputPanels();start();

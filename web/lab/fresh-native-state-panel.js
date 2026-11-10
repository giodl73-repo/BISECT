import {freshNativeStateFollowupInputs} from './fresh-native-state-results.js';
import {freshNativeAuxiliaryJson,freshNativeAuxiliarySources,freshNativeAuxiliaryKinds,validateFreshNativeMissingSource} from './fresh-native-state-auxiliary.js';
import {encodeBytes} from './package-files.js';
import {inspectJsonText,parseNativeMapPlanJson,parseGeometryJson} from './project.js';
import {planMapModel,renderPlanMap} from './plan-map.js';
import {validateFreshNativeStateSession,retainFreshNativeStateRun,freshNativeStateRetainedText,freshNativeMultiscaleSamplers,isFreshNativeMultiscaleSampler} from './fresh-native-state-session.js';
import {parseFreshNativeStateRetainedRun} from './fresh-native-state-retained-view.js';

const MAX_INPUT=128*1024*1024;
const sourceKinds=['adjacency','geoids','centroids','coi','partisan','demographics','economic','housing','fine_adjacency','fine_geoids','fine_centroids'];
const {native64,native32}=freshNativeMultiscaleSamplers;
const initialArguments=['--state','RI','--year','2020','--districts','2','--seed','42','--metis-engine','metis-core'];
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
function download(name,text){
 const url=URL.createObjectURL(new Blob([text],{type:'application/json'})),link=document.createElement('a');
 link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
async function readText(file,limit){
 if(!file||file.size>limit)throw Error('Select a file of at most '+Math.floor(limit/1024/1024)+' MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());
 if(bytes.length>limit)throw Error('Selected file exceeds its byte limit.');
 return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}
function savedInput(text){
 if(new TextEncoder().encode(text).length>MAX_INPUT)throw Error('Saved inputs exceed 128 MiB.');
 inspectJsonText(text,4000000);
 const value=JSON.parse(text);
 if(value?.schema_version!=='bisect-fresh-native-state-inputs-v1'||Object.keys(value).some(key=>!['input','schema_version','display_geometry_json','load_c_metis'].includes(key))||!value.input||typeof value.input!=='object'||!Array.isArray(value.input.arguments))throw Error('Select a fresh native State inputs file. Opening inputs never executes a run.');
 const input=value.input;
 if(Object.keys(input).some(key=>!['arguments','source','auxiliary_json','auxiliary_source_name','label','chamber','created_at','multiscale','native_recursive_backend'].includes(key))||!input.source||Object.keys(input.source).some(key=>!['state','year','sources','require_geometry'].includes(key))||!input.source.sources||Object.keys(input.source.sources).some(key=>!['adjacency','geoids','centroids'].includes(key)))throw Error('Saved inputs contain unsupported fields. No plan, context or execution receipt is accepted as input authority.');
 if(input.native_recursive_backend!==undefined&&!['metis-core','c-ffi'].includes(input.native_recursive_backend))throw Error('Select a supported native execution profile.');
 if(input.source.require_geometry===true)throw Error('This panel uses optional raw geometry. Review required-geometry inputs through the request interface.');
 if(value.load_c_metis!==undefined&&typeof value.load_c_metis!=='boolean')throw Error('Invalid saved C runtime choice.');
 if(input.multiscale!=null){
  const m=input.multiscale;if(typeof m!=='object'||Array.isArray(m)||Object.keys(m).some(key=>!['sampler','fine_source'].includes(key))||!isFreshNativeMultiscaleSampler(m.sampler))throw Error('Select a supported native multiscale sampling profile.');
  if(m.fine_source!=null){const f=m.fine_source;if(typeof f!=='object'||Object.keys(f).some(key=>!['state','year','sources','require_geometry'].includes(key))||!f.sources||Object.keys(f.sources).some(key=>!['adjacency','geoids','centroids'].includes(key))||f.require_geometry===true||f.state!==input.source.state||f.year!==input.source.year)throw Error('Saved fine sources must match the primary state/year and supported raw source shape.');}
 }
 if(input.auxiliary_json!=null){
  inspectJsonText(input.auxiliary_json,4000000);freshNativeAuxiliarySources(input.auxiliary_json);
 }
 const displayGeometry=value.display_geometry_json??null;
 if(displayGeometry!==null){if(typeof displayGeometry!=='string'||new TextEncoder().encode(displayGeometry).length>64*1024*1024)throw Error('Saved display geometry must be JSON text of at most 64 MiB.');inspectJsonText(displayGeometry,4000000);JSON.parse(displayGeometry);}
 return {input,displayGeometry,loadC:value.load_c_metis??false};
}

// A dedicated worker is created only on Review, and released on edits/cancel.
// Saved input JSON supplies data only; opening it never constructs a worker.
export function mountFreshNativeStatePanel(anchor){
 const section=document.createElement('section');section.id='fresh-native-state';
 section.innerHTML=`<details><summary>Fresh native State run from local adjacency</summary>
 <p>Select raw native adjacency and optional companions. Review parses inputs without partitioning. Run uses the native State build/retry rules on this device. Multiscale runs require a sampler profile that matches the native run and any requested fine graph. Optional native report observations remain separate.</p>
 <label>Adjacency (.adj.bin)<input data-fresh="adjacency" type="file"></label>
 <label>GEOID companion (optional)<input data-fresh="geoids" type="file" accept=".json"></label>
 <label>Centroid companion (optional)<input data-fresh="centroids" type="file" accept=".json"></label>
 <label>Original COI weights (optional)<input data-fresh="coi" type="file" accept=".json"></label>
 <label>Original partisan shares (optional TSV)<input data-fresh="partisan" type="file"></label>
 <label>Original demographics (optional native CSV)<input data-fresh="demographics" type="file"></label>
 <label>Original economic character (optional native LODES CSV)<input data-fresh="economic" type="file"></label>
 <label>Original housing character (optional native ACS CSV)<input data-fresh="housing" type="file"></label>
 <p>Choose the corresponding native weighting flags. Selected files retain their original bytes; no missing-file or source authenticity claim is inferred.</p>
 <fieldset><legend>Record missing native auxiliary sources</legend>
 <p>Make an explicit claim for this run only. This does not inspect your disk. Native behavior may preserve existing weights, warn, skip loading or reject the run.</p>
 ${freshNativeAuxiliaryKinds.map(kind=>'<label>Missing '+kind+' recorded filename<input data-fresh="'+kind+'-missing-name" maxlength="4096"></label><button data-fresh="'+kind+'-mark-missing" type="button">Use missing '+kind+' claim</button>').join('')}
 </fieldset>
 <label><input data-fresh="multiscale" type="checkbox"> Use native multiscale sampling (match the State search and level flags)</label>
 <label>Native sampler width<select data-fresh="multiscale-sampler"><option value="${native64}">64-bit native</option><option value="${native32}">32-bit native</option></select></label>
 <label>Fine adjacency (required for block-group multiscale)<input data-fresh="fine_adjacency" type="file"></label>
 <label>Fine GEOID companion<input data-fresh="fine_geoids" type="file" accept=".json"></label>
 <label>Fine centroid companion (optional)<input data-fresh="fine_centroids" type="file" accept=".json"></label>
 <label>Native execution profile<select data-fresh="backend-profile"><option value="metis-core">Portable Rust (metis-core)</option><option value="c-ffi">Windows C METIS (msvcrt-v1)</option></select></label>
 <p>Choose the native build to compare. Runs that use recursive METIS follow that build; their backend argument is retained but unused. Prime-factor follows the argument. Runs that do not use METIS need no C runtime. Review shows the resolved backend.</p>
 <label><input data-fresh="load-c" type="checkbox"> Allow loading verified C METIS on explicit Run when needed (requires a C-enabled package)</label>
 <button data-fresh="clear-companions" type="button">Clear optional companions</button>
 <p data-fresh="sources"></p>
 <label>Source state claim<input data-fresh="state" value="RI" maxlength="2"></label>
 <label>Source year claim<input data-fresh="year" value="2020" maxlength="4"></label>
 <label>Native State arguments (JSON array; adjacency, COI and partisan paths are added from selected filenames)<textarea data-fresh="arguments" rows="7" spellcheck="false"></textarea></label>
 <p>Include a seed to reproduce a run. Decimal seeds are kept as strings, including the full unsigned 64-bit range. Paths in arguments identify selected sources; the browser does not open them.</p>
 <label>Export label<input data-fresh="label" value="Fresh native State run" maxlength="200"></label>
 <label>Chamber<input data-fresh="chamber" value="congressional" maxlength="200"></label>
 <label>Export timestamp<input data-fresh="created" maxlength="200"></label>
 <label>Display polygons (optional WGS84 GeoJSON; exact unit IDs)<input data-fresh="geometry" type="file" accept=".json,.geojson"></label>
 <p>Display polygons are used only to draw the completed assignments. They do not supply native report geometry or authenticate the source.</p>
 <button data-fresh="review" type="button">Review selected inputs</button>
 <button data-fresh="run" type="button" disabled>Run reviewed inputs</button>
 <button data-fresh="cancel" type="button" disabled>Cancel</button>
 <button data-fresh="save" type="button" disabled>Save inputs</button>
 <label>Open saved inputs<input data-fresh="open" type="file" accept=".json"></label>
 <p data-fresh="status" role="status">Select sources, then review.</p>
 <p data-fresh="seed"></p><pre data-fresh="review-text"></pre>
 <button data-fresh="plan" type="button" disabled>Download RPLAN</button>
 <button data-fresh="context" type="button" disabled>Download RCTX</button>
 <button data-fresh="report" type="button" disabled>Download run record</button>
 <button data-fresh="audit" type="button" disabled>Use for audit</button>
 <button data-fresh="analyze" type="button" disabled>Use for analysis</button>
 <button data-fresh="improve" type="button" disabled>Use for improvement</button>
 <button data-fresh="view-retained" type="button" disabled>View saved run (unverified)</button>
 <button data-fresh="download-retained" type="button" disabled>Download saved run data</button>
 <div data-fresh="map"></div><pre data-fresh="result"></pre>
 </details>`;
 anchor.after(section);
 const get=name=>section.querySelector('[data-fresh="'+name+'"]');
 get('arguments').value=JSON.stringify(initialArguments,null,2);get('created').value=new Date().toISOString();
 let revision=0,busy=false,worker=null,pending=null,serial=0,reviewed=null,completed=null,sources={},geometry=null;
 let externalBusy=false,lastBusy=false,touched=false,suppress=false,retained=null,retainedArtifact=null;
 let changeListener=null,busyListener=null,handoffListener=null;
 const editable=[...section.querySelectorAll('input,textarea,select')];
 const status=text=>{get('status').textContent=text;};
 function controls(){
  const locked=busy||externalBusy;
  for(const input of editable)input.disabled=locked;
  get('multiscale-sampler').disabled=locked||!get('multiscale').checked;
  get('review').disabled=locked;get('clear-companions').disabled=locked;get('run').disabled=locked||!reviewed;
  get('cancel').disabled=!busy;get('save').disabled=locked||!reviewed;
  for(const name of ['plan','context'])get(name).disabled=locked||!completed?.export;
  get('report').disabled=locked||!completed;
  for(const action of ['audit','analyze','improve'])get(action).disabled=locked||!completed?.export||!reviewed||!handoffListener;
  for(const name of ['view-retained','download-retained'])get(name).disabled=locked||!retained;
  for(const kind of freshNativeAuxiliaryKinds)get(kind+'-mark-missing').disabled=locked;
  get('sources').textContent=sourceKinds.map(kind=>kind+': '+(sources[kind]?.missing===true?'missing claim ('+sources[kind].name+')':sources[kind]?.name??'none')).join(' · ')+' · display polygons: '+(geometry?'available':'none');
  if(lastBusy!==busy){lastBusy=busy;busyListener?.(busy);}
 }
 function snapshot(){
  if(!touched)return null;
  const selected=Object.fromEntries(Object.entries(sources).filter(([,source])=>source!=null));
  const value={schema_version:'bisect-fresh-native-state-session-v1',fields:{arguments_json:get('arguments').value,state:get('state').value,year:get('year').value,label:get('label').value,chamber:get('chamber').value,created_at:get('created').value},sources:structuredClone(selected),display_geometry_json:geometry,retained_run:retained,runtime_artifact:retainedArtifact};
  value.native_recursive_backend=get('backend-profile').value;
  if(get('multiscale').checked)value.multiscale_sampler=get('multiscale-sampler').value;if(get('load-c').checked)value.load_c_metis=true;
  return validateFreshNativeStateSession(value);
 }
 function publish(){
  if(suppress||!changeListener||busy||!touched)return;
  try{changeListener(snapshot());}catch(error){status('Project persistence: '+error.message);}
 }
 function retire(message='Inputs changed. Review again before Run.'){
  revision++;worker?.terminate();worker=null;
  const active=pending;pending=null;active?.reject(Error('Inputs or worker changed. Review again.'));
  busy=false;reviewed=null;completed=null;retained=null;retainedArtifact=null;touched=true;get('review-text').textContent='';get('result').textContent='';get('seed').textContent='';get('map').replaceChildren();status(message);controls();publish();
 }
 function call(message){
  return new Promise((resolve,reject)=>{if(pending)return reject(Error('Finish the current request first.'));pending={resolve,reject,id:message.id??null};worker.postMessage(message);});
 }
 async function initialize(){
  worker=new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'});
  const selectedWorker=worker;
  worker.onmessage=({data})=>{
   if(worker!==selectedWorker)return;
   if(data.type==='progress'){status('Running: '+(data.progress?.stage??'working'));return;}
   if(!pending||data.id!=null&&data.id!==pending.id)return;
   if(!['ready','result','error'].includes(data.type))return;
   const active=pending;pending=null;data.type==='error'?active.reject(Error(data.error)):active.resolve(data);
  };
  worker.onerror=event=>{if(worker!==selectedWorker)return;const active=pending;pending=null;active?.reject(Error(event.message||'Browser worker failed.'));};
  return call({type:'initialize',url:new URL('./bisect_wasm.wasm',import.meta.url).href});
 }
 async function input(){
  if(!sources.adjacency)throw Error('Select native adjacency bytes first.');
  const arguments_=JSON.parse(get('arguments').value);
  if(!Array.isArray(arguments_)||!arguments_.every(value=>typeof value==='string')||arguments_.length>400)throw Error('Native arguments must be a JSON array of at most 400 strings.');
  const args=[...arguments_];
  if(args.some(value=>value==='--adjacency'||value.startsWith('--adjacency=')||value==='--coi-weights'||value.startsWith('--coi-weights=')||value==='--partisan-shares'||value.startsWith('--partisan-shares=')))throw Error('Remove adjacency/COI/partisan path flags; this panel adds the selected filenames.');
  args.push('--adjacency',sources.adjacency.name);
  const value={arguments:args,native_recursive_backend:get('backend-profile').value,source:{state:get('state').value,year:get('year').value,sources:{adjacency:sources.adjacency,geoids:sources.geoids??null,centroids:sources.centroids??null}},label:get('label').value,chamber:get('chamber').value,created_at:get('created').value};
  if(sources.coi)args.push('--coi-weights',sources.coi.name);
  if(sources.partisan)args.push('--partisan-shares',sources.partisan.name);
  const auxiliary=freshNativeAuxiliaryJson(sources);
  if(auxiliary){value.auxiliary_json=auxiliary;value.auxiliary_source_name=Object.keys(sources).some(key=>['partisan','demographics','economic','housing'].includes(key))?'selected-native-auxiliary.json':'selected-native-coi.json';}
  const anyFine=['fine_adjacency','fine_geoids','fine_centroids'].some(key=>sources[key]);
  if(anyFine&&!get('multiscale').checked)throw Error('Enable native multiscale before using fine graph sources.');
  if(get('multiscale').checked)value.multiscale={sampler:get('multiscale-sampler').value,fine_source:anyFine?{state:value.source.state,year:value.source.year,sources:{adjacency:sources.fine_adjacency,geoids:sources.fine_geoids??null,centroids:sources.fine_centroids??null}}:null};
  if(geometry){inspectJsonText(geometry,4000000);JSON.parse(geometry);}
  return value;
 }
 function syncMissingDrafts(){for(const kind of freshNativeAuxiliaryKinds)get(kind+'-missing-name').value=sources[kind]?.missing===true?sources[kind].name:'';}
 for(const kind of freshNativeAuxiliaryKinds){
  get(kind+'-missing-name').oninput=()=>{if(sources[kind]?.missing===true)delete sources[kind];retire();};
  get(kind+'-mark-missing').onclick=()=>{retire();try{const claim=validateFreshNativeMissingSource({name:get(kind+'-missing-name').value,missing:true});sources[kind]=claim;get(kind).value='';status('Recorded missing '+kind+' for this run. Review to see native fallback or refusal.');}catch(error){status(error.message);}controls();publish();};
 }
 for(const name of sourceKinds)get(name).onchange=async()=>{
  const file=get(name).files?.[0];retire();const token=revision;
  if(!file){delete sources[name];controls();publish();return;}
  busy=true;controls();
  try{const limit=(name.endsWith('adjacency')?64:['partisan','demographics','economic','housing'].includes(name)?8:16)*1024*1024;if(file.size>limit)throw Error('Selected '+name+' exceeds its byte limit.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length>limit)throw Error('Selected source exceeds its byte limit.');if(token!==revision)return;sources[name]={name:file.name,base64:encodeBytes(bytes)};status('Selected '+file.name+'. Review before Run.');}
  catch(error){if(token===revision){delete sources[name];status(error.message);}}
  finally{if(token===revision){busy=false;controls();publish();}}
 };
 for(const name of ['state','year','arguments','label','chamber','created'])get(name).oninput=()=>retire();
 for(const name of ['multiscale','multiscale-sampler','load-c','backend-profile'])get(name).onchange=()=>retire();
 get('geometry').onchange=async()=>{const file=get('geometry').files?.[0];retire();const token=revision;geometry=null;if(!file){publish();return;}busy=true;controls();try{const text=await readText(file,64*1024*1024);inspectJsonText(text,4000000);JSON.parse(text);if(token===revision){geometry=text;status('Display geometry selected. Review inputs again.');}}catch(error){if(token===revision)status(error.message);}finally{if(token===revision){busy=false;controls();publish();}}};
 get('clear-companions').onclick=()=>{retire();for(const name of sourceKinds.filter(key=>key!=='adjacency')){delete sources[name];get(name).value='';}geometry=null;get('geometry').value='';syncMissingDrafts();controls();publish();};
 get('cancel').onclick=()=>retire('Cancelled. No current result; review to start again.');
 get('review').onclick=async()=>{
  retire('Reviewing selected inputs…');const token=revision;busy=true;controls();
  try{
   const selected=await input();if(token!==revision)return;
   const ready=await initialize();if(token!==revision)return;
   const reply=await call({type:'run',id:++serial,request:{operation:'prepare-fresh-native-state-run',input:selected}});if(token!==revision)return;
   const result=reply.result;
   if(result.schema_version!=='bisect-fresh-native-state-preparation-v1'||result.execution_performed!==false||result.recorded_paths_opened!==false||!result.input_sha256)throw Error('Unexpected fresh State review response.');
   if(result.native_backend?.schema_version!=='bisect-fresh-native-backend-profile-v1'||result.native_backend.recursive_backend!==selected.native_recursive_backend||JSON.stringify(result.native_backend)!==JSON.stringify(result.prepared.native_backend))throw Error('Review does not bind the selected native execution profile.');
   if(result.native_backend.effective_backend==='c-ffi'&&!get('load-c').checked)throw Error('This reviewed run needs C METIS. Allow loading C on explicit Run, then review again.');
   reviewed={input:selected,sha256:result.input_sha256,artifact:ready.runtimeArtifact,backend:result.native_backend,loadC:get('load-c').checked};
   get('review-text').textContent=JSON.stringify({native_backend:result.native_backend,selected_sources:result.selected_sources,review:result.prepared.prepared_review,engine:ready.runtimeArtifact},null,2);
   const template=JSON.parse(result.prepared.prepared_request_json),seed=(template.request??template.input?.request)?.options?.seed;
   const supplied=result.prepared.native_command?.fields?.some(field=>field.id==='seed'&&field.source==='command-line');
   get('seed').textContent='Resolved seed: '+(seed??'native default')+(seed==null&&supplied?' (the supplied seed was not retained by these State settings)':'');
   status('Reviewed selected bytes. No partition executed. Choose Run to compute.');
  }catch(error){if(token===revision){reviewed=null;worker?.terminate();worker=null;status(error.message);}}
  finally{if(token===revision){busy=false;controls();publish();}}
 };
 get('run').onclick=async()=>{
  if(!reviewed||busy)return;const token=revision,grant=reviewed;busy=true;completed=null;controls();status('Running reviewed inputs…');
  try{
   const reply=await call({type:'run',id:++serial,cMetis:grant.loadC,request:{operation:'run-fresh-native-state',input:grant.input}});if(token!==revision)return;
   const result=reply.result;
   if(result.schema_version!=='bisect-fresh-native-state-run-v1'||result.execution_performed!==true||result.recorded_paths_opened!==false||result.input_sha256!==grant.sha256||reply.runtimeArtifact?.sha256!==grant.artifact.sha256||JSON.stringify(result.native_backend)!==JSON.stringify(grant.backend))throw Error('Run differs from the reviewed input or engine identity.');
   completed=result;
   try{retained=retainFreshNativeStateRun(result);retainedArtifact=grant.artifact;}catch(error){retained=null;retainedArtifact=null;status(error.message);}
   get('seed').textContent='Accepted seed: '+((result.executed_request?.request??result.executed_request)?.options?.seed??'native default');
   get('result').textContent=JSON.stringify({native_backend:result.native_backend,assignments:result.result.assignments,metrics:result.result.metrics,build_report:result.result.build_report,export_error:result.export_error},null,2);
   status(result.export_error?'Run completed; export failed: '+result.export_error:'Run completed. Plan and physical context are ready to download.');
   if(geometry&&result.export){try{renderPlanMap(get('map'),planMapModel(result.export.document,JSON.parse(geometry)),grant.input.label);}catch(error){status('Run completed; display map unavailable: '+error.message);}}
   else get('map').textContent='Select matching display polygons before Review to draw the district map.';
  }catch(error){if(token===revision){reviewed=null;worker?.terminate();worker=null;status(error.message);}}
  finally{if(token===revision){busy=false;controls();publish();}}
 };
 get('save').onclick=()=>{if(reviewed&&!busy){const text=JSON.stringify({schema_version:'bisect-fresh-native-state-inputs-v1',input:reviewed.input,display_geometry_json:geometry,load_c_metis:reviewed.loadC},null,2);if(new TextEncoder().encode(text).length>MAX_INPUT){status('Combined saved inputs exceed 128 MiB. Keep the original source files and download the run record separately.');return;}download('fresh-native-state-inputs.json',text);}};
 get('open').onchange=async()=>{
  const file=get('open').files?.[0];retire('Opening inputs without execution…');const token=revision;if(!file)return;busy=true;controls();
  try{
   const restored=savedInput(await readText(file,MAX_INPUT)),value=restored.input;if(token!==revision)return;
   const args=[];for(let i=0;i<value.arguments.length;i++){const flag=value.arguments[i];if(flag==='--adjacency'||flag==='--coi-weights'||flag==='--partisan-shares'){i++;continue;}if(flag.startsWith('--adjacency=')||flag.startsWith('--coi-weights=')||flag.startsWith('--partisan-shares='))continue;args.push(flag);}
   const next={...value.source.sources};
   const fine=value.multiscale?.fine_source?.sources;
   if(fine)for(const name of ['adjacency','geoids','centroids'])if(fine[name])next['fine_'+name]=fine[name];
   if(value.auxiliary_json)Object.assign(next,freshNativeAuxiliarySources(value.auxiliary_json));
   sources=next;syncMissingDrafts();geometry=restored.displayGeometry;for(const name of [...sourceKinds,'geometry'])get(name).value='';
   get('multiscale').checked=value.multiscale!=null;get('multiscale-sampler').value=value.multiscale?.sampler??native64;get('load-c').checked=restored.loadC;get('backend-profile').value=value.native_recursive_backend??'metis-core';
   for(const [name,text]of [['state',value.source.state],['year',value.source.year],['label',value.label],['chamber',value.chamber],['created',value.created_at]]){if(typeof text!=='string')throw Error('Saved input metadata must be text.');get(name).value=text;}
   get('arguments').value=JSON.stringify(args,null,2);status('Inputs opened. Nothing executed. Review selected bytes before Run.');
  }catch(error){if(token===revision){sources={};status(error.message);}}
  finally{if(token===revision){busy=false;controls();publish();}}
 };
 get('plan').onclick=()=>{if(completed?.export)download('fresh-native-state.rplan',completed.export.document_json);};
 get('context').onclick=()=>{if(completed?.export)download('fresh-native-state.rctx',completed.export.context_json);};
 get('report').onclick=async()=>{if(completed&&reviewed){const input=reviewed.input,result=completed,display=geometry,artifact=reviewed.artifact;download('fresh-native-state-run.json',JSON.stringify({schema_version:'bisect-fresh-native-state-run-record-v1',input,result,engine:artifact,display_geometry_json:display,display_geometry_sha256:display===null?null:await hash(new TextEncoder().encode(display)),input_json_sha256:await hash(new TextEncoder().encode(JSON.stringify(input)))},null,2));}};
 for(const action of ['audit','analyze','improve'])get(action).onclick=()=>{
  try{
   if(busy||externalBusy||!reviewed||!completed||!handoffListener)throw Error('Complete a current Review and Run before selecting a follow-up.');
   if(completed.input_sha256!==reviewed.sha256)throw Error('Run no longer matches the reviewed inputs.');
   const inputs=freshNativeStateFollowupInputs(completed,action);
   handoffListener({action,inputs});
   status('Completed native plan and context selected for '+action+'. Execute the selected action explicitly; no follow-up ran during this handoff.');
  }catch(error){status('Follow-up selection: '+error.message);}
 };
 get('download-retained').onclick=()=>{if(retained)download('fresh-native-state-retained-run.json',freshNativeStateRetainedText(retained));};
 get('view-retained').onclick=()=>{
  try{
   if(!retained||busy||externalBusy)return;
   const value=parseFreshNativeStateRetainedRun(retained);
   if(value?.schema_version!=='bisect-fresh-native-state-run-v1')throw Error('Saved data is not a fresh native State run.');
   get('result').textContent=JSON.stringify({retained_unverified:true,native_backend:value.native_backend??null,result:value.result,export_error:value.export_error},null,2);
   get('seed').textContent='Saved, unverified seed: '+((value.executed_request?.request??value.executed_request)?.options?.seed??'native default');
   if(geometry&&typeof value.export?.document_json==='string')renderPlanMap(get('map'),planMapModel(parseNativeMapPlanJson(value.export.document_json),parseGeometryJson(geometry)),'Saved run (unverified)');
   status('Viewing saved data only. No execution or verification performed. Review and Run to produce current outputs.');
  }catch(error){status('Saved run view: '+error.message);}
 };
 controls();return {
  section,snapshot,isBusy:()=>busy,
  setExternalBusy(value){externalBusy=Boolean(value);controls();},
  subscribe({onChange,onBusy,onHandoff}){changeListener=onChange;busyListener=onBusy;handoffListener=onHandoff;controls();},
  restore(value){
   const next=value===undefined||value===null?null:validateFreshNativeStateSession(structuredClone(value));
   suppress=true;
   try{
    retire('Saved inputs restored. Review before Run.');
    sources=next?.sources??{};syncMissingDrafts();geometry=next?.display_geometry_json??null;retained=next?.retained_run??null;retainedArtifact=next?.runtime_artifact??null;touched=next!==null;
    const defaults={arguments_json:JSON.stringify(initialArguments,null,2),state:'RI',year:'2020',label:'Fresh native State run',chamber:'congressional',created_at:new Date().toISOString()};
    for(const [name,key]of [['arguments','arguments_json'],['state','state'],['year','year'],['label','label'],['chamber','chamber'],['created','created_at']])get(name).value=next?.fields[key]??defaults[key];
    for(const name of [...sourceKinds,'geometry','open'])get(name).value='';
    get('multiscale').checked=next?.multiscale_sampler!==undefined;get('multiscale-sampler').value=next?.multiscale_sampler??native64;get('load-c').checked=next?.load_c_metis===true;get('backend-profile').value=next?.native_recursive_backend??'metis-core';
    status(next?'Project inputs restored without execution. '+(retained?'Saved run data is unverified; use View saved run to inspect it.':'Review before Run.'):'Select sources, then review.');
    controls();
   }finally{suppress=false;}
  },
  destroy(){retire('Panel closed.');section.remove();}
 };
}

const anchor=typeof document==='undefined'?null:document.getElementById('practitioner-flow');
export const freshNativeStatePanel=anchor?mountFreshNativeStatePanel(anchor):null;

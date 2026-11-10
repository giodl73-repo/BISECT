import {encodeStoppedTreeProofSession} from './native-tree-generate.js';
import {nativeModelPreparationZip} from './native-model-prepare.js';
import {MAX_TREE_BUNDLE_BYTES} from './native-tree-bundle.js';
import {nativeTreePlanZip} from './native-tree-plan.js';
import {nativeMapDownload} from './map-export-files.js';
const get = id => document.getElementById(id);
get('created').value = new Date().toISOString();
let stoppedGeneration = null;
let bundle = null, tree = null, worker = null, timer = null, revision = 0;
let contextJson = null, profileJson = null, planResult = null;
let geometryJson = null, mapResult = null, imageUrl = null, geometryRead = 0;
let rootInstanceJson = null, landAdjacency = null; const generationReads = {root:0,land:0};
let landInvalid = false, modelContextJson = null, preparedModel = null, modelRead = 0;
function clearMap() { mapResult = null; if (imageUrl) URL.revokeObjectURL(imageUrl); imageUrl = null; get('map-image').removeAttribute('src'); get('map-image').hidden = true; get('save-png').disabled = get('save-svg').disabled = true; }
const readTokens = {context: 0, profile: 0};
function planControls() { get('prepare-model').disabled = !modelContextJson || !!worker; get('save-model').disabled = !preparedModel || !!worker; get('export-plan').disabled = !(tree && contextJson && profileJson) || !!worker; get('render-map').disabled = !(planResult && geometryJson) || !!worker; get('generate').disabled = !rootInstanceJson || landInvalid || !!worker; }
function stop() { worker?.terminate(); worker = null; clearTimeout(timer); timer = null; get('cancel').disabled = true; get('verify').disabled = !bundle; planControls(); }
function invalidate() { stoppedGeneration=null;get('save-stopped').disabled=true;revision++; stop(); tree = null; planResult = null; clearMap(); get('render-map').disabled = get('export-plan').disabled = get('save-plan').disabled = get('save').disabled = get('save-bundle').disabled = true; get('schedule').replaceChildren(); get('leaves').replaceChildren(); get('details').textContent = ''; }
function execute(message, transfer = []) {
  const current = revision; worker = new Worker(new URL('./native-tree-worker.js', import.meta.url), {type: 'module'});
  get('generate').disabled = get('verify').disabled = get('export-plan').disabled = get('render-map').disabled = true; get('cancel').disabled = false;
  timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Watchdog stopped the worker. No verified tree.'; }, message.mode === 'open' ? 30000 : 60000);
  worker.onmessage = async ({data}) => {
    if(current!==revision)return;
    if(data.progress){const progress=data.progress;clearTimeout(timer);timer=setTimeout(()=>{if(current!==revision)return;invalidate();get('status').textContent='Generation watchdog stopped the worker. No complete tree.';},progress.phase==='solving'?(Number(get('seconds').value)+30)*1000:60000);get('status').textContent=`${progress.phase} at ${progress.node_path||'(root)'} · ${progress.completed_nodes}/${progress.total_nodes} proved nodes · seed ${progress.seed}`;return;}
    if (current !== revision) return; stop();
    if (data.error) { get('status').textContent = 'Tree refused: ' + data.error; return; }
    if(data.prepared){
      preparedModel=data.prepared;rootInstanceJson=data.prepared.root_instance_json;landAdjacency=data.prepared.land_adjacency;landInvalid=false;generationReads.root++;generationReads.land++;get('root').value=get('land').value='';
      contextJson=data.prepared.model_context_json;readTokens.context++;get('context').value='';planControls();
      get('model-summary').textContent=`Prepared ${data.prepared.districts}-seat model: ${data.prepared.weight_policy}. Original context retained separately. Model context will be used for plan export.`;
      get('status').textContent='Native model prepared. No discovery or proof has run. Press Discover, prove and generate tree.';get('details').textContent=data.prepared.scope;return;
    }
    if (data.map) {
      try { const output = await nativeMapDownload(data.map,'png'); if (current !== revision) return; mapResult = data.map; imageUrl = URL.createObjectURL(new Blob([output.bytes],{type:output.type})); get('map-image').src = imageUrl; get('map-image').hidden = false; get('save-png').disabled = get('save-svg').disabled = false; get('status').textContent = 'Native map rendered from the exported plan and supplied geometry.'; }
      catch (error) { if (current === revision) get('status').textContent = error.message; } return;
    }
    if (data.plan) { planResult = data.plan; get('status').textContent = 'Native plan exported and selected audit completed.'; get('details').textContent = data.plan.audit_certificate_json; get('save-plan').disabled = false; planControls(); return; }
    if (data.bundle) { bundle = data.bundle; stop(); get('status').textContent = 'Imported tree proof bundle is unverified. Press Verify all tree proofs.'; return; }
    if(data.generated){if(data.generated.status!=='tree-generated-and-checked'){stoppedGeneration=data.generated;get('save-stopped').disabled=false;get('status').textContent=`Generation stopped at ${data.generated.stopped_node_path||'(root)'}: ${data.generated.stopped_run?.decisions.at(-1)?.solver_status||data.generated.stopped_run?.status||'incomplete'}. No complete tree certificate.`;get('details').textContent=JSON.stringify({seed:data.generated.seed,refinement:data.generated.refinement,solver_seconds_per_decision:data.generated.solver_seconds_per_decision,runs:data.generated.runs,stopped_decision:data.generated.stopped_run?.decisions.at(-1)},null,2);return;}bundle=data.generated.bundle;data.tree=data.generated.tree;stop();}
    tree = data.tree; get('status').textContent = `Verified ${tree.k}-seat tree with ${tree.nodes.length} proved splits and ${tree.leaves.length} leaves.${data.generated ? ` Seed ${data.generated.seed} · ${data.generated.refinement}.` : ''}`;
    const table = (target, headings, rows) => { const element = document.createElement('table'), header = element.insertRow(); for (const text of headings) { const cell = document.createElement('th'); cell.textContent = text; header.append(cell); } for (const row of rows) { const tr = element.insertRow(); for (const value of row) tr.insertCell().textContent = String(value); } get(target).append(element); };
    table('schedule', ['Node path', 'Native split certificate'], tree.nodes.map(node => [node.node_path || '(root)', node.certificate_id]));
    table('leaves', ['District', 'Leaf path', 'Units'], tree.leaves.map(leaf => [leaf.district_index + 1, leaf.node_path, leaf.unit_ids.join(', ')]));
    get('details').textContent = JSON.stringify({tree_id: tree.tree_id, assignments: tree.assignments, ...(data.generated ? {generation: {seed:data.generated.seed, refinement:data.generated.refinement, runs:data.generated.runs}} : {})}, null, 2); get('save').disabled = get('save-bundle').disabled = false;
    planControls();
  };
  worker.onerror = event => { if (current !== revision) return; invalidate(); get('status').textContent = event.message || 'Tree worker failed.'; };
  worker.postMessage(message, transfer);
}
get('open').onchange = async () => {
  invalidate(); bundle = null; stop(); const file = get('open').files[0], current = revision; if (!file) return;
  get('status').textContent = 'Reading tree proof bundle…';
  try { if (file.size > MAX_TREE_BUNDLE_BYTES) throw new Error('Tree proof bundle exceeds 192 MiB.'); const bytes = await file.arrayBuffer(); if (current !== revision) return; execute({mode: 'open', bytes}, [bytes]); }
  catch (error) { if (current === revision) get('status').textContent = error.message; }
};
get('verify').onclick = () => { if (!bundle || worker) return; invalidate(); get('status').textContent = 'Native Rust is checking every proof and parent/child relationship…'; execute({mode: 'verify', bundle}); };
get('cancel').onclick = () => { invalidate(); get('status').textContent = 'Cancelled. No verified tree retained.'; };
for(const kind of ['root','land'])get(kind).onchange=async()=>{preparedModel=null;get('model-summary').textContent='';bundle=null;get('open').value='';invalidate();const token=++generationReads[kind];if(kind==='root')rootInstanceJson=null;else {landAdjacency=null;landInvalid=!!get(kind).files[0];}planControls();const file=get(kind).files[0];if(!file)return;try{if(file.size>3*1024*1024)throw new Error('Generation source exceeds 3 MiB.');const text=new TextDecoder('utf-8',{fatal:true}).decode(await file.arrayBuffer());if(token!==generationReads[kind])return;if(kind==='root')rootInstanceJson=text;else {landAdjacency=JSON.parse(text);landInvalid=false;}planControls();get('status').textContent='Generation source selected. No discovery or proof computation has run.';}catch(error){if(token===generationReads[kind])get('status').textContent=error.message;}};
for(const name of ['seed','refinement','seconds'])get(name).oninput=()=>{bundle=null;get('open').value='';invalidate();planControls();};
get('generate').onclick=()=>{if(!rootInstanceJson||landInvalid||worker)return;bundle=null;invalidate();get('status').textContent='Starting native recursive discovery and proof generation…';execute({mode:'generate',rootInstanceJson,landAdjacency,seed:get('seed').value,refinement:get('refinement').value,seconds:Number(get('seconds').value)});};
for (const kind of ['context', 'profile']) get(kind).onchange = async () => {
  revision++; stop(); planResult = null; clearMap(); get('save-plan').disabled = true;
  if (kind === 'context') contextJson = null; else profileJson = null; planControls();
  const token = ++readTokens[kind], file = get(kind).files[0]; if (!file) return;
  try { if (file.size > (kind === 'context' ? 8 : 3) * 1024 * 1024) throw new Error('Plan source exceeds its size limit.'); const text = new TextDecoder('utf-8', {fatal: true}).decode(await file.arrayBuffer()); if (token !== readTokens[kind]) return; if (kind === 'context') contextJson = text; else profileJson = text; planControls(); get('status').textContent = 'Plan source loaded. Export and audit requires an explicit run.'; }
  catch (error) { if (token === readTokens[kind]) get('status').textContent = error.message; }
};
for (const name of ['label', 'created']) get(name).oninput = () => { revision++; stop(); planResult = null; clearMap(); planControls(); get('save-plan').disabled = true; };
get('export-plan').onclick = () => { if (!tree || !bundle || !contextJson || !profileJson || worker) return; revision++; planResult = null; clearMap(); get('save-plan').disabled = true; get('status').textContent = 'Rechecking tree proofs, original context and selected audit…'; execute({mode: 'export-plan', bundle, tree, contextJson, profileJson, label: get('label').value, createdAt: get('created').value}); };
get('geometry').onchange = async () => { revision++; stop(); clearMap(); geometryJson = null; planControls(); const token = ++geometryRead, file = get('geometry').files[0]; if (!file) return; try { if (file.size > 64 * 1024 * 1024) throw new Error('Geometry exceeds 64 MiB.'); const text = new TextDecoder('utf-8',{fatal:true}).decode(await file.arrayBuffer()); if (token !== geometryRead) return; geometryJson = text; planControls(); get('status').textContent = 'Geometry selected. Press Render exported plan map to execute.'; } catch(error) { if (token === geometryRead) get('status').textContent = error.message; } };
get('dpi').onchange = () => { revision++; stop(); clearMap(); planControls(); };
get('render-map').onclick = () => { if (!planResult || !geometryJson || worker) return; revision++; clearMap(); get('status').textContent = 'Rendering the native district map…'; execute({mode:'map',documentJson:planResult.document_json,geometryJson,dpi:Number(get('dpi').value)}); };
function download(name, value) { const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2) + '\n'], {type: 'application/json'})), link = document.createElement('a'); link.href = url; link.download = name; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); }
get('save').onclick = () => { if (tree && !worker) download('native-external-tree.json', tree); };
get('save-bundle').onclick = () => { if (tree && bundle && !worker) download('native-tree-proof-bundle.json', {...bundle, expected_tree: tree}); };
get('save-plan').onclick = async () => { if (!planResult || worker) return; const selected = planResult, current = revision; try { const zip = await nativeTreePlanZip(selected); if (current !== revision || planResult !== selected || worker) return; const url = URL.createObjectURL(new Blob([zip], {type: 'application/zip'})), link = document.createElement('a'); link.href = url; link.download = 'external-tree-plan.zip'; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); } catch (error) { if (current === revision) get('status').textContent = error.message; } };
for (const kind of ['png','svg']) get('save-'+kind).onclick = async () => { if (!mapResult || worker) return; const selected = mapResult, current = revision; try { const output = await nativeMapDownload(selected,kind); if (current !== revision || mapResult !== selected || worker) return; const url = URL.createObjectURL(new Blob([output.bytes],{type:output.type})), link = document.createElement('a'); link.href = url; link.download = output.name; link.click(); setTimeout(() => URL.revokeObjectURL(url),1000); } catch(error) { if (current === revision) get('status').textContent = error.message; } };

function retirePreparedSource(){preparedModel=null;rootInstanceJson=null;landAdjacency=null;landInvalid=false;generationReads.root++;generationReads.land++;get('root').value=get('land').value=get('open').value='';bundle=null;get('model-summary').textContent='';invalidate();planControls();}
get('model-source').onchange=async()=>{retirePreparedSource();modelContextJson=null;const token=++modelRead;planControls();const file=get('model-source').files[0];if(!file)return;try{if(file.size>8*1024*1024)throw new Error('Model context exceeds 8 MiB.');const text=new TextDecoder('utf-8',{fatal:true}).decode(await file.arrayBuffer());if(token!==modelRead)return;modelContextJson=text;planControls();get('status').textContent='Model context selected. Preparation requires an explicit action.';}catch(error){if(token===modelRead)get('status').textContent=error.message;}};
for(const name of ['model-seats','model-policy'])get(name).oninput=retirePreparedSource;
get('prepare-model').onclick=()=>{if(!modelContextJson||worker)return;retirePreparedSource();get('status').textContent='Preparing the selected native integer model…';execute({mode:'prepare-model',contextJson:modelContextJson,districts:Number(get('model-seats').value),weightPolicy:get('model-policy').value});};
get('save-model').onclick=async()=>{if(!preparedModel||worker)return;const selected=preparedModel,current=revision;try{const bytes=await nativeModelPreparationZip(selected);if(current!==revision||preparedModel!==selected||worker)return;const url=URL.createObjectURL(new Blob([bytes],{type:'application/zip'})),link=document.createElement('a');link.href=url;link.download='native-model-preparation.zip';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}catch(error){if(current===revision)get('status').textContent=error.message;}};

get('save-stopped').onclick=()=>{if(!stoppedGeneration||worker)return;try{const bytes=encodeStoppedTreeProofSession(stoppedGeneration),url=URL.createObjectURL(new Blob([bytes],{type:'application/json'})),link=document.createElement('a');link.href=url;link.download='stopped-split.native-proof.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}catch(error){get('status').textContent=error.message;}};

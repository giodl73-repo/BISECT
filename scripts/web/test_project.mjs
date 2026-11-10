import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const {createProject,parseProject,validateProject,readProjectFile,parseSafeJson,readFileTextStrict,MAX_PROJECT_BYTES}=await import('../../web/lab/project.js');
const {resultPreview,MAX_PREVIEW_CHARS}=await import('../../web/lab/result-preview.js');
const {startLabelAnalysisYearsCheckpoint}=await import('../../web/lab/label-analysis-files.js');
const {startLabelReportYearsCheckpoint}=await import('../../web/lab/label-report-files.js');
const plan=JSON.parse(await readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
const context=JSON.parse(await readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'));
const certificate=JSON.parse(await readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
const saved=createProject({name:'County review',files:{plan,context,certificate},operation:'audit-plan',constraints:['population','contiguity'],result:certificate,lastOperation:'audit-plan'});
assert.deepEqual(parseProject(JSON.stringify(saved)),saved);
plan.extra='changed';assert.equal(saved.files.plan.extra,undefined);
for(const change of [{schema_version:'unknown'},{operation:'execute-code'},{constraints:['unknown']},{files:{__bad:{}}},{lastOperation:null}])assert.throws(()=>parseProject(JSON.stringify({...saved,...change})));
assert.throws(()=>parseProject('{broken'));
console.log('Project snapshot: fixture roundtrip, immutable snapshot, invalid format rejection passed.');

const encode = overrides => JSON.stringify({...saved,...overrides});
assert.throws(()=>parseProject(encode({unexpected_url:'https://attacker.invalid/'})),/fields/);
assert.throws(()=>parseProject(encode({saved_at_utc:'invalid'})),/date/);
assert.throws(()=>parseProject(encode({constraints:['population','population']})),/checks/);
assert.throws(()=>parseProject(' '.repeat(MAX_PROJECT_BYTES+1)),/25 MB/);
assert.throws(()=>parseProject(encode({files:{plan:{payload:'é'.repeat(MAX_PROJECT_BYTES/2)}}})),/25 MB/);
let nested={};for(let i=0;i<65;i++)nested={child:nested};
assert.throws(()=>parseProject(encode({files:{plan:nested}})),/complex/);
assert.throws(()=>parseProject(encode({files:{plan:{values:Array(500001).fill(1)}}})),/many|complex/);
for(const key of ['__proto__','constructor','prototype']) {
  const payload=JSON.parse('{"'+key+'":{"polluted":true}}');
  assert.throws(()=>parseProject(encode({files:{plan:payload}})),/Unsafe/);
}
assert.equal({}.polluted,undefined);
for(const literal of ['9007199254740992','9007199254740993','-9007199254740993','9223372036854775807','18446744073709551615']) {
  assert.throws(()=>parseSafeJson('{"count":'+literal+'}'),/exact JavaScript range/);
  assert.throws(()=>parseProject(encode({files:{plan:{count:0}}}).replace('"count":0','"count":'+literal)),/exact JavaScript range/);
}
assert.deepEqual(parseSafeJson('{"count":9007199254740991,"large_exact_string":"18446744073709551615"}'),{count:Number.MAX_SAFE_INTEGER,large_exact_string:'18446744073709551615'});
assert.throws(()=>parseProject(encode({files:{plan:{value:1e400}}}).replace('"value":null','"value":1e400')),/number/);
const accessor={...saved};Object.defineProperty(accessor,'name',{enumerable:true,get(){throw new Error('getter executed');}});
assert.throws(()=>validateProject(accessor),/accessor/);
const cyclic={...saved};cyclic.result=cyclic;assert.throws(()=>validateProject(cyclic),/cyclic/);
const hostileText='<img src=https://attacker.invalid/ onerror=alert(1)><script>alert(1)</script>';
const inert=parseProject(encode({name:hostileText,result:{html:hostileText,url:'https://attacker.invalid/'}}));
assert.equal(inert.result.html,hostileText); // Data remains text; UI assertions below guard rendering.
const ui=await readFile('web/lab/workbench.js','utf8');
assert.ok(!/innerHTML|outerHTML|insertAdjacentHTML|eval\s*\(|new Function|document\.write/.test(ui));
assert.ok(ui.includes("$('result').textContent="));
assert.ok(ui.includes('revision!==expectedRevision || busy'));
assert.ok(ui.includes('Imported result is unverified'));
console.log('Project security: size, UTF-8 bytes, depth, value count, prototype keys, nonfinite numbers, accessors, cycles and untrusted-result labeling passed.');

// Rejection must precede full object-tree allocation, not merely happen after it.
const hostileJson=[
  '['.repeat(20000)+'0'+']'.repeat(20000),
  '['+Array(500000).fill('0').join(',')+']',
  '{"key":1,"k\\u0065y":2}',
  '{"__proto__":{"polluted":true}}',
  '{"files":{"plan":{"constr\\u0075ctor":{"prototype":{}}}}}',
  '{"files":{"plan":{"proto\\u0074ype":{}}}}',
];
const nativeJsonParse=JSON.parse;
try {
  for(const payload of hostileJson){
    let parsedTree=false;
    JSON.parse=function(text,...args){if(text===payload)parsedTree=true;return nativeJsonParse(text,...args);};
    assert.throws(()=>parseSafeJson(payload),/complex|Duplicate|Unsafe/);
    assert.equal(parsedTree,false,'hostile tree was allocated before rejection');
  }
} finally { JSON.parse=nativeJsonParse; }
const maximumDepth='['.repeat(64)+'0'+']'.repeat(64);
assert.deepEqual(parseSafeJson(maximumDepth),nativeJsonParse(maximumDepth));
for(const payload of ['null','true','false','-1.25e+2','{"text":"[{}] \\"quoted\\" : ,","empty":[],"other":{}}']) {
  assert.deepEqual(parseSafeJson(payload),nativeJsonParse(payload));
}
for(const payload of ['["key":0]','{"key":}','{"key":1,}','[1 2]','"unterminated'])assert.throws(()=>parseSafeJson(payload));
console.log('Pre-parse security: deep trees, excessive values and escaped duplicate keys rejected before object allocation; valid depth boundary and JSON syntax checks passed.');

const originalWorker=globalThis.Worker, originalTimeout=globalThis.setTimeout, originalClear=globalThis.clearTimeout;
let activeWorker, timeout;
try {
  globalThis.Worker=class {
    constructor(url){assert.ok(url.pathname.endsWith('/project-worker.js'));activeWorker=this;this.terminated=false;}
    postMessage(file){this.file=file;}
    terminate(){this.terminated=true;}
  };
  globalThis.setTimeout=callback=>{timeout=callback;return 1;};
  globalThis.clearTimeout=()=>{};
  await assert.rejects(readProjectFile({size:MAX_PROJECT_BYTES+1}),/25 MB/);
  assert.equal(activeWorker,undefined);
  const accepted=readProjectFile({size:100});activeWorker.onmessage({data:{ok:true,project:saved}});
  assert.deepEqual(await accepted,saved);assert.equal(activeWorker.terminated,true);
  const rejected=readProjectFile({size:100});activeWorker.onmessage({data:{ok:false,error:'Unsafe project property.'}});
  await assert.rejects(rejected,/Unsafe/);assert.equal(activeWorker.terminated,true);
  const expired=readProjectFile({size:100});timeout();
  await assert.rejects(expired,/timed out/);assert.equal(activeWorker.terminated,true);
  const failed=readProjectFile({size:100});activeWorker.onerror();
  await assert.rejects(failed,/failed/);assert.equal(activeWorker.terminated,true);
  const controller=new AbortController();
  const cancelled=readProjectFile({size:100},{signal:controller.signal});controller.abort();
  await assert.rejects(cancelled,/cancelled/);assert.equal(activeWorker.terminated,true);
  activeWorker=undefined;
  await assert.rejects(readProjectFile({size:100},{signal:controller.signal}),/cancelled/);
  assert.equal(activeWorker,undefined);
  const unreadable=readProjectFile({size:100});activeWorker.onmessageerror();
  await assert.rejects(unreadable,/failed/);assert.equal(activeWorker.terminated,true);
  for(const data of [null,undefined,{}, {ok:'yes',project:saved}, {ok:true}, {ok:false,error:{toString:'hostile'}}]){
    const malformed=readProjectFile({size:100});activeWorker.onmessage({data});
    await assert.rejects(malformed,/failed/);assert.equal(activeWorker.terminated,true);
  }
  globalThis.Worker=class {postMessage(){throw new Error('Cannot clone');}terminate(){this.terminated=true;activeWorker=this;}};
  await assert.rejects(readProjectFile({size:100}),/failed/);assert.equal(activeWorker.terminated,true);
} finally {
  globalThis.Worker=originalWorker;globalThis.setTimeout=originalTimeout;globalThis.clearTimeout=originalClear;
}
console.log('Import worker: pre-read size rejection, completion, rejection, timeout and cleanup passed.');

// Exercise real UI handlers with delayed reads: stale completions must not commit.
const {runInNewContext}=await import('node:vm');
const nodes=new Map();
function node(id){
  if(!nodes.has(id))nodes.set(id,{value:id==='operation'?'audit-plan':id==='project-name'?'Current project':'',files:[],options:[],selectedOptions:[],disabled:false,textContent:'',parentElement:{hidden:false},append(){},after(){},removeAttribute(name){delete this[name];},replaceChildren(...children){this.children=children;this.value=children[0]?.value||'';},addEventListener(type,handler){this[type]=handler;}});
  return nodes.get(id);
}
let restoreRead=()=>Promise.resolve(null), importRead=()=>Promise.resolve(saved);
let lastSnapshot;
let uiWorker;const posted=[];
const packageTools=await import('../../web/lab/package-files.js');
const doctorTools=await import('../../web/lab/doctor-files.js');
const manifestTools=await import('../../web/lab/manifest-files.js');
const labelImportTools=await import('../../web/lab/label-import-files.js');
const planHubTools=await import('../../web/lab/plan-hub-files.js');
const labelInventoryTools=await import('../../web/lab/label-inventory-files.js');
const labelRenameTools=await import('../../web/lab/label-rename-files.js');
const labelBuildTools=await import('../../web/lab/label-build-files.js');
const labelChainTools=await import('../../web/lab/label-chain-files.js');
const migrationTools=await import('../../web/lab/migration-files.js');
const candidateTools=await import('../../web/lab/candidate-files.js');
const civicTools=await import('../../web/lab/civic-files.js');
const civicSnapshotTools=await import('../../web/lab/civic-snapshot-files.js');
const workerFailureTools=await import('../../web/lab/worker-failure.js');
const suiteAssemblyTools=await import('../../web/lab/suite-assembly-files.js');
const planDoctorTools=await import('../../web/lab/plan-doctor-files.js');
const tutorialTools=await import('../../web/lab/tutorial-files.js');
const acquisitionTools=await import('../../web/lab/fetch-acquisition.js');let acquisitionRead=()=>{throw new Error('Unexpected public request');};
const workbenchArchiveTools=await import('../../web/lab/workbench-archive.js');
const sandbox={
  ...(await import('../../web/lab/native-state-workflow.js')),
  ...(await import('../../web/lab/native-selected-state-source.js')),
  ...(await import('../../web/lab/stream-native-run-years.js')),
  ...(await import('../../web/lab/national-map-directory.js')),
  ...(await import('../../web/lab/native-round-progress.js')),
  ...(await import('../../web/lab/state-command-files.js')),
  ...(await import('../../web/lab/state-graph-files.js')),
  ...(await import('../../web/lab/native-state-batch-output.js')),
  ...(await import('../../web/lab/native-state-output.js')),
  ...(await import('../../web/lab/native-state-output-ui.js')),
  ...(await import('../../web/lab/native-state-job.js')),
  ...(await import('../../web/lab/native-states-job.js')),
  ...(await import('../../web/lab/native-state-followup.js')),
  ...(await import('../../web/lab/native-state-sources.js')),
  ...(await import('../../web/lab/migration-base-files.js')),
  ...(await import('../../web/lab/native-plan-import-files.js')),
  ...(await import('../../web/lab/export-folder-files.js')),
  ...(await import('../../web/lab/native-character-producer.js')),
  ...(await import('../../web/lab/native-folder-files.js')),
  ...(await import('../../web/lab/commission-preview.js')),
  ...(await import('../../web/lab/generated-report-files.js')),
  ...(await import('../../web/lab/chamber-count-files.js')),
  ...(await import('../../web/lab/prepared-native-job.js')),
  ...(await import('../../web/lab/prepared-graph-files.js')),
  ...(await import('../../web/lab/prepared-census-job.js')),
  ...(await import('../../web/lab/prepared-state-job.js')),
  ...(await import('../../web/lab/prepared-state-output.js')),
  ...(await import('../../web/lab/state-source-job.js')),
  ...(await import('../../web/lab/prepared-engine-results.js')),
  ...(await import('../../web/lab/prepared-engine-files.js')),
  ...(await import('../../web/lab/native-character-engine-files.js')),
  ...(await import('../../web/lab/native-character-weight-files.js')),
  ...(await import('../../web/lab/project-handoff.js')),
  // Navigation has a separate real-browser gate; this fixture tests project handlers.
  // Practitioner/target-action composition has real-browser gates. This VM
  // deliberately exercises project import handlers with a minimal DOM only.
  installPractitionerFlow(){return {refresh(){}};},
  installNativeBulkWorkspaceControls(){return {update(){},preserveSavedRevision(){}};},
  installNativeStateBeforeRunControls(){return {update(){},receive(){return false;},preserveSavedRevision(){}};},
  ...(await import('../../web/lab/native-output-verification.js')),
  ...(await import('../../web/lab/state-source-acquisition-recovery.js')),
  installAdjacencyReleaseControls(){return {update(){}};},
  installElectionSourceFormatControls(){return {update(){}};},
  installWorkbenchNavigation(){return {refresh(){}};},
  installWorkflowMapPreview(){return {clear(){},update(){}};},
  ...(await import('../../web/lab/native-aggregate-files.js')),
  ...(await import('../../web/lab/multi-summary-files.js')),
  ...(await import('../../web/lab/text-comparison-files.js')),
  ...(await import('../../web/lab/certified-context-files.js')),
  ...(await import('../../web/lab/proof-model-files.js')),
  document:{getElementById:node,querySelectorAll:()=>[],createTextNode:text=>({textContent:text}),createElement:()=>({value:"",textContent:"",append(){},setAttribute(){}})},
  Worker:class {constructor(){uiWorker=this;}postMessage(value){posted.push(value);}terminate(){}}, URL, AbortController,structuredClone,TextEncoder,TextDecoder,atob,...doctorTools,...manifestTools,...tutorialTools,...planDoctorTools,...suiteAssemblyTools,...civicTools,...civicSnapshotTools,...workerFailureTools,...labelInventoryTools,...planHubTools,...labelImportTools,...labelChainTools,...migrationTools,...candidateTools,
  ...(await import('../../web/lab/native-character-files.js')),  ...(await import('../../web/lab/state-workflow-files.js')),...(await import('../../web/lab/map-export-files.js')),...(await import('../../web/lab/national-map-batch.js')),...(await import('../../web/lab/state-batch-files.js')),...(await import('../../web/lab/state-run-batch.js')),...(await import('../../web/lab/state-source-job.js')),...(await import('../../web/lab/state-source-acquisition.js')),...(await import('../../web/lab/local-state-output.js')),...(await import('../../web/lab/stream-state-source-batch.js')),...(await import('../../web/lab/native-adjacency-files.js')),...(await import('../../web/lab/workbench-archive.js')),workbenchArchiveSnapshot:data=>workbenchArchiveTools.workbenchArchiveSnapshot(structuredClone(data)),...(await import('../../web/lab/census-archive.js')),...(await import('../../web/lab/tract-input-files.js')),...(await import('../../web/lab/tiger-files.js')),...(await import('../../web/lab/pl94-files.js')),...(await import('../../web/lab/fetch-data-files.js')), ...acquisitionTools,acquireFetchSource:(r,i,o)=>acquisitionTools.acquireFetchSource(r,i,{...o,fetcher:acquisitionRead}), ...(await import('../../web/lab/fetch-plan-files.js')),...(await import('../../web/lab/fletch-cache-files.js')),...(await import('../../web/lab/fletch-source-files.js')),...(await import('../../web/lab/fletch-verify-files.js')),...(await import('../../web/lab/fletch-merge-files.js')),...(await import('../../web/lab/fletch-stage-files.js')),...(await import('../../web/lab/fletch-cache-store.js')), ...(await import('../../web/lab/lodes-files.js')), ...(await import('../../web/lab/acs-housing-files.js')), ...(await import('../../web/lab/sweep-files.js')), ...(await import('../../web/lab/verify-assignments-files.js')), ...(await import('../../web/lab/assignment-discovery-files.js')), ...(await import('../../web/lab/engine-replay-files.js')), ...(await import('../../web/lab/assignment-verification-files.js')), ...(await import('../../web/lab/package-verification-files.js')),...(await import('../../web/lab/ilp-audit-files.js')),...(await import('../../web/lab/label-package-merge-files.js')),...(await import('../../web/lab/label-reconcile-files.js')),...(await import('../../web/lab/label-report-files.js')),...(await import('../../web/lab/label-analysis-files.js')),...(await import('../../web/lab/label-years-resume.js')),...(await import('../../web/lab/analysis-package.js')),...(await import('../../web/lab/analysis-collection.js')),...(await import('../../web/lab/label-build-archive.js')),...labelBuildTools,...labelRenameTools,setTimeout:()=>1,clearTimeout(){},
  ...(await import('../../web/lab/tui-run-files.js')),...(await import('../../web/lab/tui-verify-files.js')),...(await import('../../web/lab/tui-doctor-files.js')),...(await import('../../web/lab/tui-compare-files.js')),...(await import("../../web/lab/tui-session-files.js")),...(await import("../../web/lab/tui-library-files.js")),...(await import("../../web/lab/round-capture-files.js")),renderCountResult(){},planMapModel(){return {};},renderPlanMap(){},parseNativeNationalMapJson:(await import("../../web/lab/project.js")).parseNativeNationalMapJson,parseGeometryJson:(await import('../../web/lab/project.js')).parseGeometryJson,parseSafeJson,resultPreview,...(await import('../../web/lab/policy-files.js')),...(await import('../../web/lab/config-catalog.js')),...(await import('../../web/lab/config-catalog-preparation.js')),...(await import('../../web/lab/config-run.js')),...packageTools,...(await import("../../web/lab/config-template.js")),...(await import("../../web/lab/config-files.js")),...(await import('../../web/lab/national-files.js')),...(await import('../../web/lab/commission-files.js')),...(await import('../../web/lab/bloc-files.js')),...(await import('../../web/lab/compactness-files.js')),...(await import('../../web/lab/statistics-files.js')),...(await import('../../web/lab/election-analysis-settings.js')),...(await import('../../web/lab/stability-files.js')),...(await import('../../web/lab/suite-files.js')),...(await import('../../web/lab/research-files.js')),...(await import('../../web/lab/reproducibility-files.js')),...(await import('../../web/lab/report-output.js')),
  createProject(data){lastSnapshot=data;return data;},readProjectFile:(...args)=>importRead(...args),
  readJsonFile:async file=>parseSafeJson(await file.text()),
  configuredRuntimeText:()=>"No recorded configured run engine identity.",validateConfiguredRuntimeArtifacts:v=>v,labelBuildJobPolicies:()=>[],setLabelBuildJobAbsence:()=>{throw new Error("Job policy editing is tested separately.");},
  readConfiguredSourceBundle:async()=>{throw new Error("Source ZIP import is tested separately.");},
  configuredSourceBundle:async()=>{throw new Error("Source export is tested separately.");},
  prepareNativeDemographicAdoption:async()=>{throw new Error("Explicit native demographic handoff is tested separately.");},
  readAutosave:()=>restoreRead(),writeAutosave:()=>Promise.resolve(),
};
// Preserve real imported validators/readers as the workbench grows. Explicit
// DOM/worker adapters above remain scoped to this handler-only fixture.
for(const [,specifier] of ui.matchAll(/^import\s*.*?\s*from\s*['"]([^'"]+)['"];$/gm)){
  const exports=await import(new URL(specifier,new URL('../../web/lab/workbench.js',import.meta.url)));
  for(const [name,value] of Object.entries(exports))if(!(name in sandbox))sandbox[name]=value;
}
runInNewContext(ui.replace(/^import(?:\s|\{).*;\r?\n/gm,'').replaceAll('import.meta.url',JSON.stringify(new URL('../../web/lab/workbench.js',import.meta.url).href)),sandbox);
const deferred=()=>{let resolve;const promise=new Promise(r=>resolve=r);return {promise,resolve};};
const staleRestore=deferred();restoreRead=()=>staleRestore.promise;
const restoreAttempt=node('restore-project').onclick();
node('project-name').value='Edited while restoring';node('project-name').oninput();
staleRestore.resolve(saved);await restoreAttempt;
assert.equal(node('project-name').value,'Edited while restoring');
assert.match(node('project-status').textContent,/changed during restore/);

const staleInput=deferred();
const inputAttempt=node('plan-file').change({target:{files:[{name:'old.rplan',text:()=>staleInput.promise}]}});
await node('open-project').onchange({target:{files:[{name:'new.bisect'}],value:'new.bisect'}});
staleInput.resolve('{"stale":true}');await inputAttempt;
node('save-project').onclick(); // Snapshot occurs before unavailable download APIs.
assert.equal(lastSnapshot.files.plan.stale,undefined);
assert.equal(lastSnapshot.files.plan.schema_version,saved.files.plan.schema_version);
assert.match(node('result-status').textContent,/changed while reading/);

let cancelledImport=false;
importRead=(_file,{signal})=>new Promise((_resolve,reject)=>signal.addEventListener('abort',()=>{cancelledImport=true;reject(new Error('cancelled'));}));
const pendingOpen=node('open-project').onchange({target:{files:[{name:'old.bisect'}],value:'old.bisect'}});
restoreRead=()=>Promise.resolve(saved);
await node('restore-project').onclick();await pendingOpen;
assert.equal(cancelledImport,true);
assert.equal(node('project-name').value,saved.name);
assert.notEqual(node('project-status').textContent,'cancelled');
console.log('Open/restore UI: stale autosave and input reads preserve current work; superseded imports cancel without stale errors.');

assert.equal(resultPreview({ok:true,values:[1,'text',null]}),JSON.stringify({ok:true,values:[1,'text',null]},null,2));
const largeResult={payload:'x'.repeat(MAX_PREVIEW_CHARS*3)};
const preview=resultPreview(largeResult);
assert.match(preview,/Preview shortened/);
assert.ok(preview.length < MAX_PREVIEW_CHARS+100);
assert.equal(largeResult.payload.length,MAX_PREVIEW_CHARS*3);
// Do not traverse data beyond the display budget.
const limited={first:'x'.repeat(MAX_PREVIEW_CHARS*2)};
Object.defineProperty(limited,'later',{enumerable:true,get(){throw new Error('Preview traversed past its budget');}});
assert.match(resultPreview(limited),/Preview shortened/);
assert.equal(resultPreview({html:hostileText}),JSON.stringify({html:hostileText},null,2));
assert.throws(()=>resultPreview({},Infinity),/limit/);
const html=await readFile('web/lab/workbench.html','utf8');
for(const directive of ["default-src 'self'","script-src 'self' 'wasm-unsafe-eval'","worker-src 'self'","connect-src 'self'","object-src 'none'","base-uri 'none'","form-action 'none'"])assert.ok(html.includes(directive));
const renderer=await readFile('web/lab/count-results.js','utf8');
assert.ok(!/innerHTML|outerHTML|insertAdjacentHTML|eval\s*\(|new Function|document\.write/.test(renderer));
console.log('Result preview: bounded display, no traversal beyond budget, exact small results, unchanged exports and inert markup/CSP passed.');

// Avoid parser disagreement and silent byte replacement in every local JSON path.
for(const text of ['{"name":"first","name":"second"}',
  '{"name":1,"\\u006eame":2}', '{"files":{"plan":{"x":1,"x":2}}}',
  '[{"x":1,"x":2}]'])assert.throws(()=>parseSafeJson(text),/Duplicate/);
assert.deepEqual(parseSafeJson('{"x":{"key":1},"y":{"key":2},"text":"\\\"key\\\":{}"}'),{x:{key:1},y:{key:2},text:'"key":{}'});
assert.equal(await readFileTextStrict(new Blob(['{"ok":true}'])),'{"ok":true}');
await assert.rejects(readFileTextStrict(new Blob([Uint8Array.from([0xc3,0x28])])),/UTF-8/);
await assert.rejects(readFileTextStrict({size:1,arrayBuffer:async()=>new ArrayBuffer(MAX_PROJECT_BYTES+1)}),/25 MB/);
for(const size of [-1,NaN,Infinity,undefined])await assert.rejects(readProjectFile({size}),/25 MB/);
assert.ok(ui.includes('await readJsonFile(file)'));
assert.ok(!ui.includes('JSON.parse(await file.text())'));
// Execute actual worker handlers with hostile bytes, rather than mocking parsing.
for(const [name,parser]of [['project-worker.js',parseProject],['laboratory-project-worker.js',(await import('../../web/lab/laboratory-project.js')).parseLabProject],['json-worker.js',parseSafeJson]]) {
  let response;const self={postMessage:value=>{response=value;}};
  const source=await readFile('web/lab/'+name,'utf8');
  runInNewContext(source.replace(/^import(?:\s|\{).*;\r?\n/gm,''),{self,readFileTextStrict,parseProject:parser,parseLabProject:parser,parseSafeJson:parser});
  if(name!=='laboratory-project-worker.js'){
    await self.onmessage({data:new Blob([JSON.stringify(name==='project-worker.js'?saved:{ok:true})])});
    assert.equal(response.ok,true);
  }
  await self.onmessage({data:new Blob([Uint8Array.from([0xc3,0x28])])});
  assert.equal(response.ok,false);assert.match(response.error,/UTF-8/);
  await self.onmessage({data:new Blob(['{"x":1,"x":2}'])});
  assert.equal(response.ok,false);assert.match(response.error,/Duplicate/);
  await self.onmessage({data:new Blob(['{"count":18446744073709551615}'])});
  assert.equal(response.ok,false);assert.match(response.error,/exact JavaScript range/);
}
console.log('All local JSON workers: duplicate and escaped-duplicate fields, invalid UTF-8 and dishonest/invalid size rejection passed.');

// An import result may carry an exact-byte archive, but cannot bypass its limits.
const archiveResult={schema_version:'bisect-election-import-v1',package_files:{'manifest.json':'e30='}};
assert.deepEqual(parseProject(encode({result:archiveResult})).result,archiveResult);
for(const package_files of [undefined,{}, {'manifest.json':'bad'}, {'manifest.json':'e30=','../escape':'AA=='}]){
  assert.throws(()=>parseProject(encode({result:{...archiveResult,package_files}})),/package|manifest|encoding|path/i);
}
console.log('Imported election results: embedded archives obey the same path, encoding and size validation as inputs.');

// Exercise the import UI's metadata/source gates and explicit verification flow.
const metadata={country:'US',state:'RI',county:'Synthetic County',date:'2024-11-05',election_type:'general',scope:'county',status:'canvassed'};
node('operation').value='import-statement-csv';
let sourceBytes=Uint8Array.from([65,13,10,255]);
await node('election-source').onchange({target:{files:[{name:'source.csv',size:sourceBytes.length,arrayBuffer:async()=>sourceBytes.buffer}]}});
const beforeImport=posted.length;node('execute').onclick();assert.equal(posted.length,beforeImport);
assert.match(node('result-status').textContent,/metadata/);
for(const [key,value]of Object.entries(metadata)){node('import-'+key).value=value;node('import-'+key).oninput();}
node('execute').onclick();const importRequest=JSON.parse(JSON.stringify(posted.at(-1).request));
assert.equal(importRequest.operation,'import-statement-csv');assert.equal(importRequest.source_base64,packageTools.encodeBytes(sourceBytes));assert.deepEqual(importRequest.metadata,metadata);
uiWorker.onmessage({data:{type:'result',result:{...archiveResult,status:'pass',verification:{checks:[]}}}});
assert.equal(node('use-imported-package').disabled,false);assert.equal(node('download-package').disabled,false);
node('save-project').onclick();const importProject=createProject(lastSnapshot);
importRead=()=>Promise.resolve(importProject);
await node('open-project').onchange({target:{files:[{name:'import.bisect'}],value:'import.bisect'}});
assert.match(node('result-status').textContent,/unverified/);assert.equal(node('import-county').value,metadata.county);
const beforeUse=posted.length;node('use-imported-package').onclick();assert.equal(posted.length,beforeUse);
assert.equal(node('operation').value,'verify-count-files');node('execute').onclick();
assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'verify-count-files',files:{'manifest.json':[123,125]}});
uiWorker.onmessage({data:{type:'result',result:{status:'pass',checks:[]}}});
assert.equal(node('use-imported-package').disabled,true);
const archiveText=JSON.stringify({schema_version:'bisect-rcount-files-v1',files:archiveResult.package_files});
await node('package-archive').onchange({target:{files:[{name:'package.json',text:async()=>archiveText}]}});
assert.equal(node('operation').value,'verify-count-files');
const staleSource=deferred();const staleSourceRead=node('election-source').onchange({target:{files:[{name:'stale.csv',size:1,arrayBuffer:()=>staleSource.promise}]}});
node('import-country').value='CA';node('import-country').oninput();staleSource.resolve(new Uint8Array([1]).buffer);await staleSourceRead;
node('save-project').onclick();assert.equal(lastSnapshot.files.sourceFile.name,'source.csv');
assert.match(node('result-status').textContent,/changed while reading/);
console.log('Election workbench UI: explicit metadata gate, exact source request, result controls, Save/Open, imported-result labeling, explicit package verification, archive reopening and stale source rejection passed.');
node('operation').value='import-ri2024-rep28-rla';node('operation').change();
assert.equal(node('ri-import-fields').hidden,false);assert.equal(node('election-import-fields').hidden,true);
assert.equal(node('audit-check-fields').hidden,true);
const beforeRi=posted.length;node('execute').onclick();assert.equal(posted.length,beforeRi);
assert.match(node('result-status').textContent,/Select the RI/);
for(const key of ['auditReport','ballotManifest','ballotRetrieval'])await node('ri-'+key).onchange({target:{files:[{name:key+'.csv',size:1,arrayBuffer:async()=>new Uint8Array([65]).buffer}]}});
node('save-project').onclick();const riProject=createProject(lastSnapshot);assert.equal(Object.keys(riProject.files.riSources).length,3);
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'import-ri2024-rep28-rla',audit_report_base64:'QQ==',ballot_manifest_base64:'QQ==',ballot_retrieval_base64:'QQ=='});
uiWorker.onmessage({data:{type:'result',result:{...archiveResult,status:'pass',verification:{checks:[]}}}});
importRead=()=>Promise.resolve(riProject);await node('open-project').onchange({target:{files:[{name:'ri.bisect'}],value:'ri.bisect'}});
assert.equal(node('ri-auditReport-name').textContent,'auditReport.csv');assert.equal(node('ri-import-fields').hidden,false);
const heldRi=deferred();const oldRiRead=node('ri-auditReport').onchange({target:{files:[{name:'stale.csv',size:1,arrayBuffer:()=>heldRi.promise}]}});
node('project-name').oninput();heldRi.resolve(new Uint8Array([1]).buffer);await oldRiRead;
node('save-project').onclick();assert.equal(lastSnapshot.files.riSources.auditReport.name,'auditReport.csv');
console.log('RI workbench UI: three-source gate/request, partial project persistence, operation panels, restored filenames and stale source rejection passed.');
node('operation').value='aggregate-districts';node('operation').change();assert.equal(node('aggregation-fields').hidden,false);assert.equal(node('audit-check-fields').hidden,true);
node('aggregation-status').value='canvassed';node('aggregation-format').value='pretty-json';const beforeAggregation=posted.length;node('execute').onclick();assert.equal(posted.length,beforeAggregation);
node('aggregation-contest').value='mayor';node('aggregation-contest').input();
await node('crosswalk-file').onchange({target:{files:[{name:'crosswalk.ndjson',size:2,arrayBuffer:async()=>new Uint8Array([123,125]).buffer}]}});
node('execute').onclick();const aggregateRequest=JSON.parse(JSON.stringify(posted.at(-1).request));assert.equal(aggregateRequest.operation,'aggregate-districts');assert.equal(aggregateRequest.contest_id,'mayor');assert.equal(aggregateRequest.crosswalk_base64,'e30=');assert.equal(aggregateRequest.output_format,'pretty-json');assert.equal(aggregateRequest.document.schema_version,saved.files.plan.schema_version);
const aggregationResult={schema_version:'bisect-district-aggregation-v1',status:'pass',transcript_base64:packageTools.encodeBytes(new TextEncoder().encode('{"count":9007199254740993}')),district_totals:[{district_label:'1',counted_ballots:'9007199254740993',totals:[]}],verification:{checks:[]}};
uiWorker.onmessage({data:{type:'result',result:aggregationResult}});assert.equal(node('download-aggregation').disabled,false);node('save-project').onclick();const aggregationProject=createProject(lastSnapshot);assert.deepEqual(aggregationProject.result,aggregationResult);
importRead=()=>Promise.resolve(aggregationProject);await node('open-project').onchange({target:{files:[{name:'aggregate.bisect'}],value:'aggregate.bisect'}});
assert.equal(node('aggregation-contest').value,'mayor');assert.equal(node('crosswalk-name').textContent,'crosswalk.ndjson');assert.match(node('result-status').textContent,/unverified/);
assert.throws(()=>parseProject(JSON.stringify({...aggregationProject,result:{...aggregationResult,transcript_base64:'bad'}})),/encoding/);
console.log('Aggregation UI: required inputs/settings, exact crosswalk request, output format, large-count result persistence, restored settings/source names and transcript encoding rejection passed.');
node('operation').value='verify-history-files';node('operation').change();
assert.equal(node('history-fields').hidden,false);assert.equal(node('aggregation-fields').hidden,true);assert.equal(node('audit-check-fields').hidden,true);
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'verify-history-files');
assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request.files)),JSON.parse(JSON.stringify(packageTools.decodePackageFiles(aggregationProject.files.packageFiles))));
const historyResult={status:'pass',package_id:'syn-history',package_content_hash:'sha256:'+'a'.repeat(64),checks:['lineage_cardinality'],error:null};
uiWorker.onmessage({data:{type:'result',result:historyResult}});node('save-project').onclick();const historyProject=createProject(lastSnapshot);
assert.deepEqual(historyProject.result,historyResult);assert.equal(node('download-aggregation').disabled,true);
importRead=()=>Promise.resolve(historyProject);const beforeHistoryOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'history.bisect'}],value:'history.bisect'}});
assert.equal(posted.length,beforeHistoryOpen);assert.equal(node('operation').value,'verify-history-files');assert.match(node('result-status').textContent,/unverified/);
console.log('History workbench: exact package request, operation panels, Save/Open, no execution on Open and unverified labeling passed.');

node('operation').value='attach-demographic-csv';node('operation').change();assert.equal(node('demographic-context-fields').hidden,false);assert.equal(node('vra-policy-fields').hidden,true);
node('demographic-basis').value='voting-age-population';node('demographic-label').value='SYNTHETIC count fixture';
const countBytes=new TextEncoder().encode('GEOID,total_vap,minority_vap\n53001000100,100,50\n');
await node('demographic-source').onchange({target:{files:[{name:'Synthetic.csv',size:countBytes.length,arrayBuffer:async()=>countBytes.buffer}]}});
node('execute').onclick();const countRequest=JSON.parse(JSON.stringify(posted.at(-1).request));assert.equal(countRequest.operation,'attach-demographic-csv');assert.equal(countRequest.source_base64,packageTools.encodeBytes(countBytes));assert.equal(countRequest.basis,'voting-age-population');assert.equal(countRequest.source_label,'SYNTHETIC count fixture');assert.equal(node('demographic-source').disabled,true);
const preparedContext={schema_version:'bisect-demographic-context-v1',basis:'voting-age-population',source_label:'SYNTHETIC count fixture',unit_count:9,context:{...saved.files.context,context_hash:'sha256:'+'a'.repeat(64),demographics:{total_vap:Array(9).fill(100),minority_vap:Array(9).fill(50)}},context_hash:'sha256:'+'a'.repeat(64),district_totals:[]};
uiWorker.onmessage({data:{type:'result',result:preparedContext}});assert.equal(node('use-demographic-context').disabled,false);assert.equal(node('download-demographic-context').disabled,false);assert.equal(node('demographic-source').disabled,false);
node('save-project').onclick();const countProject=createProject(lastSnapshot);assert.deepEqual(countProject.files.demographicSettings,{basis:'voting-age-population',source_label:'SYNTHETIC count fixture'});assert.equal(countProject.files.demographicSource.base64,countRequest.source_base64);
importRead=()=>Promise.resolve(countProject);const beforeCountOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'counts.bisect'}],value:'counts.bisect'}});assert.equal(posted.length,beforeCountOpen);assert.equal(node('demographic-source-name').textContent,'Synthetic.csv');assert.equal(node('demographic-basis').value,'voting-age-population');assert.match(node('result-status').textContent,/unverified/);
node('use-demographic-context').onclick();assert.equal(node('operation').value,'audit-plan');assert.equal(node('vra-policy-fields').hidden,false);
const explicitProfile={...certificate.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};delete explicitProfile.legal_disclaimer;
await node('profile-file').change({target:{files:[{name:'profile.json',text:async()=>JSON.stringify(explicitProfile)}]}});
node('vra-group').value='Explicit synthetic group';node('vra-report-threshold').value='50';node('apply-vra-policy').onclick();node('execute').onclick();const countAudit=JSON.parse(JSON.stringify(posted.at(-1).request));assert.deepEqual(countAudit.context,preparedContext.context);assert.deepEqual(countAudit.profile.vra_policy,{type:'report-opportunity-districts',minority_group:'Explicit synthetic group',vap_threshold:0.5});
uiWorker.onmessage({data:{type:'result',result:{status:'pass'}}});node('save-project').onclick();const auditCountProject=createProject(lastSnapshot);assert.deepEqual(auditCountProject.files.profile.vra_policy,countAudit.profile.vra_policy);
const heldCounts=deferred();const staleCounts=node('demographic-source').onchange({target:{files:[{name:'stale.csv',size:1,arrayBuffer:()=>heldCounts.promise}]}});node('project-name').oninput();heldCounts.resolve(new Uint8Array([1]).buffer);await staleCounts;node('save-project').onclick();assert.equal(lastSnapshot.files.demographicSource.name,'Synthetic.csv');
for(const source of [{name:'bad.csv',base64:'bad'},{name:'x'.repeat(201),base64:'AA=='}])assert.throws(()=>createProject({...countProject,files:{...countProject.files,demographicSource:source}}));
for(const settings of [{basis:'total-population',source_label:'x'},{basis:'voting-age-population',source_label:'é'.repeat(101)},{basis:'voting-age-population',source_label:'x',extra:1}])assert.throws(()=>createProject({...countProject,files:{...countProject.files,demographicSettings:settings}}));
console.log('Count-based VRA workbench: exact CSV request and byte retention, explicit context adoption/profile policy, Save/Open without execution, restored settings, busy controls, stale source rejection and invalid source/settings rejection passed.');

const comparisonInput=createProject({name:'Two plan comparison',files:{plan:saved.files.plan,comparisonPlan:saved.files.plan,context:saved.files.context},operation:'compare-plans',constraints:[],result:null,lastOperation:null});
importRead=()=>Promise.resolve(comparisonInput);const beforeComparisonOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'comparison.bisect'}],value:'comparison.bisect'}});
assert.equal(posted.length,beforeComparisonOpen);assert.equal(node('comparison-fields').hidden,false);
node('execute').onclick();const comparisonRequest=JSON.parse(JSON.stringify(posted.at(-1).request));
assert.deepEqual(comparisonRequest,{operation:'compare-plans',plan_a:comparisonInput.files.plan,plan_b:comparisonInput.files.comparisonPlan,context:comparisonInput.files.context});
await node('comparisonPlan-file').change({target:{files:[{name:'busy.json',text:async()=>'{"changed":true}'}]}});
assert.match(node('result-status').textContent,/Finish or cancel/);
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-plan-comparison-v1',similarity:{a_to_b:1,b_to_a:1}}}});
node('save-project').onclick();const comparisonSaved=createProject(lastSnapshot);assert.deepEqual(comparisonSaved.files.comparisonPlan,comparisonInput.files.comparisonPlan);assert.equal(comparisonSaved.lastOperation,'compare-plans');
importRead=()=>Promise.resolve(comparisonSaved);const beforeComparisonRestore=posted.length;
await node('open-project').onchange({target:{files:[{name:'comparison-saved.bisect'}],value:'comparison-saved.bisect'}});
assert.equal(posted.length,beforeComparisonRestore);assert.match(node('result-status').textContent,/unverified/);
console.log('Comparison workbench: two exact plan inputs, optional shared context, operation panel, busy input guard, Save/Open and no execution on Open passed.');

const reportBytes=new TextEncoder().encode('{"num_districts":2}');
let ignoredReportReads=0;
const reportSelection=[{name:'manifest.json',webkitRelativePath:'report/manifest.json',size:reportBytes.length,arrayBuffer:async()=>reportBytes.buffer},{name:'map.geojson',webkitRelativePath:'report/map.geojson',size:100000000,arrayBuffer:async()=>{ignoredReportReads++;throw new Error('Unrelated file read');}}];
const selectedReport=await packageTools.readSelectedReport(reportSelection);assert.equal(ignoredReportReads,0);assert.deepEqual(Object.keys(selectedReport),['manifest.json']);
for(const invalid of [[...reportSelection,reportSelection[0]],[reportSelection[0],{...reportSelection[1],webkitRelativePath:'report/../map.geojson'}],[reportSelection[0],{...reportSelection[1],webkitRelativePath:'other/map.geojson'}]])await assert.rejects(()=>packageTools.readSelectedReport(invalid));
await assert.rejects(()=>packageTools.readSelectedReport([{...reportSelection[0],size:1,arrayBuffer:async()=>new Uint8Array(4*1024*1024+1).buffer}]));
const reportInput=createProject({name:'Imported report',files:{reportFiles:selectedReport,reportSettings:{label:'Literal / fallback',leaning_threshold:0.55}},operation:'inspect-report-side',constraints:[],result:null,lastOperation:null});
importRead=()=>Promise.resolve(reportInput);const beforeReportOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'report.bisect'}],value:'report.bisect'}});assert.equal(posted.length,beforeReportOpen);assert.equal(node('report-fields').hidden,false);assert.equal(node('report-label').value,'Literal / fallback');assert.equal(node('report-threshold').value,'55');
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'inspect-report-side',files:{'manifest.json':Array.from(reportBytes)},label:'Literal / fallback',leaning_threshold:0.55});assert.equal(node('report-folder').disabled,true);
await node('report-folder').onchange({target:{files:reportSelection}});assert.match(node('result-status').textContent,/Finish or cancel/);
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-report-side-inspection-v1',native_report_side:{label:'Literal / fallback',n_districts:2},usable_analysis:{partisan:false,demographic:false,compactness:false}}}});assert.equal(node('report-folder').disabled,false);
node('save-project').onclick();const savedReport=createProject(lastSnapshot);assert.deepEqual(savedReport.files.reportFiles,reportInput.files.reportFiles);assert.deepEqual(savedReport.files.reportSettings,reportInput.files.reportSettings);
importRead=()=>Promise.resolve(savedReport);const beforeReportRestore=posted.length;await node('open-project').onchange({target:{files:[{name:'report-saved.bisect'}],value:'report-saved.bisect'}});assert.equal(posted.length,beforeReportRestore);assert.match(node('result-status').textContent,/unverified/);
for(const settings of [{label:'x',leaning_threshold:2},{label:'é'.repeat(101),leaning_threshold:0.5},{label:'x',leaning_threshold:0.5,extra:true}])assert.throws(()=>createProject({...reportInput,files:{...reportInput.files,reportSettings:settings}}));
const heldReport=deferred();const staleReport=node('report-folder').onchange({target:{files:[{...reportSelection[0],arrayBuffer:()=>heldReport.promise}]}});node('project-name').oninput();heldReport.resolve(reportBytes.buffer);await staleReport;assert.match(node('result-status').textContent,/Project changed/);
console.log('Report inspection UI: supported-file filtering, ignored files not read, path/size limits, exact request, busy/stale guards, settings/byte persistence and unverified Open without execution passed.');

// Exercise the real report request, local save/restore and computed-only exports.
const reportOutputTools=await import('../../web/lab/report-output.js');
const {createHash}=await import('node:crypto');
const reportHash=text=>createHash('sha256').update(text).digest('hex');
const reportResult={schema_version:'bisect-imported-report-comparison-v1',draft:true,html:'<!doctype html><p>Draft é report</p>',markdown:'Draft é report',receipt:{schema_version:'bisect-report-byte-receipt-v1'}};
reportResult.receipt.html_sha256=reportHash(reportResult.html);reportResult.receipt.markdown_sha256=reportHash(reportResult.markdown);
const reportComparisonInput=createProject({name:'Report comparison',files:{reportFiles:selectedReport,comparisonReportFiles:selectedReport,reportComparisonSettings:{label_a:'A',label_b:'B',leaning_threshold:0.55,close_call_band:0.02,generated_at:'2026-10-04T00:00:00Z'}},operation:'render-report-comparison',constraints:[],result:null,lastOperation:null});
importRead=()=>Promise.resolve(reportComparisonInput);const beforeReportComparisonOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'report-comparison.bisect'}],value:'report-comparison.bisect'}});
assert.equal(posted.length,beforeReportComparisonOpen);assert.equal(node('report-fields').hidden,false);assert.equal(node('report-comparison-fields').hidden,false);assert.equal(node('report-close-call').value,'2');assert.equal(node('plan-file').parentElement.hidden,true);assert.equal(node('verify-result').hidden,true);assert.equal(node('download-report-html').hidden,false);assert.equal(node('comparison-report-label').value,'B');
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'render-report-comparison',files_a:{'manifest.json':Array.from(reportBytes)},files_b:{'manifest.json':Array.from(reportBytes)},...reportComparisonInput.files.reportComparisonSettings});
assert.equal(node('comparison-report-folder').disabled,true);assert.equal(node('download-report-html').disabled,true);
await node('comparison-report-folder').onchange({target:{files:reportSelection}});assert.match(node('result-status').textContent,/Finish or cancel/);
uiWorker.onmessage({data:{type:'result',result:reportResult}});assert.equal(node('download-report-html').disabled,false);assert.equal(node('comparison-report-folder').disabled,false);
node('save-project').onclick();const savedComparisonReport=createProject(lastSnapshot);assert.deepEqual(savedComparisonReport.result,reportResult);assert.deepEqual(savedComparisonReport.files.reportComparisonSettings,reportComparisonInput.files.reportComparisonSettings);
importRead=()=>Promise.resolve(savedComparisonReport);const beforeReportComparisonRestore=posted.length;
await node('open-project').onchange({target:{files:[{name:'saved-report.bisect'}],value:'saved-report.bisect'}});assert.equal(posted.length,beforeReportComparisonRestore);assert.match(node('result-status').textContent,/unverified/);assert.equal(node('download-report-html').disabled,true);
await node('download-report-html').onclick();assert.match(node('result-status').textContent,/Run the report again/);
for(const kind of ['html','markdown','receipt']){const file=await reportOutputTools.reportDownload(reportResult,kind);assert.equal(new TextDecoder().decode(file.bytes),kind==='receipt'?JSON.stringify(reportResult.receipt,null,2):reportResult[kind]);}
await assert.rejects(()=>reportOutputTools.reportDownload({...reportResult,html:reportResult.html+'tampered'},'html'),/match the receipt/);
await assert.rejects(()=>reportOutputTools.reportDownload({...reportResult,markdown:reportResult.markdown+'tampered'},'receipt'),/match the receipt/);
assert.throws(()=>createProject({...savedComparisonReport,result:{...reportResult,draft:false}}));
assert.throws(()=>createProject({...savedComparisonReport,result:{...reportResult,html:'x'.repeat(2*1024*1024+1)}}));
for(const settings of [{...reportComparisonInput.files.reportComparisonSettings,close_call_band:2},{...reportComparisonInput.files.reportComparisonSettings,label_b:'é'.repeat(101)},{...reportComparisonInput.files.reportComparisonSettings,generated_at:'é'.repeat(51)}])assert.throws(()=>createProject({...reportComparisonInput,files:{...reportComparisonInput.files,reportComparisonSettings:settings}}));
const heldComparisonReport=deferred();const staleComparisonReport=node('comparison-report-folder').onchange({target:{files:[{...reportSelection[0],arrayBuffer:()=>heldComparisonReport.promise}]}});node('project-name').oninput();heldComparisonReport.resolve(reportBytes.buffer);await staleComparisonReport;assert.match(node('result-status').textContent,/Project changed/);
node('report-close-call').value='';const beforeEmptyBand=posted.length;node('execute').onclick();assert.equal(posted.length,beforeEmptyBand);assert.match(node('result-status').textContent,/Supply the leaning threshold/);node('report-close-call').value='2';
node('report-generated-at').value='';node('execute').onclick();assert.ok(Number.isFinite(Date.parse(posted.at(-1).request.generated_at)));uiWorker.onmessage({data:{type:'result',result:reportResult}});
// Verify actual handler Blob content, not just the helper's return value.
sandbox.Blob=Blob;let downloadedLink;sandbox.document.createElement=()=>({click(){downloadedLink=this;}});
for(const kind of ['html','markdown','receipt']){await node('download-report-'+kind).onclick();assert.ok(downloadedLink.download.includes('bisect-comparison-report'));assert.equal(await (await fetch(downloadedLink.href)).text(),kind==='receipt'?JSON.stringify(reportResult.receipt,null,2):reportResult[kind]);URL.revokeObjectURL(downloadedLink.href);}
const spoofedReport={...reportResult,html:'<script>untrusted imported result</script>',receipt:{...reportResult.receipt,html_sha256:reportHash('<script>untrusted imported result</script>')}};
const spoofedProject=createProject({...savedComparisonReport,result:spoofedReport});importRead=()=>Promise.resolve(spoofedProject);
const beforeSpoofOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'spoofed-report.bisect'}],value:'spoofed-report.bisect'}});
assert.equal(posted.length,beforeSpoofOpen);assert.equal(node('download-report-html').disabled,true);assert.ok(!node('result').textContent.includes('<script>untrusted imported result</script>'));
await node('download-report-html').onclick();assert.match(node('result-status').textContent,/Run the report again/);
console.log('Report generation UI: two exact folders/settings, busy/stale guards, default timestamp, Save/Open without execution, computed-only exports, output tamper rejection and exact HTML/Markdown/receipt download bytes passed.');

const reportWithPlans=createProject({...savedComparisonReport,files:{...savedComparisonReport.files,plan:saved.files.plan,comparisonPlan:saved.files.plan,context:saved.files.context,reportPlanSettings:{include_plan_measurements:true}}});
importRead=()=>Promise.resolve(reportWithPlans);await node('open-project').onchange({target:{files:[{name:'report-plans.bisect'}],value:'report-plans.bisect'}});
assert.equal(node('report-plan-evidence').checked,true);assert.equal(node('plan-file').parentElement.hidden,false);assert.equal(node('context-file').parentElement.hidden,false);assert.equal(node('comparison-fields').hidden,false);
node('execute').onclick();const reportPlanRequest=JSON.parse(JSON.stringify(posted.at(-1).request));assert.deepEqual(reportPlanRequest.plan_a,saved.files.plan);assert.deepEqual(reportPlanRequest.plan_b,saved.files.plan);assert.deepEqual(reportPlanRequest.context,saved.files.context);assert.equal(node('report-plan-evidence').disabled,true);
uiWorker.onmessage({data:{type:'result',result:reportResult}});node('save-project').onclick();assert.deepEqual(lastSnapshot.files.reportPlanSettings,{include_plan_measurements:true});
node('report-plan-evidence').checked=false;node('report-plan-evidence').onchange();node('execute').onclick();assert.equal(Object.hasOwn(posted.at(-1).request,'plan_a'),false);assert.equal(Object.hasOwn(posted.at(-1).request,'context'),false);assert.equal(node('plan-file').parentElement.hidden,true);uiWorker.onmessage({data:{type:'result',result:reportResult}});
for(const settings of [{include_plan_measurements:'yes'},{include_plan_measurements:true,extra:1},{other:true}])assert.throws(()=>createProject({...reportWithPlans,files:{...reportWithPlans.files,reportPlanSettings:settings}}));
console.log('Report plan inputs: explicit opt-in, exact two-plan/context request, busy guard, saved setting, restored panels, and opt-out without silently using retained plan inputs passed.');

node('operation').value='render-report-comparison';node('operation').change();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:reportResult}});
assert.equal(node('download-report-html').disabled,false);node('project-name').value='Renamed project';node('project-name').oninput();assert.equal(node('download-report-html').disabled,false);
node('report-threshold').value='56';node('report-threshold').oninput();assert.equal(node('download-report-html').disabled,true);assert.match(node('result-status').textContent,/Inputs changed/);
await node('download-report-html').onclick();assert.match(node('result-status').textContent,/Run the report again/);
console.log('Report export freshness: project renaming preserves output export; changing report settings requires an explicit fresh run.');

for(const [field,value] of [['report-threshold',''],['report-close-call','101'],['report-generated-at','é'.repeat(51)]]){
  node('report-threshold').value='55';node('report-close-call').value='2';node('report-generated-at').value='2026-10-04T00:00:00Z';
  node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:reportResult}});assert.equal(node('download-report-html').disabled,false);
  node(field).value=value;node(field).oninput();
  for(const kind of ['html','markdown','receipt'])assert.equal(node('download-report-'+kind).disabled,true);
  await node('download-report-html').onclick();assert.match(node('result-status').textContent,/Run the report again/);
}
console.log('Invalid report edits also invalidate exports: blank threshold, out-of-range band and oversized timestamp.');

const exactText='{"populations":[9007199254740993],"literal":"<script>inert</script>"}';
const exactBytes=new TextEncoder().encode(exactText);
node('operation').value='solve-certified-split';node('operation').change();assert.equal(node('exact-fields').hidden,false);assert.equal(node('plan-file').parentElement.hidden,true);
await node('exact-source').onchange({target:{files:[{name:'exact.json',size:exactBytes.length,arrayBuffer:async()=>exactBytes.buffer}]}});
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'solve-certified-split',instance_json:exactText});assert.equal(node('exact-source').disabled,true);
const exactResult={schema_version:'bisect-certified-split-output-v1',status:'optimal',artifacts_json:exactText,certificate_id:'sha256:'+'a'.repeat(64)};
uiWorker.onmessage({data:{type:'result',result:exactResult}});assert.equal(node('download-exact-artifact').disabled,false);
node('download-exact-artifact').onclick();assert.equal(await (await fetch(downloadedLink.href)).text(),exactText);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const exactProject=createProject(lastSnapshot);assert.equal(exactProject.files.exactSource.base64,packageTools.encodeBytes(exactBytes));assert.equal(parseProject(JSON.stringify(exactProject)).result.artifacts_json,exactText);
node('use-exact-artifact').onclick();assert.equal(node('operation').value,'verify-certified-split');node('execute').onclick();assert.equal(posted.at(-1).request.artifacts_json,exactText);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
importRead=()=>Promise.resolve(exactProject);const beforeExactRestore=posted.length;await node('open-project').onchange({target:{files:[{name:'exact.bisect'}],value:'exact.bisect'}});assert.equal(posted.length,beforeExactRestore);assert.equal(node('download-exact-artifact').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('download-exact-artifact').onclick();assert.match(node('result-status').textContent,/Run exact solving again/);
let tooLargeRead=false;await node('exact-source').onchange({target:{files:[{name:'huge.json',size:3*1024*1024+1,arrayBuffer:async()=>{tooLargeRead=true;}}]}});assert.equal(tooLargeRead,false);assert.match(node('result-status').textContent,/3 MiB/);
console.log('Exact workbench: lossless large-integer source request, byte-retaining Save/Open, busy and pre-read size guards, exact artifact download, explicit proof adoption and unverified restore without execution passed.');

node('operation').value='solve-certified-single';node('operation').change();node('execute').onclick();assert.equal(posted.at(-1).request.operation,'solve-certified-single');assert.equal(posted.at(-1).request.instance_json,exactText);
const singleResult={schema_version:'bisect-certified-single-output-v1',status:'trivial-single-district',districts:1,population_total:'9007199254740993',artifacts_json:exactText,certificate_id:'sha256:'+'b'.repeat(64)};
uiWorker.onmessage({data:{type:'result',result:singleResult}});node('download-exact-artifact').onclick();assert.equal(downloadedLink.download,'certified-single-artifacts.json');assert.equal(await (await fetch(downloadedLink.href)).text(),exactText);URL.revokeObjectURL(downloadedLink.href);
node('use-exact-artifact').onclick();assert.equal(node('operation').value,'verify-certified-single');assert.equal(node('exact-source').value,'');node('execute').onclick();assert.equal(posted.at(-1).request.artifacts_json,exactText);uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-certified-single-verification-v1',verified:true,verification_method:'Checked trivial single-district certificate'}}});
console.log('Single-district UI: lossless build request, correctly named exact download, explicit adoption, cleared stale picker label and verification request passed.');

node('operation').value='solve-certified-canonical';node('operation').change();node('execute').onclick();assert.equal(posted.at(-1).request.operation,'solve-certified-canonical');assert.equal(posted.at(-1).request.instance_json,exactText);
const canonicalResult={schema_version:'bisect-certified-canonical-output-v1',status:'optimal',districts:2,artifacts_json:exactText,certificate_id:'sha256:'+'c'.repeat(64)};
uiWorker.onmessage({data:{type:'result',result:canonicalResult}});node('download-exact-artifact').onclick();assert.equal(downloadedLink.download,'certified-canonical-artifacts.json');assert.equal(await (await fetch(downloadedLink.href)).text(),exactText);URL.revokeObjectURL(downloadedLink.href);
node('use-exact-artifact').onclick();assert.equal(node('operation').value,'verify-certified-canonical');node('execute').onclick();assert.equal(posted.at(-1).request.artifacts_json,exactText);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
console.log('Canonical benchmark UI: exact instance request, correctly named artifact download and explicit canonical verification adoption passed.');

node('ceremony-full-tree').checked=false;node('operation').value='start-ceremony';node('operation').change();assert.equal(node('ceremony-fields').hidden,false);assert.equal(node('exact-fields').hidden,true);assert.equal(node('plan-file').parentElement.hidden,true);
for(const key of ['ceremonyTree','ceremonyConfig'])await node(key+'-file').onchange({target:{files:[{name:key+'.json',size:exactBytes.length,arrayBuffer:async()=>exactBytes.buffer}]}});
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'start-ceremony',tree_json:exactText,config_json:exactText});assert.equal(node('ceremony-time').disabled,true);
const ceremonyOutput={schema_version:'bisect-ceremony-output-v1',status:'open',transcript_json:exactText,transcript_id:'sha256:'+'d'.repeat(64),released_rounds:0};
uiWorker.onmessage({data:{type:'result',result:ceremonyOutput}});node('download-ceremony').onclick();assert.equal(downloadedLink.download,'ceremony-transcript.json');assert.equal(await (await fetch(downloadedLink.href)).text(),exactText);URL.revokeObjectURL(downloadedLink.href);
node('use-ceremony').onclick();assert.equal(node('operation').value,'announce-ceremony');assert.equal(node('ceremony-time').value,'');assert.equal(node('ceremony-receipts').value,'');node('ceremony-time').value='9007199254740993';node('ceremony-receipts').value='receipt-A\nreceipt-B';node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'announce-ceremony',tree_json:exactText,transcript_json:exactText,published_unix_seconds:'9007199254740993',witness_receipts:['receipt-A','receipt-B']});
uiWorker.onmessage({data:{type:'result',result:ceremonyOutput}});node('save-project').onclick();const ceremonyProject=createProject(lastSnapshot);assert.equal(ceremonyProject.files.ceremonySettings.published_unix_seconds,'9007199254740993');assert.deepEqual(parseProject(JSON.stringify(ceremonyProject)),ceremonyProject);
node('ceremony-time').value='01';node('ceremony-time').oninput();assert.equal(node('download-ceremony').disabled,true);assert.match(node('result-status').textContent,/canonical decimal/);
node('operation').value='verify-ceremony';node('operation').change();node('execute').onclick();assert.equal(posted.at(-1).request.tree_json,null);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});node('ceremony-full-tree').checked=true;node('ceremony-full-tree').onchange();node('execute').onclick();assert.equal(posted.at(-1).request.tree_json,exactText);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
node('operation').value='inspect-ceremony-prefix';node('operation').change();assert.equal(node('ceremony-tree-field').hidden,true);assert.equal(node('ceremony-scope-field').hidden,true);node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'inspect-ceremony-prefix',transcript_json:exactText});uiWorker.onmessage({data:{type:'result',result:{verified:true,verification_scope:'released-prefix-only'}}});node('save-project').onclick();assert.equal(parseProject(JSON.stringify(createProject(lastSnapshot))).operation,'inspect-ceremony-prefix');
node('operation').value='halt-ceremony';node('ceremony-reason').value='Synthetic pause';node('execute').onclick();assert.equal(posted.at(-1).request.reason,'Synthetic pause');uiWorker.onmessage({data:{type:'result',result:{...ceremonyOutput,status:'halted'}}});node('use-ceremony').onclick();assert.equal(node('operation').value,'verify-ceremony');
importRead=()=>Promise.resolve(ceremonyProject);const beforeCeremonyOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'ceremony.bisect'}],value:'ceremony.bisect'}});assert.equal(posted.length,beforeCeremonyOpen);assert.equal(node('download-ceremony').disabled,true);assert.equal(node('ceremony-time').value,'9007199254740993');assert.match(node('result-status').textContent,/unverified/);
let oversizedCeremonyRead=false;await node('ceremonyTranscript-file').onchange({target:{files:[{name:'large.json',size:3*1024*1024+1,arrayBuffer:async()=>{oversizedCeremonyRead=true;}}]}});assert.equal(oversizedCeremonyRead,false);
for(const settings of [{published_unix_seconds:'01',witness_receipts:[],reason:'',include_tree:false},{published_unix_seconds:'',witness_receipts:[],reason:'',include_tree:'true'}])assert.throws(()=>createProject({...ceremonyProject,files:{...ceremonyProject.files,ceremonySettings:settings}}));
console.log('Ceremony UI: exact tree/config/transcript bytes, four native requests, lossless timestamps, explicit verification scope, busy/size guards, transcript download/adoption, invalid-edit export guard and inert saved-project restore passed.');

const staleCeremonyRead=deferred();const staleCeremonyImport=node('ceremonyConfig-file').onchange({target:{files:[{name:'late.json',size:exactBytes.length,arrayBuffer:()=>staleCeremonyRead.promise}]}});node('project-name').oninput();staleCeremonyRead.resolve(exactBytes.buffer);await staleCeremonyImport;assert.match(node('result-status').textContent,/Project changed while reading ceremony/);
node('save-project').onclick();assert.equal(lastSnapshot.files.ceremonyConfig.name,'ceremonyConfig.json');
console.log('Ceremony delayed file reads reject stale completion and preserve the saved configuration.');

node('operation').value='solve-certified-tree';node('operation').change();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-certified-tree-output-v1',tree_json:exactText,districts:4}}});assert.equal(node('use-tree-for-ceremony').disabled,false);node('use-tree-for-ceremony').onclick();assert.equal(node('operation').value,'start-ceremony-from-sources');const beforeSourceStart=posted.length;node('execute').onclick();assert.equal(posted.length,beforeSourceStart);assert.match(node('result-status').textContent,/original rule-profile/);node('operation').value='start-ceremony';node('operation').change();node('execute').onclick();assert.equal(posted.at(-1).request.tree_json,exactText);uiWorker.onmessage({data:{type:'result',result:ceremonyOutput}});
console.log('Generated certified tree adoption selects ceremony start and preserves its exact artifact text.');

node('operation').value='solve-branch-price';node('operation').change();
assert.equal(node('exact-fields').hidden,false);assert.equal(node('column-help').hidden,false);assert.equal(node('certified-help').hidden,true);assert.equal(node('download-exact-artifact').hidden,true);
await node('exact-source').onchange({target:{files:[{name:'column.json',size:exactBytes.length,arrayBuffer:async()=>exactBytes.buffer}]}});
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'solve-branch-price',instance_json:exactText});
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-branch-price-browser-output-v1',report:{status:'formulation-only',solution:null,lower_bound:null,upper_bound:null,gap:null}}}});
node('save-project').onclick();const columnProject=createProject(lastSnapshot);assert.equal(columnProject.operation,'solve-branch-price');assert.equal(columnProject.files.exactSource.base64,Buffer.from(exactText).toString('base64'));
importRead=()=>Promise.resolve(columnProject);const beforeColumnOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'column.bisect'}],value:'column.bisect'}});assert.equal(posted.length,beforeColumnOpen);assert.match(node('result-status').textContent,/unverified/);
console.log('Column benchmark UI: lossless request, dedicated scope panel, artifact controls hidden, byte-preserving project and unverified Open without execution passed.');

const columnContextText=await readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'),columnContextBytes=new TextEncoder().encode(columnContextText);
node('operation').value='solve-branch-price-context';node('operation').change();assert.equal(node('column-context-fields').hidden,false);assert.equal(node('exact-fields').hidden,false);assert.equal(node('column-help').hidden,true);
for(const [key,value] of Object.entries({k:'3',tolerance:'0',limit:'12',label:'Grid benchmark',chamber:'congressional',created:'2026-10-04T00:00:00Z'}))node('column-'+key).value=value;node('column-formulation').checked=false;
await node('exact-source').onchange({target:{files:[{name:'grid.rctx',size:columnContextBytes.length,arrayBuffer:async()=>columnContextBytes.buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'solve-branch-price-context');assert.equal(posted.at(-1).request.context_json,columnContextText);assert.equal(posted.at(-1).request.settings.k,3);assert.equal(posted.at(-1).request.settings.tolerance,0);assert.equal(node('column-k').disabled,true);
const contextResult={schema_version:'bisect-branch-price-context-output-v1',report:{status:'exact-fixture-optimal'},document_json:JSON.stringify(plan),context_json:columnContextText};
uiWorker.onmessage({data:{type:'result',result:contextResult}});assert.equal(node('download-column-plan').disabled,false);node('download-column-plan').onclick();assert.equal(downloadedLink.download,'column-benchmark.rplan');assert.equal(await (await fetch(downloadedLink.href)).text(),contextResult.document_json);URL.revokeObjectURL(downloadedLink.href);
node('download-column-context').onclick();assert.equal(downloadedLink.download,'column-context.rctx');assert.equal(await (await fetch(downloadedLink.href)).text(),columnContextText);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const contextProject=createProject(lastSnapshot);assert.equal(contextProject.files.columnSettings.k,3);assert.equal(contextProject.files.exactSource.base64,Buffer.from(columnContextText).toString('base64'));
node('column-tolerance').value='';node('column-tolerance').oninput();assert.equal(node('download-column-plan').disabled,true);assert.equal(node('download-column-context').disabled,true);
importRead=()=>Promise.resolve(contextProject);const beforeContextOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'grid.bisect'}],value:'grid.bisect'}});assert.equal(posted.length,beforeContextOpen);assert.equal(node('column-tolerance').value,'0');assert.equal(node('download-column-plan').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:{...contextResult,report:{status:'formulation-only'},document_json:null}}});assert.equal(node('download-column-plan').disabled,true);assert.equal(node('download-column-context').disabled,false);
console.log('RCTX column UI: original-byte request, settings/panels, busy guard, exact RPLAN/RCTX downloads, invalid-edit freshness, no-plan formulation and inert Save/Open passed.');
// A native context can be larger than the exact-certificate input cap. Retain
// its bytes across a tool switch without permitting a larger exact request.
// The mock DOM does not coerce input.value assignments as a browser does.
for(const key of ['k','tolerance','limit','label','chamber','created'])node('column-'+key).value=String(node('column-'+key).value);
const largeColumnText=columnContextText+' '.repeat(3*1024*1024),largeColumnBytes=new TextEncoder().encode(largeColumnText);
await node('exact-source').onchange({target:{files:[{name:'large-grid.rctx',size:largeColumnBytes.length,arrayBuffer:async()=>largeColumnBytes.buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.context_json,largeColumnText);
uiWorker.onmessage({data:{type:'result',result:contextResult}});
node('save-project').onclick();const largeColumnProject=createProject(lastSnapshot);
assert.equal(parseProject(JSON.stringify(largeColumnProject)).files.exactSource.base64,Buffer.from(largeColumnText).toString('base64'));
importRead=()=>Promise.resolve(largeColumnProject);const beforeLargeColumnOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'large-column.bisect'}],value:'large-column.bisect'}});
assert.equal(posted.length,beforeLargeColumnOpen);assert.match(node('result-status').textContent,/unverified/);
for(const key of ['k','tolerance','limit','label','chamber','created'])node('column-'+key).value=String(node('column-'+key).value);
node('execute').onclick();assert.equal(posted.at(-1).request.context_json,largeColumnText);uiWorker.onmessage({data:{type:'result',result:contextResult}});
node('operation').value='solve-branch-price';node('operation').change();const beforeWrongKind=posted.length;
node('execute').onclick();assert.equal(posted.length,beforeWrongKind);assert.match(node('result-status').textContent,/3 MiB/);
node('save-project').onclick();assert.equal(createProject(lastSnapshot).files.exactSource.base64,Buffer.from(largeColumnText).toString('base64'));
node('operation').value='solve-branch-price-context';node('operation').change();
let oversizedColumnRead=false;
await node('exact-source').onchange({target:{files:[{name:'oversized.rctx',size:8*1024*1024+1,arrayBuffer:async()=>{oversizedColumnRead=true;}}]}});
assert.equal(oversizedColumnRead,false);assert.match(node('result-status').textContent,/8 MiB/);
assert.throws(()=>createProject({...lastSnapshot,files:{...lastSnapshot.files,exactSource:{name:'oversized.rctx',base64:Buffer.alloc(8*1024*1024+1,32).toString('base64')}}}),/8 MiB/);
await node('exact-source').onchange({target:{files:[{name:'bad.rctx',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}]}});
assert.match(node('result-status').textContent,/encoded|encoding|UTF|valid/i);
await node('exact-source').onchange({target:{files:[{name:'grid.rctx',size:columnContextBytes.length,arrayBuffer:async()=>columnContextBytes.buffer}]}});
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:contextResult}});
console.log('Large column RCTX: exact >3 MiB chooser/request, inert save/open, retained tool switches, 8 MiB pre-read/project cap and strict UTF-8 passed; exact requests retain 3 MiB.');


uiWorker.onmessage({data:{type:'result',result:{...contextResult,algorithm_lineage_json:JSON.stringify({method:'branch-and-price'})}}});node('use-column-audit').onclick();assert.equal(node('operation').value,'audit-rplan-text');assert.equal(node('text-audit-fields').hidden,false);assert.equal(node('audit-check-fields').hidden,false);node('text-audit-time').value='2026-10-04T00:00:00Z';
const auditProfileBytes=new TextEncoder().encode(JSON.stringify(certificate.legal_profile));await node('auditProfileText-file').onchange({target:{files:[{name:'profile.json',size:auditProfileBytes.length,arrayBuffer:async()=>auditProfileBytes.buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'audit-rplan-text');assert.equal(posted.at(-1).request.document_json,contextResult.document_json);assert.equal(posted.at(-1).request.context_json,columnContextText);assert.equal(posted.at(-1).request.profile_json,new TextDecoder().decode(auditProfileBytes));assert.equal(node('auditPlanText-file').disabled,true);
const auditTextResult={schema_version:'bisect-text-plan-audit-v1',result:'pass',certificate_json:exactText};uiWorker.onmessage({data:{type:'result',result:auditTextResult}});node('download-text-audit').onclick();assert.equal(downloadedLink.download,'exact-audit-certificate.json');assert.equal(await (await fetch(downloadedLink.href)).text(),exactText);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const textAuditProject=createProject(lastSnapshot);assert.equal(textAuditProject.files.auditPlanText.base64,Buffer.from(contextResult.document_json).toString('base64'));assert.equal(textAuditProject.files.auditContextText.base64,Buffer.from(columnContextText).toString('base64'));
node('text-audit-time').value='x'.repeat(201);node('text-audit-time').oninput();assert.equal(node('download-text-audit').disabled,true);
importRead=()=>Promise.resolve(textAuditProject);const beforeTextAuditOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'text-audit.bisect'}],value:'text-audit.bisect'}});assert.equal(posted.length,beforeTextAuditOpen);assert.equal(node('download-text-audit').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:auditTextResult}});node('use-text-audit').onclick();assert.equal(node('operation').value,'verify-audit-text');assert.equal(node('audit-profile-text-field').hidden,true);assert.equal(node('audit-certificate-text-field').hidden,false);node('execute').onclick();assert.equal(posted.at(-1).request.certificate_json,exactText);assert.equal(posted.at(-1).request.context_json,columnContextText);assert.equal(posted.at(-1).request.document_json,contextResult.document_json);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
let oversizedTextAuditRead=false;await node('auditPlanText-file').onchange({target:{files:[{name:'oversized.rplan',size:8*1024*1024+1,arrayBuffer:async()=>{oversizedTextAuditRead=true;return new ArrayBuffer(0);}}]}});assert.equal(oversizedTextAuditRead,false);
const largeAuditPlanText=contextResult.document_json+' '.repeat(3*1024*1024),largeAuditPlanBytes=new TextEncoder().encode(largeAuditPlanText);
await node('auditPlanText-file').onchange({target:{files:[{name:'large-state.rplan',size:largeAuditPlanBytes.length,arrayBuffer:async()=>largeAuditPlanBytes.buffer}]}});
node('operation').value='audit-rplan-text';node('operation').change();node('text-audit-time').value='2026-10-04T00:00:00Z';node('execute').onclick();
assert.equal(posted.at(-1).request.document_json,largeAuditPlanText);uiWorker.onmessage({data:{type:'result',result:auditTextResult}});
node('save-project').onclick();assert.equal(createProject(lastSnapshot).files.auditPlanText.base64,Buffer.from(largeAuditPlanText).toString('base64'));
let oversizedProfileRead=false;await node('auditProfileText-file').onchange({target:{files:[{name:'too-large-profile.json',size:3*1024*1024+1,arrayBuffer:async()=>{oversizedProfileRead=true;return new ArrayBuffer(0);}}]}});assert.equal(oversizedProfileRead,false);
const smallAuditPlanBytes=new TextEncoder().encode(contextResult.document_json);await node('auditPlanText-file').onchange({target:{files:[{name:'small.rplan',size:smallAuditPlanBytes.length,arrayBuffer:async()=>smallAuditPlanBytes.buffer}]}});
console.log('Text audit UI: explicit column-plan adoption, exact file requests/certificate downloads, busy/size guards, freshness invalidation, byte-preserving Save/Open and explicit verification passed.');
const largeContextText=columnContextText+' '.repeat(3*1024*1024),largeContextBytes=new TextEncoder().encode(largeContextText);
await node('auditContextText-file').onchange({target:{files:[{name:'state.rctx',size:largeContextBytes.length,arrayBuffer:async()=>largeContextBytes.buffer}]}});
node('operation').value='audit-rplan-text';node('operation').change();node('text-audit-time').value='2026-10-04T00:00:00Z';
node('execute').onclick();assert.equal(posted.at(-1).request.context_json,largeContextText);
uiWorker.onmessage({data:{type:'result',result:auditTextResult}});
let oversizedContextRead=false;await node('auditContextText-file').onchange({target:{files:[{name:'too-large.rctx',size:8*1024*1024+1,arrayBuffer:async()=>{oversizedContextRead=true;return new ArrayBuffer(0);}}]}});
assert.equal(oversizedContextRead,false);
// Subsequent fixtures retain the original small context.
await node('auditContextText-file').onchange({target:{files:[{name:'grid.rctx',size:columnContextBytes.length,arrayBuffer:async()=>columnContextBytes.buffer}]}});
console.log('Large RPLAN/RCTX UI: exact >3 MiB requests and 8 MiB pre-read bounds passed; profile/certificate/lineage inputs retain 3 MiB.');

const improvementFixture={operation:'improve-rplan-text',document_json:contextResult.document_json,context_json:columnContextText,settings:{tolerance_percent:34,label:'Improvement fixture',generated_at_utc:'2026-10-04T00:00:00Z'}};
const improvementOutput={schema_version:'bisect-local-improvement-output-v1',files:{'improved.rplan':contextResult.document_json,'improved.rctx':columnContextText,'local-search-summary.json':exactText,'audit-certificate.json':exactText,'manifest.json':exactText}};
node('operation').value='improve-rplan-text';node('operation').change();assert.equal(node('improvement-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,false);assert.equal(node('audit-profile-text-field').hidden,true);
for(const [key,text] of [['auditPlanText',improvementFixture.document_json],['auditContextText',improvementFixture.context_json]]){const bytes=new TextEncoder().encode(text);await node(key+'-file').onchange({target:{files:[{name:key+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer}]}});}
node('improvement-tolerance').value='34';node('improvement-label').value='Improvement fixture';node('improvement-time').value='2026-10-04T00:00:00Z';node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),improvementFixture);assert.equal(node('improvement-tolerance').disabled,true);
uiWorker.onmessage({data:{type:'result',result:improvementOutput}});
for(const [name,text] of Object.entries(improvementOutput.files)){node('improvement-output').value=name;node('download-improvement').onclick();assert.equal(downloadedLink.download,name);assert.equal(await (await fetch(downloadedLink.href)).text(),text);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const improvementProject=createProject(lastSnapshot);assert.equal(improvementProject.files.improvementSettings.tolerance_percent,34);
node('project-name').oninput();assert.equal(node('download-improvement').disabled,false);
node('improvement-tolerance').value='';node('improvement-tolerance').oninput();assert.equal(node('download-improvement').disabled,true);assert.equal(node('verify-improvement').disabled,true);
importRead=()=>Promise.resolve(improvementProject);const beforeImprovementOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'improvement.bisect'}],value:'improvement.bisect'}});assert.equal(posted.length,beforeImprovementOpen);assert.equal(node('download-improvement').disabled,true);assert.equal(node('improvement-tolerance').value,34);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:improvementOutput}});let largeImprovementRead=false;await node('auditPlanText-file').onchange({target:{files:[{name:'oversized.rplan',size:3*1024*1024+1,arrayBuffer:async()=>{largeImprovementRead=true;}}]}});assert.equal(largeImprovementRead,false);assert.equal(node('download-improvement').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:improvementOutput}});node('verify-improvement').onclick();assert.equal(node('operation').value,'verify-audit-text');node('execute').onclick();assert.equal(posted.at(-1).request.document_json,improvementOutput.files['improved.rplan']);assert.equal(posted.at(-1).request.context_json,improvementOutput.files['improved.rctx']);assert.equal(posted.at(-1).request.certificate_json,improvementOutput.files['audit-certificate.json']);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
console.log('Improvement UI: exact requests, all five native downloads, busy guard, edit freshness, oversized pre-read rejection, inert Save/Open and explicit certificate verification passed.');

node('operation').value='sample-smc-context';node('operation').change();assert.equal(node('smc-fields').hidden,false);assert.equal(node('auditPlanText-file').parentElement.hidden,true);
for(const [key,value] of Object.entries({k:'3',particles:'32',tolerance:'10',resample:'0.5',seed:'18446744073709551615'}))node('smc-'+key).value=value;
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'sample-smc-context');assert.equal(posted.at(-1).request.context_json,improvementOutput.files['improved.rctx']);assert.equal(posted.at(-1).request.settings.base_seed,'18446744073709551615');assert.equal(node('smc-seed').disabled,true);
const smcOutput={schema_version:'bisect-smc-context-output-v1',base_seed:'18446744073709551615',ndjson:'{"base_seed":18446744073709551615}\n'};
uiWorker.onmessage({data:{type:'result',result:smcOutput}});node('download-smc').onclick();assert.equal(downloadedLink.download,'smc-ensemble.ndjson');assert.equal(await (await fetch(downloadedLink.href)).text(),smcOutput.ndjson);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const smcProject=createProject(lastSnapshot);assert.equal(smcProject.files.smcSettings.base_seed,'18446744073709551615');
node('project-name').oninput();assert.equal(node('download-smc').disabled,false);node('smc-seed').value='01';node('smc-seed').oninput();assert.equal(node('download-smc').disabled,true);
importRead=()=>Promise.resolve(smcProject);const beforeSmcOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'ensemble.bisect'}],value:'ensemble.bisect'}});assert.equal(posted.length,beforeSmcOpen);assert.equal(node('download-smc').disabled,true);assert.equal(node('smc-seed').value,'18446744073709551615');assert.match(node('result-status').textContent,/unverified/);
console.log('SMC UI: exact context request, visible lossless seed, busy/settings guards, exact NDJSON download, rename preservation and inert project reopening passed.');

node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:smcOutput}});node('use-smc').onclick();assert.equal(node('operation').value,'select-smc-plan');assert.equal(node('smc-selection-fields').hidden,false);node('smc-selection-index').value='0';node('smc-selection-label').value='Sample 0';node('smc-selection-chamber').value='congressional';node('smc-selection-created').value='2026-10-04T00:00:00Z';node('execute').onclick();assert.equal(posted.at(-1).request.ensemble_ndjson,smcOutput.ndjson);assert.equal(posted.at(-1).request.settings.particle_index,0);assert.equal(node('smc-selection-index').disabled,true);
const selectedSmc={schema_version:'bisect-smc-selected-plan-v1',document_json:contextResult.document_json,context_json:columnContextText,algorithm_lineage_json:exactText};uiWorker.onmessage({data:{type:'result',result:selectedSmc}});
for(const [id,field] of [['download-smc-plan','document_json'],['download-smc-lineage','algorithm_lineage_json']]){node(id).onclick();assert.equal(await (await fetch(downloadedLink.href)).text(),selectedSmc[field]);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const selectionProject=createProject(lastSnapshot);node('smc-selection-index').value='';node('smc-selection-index').oninput();assert.equal(node('download-smc-plan').disabled,true);
importRead=()=>Promise.resolve(selectionProject);const beforeSelectionOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'selection.bisect'}],value:'selection.bisect'}});assert.equal(posted.length,beforeSelectionOpen);assert.equal(node('download-smc-plan').disabled,true);assert.equal(node('smc-selection-index').value,0);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:selectedSmc}});assert.equal(node('use-column-audit').disabled,false);node('use-column-audit').onclick();assert.equal(node('operation').value,'audit-rplan-text');node('text-audit-time').value='2026-10-04T00:00:00Z';node('execute').onclick();assert.equal(posted.at(-1).request.document_json,selectedSmc.document_json);assert.equal(posted.at(-1).request.lineage_json,exactText);uiWorker.onmessage({data:{type:'result',result:auditTextResult}});
console.log('SMC selection UI: explicit ensemble adoption, exact replay request, plan/lineage downloads, busy/edit guards, inert project reopening and exact lineage audit handoff passed.');

node('operation').value='validate-rplan-text';node('operation').change();
assert.equal(node('text-audit-fields').hidden,false);
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'validate-rplan-text');
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-text-plan-validation-v1',document_json:'{}'}}});
const mapBytes=new TextEncoder().encode('{"type":"FeatureCollection","features":[]}');
await node('plan-geometry-file').onchange({target:{files:[{name:'units.geojson',size:mapBytes.length,arrayBuffer:async()=>mapBytes.buffer}]}});
node('save-project').onclick();assert.equal(lastSnapshot.files.planGeometry.base64,Buffer.from(mapBytes).toString('base64'));
let oversizedGeometryRead=false;await node('plan-geometry-file').onchange({target:{files:[{name:'big.geojson',size:16*1024*1024+1,arrayBuffer:async()=>{oversizedGeometryRead=true;}}]}});assert.equal(oversizedGeometryRead,false);assert.match(node('result-status').textContent,/16 MiB/);
console.log('Plan map UI: explicit exact validation request, geometry byte persistence and pre-read size guard passed.');

const paretoFixture=await packageTools.paretoRequest({auditContextText:{name:'grid.rctx',base64:Buffer.from(columnContextText).toString('base64')}},{k:3,population:12,generations:5,tolerance:0.2,base_seed:'18446744073709551615',democratic_shares:null,minority_shares:null});
await node('open-project').onchange({target:{files:[{name:'pareto.bisect'}],value:'pareto.bisect'}});
for(const [key,value] of Object.entries({k:3,population:12,generations:5,tolerance:20,seed:'18446744073709551615',democratic:'',minority:''}))node('pareto-'+key).value=String(value);
const paretoContextBytes=new TextEncoder().encode(columnContextText);await node('auditContextText-file').onchange({target:{files:[{name:'grid.rctx',size:paretoContextBytes.length,arrayBuffer:async()=>paretoContextBytes.buffer}]}});
node('operation').value='sample-pareto-context';node('operation').change();assert.equal(node('pareto-fields').hidden,false);assert.equal(node('auditPlanText-file').parentElement.hidden,true);
node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),paretoFixture);assert.equal(node('pareto-seed').disabled,true);
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-pareto-context-output-v1',ndjson:'native frontier bytes\n'}}});assert.equal(node('download-pareto').disabled,false);
node('save-project').onclick();assert.equal(lastSnapshot.files.paretoSettings.base_seed,'18446744073709551615');
node('pareto-generations').value='';node('pareto-generations').input();assert.equal(node('download-pareto').disabled,true);assert.match(node('result-status').textContent,/Supply Pareto/);
console.log('Pareto UI: exact context request, visible u64 seed, busy controls, optional share fields, project retention and invalid-edit download freshness passed.');
node('pareto-generations').value='5';node('pareto-generations').input();node('operation').value='sample-pareto-context';node('execute').onclick();
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-pareto-context-output-v1',ndjson:'native frontier bytes\n'}}});node('use-pareto').onclick();assert.equal(node('operation').value,'select-pareto-plan');assert.equal(node('pareto-selection-fields').hidden,false);assert.equal(node('pareto-frontier-name').textContent,'pareto-frontier.ndjson');
node('pareto-selection-index').value='1';node('pareto-selection-label').value='Selected fixture';node('pareto-selection-time').value='2026-10-06T00:00:00Z';node('execute').onclick();assert.equal(posted.at(-1).request.operation,'select-pareto-plan');assert.equal(posted.at(-1).request.frontier_ndjson,'native frontier bytes\n');assert.equal(posted.at(-1).request.selection.frontier_index,1);assert.equal(node('pareto-frontier-file').disabled,true);
const selectedParetoOutput={schema_version:'bisect-pareto-selected-plan-v1',document_json:contextResult.document_json,context_json:columnContextText,algorithm_lineage_json:'{}',files:{'selected-frontier.rplan':contextResult.document_json,'selected-frontier.rctx':columnContextText,'audit-certificate.json':exactText,'manifest.json':exactText}};
uiWorker.onmessage({data:{type:'result',result:selectedParetoOutput}});
for(const [name,text] of Object.entries(selectedParetoOutput.files)){node('pareto-output').value=name;node('download-pareto-package').onclick();assert.equal(downloadedLink.download,name);assert.equal(await (await fetch(downloadedLink.href)).text(),text);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const selectedParetoProject=createProject(lastSnapshot);assert.equal(selectedParetoProject.files.paretoSelection.frontier_index,1);assert.equal(selectedParetoProject.files.paretoFrontier.base64,Buffer.from('native frontier bytes\n').toString('base64'));assert.equal(node('use-column-audit').disabled,false);
importRead=()=>Promise.resolve(selectedParetoProject);const beforeParetoSelectionOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'selected.bisect'}],value:'selected.bisect'}});assert.equal(posted.length,beforeParetoSelectionOpen);assert.equal(node('download-pareto-package').disabled,true);assert.equal(node('pareto-selection-index').value,1);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:selectedParetoOutput}});node('verify-pareto').onclick();assert.equal(node('operation').value,'verify-audit-text');node('execute').onclick();assert.equal(posted.at(-1).request.document_json,selectedParetoOutput.document_json);assert.equal(posted.at(-1).request.certificate_json,exactText);uiWorker.onmessage({data:{type:'result',result:{verified:true}}});
console.log('Pareto selection UI: explicit frontier adoption, exact replay request, four native downloads, busy controls, exact project retention, inert reopening and certificate handoff passed.');

// Interchange files must come from an explicit native run, including after reopening.
node('operation').value='validate-rplan-text';node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-text-plan-validation-v1',document_json:contextResult.document_json}}});
assert.equal(node('use-interchange').disabled,false);node('use-interchange').onclick();assert.equal(node('operation').value,'export-rplan-interchange');assert.equal(node('auditContextText-file').parentElement.hidden,true);assert.equal(node('interchange-fields').hidden,false);
node('execute').onclick();assert.equal(posted.at(-1).request.document_json,contextResult.document_json);assert.equal(posted.at(-1).request.operation,'export-rplan-interchange');assert.equal(node('interchange-output').disabled,true);
const interchangeOutput={schema_version:'bisect-plan-interchange-v1',document_json:contextResult.document_json,files:{'assignments.csv':'geoid,district\nu1,1\n','gerrychain.json':'{"assignment":{"u1":1}}','plan.geojson':'{"type":"FeatureCollection","features":[]}'}};
uiWorker.onmessage({data:{type:'result',result:interchangeOutput}});
for(const [name,text] of Object.entries(interchangeOutput.files)){node('interchange-output').value=name;node('download-interchange').onclick();assert.equal(downloadedLink.download,name);assert.equal(await (await fetch(downloadedLink.href)).text(),text);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const interchangeProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(interchangeProject);const beforeInterchangeOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'export.bisect'}],value:'export.bisect'}});assert.equal(posted.length,beforeInterchangeOpen);assert.equal(node('download-interchange').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:interchangeOutput}});node('operation').value='validate-rplan-text';node('operation').change();assert.equal(node('download-interchange').disabled,true);
console.log('Interchange UI: exact source/adoption, three byte-exact downloads, busy controls, context hiding, inert Open and stale export rejection passed.');

node('operation').value='import-plan-interchange';node('operation').change();assert.equal(node('plan-import-fields').hidden,false);assert.equal(node('auditPlanText-file').parentElement.hidden,true);assert.equal(node('auditContextText-file').parentElement.hidden,false);
for(const [key,value] of Object.entries({format:'gerrychain',state:'WA',year:'2020',label:'Imported civic plan',time:'2026-10-06T00:00:00Z',submitter:'Fixture organization',submitted:'2026-10-06T00:00:00Z'}))node('plan-import-'+key).value=value;node('plan-import-civic').checked=true;
const externalText='{"assignment":{"53001000100":1}}',externalBytes=new TextEncoder().encode(externalText);await node('plan-import-source').onchange({target:{files:[{name:'external.json',size:externalBytes.length,arrayBuffer:async()=>externalBytes.buffer}]}});
node('execute').onclick();const externalRequest=posted.at(-1).request;assert.equal(externalRequest.operation,'import-plan-interchange');assert.equal(externalRequest.source_text,externalText);assert.equal(externalRequest.settings.submitted_by,'Fixture organization');assert.equal(externalRequest.settings.as_civic_counter_proposal,true);assert.equal(node('plan-import-source').disabled,true);
const externalOutput={schema_version:'bisect-plan-import-v1',document_json:contextResult.document_json,files:{'imported.rplan':contextResult.document_json,'legacy.rplan':exactText,'final_assignments.json':exactText,'manifest.json':exactText}};uiWorker.onmessage({data:{type:'result',result:externalOutput}});
for(const [name,text] of Object.entries(externalOutput.files)){node('plan-import-output').value=name;node('download-plan-import').onclick();assert.equal(downloadedLink.download,name);assert.equal(await(await fetch(downloadedLink.href)).text(),text);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const externalProject=createProject(lastSnapshot);assert.equal(externalProject.files.planImportSource.base64,Buffer.from(externalText).toString('base64'));assert.equal(externalProject.files.planImportSettings.submitted_by,'Fixture organization');importRead=()=>Promise.resolve(externalProject);const beforeExternalOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'external.bisect'}],value:'external.bisect'}});assert.equal(posted.length,beforeExternalOpen);assert.equal(node('download-plan-import').disabled,true);assert.equal(node('plan-import-civic').checked,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:externalOutput}});node('plan-import-label').value='';node('plan-import-label').oninput();assert.equal(node('download-plan-import').disabled,true);node('plan-import-label').value='External plan';node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:externalOutput}});let externalOversizeRead=false;await node('plan-import-source').onchange({target:{files:[{name:'huge.json',size:3*1024*1024+1,arrayBuffer:async()=>{externalOversizeRead=true;}}]}});assert.equal(externalOversizeRead,false);assert.equal(node('download-plan-import').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:externalOutput}});node('audit-plan-import').onclick();assert.equal(node('operation').value,'audit-rplan-text');node('execute').onclick();assert.equal(posted.at(-1).request.document_json,externalOutput.document_json);uiWorker.onmessage({data:{type:'result',result:{result:'pass'}}});
console.log('Plan import UI: exact source/settings, four exact downloads, civic retention, busy controls, context panel, inert Open, invalid/oversized freshness and explicit audit adoption passed.');

node('operation').value='import-plan-interchange';node('operation').change();
node('plan-import-format').value='geojson-polygons';node('plan-import-format').oninput();
assert.equal(node('plan-import-centroid-field').hidden,false);
const centroidText='{"53001000100":[-122.5,47.5]}',centroidBytes=new TextEncoder().encode(centroidText);
await node('plan-import-centroids').onchange({target:{files:[{name:'centroids.json',size:centroidBytes.length,arrayBuffer:async()=>centroidBytes.buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.centroids_json,centroidText);assert.equal(node('plan-import-centroids').disabled,true);
uiWorker.onmessage({data:{type:'result',result:externalOutput}});
node('save-project').onclick();const polygonProject=createProject(lastSnapshot);
assert.equal(polygonProject.files.planImportCentroids.base64,Buffer.from(centroidText).toString('base64'));
importRead=()=>Promise.resolve(polygonProject);const beforePolygonOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'polygon.bisect'}],value:'polygon.bisect'}});
assert.equal(posted.length,beforePolygonOpen);assert.equal(node('download-plan-import').disabled,true);
assert.equal(node('plan-import-centroid-field').hidden,false);
node('execute').onclick();assert.equal(posted.at(-1).request.centroids_json,centroidText);uiWorker.onmessage({data:{type:'result',result:externalOutput}});
let centroidOversizeRead=false;await node('plan-import-centroids').onchange({target:{files:[{name:'huge.json',size:3*1024*1024+1,arrayBuffer:async()=>{centroidOversizeRead=true;}}]}});
assert.equal(centroidOversizeRead,false);assert.equal(node('download-plan-import').disabled,true);
node('plan-import-format').value='gerrychain';node('plan-import-format').oninput();assert.equal(node('plan-import-centroid-field').hidden,true);
console.log('Polygon import UI: explicit centroid request, busy controls, exact Save/Open retention, inert reopening, conditional field and pre-read size guard passed.');

node('operation').value='export-reproducibility-package';node('operation').change();assert.equal(node('reproducibility-fields').hidden,false);assert.equal(node('auditContextText-file').parentElement.hidden,true);
node('reproducibility-label').value='fixture';node('reproducibility-label').oninput();
const reproManifest='{ "recorded": true }\n',reproManifestBytes=new TextEncoder().encode(reproManifest);
await node('reproducibility-manifest').onchange({target:{files:[{name:'manifest.json',size:reproManifestBytes.length,arrayBuffer:async()=>reproManifestBytes.buffer}]}});
const reproAnalysis='{ "supplied": true }\n',reproAnalysisBytes=new TextEncoder().encode(reproAnalysis);
await node('reproducibility-analysis').onchange({target:{files:[{name:'population.json',size:reproAnalysisBytes.length,arrayBuffer:async()=>reproAnalysisBytes.buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.manifest_json,reproManifest);assert.deepEqual(posted.at(-1).request.analysis_files,{'population.json':reproAnalysis});assert.equal(node('reproducibility-analysis').disabled,true);
const reproOutput={schema_version:'bisect-reproducibility-package-v1',files:{'fixture.rplan':'{}','audit.json':'{}','verify.sh':'# unavailable','manifest.json':reproManifest,'analysis/population.json':reproAnalysis},file_sha256:{}};
uiWorker.onmessage({data:{type:'result',result:reproOutput}});node('download-reproducibility').onclick();assert.equal(downloadedLink.download,'bisect-reproducibility.zip');const reproZip=new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer());assert.equal(new DataView(reproZip.buffer).getUint32(0,true),0x04034b50);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const reproProject=createProject(lastSnapshot);assert.equal(reproProject.files.reproducibilityInputs.manifest.base64,Buffer.from(reproManifest).toString('base64'));
importRead=()=>Promise.resolve(reproProject);const beforeReproOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'repro.bisect'}],value:'repro.bisect'}});assert.equal(posted.length,beforeReproOpen);assert.equal(node('download-reproducibility').disabled,true);assert.equal(node('reproducibility-label').value,'fixture');assert.match(node('reproducibility-analysis-name').textContent,/population.json/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:reproOutput}});node('reproducibility-label').value='../bad';node('reproducibility-label').oninput();assert.equal(node('download-reproducibility').disabled,true);node('reproducibility-label').value='fixture';node('reproducibility-label').oninput();
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:reproOutput}});let reproLargeRead=false;await node('reproducibility-analysis').onchange({target:{files:[{name:'large.json',size:3*1024*1024+1,arrayBuffer:async()=>{reproLargeRead=true;}}]}});assert.equal(reproLargeRead,false);assert.equal(node('download-reproducibility').disabled,true);
const delayedRepro=deferred();const delayedRead=node('reproducibility-manifest').onchange({target:{files:[{name:'old.json',size:reproManifestBytes.length,arrayBuffer:()=>delayedRepro.promise}]}});node('project-name').value='Changed during read';node('project-name').oninput();delayedRepro.resolve(reproManifestBytes.buffer);await delayedRead;node('save-project').onclick();assert.equal(lastSnapshot.files.reproducibilityInputs.manifest.name,'manifest.json');
console.log('Reproducibility UI: exact manifest/analysis request, busy controls, ZIP download, byte-preserving Save/Open, inert reopening, invalid-edit freshness, pre-read size and stale-read guards passed.');

node('operation').value='validate-research-ensemble';node('operation').change();assert.equal(node('research-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,true);
node('research-rhat').value='1.05';node('research-ess').value='1';node('research-rhat').oninput();
const researchFixture={base:{operation:'validate-research-ensemble',chains_jsonl:Object.fromEntries(Array.from({length:4},(_,i)=>['chain-'+i+'.jsonl','{"metrics":{"x":2}}\n'.repeat(4)])),enacted_json:'{"x":2}',rhat_threshold:1.05,ess_min:1,strict:false},output:{schema_version:'bisect-research-ensemble-output-v1',report_json:'{"schema_version":"validate-ensemble v1","n_chains":4,"n_per_chain":4,"all_pass":true}'}};
const chainFiles=Object.entries(researchFixture.base.chains_jsonl).map(([name,text])=>{const bytes=new TextEncoder().encode(text);return {name,size:bytes.length,arrayBuffer:async()=>bytes.buffer};});
await node('research-chains').onchange({target:{files:chainFiles.slice(0,2)}});node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.researchInputs.chains).length,2);const beforeFewChains=posted.length;node('execute').onclick();assert.equal(posted.length,beforeFewChains);assert.match(node('result-status').textContent,/four/);
await node('research-chains').onchange({target:{files:chainFiles}});const enactedBytes=new TextEncoder().encode(researchFixture.base.enacted_json);await node('research-enacted').onchange({target:{files:[{name:'enacted.json',size:enactedBytes.length,arrayBuffer:async()=>enactedBytes.buffer}]}});
node('execute').onclick();assert.deepEqual(posted.at(-1).request,researchFixture.base);assert.equal(node('research-chains').disabled,true);assert.equal(node('research-rhat').disabled,true);
uiWorker.onmessage({data:{type:'result',result:researchFixture.output}});node('download-research').onclick();assert.equal(downloadedLink.download,'validate-ensemble.json');assert.equal(await(await fetch(downloadedLink.href)).text(),researchFixture.output.report_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const researchProject=createProject(lastSnapshot);assert.equal(researchProject.files.researchInputs.enacted.base64,Buffer.from(researchFixture.base.enacted_json).toString('base64'));
importRead=()=>Promise.resolve(researchProject);const beforeResearchOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'research.bisect'}],value:'research.bisect'}});assert.equal(posted.length,beforeResearchOpen);assert.equal(node('download-research').disabled,true);assert.equal(node('research-ess').value,1);assert.match(node('research-chains-name').textContent,/chain-3.jsonl/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:researchFixture.output}});node('research-rhat').value='';node('research-rhat').oninput();assert.equal(node('download-research').disabled,true);node('research-rhat').value='1.05';node('research-rhat').oninput();
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:researchFixture.output}});let researchLargeRead=false;await node('research-chains').onchange({target:{files:[{name:'huge.jsonl',size:3*1024*1024+1,arrayBuffer:async()=>{researchLargeRead=true;}}]}});assert.equal(researchLargeRead,false);assert.equal(node('download-research').disabled,true);
const delayedResearch=deferred();const researchRead=node('research-enacted').onchange({target:{files:[{name:'old-enacted.json',size:enactedBytes.length,arrayBuffer:()=>delayedResearch.promise}]}});node('project-name').value='Changed during research read';node('project-name').oninput();delayedResearch.resolve(enactedBytes.buffer);await researchRead;node('save-project').onclick();assert.equal(lastSnapshot.files.researchInputs.enacted.name,'enacted.json');
await node('research-enacted').onchange({target:{files:[]}});node('execute').onclick();assert.equal(posted.at(-1).request.enacted_json,null);uiWorker.onmessage({data:{type:'result',result:researchFixture.output}});
console.log('Research UI: partial source retention, four-chain Run gate, exact requests/report downloads, optional enacted removal, threshold/busy controls, inert Save/Open, invalid-edit freshness, pre-read size and stale-read guards passed.');

node('operation').value='validate-suite-nesting';node('operation').change();node('suite-name').value='fixture';node('suite-ratio').value='';node('suite-name').oninput();
assert.equal(node('suite-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,true);
const suiteSource=(name,text)=>{const bytes=new TextEncoder().encode(text);return {name,size:bytes.length,arrayBuffer:async()=>bytes.buffer};};
const suiteOutput={schema_version:'bisect-suite-nesting-output-v1',report:{suite_name:'fixture',valid:true,nesting:{senate_to_house_map:{'1':[1,2]},violations:[]}},required_ratio:2,ratio_explicit:false,report_json:'{"suite_name":"fixture","valid":true}'};
await node('suite-house').onchange({target:{files:[suiteSource('house.rplan','{"house":1}')]}});node('save-project').onclick();assert.equal(lastSnapshot.files.suiteInputs.senate,null);
const beforeSuiteMissing=posted.length;node('execute').onclick();assert.equal(posted.length,beforeSuiteMissing);assert.match(node('result-status').textContent,/both/);
await node('suite-senate').onchange({target:{files:[suiteSource('senate.rplan','{"senate":2}')]}});node('execute').onclick();assert.deepEqual(posted.at(-1).request,{operation:'validate-suite-nesting',house_json:'{"house":1}',senate_json:'{"senate":2}',suite_name:'fixture',required_ratio:null});assert.equal(node('suite-ratio').disabled,true);
uiWorker.onmessage({data:{type:'result',result:suiteOutput}});node('download-suite').onclick();assert.equal(downloadedLink.download,'validate-suite.json');assert.equal(await(await fetch(downloadedLink.href)).text(),suiteOutput.report_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const suiteProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(suiteProject);const beforeSuiteOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'suite.bisect'}],value:'suite.bisect'}});assert.equal(posted.length,beforeSuiteOpen);assert.equal(node('download-suite').disabled,true);assert.equal(node('suite-house-name').textContent,'house.rplan');assert.equal(node('suite-ratio').value,'');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:suiteOutput}});node('suite-ratio').value='1';node('suite-ratio').oninput();assert.equal(node('download-suite').disabled,true);assert.equal(node('inspect-suite-house').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.required_ratio,1);uiWorker.onmessage({data:{type:'result',result:suiteOutput}});
node('suite-ratio').value='0';node('suite-ratio').oninput();assert.equal(node('download-suite').disabled,true);node('suite-ratio').value='';node('suite-ratio').oninput();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:suiteOutput}});
let largeSuiteRead=false;await node('suite-house').onchange({target:{files:[{name:'large.rplan',size:3*1024*1024+1,arrayBuffer:async()=>{largeSuiteRead=true;}}]}});assert.equal(largeSuiteRead,false);assert.equal(node('download-suite').disabled,true);
const delayedSuite=deferred();const pendingSuite=node('suite-house').onchange({target:{files:[{name:'old.rplan',size:11,arrayBuffer:()=>delayedSuite.promise}]}});node('suite-name').value='changed';node('suite-name').oninput();delayedSuite.resolve(new TextEncoder().encode('{"house":1}').buffer);await pendingSuite;node('save-project').onclick();assert.equal(lastSnapshot.files.suiteInputs.house.name,'house.rplan');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:suiteOutput}});const beforeSuiteAdoption=posted.length;node('inspect-suite-house').onclick();assert.equal(posted.length,beforeSuiteAdoption);assert.equal(node('operation').value,'validate-rplan-text');assert.equal(node('download-suite').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.document_json,'{"house":1}');uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-text-plan-validation-v1'}}});
console.log('Suite UI: partial retention, exact request/download, ratio and busy controls, inert Open, freshness, pre-read size, stale reads and explicit chamber inspection passed.');

node('operation').value='analyze-suite-stability';node('operation').change();assert.equal(node('stability-fields').hidden,false);assert.equal(node('suite-fields').hidden,true);assert.equal(node('text-audit-fields').hidden,true);
const stabilityFiles=[suiteSource('a.rplan','{"a":1}'),suiteSource('b.rplan','{"b":2}')];
await node('stability-plans').onchange({target:{files:stabilityFiles.slice(0,1)}});node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.stabilityInputs.plans).length,1);const beforeFewPlans=posted.length;node('execute').onclick();assert.equal(posted.length,beforeFewPlans);assert.match(node('result-status').textContent,/two/);
await node('stability-plans').onchange({target:{files:stabilityFiles}});node('execute').onclick();assert.deepEqual(posted.at(-1).request,{operation:'analyze-suite-stability',plans_json:{'a.rplan':'{"a":1}','b.rplan':'{"b":2}'}});assert.equal(node('stability-plans').disabled,true);
const stabilityOutput={schema_version:'bisect-suite-stability-output-v1',report_json:'{"num_plans":2,"num_pairs":1,"stable":true}'};uiWorker.onmessage({data:{type:'result',result:stabilityOutput}});node('download-stability').onclick();assert.equal(downloadedLink.download,'stability.json');assert.equal(await(await fetch(downloadedLink.href)).text(),stabilityOutput.report_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const stabilityProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(stabilityProject);const beforeStabilityOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'stability.bisect'}],value:'stability.bisect'}});assert.equal(posted.length,beforeStabilityOpen);assert.equal(node('download-stability').disabled,true);assert.match(node('stability-plans-name').textContent,/b.rplan/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:stabilityOutput}});let largeStabilityRead=false;await node('stability-plans').onchange({target:{files:[{name:'large.rplan',size:3*1024*1024+1,arrayBuffer:async()=>{largeStabilityRead=true;}}]}});assert.equal(largeStabilityRead,false);assert.equal(node('download-stability').disabled,true);
const delayedStability=deferred();const pendingStability=node('stability-plans').onchange({target:{files:[{name:'old.rplan',size:7,arrayBuffer:()=>delayedStability.promise}]}});node('project-name').value='changed during stability read';node('project-name').oninput();delayedStability.resolve(new TextEncoder().encode('{"a":1}').buffer);await pendingStability;node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.stabilityInputs.plans).length,2);assert.equal(lastSnapshot.files.stabilityInputs.plans['old.rplan'],undefined);
let duplicatePlanRead=false;await node('stability-plans').onchange({target:{files:[{...stabilityFiles[0],arrayBuffer:async()=>{duplicatePlanRead=true;}},stabilityFiles[0]]}});assert.equal(duplicatePlanRead,false);assert.match(node('result-status').textContent,/unique/);
console.log('Stability UI: partial files, two-plan Run gate, exact requests/native downloads, busy controls, inert Open, freshness, oversized/duplicate pre-read rejection and stale reads passed.');

node('operation').value='analyze-plan-election';node('operation').change();node('election-analysis-seed').value='42';node('election-analysis-replicates').value='1000';assert.equal(node('election-analysis-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,false);assert.equal(node('auditContextText-file').parentElement.hidden,true);assert.equal(node('download-text-audit').hidden,true);
await node('auditPlanText-file').onchange({target:{files:[suiteSource('election-plan.rplan','{"plan":1}')]}});const beforeElectionMissing=posted.length;node('execute').onclick();assert.equal(posted.length,beforeElectionMissing);assert.match(node('result-status').textContent,/election CSV/);
const electionText='geoid,dem_votes,rep_votes\nu1,10.5,2\n';await node('election-analysis-source').onchange({target:{files:[suiteSource('election.csv',electionText)]}});node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'analyze-plan-election',document_json:'{"plan":1}',election_csv:electionText,settings:{bootstrap_seed:'42',bootstrap_replicates:1000}});assert.equal(node('election-analysis-source').disabled,true);
const electionOutput={schema_version:'bisect-plan-election-analysis-v1',files:{'political.json':'{"political":true}','proportionality.json':'{"proportionality":true}'}};uiWorker.onmessage({data:{type:'result',result:electionOutput}});
for(const kind of ['political','proportionality']){node('download-election-'+kind).onclick();assert.equal(downloadedLink.download,kind+'.json');assert.equal(await(await fetch(downloadedLink.href)).text(),electionOutput.files[kind+'.json']);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const electionProject=createProject(lastSnapshot);assert.equal(electionProject.files.electionAnalysisSource.base64,Buffer.from(electionText).toString('base64'));importRead=()=>Promise.resolve(electionProject);const beforeElectionOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'election.bisect'}],value:'election.bisect'}});assert.equal(posted.length,beforeElectionOpen);assert.equal(node('download-election-political').disabled,true);assert.equal(node('election-analysis-name').textContent,'election.csv');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:electionOutput}});let hugeElectionRead=false;await node('election-analysis-source').onchange({target:{files:[{name:'huge.csv',size:3*1024*1024+1,arrayBuffer:async()=>{hugeElectionRead=true;}}]}});assert.equal(hugeElectionRead,false);assert.equal(node('download-election-political').disabled,true);
const delayedElectionAnalysis=deferred();const pendingElectionAnalysis=node('election-analysis-source').onchange({target:{files:[{name:'old.csv',size:Buffer.byteLength(electionText),arrayBuffer:()=>delayedElectionAnalysis.promise}]}});node('project-name').value='changed while reading election analysis';node('project-name').oninput();delayedElectionAnalysis.resolve(new TextEncoder().encode(electionText).buffer);await pendingElectionAnalysis;node('save-project').onclick();assert.equal(lastSnapshot.files.electionAnalysisSource.name,'election.csv');
await node('election-analysis-source').onchange({target:{files:[]}});const beforeElectionCleared=posted.length;node('execute').onclick();assert.equal(posted.length,beforeElectionCleared);assert.equal(node('download-election-proportionality').disabled,true);
console.log('Election analysis UI: exact plan/CSV request, both native downloads, busy controls, inert byte-preserving Save/Open, missing-source gate, freshness and pre-read/stale guards passed.');

node('election-analysis-seed').value='18446744073709551615';node('election-analysis-replicates').value='64';node('election-analysis-seed').oninput();await node('election-analysis-source').onchange({target:{files:[suiteSource('election.csv',electionText)]}});node('execute').onclick();assert.equal(posted.at(-1).request.settings.bootstrap_seed,'18446744073709551615');assert.equal(node('election-analysis-seed').disabled,true);assert.equal(node('election-analysis-replicates').disabled,true);
const partisanUiOutput={...electionOutput,files:{...electionOutput.files,'partisan-metrics.json':'{"bootstrap_seed":"18446744073709551615","bootstrap_protocol":"chacha12-u64-index-v1"}','partisan.json':'{"analyzer":"partisan","available":true}'}};uiWorker.onmessage({data:{type:'result',result:partisanUiOutput}});node('download-election-partisan-metrics').onclick();assert.equal(downloadedLink.download,'partisan-metrics.json');assert.equal(await(await fetch(downloadedLink.href)).text(),partisanUiOutput.files['partisan-metrics.json']);URL.revokeObjectURL(downloadedLink.href);
node('download-election-partisan').onclick();assert.equal(downloadedLink.download,'partisan.json');assert.equal(await(await fetch(downloadedLink.href)).text(),partisanUiOutput.files['partisan.json']);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const partisanUiProject=createProject(lastSnapshot);assert.equal(partisanUiProject.files.electionAnalysisSettings.bootstrap_seed,'18446744073709551615');importRead=()=>Promise.resolve(partisanUiProject);const beforePartisanOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'partisan.bisect'}],value:'partisan.bisect'}});assert.equal(posted.length,beforePartisanOpen);assert.equal(node('election-analysis-seed').value,'18446744073709551615');assert.equal(node('election-analysis-replicates').value,64);assert.equal(node('download-election-partisan-metrics').disabled,true);assert.equal(node('download-election-partisan').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:partisanUiOutput}});node('election-analysis-seed').value='18446744073709551616';node('election-analysis-seed').oninput();assert.equal(node('download-election-partisan-metrics').disabled,true);assert.equal(node('download-election-partisan').disabled,true);node('election-analysis-seed').value='42';node('election-analysis-replicates').value='';node('election-analysis-replicates').oninput();const beforeBadReplicates=posted.length;node('execute').onclick();assert.equal(posted.length,beforeBadReplicates);assert.match(node('result-status').textContent,/replicate count/);
console.log('Partisan UI: visible maximum-u64 seed, replicates, exact metric download, busy controls, lossless inert Save/Open and invalid-setting freshness passed.');

node('operation').value='analyze-plan-statistics';node('operation').change();node('statistics-tolerance').value='0.5';assert.equal(node('statistics-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,false);assert.equal(node('auditContextText-file').parentElement.hidden,true);
await node('auditPlanText-file').onchange({target:{files:[suiteSource('statistics.rplan','{"statistics":1}')]}});const beforeNoStatistics=posted.length;node('execute').onclick();assert.equal(posted.length,beforeNoStatistics);assert.match(node('result-status').textContent,/at least one/);
const demoUiText='GEOID,total_pop,white_non_hispanic,black_non_hispanic,asian_non_hispanic,hispanic,other\nu1,9007199254740993,9007199254740993,0,0,0,0\n';
await node('statistics-demographic').onchange({target:{files:[suiteSource('demographic.csv',demoUiText)]}});node('execute').onclick();assert.deepEqual(posted.at(-1).request,{operation:'analyze-plan-statistics',document_json:'{"statistics":1}',demographic_csv:demoUiText,urban_csv:null,election_csv:null,tolerance_percent:0.5});assert.equal(node('statistics-urban').disabled,true);assert.equal(node('statistics-tolerance').disabled,true);
const statisticsOutput={schema_version:'bisect-plan-statistics-v1',files:{'summary.json':'{"total_pop":9007199254740993}','demographic.json':'{"total_pop":9007199254740993}'}};uiWorker.onmessage({data:{type:'result',result:statisticsOutput}});assert.equal(node('download-statistics-urban').disabled,true);
for(const kind of ['summary','demographic']){node('download-statistics-'+kind).onclick();assert.equal(downloadedLink.download,kind+'.json');assert.equal(await(await fetch(downloadedLink.href)).text(),statisticsOutput.files[kind+'.json']);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const statisticsProject=createProject(lastSnapshot);assert.equal(statisticsProject.files.statisticsInputs.demographic.base64,Buffer.from(demoUiText).toString('base64'));importRead=()=>Promise.resolve(statisticsProject);const beforeStatisticsOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'statistics.bisect'}],value:'statistics.bisect'}});assert.equal(posted.length,beforeStatisticsOpen);assert.equal(node('download-statistics-summary').disabled,true);assert.equal(node('statistics-tolerance').value,0.5);assert.equal(node('statistics-demographic-name').textContent,'demographic.csv');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:statisticsOutput}});node('statistics-tolerance').value='';node('statistics-tolerance').oninput();assert.equal(node('download-statistics-summary').disabled,true);node('statistics-tolerance').value='5';node('statistics-tolerance').oninput();
let hugeStatsRead=false;await node('statistics-urban').onchange({target:{files:[{name:'huge.csv',size:3*1024*1024+1,arrayBuffer:async()=>{hugeStatsRead=true;}}]}});assert.equal(hugeStatsRead,false);
const delayedStats=deferred();const pendingStats=node('statistics-demographic').onchange({target:{files:[{name:'old.csv',size:Buffer.byteLength(demoUiText),arrayBuffer:()=>delayedStats.promise}]}});node('statistics-tolerance').value='10';node('statistics-tolerance').oninput();delayedStats.resolve(new TextEncoder().encode(demoUiText).buffer);await pendingStats;node('save-project').onclick();assert.equal(lastSnapshot.files.statisticsInputs.demographic.name,'demographic.csv');
await node('statistics-urban').onchange({target:{files:[suiteSource('urban.csv','GEOID,place_name,place_pop\nu1,Place,100\n')]}});await node('statistics-demographic').onchange({target:{files:[]}});node('execute').onclick();assert.equal(posted.at(-1).request.demographic_csv,null);assert.match(posted.at(-1).request.urban_csv,/Place/);assert.equal(posted.at(-1).request.tolerance_percent,10);uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-plan-statistics-v1',files:{'summary.json':'{}','urban.json':'{}'}}}});
console.log('Statistics UI: optional sources, exact i64 source/report bytes, request/busy controls, available-only downloads, inert Save/Open, tolerance freshness, pre-read size/stale guards and sparse-source run passed.');

assert.equal(node('remove-statistics-urban').disabled,false);node('remove-statistics-urban').onclick();assert.equal(node('download-statistics-summary').disabled,true);assert.equal(node('remove-statistics-urban').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.statisticsInputs.urban,null);
console.log('Statistics source removal: explicit removal clears retained bytes and invalidates reports.');

node('operation').value='analyze-plan-compactness';node('operation').change();node('compactness-basis').value='projected-metres';node('compactness-crs').value='Synthetic metre fixture';assert.equal(node('compactness-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,false);assert.equal(node('auditContextText-file').parentElement.hidden,true);
await node('auditPlanText-file').onchange({target:{files:[suiteSource('compactness.rplan','{"geometry-plan":1}')]}});const beforeMissingGeometry=posted.length;node('execute').onclick();assert.equal(posted.length,beforeMissingGeometry);assert.match(node('result-status').textContent,/unit geometry/);
const geometryUiText='{"type":"FeatureCollection","features":[]}\n';await node('compactness-source').onchange({target:{files:[suiteSource('units.json',geometryUiText)]}});node('execute').onclick();assert.deepEqual(posted.at(-1).request,{operation:'analyze-plan-compactness',document_json:'{"geometry-plan":1}',unit_geometry_json:geometryUiText,coordinate_basis:'projected-metres',crs_label:'Synthetic metre fixture'});assert.equal(node('compactness-basis').disabled,true);assert.equal(node('compactness-source').disabled,true);
const compactnessOutput={schema_version:'bisect-plan-compactness-v1',report_json:'{"area_m2":3}'};uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('download-compactness').onclick();assert.equal(downloadedLink.download,'compactness.json');assert.equal(await(await fetch(downloadedLink.href)).text(),compactnessOutput.report_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const compactnessProject=createProject(lastSnapshot);assert.equal(compactnessProject.files.compactnessInputs.geometry.base64,Buffer.from(geometryUiText).toString('base64'));importRead=()=>Promise.resolve(compactnessProject);const beforeCompactnessOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'compactness.bisect'}],value:'compactness.bisect'}});assert.equal(posted.length,beforeCompactnessOpen);assert.equal(node('download-compactness').disabled,true);assert.equal(node('compactness-crs').value,'Synthetic metre fixture');assert.equal(node('compactness-source-name').textContent,'units.json');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('compactness-crs').value='';node('compactness-crs').oninput();assert.equal(node('download-compactness').disabled,true);node('compactness-crs').value='EPSG:4326';node('compactness-basis').value='wgs84-degrees';node('compactness-basis').oninput();
let oversizedCompactnessRead=false;await node('compactness-source').onchange({target:{files:[{name:'huge.json',size:64*1024*1024+1,arrayBuffer:async()=>{oversizedCompactnessRead=true;}}]}});assert.equal(oversizedCompactnessRead,false);
const delayedCompactness=deferred();const pendingCompactness=node('compactness-source').onchange({target:{files:[{name:'old.json',size:Buffer.byteLength(geometryUiText),arrayBuffer:()=>delayedCompactness.promise}]}});node('project-name').value='Geometry changed while reading';node('project-name').oninput();delayedCompactness.resolve(new TextEncoder().encode(geometryUiText).buffer);await pendingCompactness;node('save-project').onclick();assert.equal(lastSnapshot.files.compactnessInputs.geometry.name,'units.json');
await node('compactness-source').onchange({target:{files:[]}});const beforeGeometryCleared=posted.length;node('execute').onclick();assert.equal(posted.length,beforeGeometryCleared);assert.equal(node('download-compactness').disabled,true);
console.log('Compactness UI: exact source/request/report download, CRS/busy controls, inert Save/Open, missing-source gate, invalid-setting freshness and oversized/stale-read guards passed.');

const blocUiDefaults=(await import('../../web/lab/bloc-files.js')).BLOC_DEFAULTS;for(const [key,value]of Object.entries(blocUiDefaults))node('bloc-'+key).value=String(value);node('bloc-bootstrap_seed').value='18446744073709551615';node('bloc-bootstrap_samples').value='64';node('operation').value='analyze-bloc-voting';node('operation').change();assert.equal(node('bloc-fields').hidden,false);assert.equal(node('text-audit-fields').hidden,true);assert.equal(node('download-text-audit').hidden,true);assert.equal(node('use-text-audit').hidden,true);const blocUiMissing=posted.length;node('execute').onclick();assert.equal(posted.length,blocUiMissing);assert.match(node('result-status').textContent,/attestation/);
const blocUiTsv='candidate_name\tprecinct_id\nSynthetic\tp1\n',blocUiCsv='candidate_name,attestation_doc_path\nSynthetic,attestations/alice.pdf\n',blocUiDoc='%PDF-1.4 synthetic bytes';await node('bloc-precincts').onchange({target:{files:[suiteSource('precincts.tsv',blocUiTsv)]}});await node('bloc-annotations').onchange({target:{files:[suiteSource('candidate-race.csv',blocUiCsv)]}});const blocUiAttachment={...suiteSource('alice.pdf',blocUiDoc),webkitRelativePath:'selected-root/attestations/alice.pdf'};await node('bloc-attestations-folder').onchange({target:{files:[blocUiAttachment]}});node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'analyze-bloc-voting',precincts_tsv:blocUiTsv,annotation_csv:blocUiCsv,annotation_source_name:'candidate-race.csv',attestation_files:{'attestations/alice.pdf':Buffer.from(blocUiDoc).toString('base64')},settings:{...blocUiDefaults,bootstrap_seed:'18446744073709551615',bootstrap_samples:64}});assert.equal(node('bloc-bootstrap_seed').disabled,true);assert.equal(node('bloc-attestations').disabled,true);assert.equal(node('bloc-precincts').disabled,true);
const blocUiOutput={schema_version:'bisect-bloc-voting-browser-v1',files:{'bloc_voting.json':'{"bootstrap_seed":"18446744073709551615"}','bloc_voting_summary.md':'[DRAFT] Synthetic native report\n'}};uiWorker.onmessage({data:{type:'result',result:blocUiOutput}});for(const [kind,name]of [['json','bloc_voting.json'],['markdown','bloc_voting_summary.md']]){node('download-bloc-'+kind).onclick();assert.equal(downloadedLink.download,name);assert.equal(await(await fetch(downloadedLink.href)).text(),blocUiOutput.files[name]);URL.revokeObjectURL(downloadedLink.href);}node('download-bloc-evidence').onclick();assert.equal(downloadedLink.download,'bloc-voting-evidence.zip');assert.ok((await(await fetch(downloadedLink.href)).arrayBuffer()).byteLength>300);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const blocUiProject=createProject(lastSnapshot);assert.equal(blocUiProject.files.blocInputs.settings.bootstrap_seed,'18446744073709551615');assert.equal(blocUiProject.files.blocInputs.attestations['attestations/alice.pdf'],Buffer.from(blocUiDoc).toString('base64'));importRead=()=>Promise.resolve(blocUiProject);const blocUiBeforeOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'bloc.bisect'}],value:'bloc.bisect'}});assert.equal(posted.length,blocUiBeforeOpen);assert.equal(node('download-bloc-json').disabled,true);assert.equal(node('bloc-bootstrap_seed').value,'18446744073709551615');assert.equal(node('bloc-annotations-name').textContent,'candidate-race.csv');
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:blocUiOutput}});node('bloc-bootstrap_seed').value='01';node('bloc-bootstrap_seed').oninput();assert.equal(node('download-bloc-evidence').disabled,true);node('bloc-bootstrap_seed').value='42';node('bloc-bootstrap_seed').oninput();
let blocUiOversizeRead=false;await node('bloc-attestations').onchange({target:{files:[{name:'huge.pdf',size:8*1024*1024+1,arrayBuffer:async()=>{blocUiOversizeRead=true;}}]}});assert.equal(blocUiOversizeRead,false);const blocUiDelayed=deferred();const blocUiPending=node('bloc-precincts').onchange({target:{files:[{name:'old.tsv',size:Buffer.byteLength(blocUiTsv),arrayBuffer:()=>blocUiDelayed.promise}]}});node('bloc-alpha').value='0.1';node('bloc-alpha').oninput();blocUiDelayed.resolve(new TextEncoder().encode(blocUiTsv).buffer);await blocUiPending;node('save-project').onclick();assert.equal(lastSnapshot.files.blocInputs.precincts.name,'precincts.tsv');
await node('bloc-attestations').onchange({target:{files:[]}});const blocUiCleared=posted.length;node('execute').onclick();assert.equal(posted.length,blocUiCleared);assert.equal(node('download-bloc-markdown').disabled,true);
console.log('Bloc UI: lossless seed/source/attachment requests, folder path retention, native JSON/Markdown/evidence ZIP downloads, busy controls, inert Save/Open, missing source, invalid-setting freshness and oversized/stale-read guards passed.');

node('operation').value='analyze-plan-compactness';node('operation').change();node('compactness-basis').value='projected-metres';node('compactness-crs').value='Synthetic metre fixture';await node('compactness-source').onchange({target:{files:[suiteSource('exact-units.json',geometryUiText)]}});node('compactness-exact').checked=true;node('compactness-exact').oninput();node('execute').onclick();assert.equal(posted.at(-1).request.include_exact_reock,true);assert.equal(node('compactness-exact').disabled,true);uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('save-project').onclick();const exactCompactProject=createProject(lastSnapshot);assert.equal(exactCompactProject.files.compactnessInputs.include_exact_reock,true);importRead=()=>Promise.resolve(exactCompactProject);const beforeExactOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'exact.bisect'}],value:'exact.bisect'}});assert.equal(posted.length,beforeExactOpen);assert.equal(node('compactness-exact').checked,true);assert.equal(node('download-compactness').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('compactness-exact').checked=false;node('compactness-exact').oninput();assert.equal(node('download-compactness').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.include_exact_reock,undefined);uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});
console.log('Exact Reock UI: explicit option, busy guard, retained inert Save/Open, edit freshness and disabled-option backward compatibility passed.');

const populationUiCsv='GEOID,x,y,population\nu1,1,1,18446744073709551615\n';await node('compactness-population').onchange({target:{files:[suiteSource('centroids.csv',populationUiCsv)]}});node('execute').onclick();assert.equal(posted.at(-1).request.population_centroids_csv,populationUiCsv);assert.equal(node('compactness-population').disabled,true);assert.equal(node('remove-compactness-population').disabled,true);uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('save-project').onclick();const populationUiProject=createProject(lastSnapshot);assert.equal(populationUiProject.files.compactnessInputs.population.base64,Buffer.from(populationUiCsv).toString('base64'));importRead=()=>Promise.resolve(populationUiProject);const beforePopulationOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'population.bisect'}],value:'population.bisect'}});assert.equal(posted.length,beforePopulationOpen);assert.equal(node('download-compactness').disabled,true);assert.equal(node('compactness-population-name').textContent,'centroids.csv');
let populationOversizedRead=false;await node('compactness-population').onchange({target:{files:[{name:'huge.csv',size:3*1024*1024+1,arrayBuffer:async()=>{populationOversizedRead=true;}}]}});assert.equal(populationOversizedRead,false);const populationDelay=deferred();const populationRead=node('compactness-population').onchange({target:{files:[{name:'old.csv',size:Buffer.byteLength(populationUiCsv),arrayBuffer:()=>populationDelay.promise}]}});node('compactness-crs').value='Changed metre CRS';node('compactness-crs').oninput();populationDelay.resolve(new TextEncoder().encode(populationUiCsv).buffer);await populationRead;node('save-project').onclick();assert.equal(lastSnapshot.files.compactnessInputs.population.name,'centroids.csv');node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('remove-compactness-population').onclick();assert.equal(node('download-compactness').disabled,true);assert.equal(node('remove-compactness-population').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.compactnessInputs.population,null);node('execute').onclick();assert.equal(posted.at(-1).request.population_centroids_csv,undefined);uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});
console.log('Population compactness UI: exact u64 CSV request/retention, busy controls, inert Save/Open, source removal/freshness and oversized/stale-read guards passed.');

node('compactness-dissolve').value='balanced';node('compactness-dissolve').oninput();
assert.equal(node('download-compactness').disabled,true);node('execute').onclick();
assert.equal(posted.at(-1).request.dissolve_method,'balanced');
assert.equal(node('compactness-dissolve').disabled,true);
uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('save-project').onclick();
const balancedUiProject=createProject(lastSnapshot);assert.equal(balancedUiProject.files.compactnessInputs.dissolve_method,'balanced');
importRead=()=>Promise.resolve(balancedUiProject);const beforeBalancedOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'balanced.bisect'}],value:'balanced.bisect'}});
assert.equal(posted.length,beforeBalancedOpen);assert.equal(node('compactness-dissolve').value,'balanced');
assert.equal(node('download-compactness').disabled,true);
console.log('Balanced dissolve UI: explicit request, busy control, stale exports and inert Save/Open retention passed.');

const largeCompactnessText=JSON.stringify({type:'FeatureCollection',features:[]})+' '.repeat(9*1024*1024);
let largeGeometryRead=false;await node('compactness-source').onchange({target:{files:[{name:'large-geometry.json',size:Buffer.byteLength(largeCompactnessText),arrayBuffer:async()=>{largeGeometryRead=true;return new TextEncoder().encode(largeCompactnessText).buffer;}}]}});
assert.equal(largeGeometryRead,true);node('save-project').onclick();
assert.equal(lastSnapshot.files.compactnessInputs.geometry.base64,Buffer.from(largeCompactnessText).toString('base64'));
node('execute').onclick();assert.equal(posted.at(-1).request.unit_geometry_json,largeCompactnessText);
uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});
console.log('Large compactness geometry: >8 MiB byte-exact load, retention and request passed; other artifacts retain their existing limits.');

node('compactness-dissolve').value='spatial-balanced';node('compactness-dissolve').oninput();
assert.equal(node('download-compactness').disabled,true);node('execute').onclick();
assert.equal(posted.at(-1).request.dissolve_method,'spatial-balanced');assert.equal(node('compactness-dissolve').disabled,true);
uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('save-project').onclick();
const spatialUiProject=createProject(lastSnapshot);assert.equal(spatialUiProject.files.compactnessInputs.dissolve_method,'spatial-balanced');
importRead=()=>Promise.resolve(spatialUiProject);const beforeSpatialOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'spatial.bisect'}],value:'spatial.bisect'}});
assert.equal(posted.length,beforeSpatialOpen);assert.equal(node('compactness-dissolve').value,'spatial-balanced');assert.equal(node('download-compactness').disabled,true);
console.log('Spatial dissolve: explicit request, busy/freshness and inert Save/Open setting retention passed.');

node('compactness-dissolve').value='batch';node('compactness-dissolve').oninput();
assert.equal(node('download-compactness').disabled,true);node('execute').onclick();
assert.equal(posted.at(-1).request.dissolve_method,'batch');assert.equal(node('compactness-dissolve').disabled,true);
uiWorker.onmessage({data:{type:'result',result:compactnessOutput}});node('save-project').onclick();
const batchUiProject=createProject(lastSnapshot);assert.equal(batchUiProject.files.compactnessInputs.dissolve_method,'batch');
importRead=()=>Promise.resolve(batchUiProject);const beforeBatchOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'batch.bisect'}],value:'batch.bisect'}});
assert.equal(posted.length,beforeBatchOpen);assert.equal(node('compactness-dissolve').value,'batch');assert.equal(node('download-compactness').disabled,true);
console.log('Batch dissolve: explicit request, busy/freshness and inert Save/Open setting retention passed.');

node('operation').value='aggregate-national-analysis';node('operation').change();assert.equal(node('national-fields').hidden,false);
const nationalUiText=JSON.stringify({analyzer:'compactness',state:'RI',year:'2020',districts:[{district:1,count:'18446744073709551615'}]});
await node('national-reports').onchange({target:{files:[{name:'RI.json',size:Buffer.byteLength(nationalUiText),arrayBuffer:async()=>new TextEncoder().encode(nationalUiText).buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.reports_json['RI.json'],nationalUiText);assert.equal(node('national-reports').disabled,true);
const nationalUiOutput={schema_version:'bisect-national-analysis-v1',report:{analyzer:'compactness',scope:'national',state_count:1,district_count:1,districts:[{district:1,count:'18446744073709551615',state:'RI'}]},report_json:'native national json',csv:'native national csv',array_json:'native records json',provided_states:['RI'],year:'2020'};
uiWorker.onmessage({data:{type:'result',result:nationalUiOutput}});assert.equal(node('download-national-json').disabled,false);node('download-national-csv').onclick();assert.equal(await (await fetch(downloadedLink.href)).text(),nationalUiOutput.csv);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const nationalUiProject=createProject(lastSnapshot);assert.equal(nationalUiProject.files.nationalInputs.reports['RI.json'].base64,Buffer.from(nationalUiText).toString('base64'));
importRead=()=>Promise.resolve(nationalUiProject);const beforeNationalOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'national.bisect'}],value:'national.bisect'}});assert.equal(posted.length,beforeNationalOpen);assert.equal(node('download-national-json').disabled,true);
let nationalOversizeRead=false;await node('national-reports').onchange({target:{files:[{name:'big.json',size:3*1024*1024+1,arrayBuffer:async()=>{nationalOversizeRead=true;}}]}});assert.equal(nationalOversizeRead,false);
const nationalDelayed=deferred();const nationalPending=node('national-reports').onchange({target:{files:[{name:'old.json',size:Buffer.byteLength(nationalUiText),arrayBuffer:()=>nationalDelayed.promise}]}});node('operation').value='analyze-plan-compactness';node('operation').change();nationalDelayed.resolve(new TextEncoder().encode(nationalUiText).buffer);await nationalPending;node('save-project').onclick();assert.equal(lastSnapshot.files.nationalInputs.reports['RI.json'].name,'RI.json');
console.log('National UI: exact source request/download, busy guard, inert Save/Open, imported-result freshness and oversized/stale-read guards passed.');

node('operation').value='inspect-algorithm-config';node('operation').change();assert.equal(node('config-fields').hidden,false);
const configUiText='name: example\nalgorithm:\n  structure: apportion-regions\n  seed: 18446744073709551615\n';
await node('config-source').onchange({target:{files:[{name:'example.yml',size:Buffer.byteLength(configUiText),arrayBuffer:async()=>new TextEncoder().encode(configUiText).buffer}]}});
node('execute').onclick();assert.equal(posted.at(-1).request.yaml_text,configUiText);assert.equal(node('config-source').disabled,true);
uiWorker.onmessage({data:{type:'result',result:{schema_version:'bisect-algorithm-config-inspection-v1',seed:'18446744073709551615'}}});
node('save-project').onclick();const configUiProject=createProject(lastSnapshot);assert.equal(configUiProject.files.configSource.base64,Buffer.from(configUiText).toString('base64'));
importRead=()=>Promise.resolve(configUiProject);const beforeConfigOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'config.bisect'}],value:'config.bisect'}});assert.equal(posted.length,beforeConfigOpen);assert.equal(node('download').disabled,false);
let configOversizeRead=false;await node('config-source').onchange({target:{files:[{name:'big.yml',size:256*1024+1,arrayBuffer:async()=>{configOversizeRead=true;}}]}});assert.equal(configOversizeRead,false);
const configDelayed=deferred();const configPending=node('config-source').onchange({target:{files:[{name:'old.yml',size:Buffer.byteLength(configUiText),arrayBuffer:()=>configDelayed.promise}]}});node('operation').value='analyze-plan-compactness';node('operation').change();configDelayed.resolve(new TextEncoder().encode(configUiText).buffer);await configPending;node('save-project').onclick();assert.equal(lastSnapshot.files.configSource.name,'example.yml');
console.log('Configuration UI: exact source request, busy guard, inert Save/Open and oversized/stale-read rejection passed.');

node('operation').value='validate-algorithm-config';node('operation').change();node('config-profile').value='court';node('config-profile').onchange();
node('execute').onclick();assert.equal(posted.at(-1).request.operation,'validate-algorithm-config');assert.equal(posted.at(-1).request.profile,'court');assert.equal(node('config-profile').disabled,true);
const configValidated={schema_version:'bisect-algorithm-config-validation-v1',native_summary:'native summary\nexact bytes',resolution_json:'{"seed":18446744073709551615}'};
uiWorker.onmessage({data:{type:'result',result:configValidated}});assert.equal(node('download-config-summary').disabled,false);node('download-config-summary').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),configValidated.native_summary);URL.revokeObjectURL(downloadedLink.href);
node('download-config-resolution').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),configValidated.resolution_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const configValidatedProject=createProject(lastSnapshot);assert.equal(configValidatedProject.files.configSettings.profile,'court');
importRead=()=>Promise.resolve(configValidatedProject);const beforeValidatedOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'validated.bisect'}],value:'validated.bisect'}});assert.equal(posted.length,beforeValidatedOpen);assert.equal(node('config-profile').value,'court');assert.equal(node('download-config-summary').disabled,true);assert.equal(node('download-config-resolution').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:configValidated}});node('config-profile').value='open';node('config-profile').onchange();assert.equal(node('download-config-summary').disabled,true);
console.log('Configuration validation UI: explicit saved profile, busy guards, exact native summary/resolution Blob exports, inert Open and edited-profile invalidation passed.');

const templateUiSettings={...(await import('../../web/lab/config-template.js')).CONFIG_TEMPLATE_DEFAULTS,seed:'18446744073709551615'};
node('operation').value='create-algorithm-config';node('operation').change();for(const [key,value]of Object.entries(templateUiSettings))node('config-new-'+key).value=Array.isArray(value)?value.join(' '):value===null?'':String(value);
node('config-new-seed').onchange();node('execute').onclick();assert.equal(posted.at(-1).request.operation,'create-algorithm-config');assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request.settings)),templateUiSettings);assert.equal(node('config-new-engine').disabled,true);
const templateUiResult={schema_version:'bisect-algorithm-config-template-v1',generated_filename:'browser_baseline.yml',yaml_text:'name: browser_baseline\nalgorithm:\n  structure: apportion-regions\n  engine: metis-core\n  seed: 18446744073709551615\n'};
uiWorker.onmessage({data:{type:'result',result:templateUiResult}});assert.equal(node('download-config-yaml').disabled,false);node('download-config-yaml').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),templateUiResult.yaml_text);assert.equal(downloadedLink.download,templateUiResult.generated_filename);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const templateUiProject=createProject(lastSnapshot);assert.equal(templateUiProject.files.configTemplate.seed,'18446744073709551615');
importRead=()=>Promise.resolve(templateUiProject);const beforeTemplateOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'template.bisect'}],value:'template.bisect'}});assert.equal(posted.length,beforeTemplateOpen);assert.equal(node('config-new-seed').value,'18446744073709551615');assert.equal(node('download-config-yaml').disabled,true);assert.equal(node('use-config-template').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:templateUiResult}});node('config-new-workers').value='6';node('config-new-workers').onchange();assert.equal(node('download-config-yaml').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:templateUiResult}});const beforeTemplateUse=posted.length;node('use-config-template').onclick();assert.equal(posted.length,beforeTemplateUse);assert.equal(node('operation').value,'validate-algorithm-config');node('save-project').onclick();assert.equal(Buffer.from(lastSnapshot.files.configSource.base64,'base64').toString('utf8'),templateUiResult.yaml_text);
node('execute').onclick();assert.equal(posted.at(-1).request.yaml_text,templateUiResult.yaml_text);uiWorker.onmessage({data:{type:'result',result:configValidated}});
console.log('Config template UI: exact seeded settings/YAML Blob bytes, busy guard, inert Save/Open, edited-input invalidation and explicit source handoff to validation passed.');

node('operation').value='run-algorithm-config';node('operation').change();
assert.equal(node('config-run-fields').hidden,false);assert.equal(node('plan-map').parentElement.hidden,false);
node('config-run-chamber').value='house';node('config-run-districts').value='2';node('config-run-chamber').onchange();
const graphUi={schema_version:1,state:'RI',year:'2020',geoids:['44001000001','44001000002'],adjacency:[[1],[0]],population:[100,100],edges:[[0,1,1]],areas:[100,100],exterior_perimeters:[30,30],centroids:[[-71,41],[-71.01,41.01]]};
const graphUiText=JSON.stringify(graphUi),jsonFile=(name,text)=>({name,size:Buffer.byteLength(text),arrayBuffer:async()=>new TextEncoder().encode(text).buffer});
const beforeMissingGraph=posted.length;node('execute').onclick();assert.equal(posted.length,beforeMissingGraph);assert.match(node('result-status').textContent,/prepared state graph/);
await node('config-run-graph').onchange({target:{files:[jsonFile('RI-2020-graph.json',graphUiText)]}});
node('execute').onclick();const configuredRequest=posted.at(-1).request;
assert.equal(configuredRequest.operation,'run-algorithm-config');assert.equal(configuredRequest.input.yaml_text,templateUiResult.yaml_text);assert.deepEqual(configuredRequest.input.graph,graphUi);assert.equal(configuredRequest.input.chamber,'house');assert.equal(node('config-run-graph').disabled,true);
const configuredUiResult={schema_version:'bisect-configured-run-v1',configuration:{yaml_text:templateUiResult.yaml_text},engine_request:{graph:graphUi,options:{districts:2,seed:'18446744073709551615'}},result:{assignments:{'44001000001':1,'44001000002':2},metrics:{max_deviation_percent:0,contiguous:true}}};
uiWorker.onmessage({data:{type:'result',result:configuredUiResult}});assert.equal(node('config-run-graph').disabled,false);assert.equal(node('result-status').textContent,'Operation finished · configured plan generated');
node('save-project').onclick();const configuredUiProject=createProject(lastSnapshot);assert.equal(configuredUiProject.files.configRunInputs.graph.base64,Buffer.from(graphUiText).toString('base64'));
importRead=()=>Promise.resolve(configuredUiProject);const beforeConfiguredOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'configured.bisect'}],value:'configured.bisect'}});assert.equal(posted.length,beforeConfiguredOpen);assert.equal(node('config-run-chamber').value,'house');assert.equal(node('config-run-districts').value,2);assert.match(node('plan-map').textContent,/Run plan validation/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:configuredUiResult}});node('config-run-districts').value='3';node('config-run-districts').onchange();assert.match(node('result-status').textContent,/Inputs changed/);
let configGraphTooLargeRead=false;await node('config-run-graph').onchange({target:{files:[{name:'big.json',size:8*1024*1024+1,arrayBuffer:async()=>{configGraphTooLargeRead=true;}}]}});assert.equal(configGraphTooLargeRead,false);
await node('config-run-graph').onchange({target:{files:[jsonFile('duplicate.json','{"state":"RI","state":"IA"}')]}});assert.match(node('result-status').textContent,/Duplicate/);
const configGraphDelayed=deferred(),graphPending=node('config-run-graph').onchange({target:{files:[{name:'old.json',size:Buffer.byteLength(graphUiText),arrayBuffer:()=>configGraphDelayed.promise}]}});node('operation').value='validate-algorithm-config';node('operation').change();configGraphDelayed.resolve(new TextEncoder().encode(graphUiText).buffer);await graphPending;node('save-project').onclick();assert.equal(lastSnapshot.files.configRunInputs.graph.name,'RI-2020-graph.json');
console.log('Configured run UI: exact YAML/graph requests, explicit Run, busy guards, lossless inert Save/Open, input freshness and oversized/duplicate/stale-source rejection passed.');
node('operation').value='run-algorithm-config';node('operation').change();
node('config-run-districts').value='2';node('config-run-districts').onchange();
node('execute').onclick();
configuredUiResult.configuration={yaml_text:templateUiResult.yaml_text,source_name:'run.yml',profile:'open',execution_chamber:'house'};
uiWorker.onmessage({data:{type:'ready'}});
uiWorker.onmessage({data:{type:'result',result:configuredUiResult}});
assert.equal(node('prepare-configured-export').disabled,false);
node('prepare-configured-export').onclick();const configuredExportRequest=posted.at(-1).request;
assert.equal(configuredExportRequest.operation,'export-configured-plan');assert.deepEqual(configuredExportRequest.assignments,configuredUiResult.result.assignments);assert.equal(configuredExportRequest.input.yaml_text,templateUiResult.yaml_text);assert.equal(configuredExportRequest.input.chamber,'house');assert.equal(node('config-run-graph').disabled,true);
const fakeNativeExport={schema_version:'bisect-configured-plan-export-v1',document_json:'{"native":"exact plan bytes"}\n',context_json:'{"native":"exact context bytes"}\n'};
uiWorker.onmessage({data:{type:'result',result:fakeNativeExport}});
assert.equal(node('download-configured-plan').disabled,false);node('save-project').onclick();
assert.deepEqual(lastSnapshot.result.result,configuredUiResult.result);assert.deepEqual(lastSnapshot.result.practitioner_export,fakeNativeExport);
node('download-configured-plan').onclick();assert.equal(downloadedLink.download,'configured.rplan');assert.equal(await (await fetch(downloadedLink.href)).text(),fakeNativeExport.document_json);URL.revokeObjectURL(downloadedLink.href);
node('download-configured-context').onclick();assert.equal(await (await fetch(downloadedLink.href)).text(),fakeNativeExport.context_json);URL.revokeObjectURL(downloadedLink.href);
const exportedUiProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(exportedUiProject);
await node('open-project').onchange({target:{files:[{name:'exported.bisect'}]}});
assert.equal(node('download-configured-plan').disabled,true);assert.equal(node('audit-configured-plan').disabled,true);
const beforeInertExport=posted.length;node('prepare-configured-export').onclick();assert.equal(posted.length,beforeInertExport);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:configuredUiResult}});
node('prepare-configured-export').onclick();uiWorker.onmessage({data:{type:'error',error:'Export rejected'}});assert.equal(node('prepare-configured-export').disabled,false);assert.equal(node('download-configured-plan').disabled,true);
node('prepare-configured-export').onclick();node('cancel').onclick();assert.equal(node('result-status').textContent,'Export cancelled. Completed run retained.');uiWorker.onmessage({data:{type:'ready'}});assert.equal(node('prepare-configured-export').disabled,false);node('save-project').onclick();assert.deepEqual(lastSnapshot.result.result,configuredUiResult.result);assert.equal(lastSnapshot.result.practitioner_export,undefined);
node('prepare-configured-export').onclick();uiWorker.onmessage({data:{type:'result',result:fakeNativeExport}});
node('audit-configured-plan').onclick();node('save-project').onclick();assert.equal(node('operation').value,'audit-rplan-text');assert.equal(Buffer.from(lastSnapshot.files.auditPlanText.base64,'base64').toString(),fakeNativeExport.document_json);assert.equal(Buffer.from(lastSnapshot.files.auditContextText.base64,'base64').toString(),fakeNativeExport.context_json);assert.equal(lastSnapshot.files.auditCertificateText,undefined);assert.equal(lastSnapshot.files.auditLineageText,undefined);
console.log('Configured native export UI: explicit request, retained run, exact downloads, inert reopening, error recovery and matching plan/context audit handoff passed.');

const originalCatalogReader=sandbox.readConfigCatalog,originalCatalogLoader=sandbox.loadConfigCatalogInputs;
const publishedFixture={manifest:{},entries:[{key:'RI:2020',state:'RI',year:'2020',name:'Rhode Island',districts:{congressional:2,house:75,senate:38}}]};
sandbox.readConfigCatalog=async()=>publishedFixture;
sandbox.document.createElement=()=>({click(){downloadedLink=this;}});
node('config-run-published').replaceChildren=()=>{};node('config-run-published').append=option=>{node('config-run-published').value=option.value;};
node('operation').value='run-algorithm-config';node('operation').change();
const beforeCatalogPosts=posted.length;await node('config-run-catalog').onclick();assert.equal(posted.length,beforeCatalogPosts);assert.equal(node('config-run-published').value,'RI:2020');assert.equal(node('config-run-load-published').disabled,false);
const publishedSources={graph:{name:'RI-2020-graph.json',base64:Buffer.from(graphUiText).toString('base64')},geometry:{name:'RI-2020-map.geojson',base64:Buffer.from('{}').toString('base64')},settings:{chamber:'congressional',districts:2},selected:publishedFixture.entries[0]};
sandbox.loadConfigCatalogInputs=async()=>structuredClone(publishedSources);
node('config-run-chamber').value='congressional';await node('config-run-load-published').onclick();assert.equal(posted.length,beforeCatalogPosts);node('save-project').onclick();assert.deepEqual(structuredClone(lastSnapshot.files.configRunInputs),{graph:publishedSources.graph});assert.deepEqual(lastSnapshot.files.planGeometry,publishedSources.geometry);assert.deepEqual(structuredClone(lastSnapshot.files.configRunSettings),publishedSources.settings);assert.equal(node('prepare-configured-export').disabled,true);
const stalePublished=deferred();sandbox.loadConfigCatalogInputs=()=>stalePublished.promise;
const pendingPublished=node('config-run-load-published').onclick();assert.equal(node('execute').disabled,true);node('execute').onclick();assert.equal(posted.length,beforeCatalogPosts);
node('project-name').value='Changed during catalog load';node('project-name').oninput();stalePublished.resolve({...publishedSources,graph:{...publishedSources.graph,name:'stale.json'}});await pendingPublished;node('save-project').onclick();assert.equal(lastSnapshot.files.configRunInputs.graph.name,publishedSources.graph.name);assert.match(node('result-status').textContent,/Project changed/);
sandbox.readConfigCatalog=originalCatalogReader;sandbox.loadConfigCatalogInputs=originalCatalogLoader;
console.log('Published YAML input UI: explicit catalog/state loading without execution, paired source retention, chamber allocation, freshness, busy Run gate and stale-load preservation passed.');
node('operation').value='run-algorithm-config';node('operation').change();node('execute').onclick();
uiWorker.onmessage({data:{type:'result',result:{...configuredUiResult,result:{...configuredUiResult.result,metrics:{...configuredUiResult.result.metrics,within_requested_tolerance:false}}}}});assert.match(node('result-status').textContent,/outside requested population tolerance/);
const displayBytes=Buffer.from(JSON.stringify({type:'FeatureCollection',features:[],padding:'a'.repeat(9*1024*1024)}));
await node('plan-geometry-file').onchange({target:{files:[{name:'large-display.geojson',size:displayBytes.length,arrayBuffer:async()=>displayBytes.buffer.slice(displayBytes.byteOffset,displayBytes.byteOffset+displayBytes.byteLength)}]}});node('save-project').onclick();assert.equal(lastSnapshot.files.planGeometry.base64,displayBytes.toString('base64'));
console.log('Large display geometry: >8 MiB exact local read/project retention with 16 MiB cap; configured balance failures are visible in status.');
node('config-run-native_build').checked=true;node('config-run-native_build').onchange();node('execute').onclick();assert.equal(posted.at(-1).request.operation,'run-algorithm-build');assert.equal(Object.hasOwn(posted.at(-1).request.input,'native_build'),false);assert.equal(node('config-run-native_build').disabled,true);
const retryUiResult={...configuredUiResult,configuration:{...configuredUiResult.configuration,execution_attempt:1,execution_seed:'0'},engine_request:{...configuredUiResult.engine_request,options:{...configuredUiResult.engine_request.options,seed:0}},build_report:{accepted_attempt:1,max_balance_retries:50,attempts:[{attempt:0,seed:'18446744073709551615',balanced:false},{attempt:1,seed:'0',balanced:true}]}};
uiWorker.onmessage({data:{type:'result',result:retryUiResult}});assert.equal(node('config-run-native_build').disabled,false);node('save-project').onclick();const retryUiProject=createProject(lastSnapshot);assert.equal(retryUiProject.files.configRunSettings.native_build,true);assert.deepEqual(retryUiProject.result.build_report,retryUiResult.build_report);
node('prepare-configured-export').onclick();assert.equal(posted.at(-1).request.attempt,1);uiWorker.onmessage({data:{type:'result',result:fakeNativeExport}});node('save-project').onclick();assert.equal(lastSnapshot.result.engine_request.options.seed,0);assert.equal(lastSnapshot.result.build_report.accepted_attempt,1);
importRead=()=>Promise.resolve(retryUiProject);const beforeRetryOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'retry.bisect'}]}});assert.equal(posted.length,beforeRetryOpen);assert.equal(node('config-run-native_build').checked,true);assert.equal(node('prepare-configured-export').disabled,true);
node('execute').onclick();uiWorker.onmessage({data:{type:'error',error:'Native population balance retry policy exhausted after 51 attempts'}});assert.match(node('result-status').textContent,/exhausted after 51/);assert.equal(node('prepare-configured-export').disabled,true);assert.equal(node('download-configured-plan').disabled,true);
node('execute').onclick();node('cancel').onclick();assert.match(node('result-status').textContent,/Cancelled/);assert.equal(node('prepare-configured-export').disabled,true);
console.log('Native retry UI: explicit switch/request, busy/cancel/exhaustion guards, accepted-seed export, retained report and inert saved-project restoration passed.');

uiWorker.onmessage({data:{type:'ready'}});
node('operation').value='inspect-location-policy';node('operation').change();assert.equal(node('policy-fields').hidden,false);
node('policy-query').value='RI';node('policy-query').oninput();node('execute').onclick();
assert.deepEqual(structuredClone(posted.at(-1).request),{operation:'inspect-location-policy',query:'RI',source_name:null,policy_json:null});
assert.equal(node('policy-query').disabled,true);assert.equal(node('policy-source').disabled,true);
const policyUiOutput={schema_version:'bisect-location-policy-v1',query:'RI',resolved_code:'RI',source_kind:'embedded',source_name:'location_policy.json',source_sha256:'sha256:fixture',policy:{name:'Rhode Island'},policy_json:'{ "name": "Rhode Island" }',table_text:'Native policy text\n',fields:[],locations:[]};
uiWorker.onmessage({data:{type:'result',result:policyUiOutput}});
for(const [kind,key,extension] of [['json','policy_json','json'],['text','table_text','txt']]){node('download-policy-'+kind).onclick();assert.equal(downloadedLink.download,'location-policy.'+extension);assert.equal(await (await fetch(downloadedLink.href)).text(),policyUiOutput[key]);URL.revokeObjectURL(downloadedLink.href);}
const policyText='{ "XX": {"name":"<img src=x onerror=alert(1)>","nesting_ratio":"2:1"} }\n',policyBytes=new TextEncoder().encode(policyText);
await node('policy-source').onchange({target:{files:[{name:'override.json',size:policyBytes.length,arrayBuffer:async()=>policyBytes.buffer}]}});
assert.equal(node('download-policy-json').disabled,true);node('policy-query').value='XX';node('policy-query').oninput();node('execute').onclick();assert.equal(posted.at(-1).request.policy_json,policyText);assert.equal(posted.at(-1).request.source_name,'override.json');
uiWorker.onmessage({data:{type:'result',result:{...policyUiOutput,query:'XX',resolved_code:'XX',source_kind:'local-override'}}});
node('save-project').onclick();const policyUiProject=createProject({...lastSnapshot,files:{policyInputs:lastSnapshot.files.policyInputs}});
assert.equal(policyUiProject.files.policyInputs.source.base64,Buffer.from(policyText).toString('base64'));
importRead=()=>Promise.resolve(policyUiProject);const beforePolicyOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'policy.bisect'}]}});assert.equal(posted.length,beforePolicyOpen);assert.equal(node('policy-query').value,'XX');assert.equal(node('download-policy-json').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();assert.equal(posted.at(-1).request.policy_json,policyText);uiWorker.onmessage({data:{type:'result',result:policyUiOutput}});
let policyOversizeRead=false;await node('policy-source').onchange({target:{files:[{name:'oversized.json',size:512*1024+1,arrayBuffer:async()=>{policyOversizeRead=true;}}]}});assert.equal(policyOversizeRead,false);assert.match(node('result-status').textContent,/512 KiB/);
const delayedPolicy=deferred();const delayedPolicyRead=node('policy-source').onchange({target:{files:[{name:'delayed.json',size:policyBytes.length,arrayBuffer:()=>delayedPolicy.promise}]}});
node('policy-query').value='RI';node('policy-query').oninput();delayedPolicy.resolve(policyBytes.buffer);await delayedPolicyRead;
node('save-project').onclick();assert.equal(lastSnapshot.files.policyInputs.source.name,'override.json');
await node('policy-source').onchange({target:{files:[{name:'invalid.json',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}]}});assert.match(node('result-status').textContent,/encoded|encoding|UTF|valid/i);
node('policy-embedded').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.policyInputs.source,null);
node('execute').onclick();assert.equal(posted.at(-1).request.policy_json,null);uiWorker.onmessage({data:{type:'result',result:policyUiOutput}});
node('policy-query').value='';node('policy-query').oninput();const beforeEmptyPolicy=posted.length;node('execute').onclick();assert.equal(posted.length,beforeEmptyPolicy);assert.match(node('result-status').textContent,/code or name/);
console.log('Policy UI: embedded/local exact requests, busy/freshness guards, two byte-exact downloads, inert Save/Open, override removal, empty query, pre-read size, stale read and strict UTF-8 passed.');

uiWorker.onmessage({data:{type:'ready'}});
node('operation').value='diagnose-location-inputs';node('operation').change();assert.equal(node('doctor-fields').hidden,false);assert.equal(node('policy-fields').hidden,true);
for(const [key,value]of Object.entries(doctorTools.DOCTOR_DEFAULTS))node('doctor-'+key).value=value;
node('doctor-state').oninput();node('execute').onclick();
assert.deepEqual(structuredClone(posted.at(-1).request),doctorTools.doctorRequest({settings:doctorTools.DOCTOR_DEFAULTS,policy:null,context:null}));
for(const id of ['doctor-state','doctor-year','doctor-chamber','doctor-resolution','doctor-policy','doctor-context','doctor-embedded','doctor-remove-context'])assert.equal(node(id).disabled,true);
const doctorUiOutput={schema_version:'bisect-location-preflight-v1',status:'needs-inputs',metadata:{state:'RI',year:'2020'},checks:[{check:'context',status:'missing',message:'Supply an RCTX'}],scope:'No legal certification'};
uiWorker.onmessage({data:{type:'result',result:doctorUiOutput}});assert.equal(node('download-doctor').disabled,false);
node('download-doctor').onclick();assert.equal(downloadedLink.download,'location-preflight.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),doctorUiOutput);URL.revokeObjectURL(downloadedLink.href);
const doctorContextText=JSON.stringify(context),doctorContextBytes=new TextEncoder().encode(doctorContextText);
await node('doctor-policy').onchange({target:{files:[jsonFile('doctor-policy.json',policyText)]}});
await node('doctor-context').onchange({target:{files:[jsonFile('grid.rctx',doctorContextText)]}});
assert.equal(node('download-doctor').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.policy_json,policyText);assert.equal(posted.at(-1).request.context_json,doctorContextText);
uiWorker.onmessage({data:{type:'result',result:doctorUiOutput}});node('save-project').onclick();
const doctorUiProject=createProject({...lastSnapshot,files:{doctorInputs:lastSnapshot.files.doctorInputs}});assert.equal(doctorUiProject.files.doctorInputs.context.base64,Buffer.from(doctorContextText).toString('base64'));
importRead=()=>Promise.resolve(doctorUiProject);const beforeDoctorOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'doctor.bisect'}]}});
assert.equal(posted.length,beforeDoctorOpen);assert.equal(node('download-doctor').disabled,true);assert.equal(node('doctor-state').value,'RI');assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:doctorUiOutput}});node('doctor-year').value='2010';node('doctor-year').oninput();assert.equal(node('download-doctor').disabled,true);
let doctorOversizeRead=false;await node('doctor-context').onchange({target:{files:[{name:'big.rctx',size:8*1024*1024+1,arrayBuffer:async()=>{doctorOversizeRead=true;}}]}});assert.equal(doctorOversizeRead,false);
await node('doctor-context').onchange({target:{files:[{name:'invalid.rctx',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}]}});assert.match(node('result-status').textContent,/encoded|encoding|UTF|valid/i);
const delayedDoctor=deferred();const delayedDoctorRead=node('doctor-context').onchange({target:{files:[{name:'stale.rctx',size:doctorContextBytes.length,arrayBuffer:()=>delayedDoctor.promise}]}});
node('doctor-remove-context').onclick();delayedDoctor.resolve(doctorContextBytes.buffer);await delayedDoctorRead;node('save-project').onclick();assert.equal(lastSnapshot.files.doctorInputs.context,null);
const delayedDoctorPolicy=deferred();const delayedDoctorPolicyRead=node('doctor-policy').onchange({target:{files:[{name:'stale.json',size:policyBytes.length,arrayBuffer:()=>delayedDoctorPolicy.promise}]}});
node('doctor-embedded').onclick();delayedDoctorPolicy.resolve(policyBytes.buffer);await delayedDoctorPolicyRead;node('save-project').onclick();assert.equal(lastSnapshot.files.doctorInputs.policy,null);
node('execute').onclick();node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});assert.equal(node('download-doctor').disabled,true);
node('doctor-state').value='';node('doctor-state').oninput();const beforeEmptyDoctor=posted.length;node('execute').onclick();assert.equal(posted.length,beforeEmptyDoctor);assert.match(node('result-status').textContent,/location|code or name/);
console.log('Doctor UI: exact requests, busy/freshness guards, report download, inert Open, strict UTF-8, pre-read caps, cancellation and source removal during pending reads passed.');

uiWorker.onmessage({data:{type:'ready'}});node('operation').value='verify-run-manifest';node('operation').change();assert.equal(node('manifest-fields').hidden,false);assert.equal(node('doctor-fields').hidden,true);
const beforeManifest=posted.length;node('execute').onclick();assert.equal(posted.length,beforeManifest);assert.match(node('result-status').textContent,/manifest/);
const manifestText='{ "binary_version": "0.1.0", "adjacency_file":"https://attacker.invalid/never-open" }\n';
await node('manifest-source').onchange({target:{files:[jsonFile('manifest.json',manifestText)]}});node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),{operation:'verify-run-manifest',source_name:'manifest.json',manifest_json:manifestText});assert.equal(node('manifest-source').disabled,true);
const manifestUiOutput={schema_version:'bisect-manifest-compatibility-v1',status:'version-compatible',native_exit_code:0,current_build:{version:'0.1.0'},assessment:{version_status:'matches'},scope:'No authentication'};
uiWorker.onmessage({data:{type:'result',result:manifestUiOutput}});assert.equal(node('download-manifest-check').disabled,false);node('download-manifest-check').onclick();assert.equal(downloadedLink.download,'manifest-compatibility.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),manifestUiOutput);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const manifestUiProject=createProject({...lastSnapshot,files:{manifestSource:lastSnapshot.files.manifestSource}});assert.equal(manifestUiProject.files.manifestSource.base64,Buffer.from(manifestText).toString('base64'));
importRead=()=>Promise.resolve(manifestUiProject);const beforeManifestOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'manifest.bisect'}]}});assert.equal(posted.length,beforeManifestOpen);assert.equal(node('download-manifest-check').disabled,true);assert.match(node('result-status').textContent,/unverified/);
node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:manifestUiOutput}});
let manifestOversizeRead=false;await node('manifest-source').onchange({target:{files:[{name:'big.json',size:1024*1024+1,arrayBuffer:async()=>{manifestOversizeRead=true;}}]}});assert.equal(manifestOversizeRead,false);assert.equal(node('download-manifest-check').disabled,true);
const delayedManifest=deferred(),manifestBytes=new TextEncoder().encode(manifestText);const delayedManifestRead=node('manifest-source').onchange({target:{files:[{name:'stale.json',size:manifestBytes.length,arrayBuffer:()=>delayedManifest.promise}]}});node('operation').value='inspect-location-policy';node('operation').change();delayedManifest.resolve(manifestBytes.buffer);await delayedManifestRead;node('save-project').onclick();assert.equal(lastSnapshot.files.manifestSource.name,'manifest.json');
await node('manifest-source').onchange({target:{files:[{name:'invalid.json',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}]}});assert.match(node('result-status').textContent,/encoded|encoding|UTF|valid/i);
console.log('Manifest UI: exact request/report download, busy/freshness, inert Save/Open, missing-source gate, pre-read cap, stale reads and strict UTF-8 passed.');

uiWorker.onmessage({data:{type:'ready'}});node('operation').value='verify-tutorial-files';node('operation').change();assert.equal(node('tutorial-fields').hidden,false);assert.equal(node('manifest-fields').hidden,true);node('tutorial-name').value='synthetic';node('tutorial-name').oninput();
const beforeTutorial=posted.length;node('execute').onclick();assert.equal(posted.length,beforeTutorial);assert.match(node('result-status').textContent,/tutorial/);
const tutorialText=JSON.stringify({schema_version:'tutorial-checksums v1',tutorial:'synthetic',pinned_inputs:[{name:'Input',local_path:'data/input.bin',sha256:'PIN_ON_FIRST_RUN'}],expected_outputs:[]});
await node('tutorial-checksums').onchange({target:{files:[jsonFile('checksums.json',tutorialText)]}});
const tutorialFile={name:'input.bin',webkitRelativePath:'repo/data/input.bin',size:10,slice(){throw new Error('UI must not read file bytes');}};
node('tutorial-folder').onchange({target:{files:[tutorialFile,{name:'ignored.bin',webkitRelativePath:'repo/ignored.bin',size:100}]}});node('execute').onclick();
assert.equal(posted.at(-1).request.operation,'verify-tutorial-files');assert.equal(posted.at(-1).request.checksums_json,tutorialText);assert.equal(posted.at(-1).request.files.length,1);assert.equal(posted.at(-1).request.files[0].file,tutorialFile);assert.equal(posted.at(-1).request.files[0].path,'data/input.bin');assert.equal(node('tutorial-folder').disabled,true);
const tutorialUiOutput={schema_version:'bisect-tutorial-checks-v1',status:'drift-detected',native_exit_code:1,pass:0,fail:1,missing:0,rows:[],scope:'Supplied pins only'};
uiWorker.onmessage({data:{type:'result',result:tutorialUiOutput}});assert.equal(node('download-tutorial').disabled,false);node('download-tutorial').onclick();assert.equal(downloadedLink.download,'tutorial-checks.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),tutorialUiOutput);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const tutorialUiProject=createProject({...lastSnapshot,files:{tutorialInputs:lastSnapshot.files.tutorialInputs}});assert.equal(tutorialUiProject.files.tutorialInputs.checksums.base64,Buffer.from(tutorialText).toString('base64'));assert.equal(Object.hasOwn(tutorialUiProject.files.tutorialInputs,'files'),false);
importRead=()=>Promise.resolve(tutorialUiProject);const beforeTutorialOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'tutorial.bisect'}]}});assert.equal(posted.length,beforeTutorialOpen);assert.equal(node('tutorial-name').value,'synthetic');assert.equal(node('download-tutorial').disabled,true);assert.match(node('tutorial-files-name').textContent,/No local files/);
node('execute').onclick();assert.equal(posted.at(-1).request.files.length,0);uiWorker.onmessage({data:{type:'result',result:{...tutorialUiOutput,status:'files-missing',fail:0,missing:1}}});node('tutorial-name').value='different';node('tutorial-name').oninput();assert.equal(node('download-tutorial').disabled,true);
let tutorialOversizeRead=false;await node('tutorial-checksums').onchange({target:{files:[{name:'big.json',size:512*1024+1,arrayBuffer:async()=>{tutorialOversizeRead=true;}}]}});assert.equal(tutorialOversizeRead,false);
const delayedTutorial=deferred(),tutorialBytes=new TextEncoder().encode(tutorialText);const delayedTutorialRead=node('tutorial-checksums').onchange({target:{files:[{name:'stale.json',size:tutorialBytes.length,arrayBuffer:()=>delayedTutorial.promise}]}});node('tutorial-name').value='synthetic';node('tutorial-name').oninput();delayedTutorial.resolve(tutorialBytes.buffer);await delayedTutorialRead;node('save-project').onclick();assert.equal(lastSnapshot.files.tutorialInputs.checksums.name,'checksums.json');
await node('tutorial-checksums').onchange({target:{files:[{name:'invalid.json',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}]}});assert.match(node('result-status').textContent,/encoded|encoding|UTF|valid/i);
node('tutorial-files').onchange({target:{files:[{name:'../escape',size:1}]}});assert.match(node('result-status').textContent,/relative/);
node('execute').onclick();node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});assert.equal(node('download-tutorial').disabled,true);
console.log('Tutorial UI: worker-only local file reads, referenced-folder filtering, exact checksum/project/report bytes, busy/freshness/cancel guards, inert reopen with file reselection and pre-read/stale/UTF-8/path rejection passed.');

// Plan Doctor reads only supported JSON and retains inventories as unverified input.
node('operation').value='diagnose-plan-artifacts';node('operation').change();assert.equal(node('plan-doctor-fields').hidden,false);assert.equal(node('tutorial-fields').hidden,true);
const beforePlanDoctor=posted.length;node('execute').onclick();assert.equal(posted.length,beforePlanDoctor);
const doctorArtifactFile={...jsonFile('summary.json','{"population_balance_valid":false}'),webkitRelativePath:'fixture/analysis/summary.json'};
await node('plan-doctor-folder').onchange({target:{files:[doctorArtifactFile]}});node('execute').onclick();const doctorArtifactRequest=posted.at(-1).request;assert.equal(doctorArtifactRequest.operation,'diagnose-plan-artifacts');assert.equal(doctorArtifactRequest.plans[0].analysis_json['summary.json'],'{"population_balance_valid":false}');assert.equal(node('plan-doctor-folder').disabled,true);
const doctorArtifactOutput={schema_version:'bisect-plan-doctor-v1',status:'inventory-complete',plans:[{label:'fixture',status:'no-manifest'}]};uiWorker.onmessage({data:{type:'result',result:doctorArtifactOutput}});assert.equal(node('download-plan-doctor').disabled,false);node('download-plan-doctor').onclick();assert.equal(downloadedLink.download,'plan-doctor.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),doctorArtifactOutput);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const savedPlanDoctor=createProject({...lastSnapshot,files:{planDoctorInputs:lastSnapshot.files.planDoctorInputs}});importRead=()=>Promise.resolve(savedPlanDoctor);const beforePlanDoctorOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'inventory.bisect'}]}});assert.equal(posted.length,beforePlanDoctorOpen);assert.equal(node('download-plan-doctor').disabled,true);assert.equal(node('plan-doctor-folder').value,'');node('execute').onclick();assert.deepEqual(posted.at(-1).request,doctorArtifactRequest);uiWorker.onmessage({data:{type:'result',result:doctorArtifactOutput}});
let oversizePlanDoctorRead=false;await node('plan-doctor-folder').onchange({target:{files:[{name:'manifest.json',webkitRelativePath:'fixture/manifest.json',size:1024*1024+1,arrayBuffer:async()=>{oversizePlanDoctorRead=true;}}]}});assert.equal(oversizePlanDoctorRead,false);assert.equal(node('download-plan-doctor').disabled,true);
const pendingPlanDoctor=deferred(),planDoctorBytes=new TextEncoder().encode('{}');const stalePlanDoctor=node('plan-doctor-folder').onchange({target:{files:[{name:'manifest.json',webkitRelativePath:'other/manifest.json',size:2,arrayBuffer:()=>pendingPlanDoctor.promise}]}});await node('plan-doctor-folder').onchange({target:{files:[doctorArtifactFile]}});pendingPlanDoctor.resolve(planDoctorBytes.buffer);await stalePlanDoctor;node('save-project').onclick();assert.equal(lastSnapshot.files.planDoctorInputs.plans[0].label,'fixture');
node('execute').onclick();node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});assert.equal(node('download-plan-doctor').disabled,true);
console.log('Plan Doctor UI: exact requests/downloads, busy and freshness controls, source-preserving inert Save/Open, pre-read limits, stale-read preservation and cancellation passed.');

node('operation').value='validate-native-rplan';node('operation').change();assert.equal(node('native-validation-fields').hidden,false);assert.equal(node('plan-doctor-fields').hidden,true);assert.equal(node('text-audit-fields').hidden,false);
const nativeFormatText='{"rplan_version":"0.1","assignments":{}}';await node('auditPlanText-file').onchange({target:{files:[jsonFile('native.rplan',nativeFormatText)]}});node('native-validation-strict').checked=true;node('native-validation-strict').onchange();node('execute').onclick();const nativeFormatRequest=structuredClone(posted.at(-1).request);assert.deepEqual(nativeFormatRequest,{operation:'validate-native-rplan',source_name:'native.rplan',document_json:nativeFormatText,strict:true});assert.equal(node('native-validation-strict').disabled,true);
const nativeFormatResult={schema_version:'bisect-native-format-validation-v1',status:'strict-warning-failure',native_exit_code:1,warnings:['supplied warning'],strict:true};uiWorker.onmessage({data:{type:'result',result:nativeFormatResult}});assert.equal(node('download-native-validation').disabled,false);node('download-native-validation').onclick();assert.equal(downloadedLink.download,'native-format-validation.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeFormatResult);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const nativeFormatProject=createProject({...lastSnapshot,files:{auditPlanText:lastSnapshot.files.auditPlanText,nativeValidationSettings:lastSnapshot.files.nativeValidationSettings}});importRead=()=>Promise.resolve(nativeFormatProject);const beforeNativeFormatOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'format.bisect'}]}});assert.equal(posted.length,beforeNativeFormatOpen);assert.equal(node('native-validation-strict').checked,true);assert.equal(node('download-native-validation').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),nativeFormatRequest);uiWorker.onmessage({data:{type:'result',result:nativeFormatResult}});node('native-validation-strict').checked=false;node('native-validation-strict').onchange();assert.equal(node('download-native-validation').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.strict,false);node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});
// The native CLI rejects a BOM; preserve it through the final request builder.
const bomNativeFormatText='\ufeff'+nativeFormatText;
await node('auditPlanText-file').onchange({target:{files:[jsonFile('bom.rplan',bomNativeFormatText)]}});
node('execute').onclick();assert.equal(posted.at(-1).request.document_json,bomNativeFormatText);
node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});
console.log('Native format validation UI: exact request/report bytes, strict setting, busy/freshness/cancel, source-retaining inert Save/Open and disabled imported downloads passed.');

node('operation').value='assemble-plan-suite';node('operation').change();assert.equal(node('suite-fields').hidden,false);assert.equal(node('suite-assembly-fields').hidden,false);node('suite-name').value='portable';node('suite-name').oninput();node('suite-ratio').value='';node('suite-ratio').oninput();await node('suite-house').onchange({target:{files:[suiteSource('h.rplan','{"house":1}')]}});await node('suite-senate').onchange({target:{files:[suiteSource('s.rplan','{"senate":2}')]}});node('suite-assembly-mode').value='senate-in-house';node('suite-assembly-mode').onchange();await node('suite-assembly-congressional').onchange({target:{files:[suiteSource('c.rplan','{"congress":3}')]}});node('execute').onclick();const assemblyUiRequest=structuredClone(posted.at(-1).request);assert.deepEqual(assemblyUiRequest,{operation:'assemble-plan-suite',house_json:'{"house":1}',senate_json:'{"senate":2}',congressional_json:'{"congress":3}',suite_name:'portable',nest_mode:'senate-in-house',required_ratio:null,policy_source_name:null,policy_json:null});assert.equal(node('suite-assembly-mode').disabled,true);
const assemblyUiResult={schema_version:'bisect-suite-assembly-v1',suite:{suite_name:'portable'},files:{'portable/suite.json':'{}','portable_house.rplan':'{"house":1}','portable_senate.rplan':'{"senate":2}','portable_congressional.rplan':'{"congress":3}'}};uiWorker.onmessage({data:{type:'result',result:assemblyUiResult}});assert.equal(node('download-suite-package').disabled,false);node('download-suite-package').onclick();assert.equal(downloadedLink.download,'portable.zip');assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),suiteAssemblyTools.suiteAssemblyZip(assemblyUiResult));URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const assemblyUiProject=createProject({...lastSnapshot,files:{suiteInputs:lastSnapshot.files.suiteInputs,suiteAssemblyInputs:lastSnapshot.files.suiteAssemblyInputs}});importRead=()=>Promise.resolve(assemblyUiProject);const beforeAssemblyUiOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'suite-package.bisect'}]}});assert.equal(posted.length,beforeAssemblyUiOpen);assert.equal(node('download-suite-package').disabled,true);assert.equal(node('suite-assembly-mode').value,'senate-in-house');node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),assemblyUiRequest);uiWorker.onmessage({data:{type:'result',result:assemblyUiResult}});node('suite-assembly-remove-congressional').onclick();assert.equal(node('download-suite-package').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.congressional_json,null);node('cancel').onclick();uiWorker.onmessage({data:{type:'ready'}});
let hugeAssemblyRead=false;await node('suite-assembly-policy').onchange({target:{files:[{name:'huge.json',size:512*1024+1,arrayBuffer:async()=>{hugeAssemblyRead=true;}}]}});assert.equal(hugeAssemblyRead,false);
const staleAssemblyPolicy=deferred(),assemblyPolicyRead=node('suite-assembly-policy').onchange({target:{files:[{name:'old.json',size:2,arrayBuffer:()=>staleAssemblyPolicy.promise}]}});node('suite-assembly-embedded').onclick();staleAssemblyPolicy.resolve(new TextEncoder().encode('{}').buffer);await assemblyPolicyRead;node('save-project').onclick();assert.equal(lastSnapshot.files.suiteAssemblyInputs.policy,null);
console.log('Suite assembly UI: exact chamber/policy requests, ZIP bytes, busy/freshness/cancel, explicit source removal, pre-read/stale guards and source-preserving inert Save/Open passed.');

node('operation').value='inspect-civic-coi';node('operation').change();assert.equal(node('civic-fields').hidden,false);assert.equal(node('suite-fields').hidden,true);node('civic-state').value='AL';node('civic-state').onchange();node('civic-year').value='2020';node('civic-year').onchange();node('civic-validate_mode').value='lenient';node('civic-validate_mode').onchange();
const civicUiCsv='geoid,comment_id,label,source,source_url,confidence,submitted_at\n01001000100,c1,Port,public,,high,recorded\n';await node('civic-sources').onchange({target:{files:[jsonFile('one.csv',civicUiCsv)]}});node('execute').onclick();const civicUiRequest=structuredClone(posted.at(-1).request);assert.deepEqual(civicUiRequest,{operation:'inspect-civic-coi',inputs:[{label:'one.csv',source_name:'one.csv',csv_base64:Buffer.from(civicUiCsv).toString('base64')}],state:'AL',year:'2020',validate_mode:'lenient'});assert.equal(node('civic-sources').disabled,true);
const civicUiResult={schema_version:'bisect-civic-assessment-v1',status:'assessed',inputs:[{label:'one.csv',normalized_csv:civicUiCsv}],conflicts_available:true,conflicts:{coi_overlap:[],coi_label_mismatch:[]},conflicts_json:'{}'};uiWorker.onmessage({data:{type:'result',result:civicUiResult}});assert.equal(node('download-civic').disabled,false);node('download-civic').onclick();assert.equal(downloadedLink.download,'civic-assessment.zip');assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),civicTools.civicZip(civicUiResult));URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const civicUiProject=createProject({...lastSnapshot,files:{civicInputs:lastSnapshot.files.civicInputs}});importRead=()=>Promise.resolve(civicUiProject);const beforeCivicOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'civic.bisect'}]}});assert.equal(posted.length,beforeCivicOpen);assert.equal(node('download-civic').disabled,true);assert.equal(node('civic-validate_mode').value,'lenient');node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),civicUiRequest);uiWorker.onmessage({data:{type:'result',result:civicUiResult}});node('civic-state').value='RI';node('civic-state').onchange();assert.equal(node('download-civic').disabled,true);
let civicHugeRead=false;await node('civic-sources').onchange({target:{files:[{name:'huge.csv',size:3*1024*1024+1,arrayBuffer:async()=>{civicHugeRead=true;}}]}});assert.equal(civicHugeRead,false);const civicStale=deferred(),pendingCivicRead=node('civic-sources').onchange({target:{files:[{name:'late.csv',size:2,arrayBuffer:()=>civicStale.promise}]}});node('civic-clear').onclick();civicStale.resolve(new Uint8Array([65,66]).buffer);await pendingCivicRead;node('save-project').onclick();assert.equal(lastSnapshot.files.civicInputs.sources.length,0);node('execute').onclick();assert.ok(node('result-status').textContent.includes('Select civic CSV'));await node('civic-sources').onchange({target:{files:[jsonFile('one.csv',civicUiCsv)]}});node('execute').onclick();node('cancel').onclick();assert.equal(node('download-civic').disabled,true);uiWorker.onmessage({data:{type:'ready'}});
console.log('Civic UI: exact source requests/ZIP bytes, busy/freshness/cancel, source-preserving inert Save/Open, clearing and pre-read/stale guards passed.');

node('operation').value='ingest-civic-coi';node('operation').change();assert.equal(node('civic-fields').hidden,false);assert.equal(node('civic-ingest-fields').hidden,false);node('civic-ingest-label').value='community';node('civic-ingest-label').onchange();node('civic-ingest-submitter').value='Curator';node('civic-ingest-submitter').onchange();node('civic-ingest-timestamp').value='2026-10-07T12:00:00Z';node('civic-ingest-timestamp').onchange();node('execute').onclick();const civicIngestUiRequest=structuredClone(posted.at(-1).request);assert.deepEqual(civicIngestUiRequest,{operation:'ingest-civic-coi',input:{label:'community',source_name:'one.csv',csv_base64:Buffer.from(civicUiCsv).toString('base64')},state:'RI',year:'2020',validate_mode:'lenient',settings:{label:'community',submitter:'Curator',timestamp:'2026-10-07T12:00:00Z'}});assert.equal(node('civic-ingest-now').disabled,true);
const civicIngestUiResult={schema_version:'bisect-civic-ingest-v1',status:'ingested',manifest:{label:'community'},files_base64:Object.fromEntries(['original.csv','normalized.csv','validation_log.txt','manifest.json'].map(n=>['v1/civic_inputs/community/'+n,Buffer.from(n==='original.csv'?civicUiCsv:'{}').toString('base64')]))};uiWorker.onmessage({data:{type:'result',result:civicIngestUiResult}});assert.equal(node('download-civic-ingest').disabled,false);node('download-civic-ingest').onclick();assert.equal(downloadedLink.download,'community-civic.zip');assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),civicTools.civicIngestZip(civicIngestUiResult));URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const civicIngestUiProject=createProject({...lastSnapshot,files:{civicInputs:lastSnapshot.files.civicInputs,civicIngestSettings:lastSnapshot.files.civicIngestSettings}});importRead=()=>Promise.resolve(civicIngestUiProject);const beforeCivicIngestOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'ingest.bisect'}]}});assert.equal(posted.length,beforeCivicIngestOpen);assert.equal(node('download-civic-ingest').disabled,true);assert.equal(node('civic-ingest-timestamp').value,'2026-10-07T12:00:00Z');node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),civicIngestUiRequest);uiWorker.onmessage({data:{type:'result',result:civicIngestUiResult}});node('civic-ingest-now').onclick();assert.equal(node('download-civic-ingest').disabled,true);assert.match(node('civic-ingest-timestamp').value,/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);node('execute').onclick();node('cancel').onclick();assert.equal(node('download-civic-ingest').disabled,true);uiWorker.onmessage({data:{type:'ready'}});
console.log('Civic ingest UI: exact package request/ZIP bytes, explicit timestamp, busy/freshness/cancel and source-preserving inert Save/Open passed.');

node('operation').value='inspect-civic-manifests';node('operation').change();assert.equal(node('civic-manifest-fields').hidden,false);assert.equal(node('civic-fields').hidden,true);await node('civic-manifest-files').onchange({target:{files:[jsonFile('manifest.json','{"recorded":1}')]}});node('execute').onclick();const civicInventoryUiRequest=structuredClone(posted.at(-1).request);assert.deepEqual(civicInventoryUiRequest,{operation:'inspect-civic-manifests',inputs:[{source_name:'1: manifest.json',manifest_json:'{"recorded":1}'}]});assert.equal(node('civic-manifest-files').disabled,true);
const civicInventoryUiResult={schema_version:'bisect-civic-inventory-v1',native_list_text:'native list',entries:[{native_show_json:'{"recorded":1}',status:'parsed'}]};uiWorker.onmessage({data:{type:'result',result:civicInventoryUiResult}});assert.equal(node('download-civic-inventory').disabled,false);node('download-civic-inventory').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),civicTools.civicInventoryZip(civicInventoryUiResult));URL.revokeObjectURL(downloadedLink.href);node('save-project').onclick();const civicInventoryUiProject=createProject({...lastSnapshot,files:{civicManifestSources:lastSnapshot.files.civicManifestSources}});importRead=()=>Promise.resolve(civicInventoryUiProject);const beforeInventoryOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'inventory.bisect'}]}});assert.equal(posted.length,beforeInventoryOpen);assert.equal(node('download-civic-inventory').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),civicInventoryUiRequest);uiWorker.onmessage({data:{type:'result',result:civicInventoryUiResult}});
let inventoryHugeRead=false;await node('civic-manifest-files').onchange({target:{files:[{name:'large.json',size:1024*1024+1,arrayBuffer:async()=>{inventoryHugeRead=true;}}]}});assert.equal(inventoryHugeRead,false);const inventoryStale=deferred(),pendingInventory=node('civic-manifest-files').onchange({target:{files:[{name:'old.json',size:2,arrayBuffer:()=>inventoryStale.promise}]}});node('civic-manifest-clear').onclick();inventoryStale.resolve(new TextEncoder().encode('{}').buffer);await pendingInventory;node('save-project').onclick();assert.equal(lastSnapshot.files.civicManifestSources.sources.length,0);
console.log('Civic List/Show UI: exact requests/ZIP bytes, busy/freshness, inert Save/Open, clearing and pre-read/stale source guards passed.');

node('candidate-stage').appendChild=()=>{};node('operation').value='ingest-candidate-race';node('operation').change();assert.equal(node('candidate-fields').hidden,false);assert.equal(node('civic-fields').hidden,true);for(const [k,v]of Object.entries({...candidateTools.CANDIDATE_DEFAULTS,submitter:'Curator',timestamp:'2026-10-07T12:00:00Z'})){node('candidate-'+k).value=v;node('candidate-'+k).onchange();}
await node('candidate-csv').onchange({target:{files:[jsonFile('candidate.csv','candidate annotations')]}});await node('candidate-documents').onchange({target:{files:[jsonFile('letter.pdf','binary attestation')]}});node('execute').onclick();const candidateUiRequest=structuredClone(posted.at(-1).request);assert.equal(candidateUiRequest.operation,'ingest-candidate-race');assert.equal(candidateUiRequest.source_name,'candidate.csv');assert.equal(candidateUiRequest.csv_base64,Buffer.from('candidate annotations').toString('base64'));assert.equal(candidateUiRequest.staged_path,'letter.pdf');assert.equal(candidateUiRequest.documents['letter.pdf'],Buffer.from('binary attestation').toString('base64'));assert.equal(node('candidate-documents').disabled,true);
const candidateUiResult={schema_version:'bisect-candidate-ingest-v1',status:'packaged',manifest:{label:candidateUiRequest.settings.label},staged_path:'letter.pdf',files:{'annotations.json':'[]','provenance.json':'{}','manifest.json':'{}'}};uiWorker.onmessage({data:{type:'result',result:candidateUiResult}});assert.equal(node('download-candidate').disabled,false);node('save-project').onclick();const candidateUiInputs=lastSnapshot.files.candidateInputs;node('download-candidate').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),candidateTools.candidateZip(candidateUiResult,candidateUiInputs));URL.revokeObjectURL(downloadedLink.href);const candidateUiProject=createProject({...lastSnapshot,files:{candidateInputs:candidateUiInputs}});importRead=()=>Promise.resolve(candidateUiProject);const beforeCandidateOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'candidate.bisect'}]}});assert.equal(posted.length,beforeCandidateOpen);assert.equal(node('download-candidate').disabled,true);assert.equal(node('candidate-stage').value,'letter.pdf');node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),candidateUiRequest);uiWorker.onmessage({data:{type:'result',result:candidateUiResult}});node('candidate-now').onclick();assert.equal(node('download-candidate').disabled,true);
let candidateHugeRead=false;await node('candidate-csv').onchange({target:{files:[{name:'large.csv',size:1024*1024+1,arrayBuffer:async()=>{candidateHugeRead=true;}}]}});assert.equal(candidateHugeRead,false);const candidateStale=deferred(),pendingCandidate=node('candidate-csv').onchange({target:{files:[{name:'old.csv',size:2,arrayBuffer:()=>candidateStale.promise}]}});node('candidate-clear').onclick();candidateStale.resolve(new Uint8Array([65,66]).buffer);await pendingCandidate;node('save-project').onclick();assert.equal(lastSnapshot.files.candidateInputs.csv,null);assert.equal(Object.keys(lastSnapshot.files.candidateInputs.documents).length,0);
node('candidate-label').value='../unfinished';node('candidate-label').onchange();assert.doesNotThrow(()=>node('candidate-clear').onclick());node('save-project').onclick();assert.equal(lastSnapshot.files.candidateInputs.csv,null);assert.equal(lastSnapshot.files.candidateInputs.settings.label,candidateUiInputs.settings.label);assert.equal(node('candidate-label').value,'../unfinished');
console.log('Candidate annotation UI: exact binary inputs/request/ZIP bytes, primary document/timestamp retention, busy/freshness, inert Save/Open, invalid-metadata clear and pre-read/stale guards passed.');
node('operation').value='migrate-legacy-plan';node('operation').change();assert.equal(node('migration-fields').hidden,false);assert.equal(node('candidate-fields').hidden,true);for(const [k,v]of Object.entries({...migrationTools.MIGRATION_DEFAULTS,created_at:'2026-10-07T18:00:00Z'})){node('migration-'+k).value=v;node('migration-'+k).onchange();}
const legacyFile={...jsonFile('legacy.bin','binary'),webkitRelativePath:'washington/data/legacy.bin'};await node('migration-folder').onchange({target:{files:[legacyFile]}});node('execute').onclick();const migrationUiRequest=structuredClone(posted.at(-1).request);assert.equal(migrationUiRequest.files_base64['data/legacy.bin'],Buffer.from('binary').toString('base64'));assert.equal(migrationUiRequest.settings.source_dir_name,'washington');assert.equal(node('migration-folder').disabled,true);const migrationUiResult={schema_version:'bisect-legacy-migration-v1',status:'packaged',manifest:{label:migrationUiRequest.settings.label},manifest_json:'{}'};uiWorker.onmessage({data:{type:'result',result:migrationUiResult}});assert.equal(node('download-migration').disabled,false);node('save-project').onclick();const migrationUiInputs=lastSnapshot.files.migrationInputs;node('download-migration').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),migrationTools.migrationZip(migrationUiResult,migrationUiInputs));URL.revokeObjectURL(downloadedLink.href);const migrationProject=createProject({...lastSnapshot,files:{migrationInputs:migrationUiInputs}});importRead=()=>Promise.resolve(migrationProject);const migrationBeforeOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'legacy.bisect'}]}});assert.equal(posted.length,migrationBeforeOpen);assert.equal(node('download-migration').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),migrationUiRequest);uiWorker.onmessage({data:{type:'result',result:migrationUiResult}});node('migration-now').onclick();assert.equal(node('download-migration').disabled,true);let migrationHugeRead=false;await node('migration-folder').onchange({target:{files:[{name:'huge',webkitRelativePath:'washington/huge',size:16*1024*1024+1,arrayBuffer:async()=>{migrationHugeRead=true;}}]}});assert.equal(migrationHugeRead,false);const migrationStale=deferred(),migrationPending=node('migration-folder').onchange({target:{files:[{name:'old',webkitRelativePath:'washington/old',size:2,arrayBuffer:()=>migrationStale.promise}]}});node('migration-clear').onclick();migrationStale.resolve(new Uint8Array([65,66]).buffer);await migrationPending;node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.migrationInputs.files).length,0);
console.log('Legacy migration UI: exact source/request/ZIP, busy/freshness, inert saved projects, timestamp edits and pre-read/stale clearing passed.');
node('operation').value='verify-label-chain';node('operation').change();assert.equal(node('label-chain-fields').hidden,false);assert.equal(node('migration-fields').hidden,true);for(const [k,v]of Object.entries(labelChainTools.LABEL_CHAIN_DEFAULTS)){node('label-chain-'+k).value=v;node('label-chain-'+k).onchange();}const labelRegistryText=JSON.stringify({official_proposal:{built:['2020'],analyzed:[],reported:[]}});await node('label-chain-registry').onchange({target:{files:[jsonFile('.bisect',labelRegistryText)]}});await node('label-chain-folder').onchange({target:{files:[{...jsonFile('index.json','{}'),webkitRelativePath:'repo/runs/official_proposal/2020/index.json'},{...jsonFile('.bisect',labelRegistryText),webkitRelativePath:'repo/.bisect'}]}});node('execute').onclick();const labelChainUiRequest=structuredClone(posted.at(-1).request);assert.equal(labelChainUiRequest.registry_json,labelRegistryText);assert.equal(labelChainUiRequest.files_base64['runs/official_proposal/2020/index.json'],Buffer.from('{}').toString('base64'));assert.equal(node('label-chain-folder').disabled,true);const labelChainUiResult={schema_version:'bisect-label-chain-v1',status:'failed',label:'official_proposal',native_console_text:'Verdict: FAILED\n'};uiWorker.onmessage({data:{type:'result',result:labelChainUiResult}});assert.equal(node('download-label-chain').disabled,false);node('download-label-chain').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),labelChainUiResult.native_console_text);URL.revokeObjectURL(downloadedLink.href);node('save-project').onclick();const labelChainUiInputs=lastSnapshot.files.labelChainInputs;const labelChainProject=createProject({...lastSnapshot,files:{labelChainInputs:labelChainUiInputs}});importRead=()=>Promise.resolve(labelChainProject);const labelBeforeOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'label.bisect'}]}});assert.equal(posted.length,labelBeforeOpen);assert.equal(node('download-label-chain').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),labelChainUiRequest);uiWorker.onmessage({data:{type:'result',result:labelChainUiResult}});node('label-chain-year_filter').value='2010';node('label-chain-year_filter').oninput();assert.equal(node('download-label-chain').disabled,true);let labelHugeRead=false;await node('label-chain-folder').onchange({target:{files:[{name:'huge',webkitRelativePath:'repo/huge',size:16*1024*1024+1,arrayBuffer:async()=>{labelHugeRead=true;}}]}});assert.equal(labelHugeRead,false);const labelStale=deferred(),labelPending=node('label-chain-folder').onchange({target:{files:[{name:'old',webkitRelativePath:'repo/old',size:2,arrayBuffer:()=>labelStale.promise}]}});node('label-chain-clear').onclick();labelStale.resolve(new Uint8Array([65,66]).buffer);await labelPending;node('save-project').onclick();assert.equal(lastSnapshot.files.labelChainInputs.registry,null);assert.equal(Object.keys(lastSnapshot.files.labelChainInputs.files).length,0);
console.log('Label SHA-chain UI: exact selected bytes/request/native text download, failure exports, busy/freshness, inert Save/Open, year edits and pre-read/stale clearing passed.');
{

node('operation').value='inspect-label-registry';node('operation').change();assert.equal(node('label-inventory-fields').hidden,false);assert.equal(node('label-chain-fields').hidden,true);node('label-inventory-label').value='official_proposal';node('label-inventory-label').oninput();await node('label-inventory-registry').onchange({target:{files:[jsonFile('.bisect',labelRegistryText)]}});node('execute').onclick();const inventoryUiRequest=structuredClone(posted.at(-1).request);assert.deepEqual(inventoryUiRequest,{operation:'inspect-label-registry',registry_json:labelRegistryText,label:'official_proposal'});assert.equal(node('label-inventory-registry').disabled,true);const inventoryUiResult={schema_version:'bisect-label-inventory-v1',status:'listed',selected_label:'official_proposal',show_native_exit_code:0,native_list_text:'native table\n',native_list_json:'{}\n',native_show_text:'native details\n',native_show_json:'{}\n'};uiWorker.onmessage({data:{type:'result',result:inventoryUiResult}});assert.equal(node('use-label-chain').disabled,false);for(const k of ['list-text','list-json','show-text','show-json']){assert.equal(node('download-label-'+k).disabled,false);node('download-label-'+k).onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),inventoryUiResult['native_'+k.replace('-','_')]);URL.revokeObjectURL(downloadedLink.href);}node('save-project').onclick();const inventoryUiInputs=lastSnapshot.files.labelInventoryInputs;const inventoryProject=createProject({...lastSnapshot,files:{labelInventoryInputs:inventoryUiInputs}});importRead=()=>Promise.resolve(inventoryProject);const inventoryBeforeOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'inventory.bisect'}]}});assert.equal(posted.length,inventoryBeforeOpen);assert.equal(node('download-label-list-text').disabled,true);assert.equal(node('use-label-chain').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),inventoryUiRequest);uiWorker.onmessage({data:{type:'result',result:inventoryUiResult}});const beforeInventoryHandoff=posted.length;node('use-label-chain').onclick();assert.equal(posted.length,beforeInventoryHandoff);assert.equal(node('operation').value,'verify-label-chain');assert.equal(node('label-chain-label').value,'official_proposal');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.labelChainInputs.registry,inventoryUiInputs.registry);assert.equal(node('download-label-chain').disabled,true);node('operation').value='inspect-label-registry';node('operation').change();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:inventoryUiResult}});node('label-inventory-label').value='missing';node('label-inventory-label').oninput();assert.equal(node('download-label-list-text').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:{...inventoryUiResult,status:'label-not-found',show_native_exit_code:1,native_show_text:null,native_show_json:null}}});assert.equal(node('download-label-list-text').disabled,false);assert.equal(node('download-label-show-text').disabled,true);assert.equal(node('use-label-chain').disabled,true);let inventoryHugeRead=false;await node('label-inventory-registry').onchange({target:{files:[{name:'.bisect',size:1024*1024+1,arrayBuffer:async()=>{inventoryHugeRead=true;}}]}});assert.equal(inventoryHugeRead,false);const inventoryStale=deferred(),inventoryPending=node('label-inventory-registry').onchange({target:{files:[{name:'.bisect',size:2,arrayBuffer:()=>inventoryStale.promise}]}});node('label-inventory-clear').onclick();inventoryStale.resolve(new Uint8Array([123,125]).buffer);await inventoryPending;node('save-project').onclick();assert.equal(lastSnapshot.files.labelInventoryInputs.registry,null);
console.log('Label inventory UI: native list/show text/JSON downloads, exact request, busy/freshness, inert Save/Open, missing-label exports, explicit no-run chain handoff and pre-read/stale clearing passed.');

}

{
node('operation').value='rename-label-package';node('operation').change();
assert.equal(node('label-rename-fields').hidden,false);
node('label-rename-source_label').value='a';node('label-rename-destination_label').value='b';node('label-rename-force').checked=false;node('label-rename-directories').value='reports/a/empty/';node('label-rename-source_label').oninput();
const renameRegistry=JSON.stringify({a:{built:['2020'],analyzed:[],reported:[]}});
await node('label-rename-registry').onchange({target:{files:[jsonFile('.bisect',renameRegistry)]}});
await node('label-rename-folder').onchange({target:{files:[{name:'data.bin',webkitRelativePath:'repo/runs/a/data.bin',size:2,arrayBuffer:async()=>new Uint8Array([0,255]).buffer}]}});
node('execute').onclick();const renameRequest=structuredClone(posted.at(-1).request);
assert.deepEqual(renameRequest,{operation:'rename-label-package',registry_json:renameRegistry,files_base64:{'runs/a/data.bin':'AP8='},directories:['reports/a/empty/'],source_label:'a',destination_label:'b',force:false});
assert.equal(node('label-rename-folder').disabled,true);assert.equal(node('download-label-rename').disabled,true);
const renameResult={schema_version:'bisect-label-rename-v1',status:'packaged',source_label:'a',destination_label:'b',force:false,files:[{path:'.bisect',base64:Buffer.from('{"b":{"built":["2020"],"analyzed":[],"reported":[]}}').toString('base64')},{path:'runs/b/data.bin',base64:'AP8='}],directories:['reports/b/empty/']};
uiWorker.onmessage({data:{type:'result',result:renameResult}});assert.equal(node('download-label-rename').disabled,false);
node('save-project').onclick();const renameInputs=lastSnapshot.files.labelRenameInputs;
node('download-label-rename').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),labelRenameTools.labelRenameZip(renameResult,renameInputs));URL.revokeObjectURL(downloadedLink.href);
const savedRename=createProject({...lastSnapshot,files:{labelRenameInputs:renameInputs}});importRead=()=>Promise.resolve(savedRename);const beforeRenameOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'rename.bisect'}]}});assert.equal(posted.length,beforeRenameOpen);assert.equal(node('download-label-rename').disabled,true);
node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),renameRequest);uiWorker.onmessage({data:{type:'result',result:renameResult}});
node('label-rename-destination_label').value='c';node('label-rename-destination_label').oninput();assert.equal(node('download-label-rename').disabled,true);
let renameOversizeRead=false;await node('label-rename-folder').onchange({target:{files:[{name:'big',webkitRelativePath:'repo/runs/a/big',size:16*1024*1024+1,arrayBuffer:async()=>{renameOversizeRead=true;}}]}});assert.equal(renameOversizeRead,false);
const renameStale=deferred(),renamePending=node('label-rename-folder').onchange({target:{files:[{name:'x',webkitRelativePath:'repo/runs/a/x',size:1,arrayBuffer:()=>renameStale.promise}]}});node('label-rename-clear').onclick();renameStale.resolve(new Uint8Array([1]).buffer);await renamePending;node('save-project').onclick();assert.deepEqual(Object.keys(lastSnapshot.files.labelRenameInputs.files),[]);assert.equal(lastSnapshot.files.labelRenameInputs.registry,null);
console.log('Label rename UI: exact request and ZIP, busy/freshness, inert Save/Open, pre-read size guard and stale-source clearing passed.');
}

{
node('operation').value='run-label-build';node('operation').change();assert.equal(node('label-build-fields').hidden,false);
node('label-build-label').value='batch';node('label-build-profile').value='open';node('label-build-years').value='2020';node('label-build-force').checked=false;node('label-build-created_at').value='recorded';node('label-build-label').oninput();
await node('label-build-config').onchange({target:{files:[jsonFile('batch.yml','name: batch\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n')]}});
await node('label-build-jobs').onchange({target:{files:[jsonFile('jobs.json',JSON.stringify([{graph:{state:'RI',year:'2020'},districts:2,chamber:'congressional',demographics:null}]))]}});
node('execute').onclick();const buildPosted=structuredClone(posted.at(-1));assert.equal(buildPosted.request.operation,'run-label-build-sources');assert.deepEqual(Array.from(buildPosted.request.sources.settings.years),['2020']);assert.equal(node('label-build-jobs').disabled,true);
const buildProgress={schema_version:'bisect-label-build-progress-v1',label:'batch',request_sha256:'a'.repeat(64),sequence:1,kind:'state-completed',state:{state:'RI',state_name:'rhode_island',year:'2020',status:'ok'},files:{'runs/batch/2020/rhode_island/plan.rplan':'{}','runs/batch/2020/rhode_island/context.rctx':'{}','runs/batch/2020/rhode_island/final_assignments.json':'{"0":1}'}};
uiWorker.onmessage({data:{type:'progress',id:buildPosted.id+1,progress:buildProgress}});assert.match(node('result-status').textContent,/Running/);
uiWorker.onmessage({data:{type:'build-started',id:buildPosted.id,checkpoint:labelBuildTools.startLabelBuildCheckpoint(labelBuildTools.labelBuildRequest(buildPosted.request.sources))}});uiWorker.onmessage({data:{type:'progress',id:buildPosted.id,progress:buildProgress}});assert.match(node('result-status').textContent,/1 completed/);assert.equal(node('download-label-build').disabled,true);
node('cancel').onclick();assert.equal(node('download-label-build').disabled,false);assert.match(node('result-status').textContent,/retained/);assert.equal(JSON.parse(node('result').textContent).status,'cancelled');node('save-project').onclick();assert.equal(lastSnapshot.result.status,'cancelled');assert.equal(lastSnapshot.result.years.length,0);assert.deepEqual(JSON.parse(lastSnapshot.result.files['.bisect']),{});
const savedBuild=createProject(lastSnapshot);node('download-label-build').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),labelBuildTools.labelBuildZip(savedBuild.result));URL.revokeObjectURL(downloadedLink.href);
uiWorker.onmessage({data:{type:'ready'}});importRead=()=>Promise.resolve(savedBuild);const beforeBuildOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'build.bisect'}]}});assert.equal(posted.length,beforeBuildOpen);assert.equal(node('download-label-build').disabled,true);
node('execute').onclick();const rerun=structuredClone(posted.at(-1));assert.deepEqual(rerun.request,buildPosted.request);uiWorker.onmessage({data:{type:'result',result:{...savedBuild.result,status:'built'}}});assert.equal(node('download-label-build').disabled,false);node('label-build-years').value='2010';node('label-build-years').oninput();assert.equal(node('download-label-build').disabled,true);
let buildOversize=false;await node('label-build-jobs').onchange({target:{files:[{name:'huge.json',size:16*1024*1024+1,arrayBuffer:async()=>{buildOversize=true;}}]}});assert.equal(buildOversize,false);
const buildStale=deferred(),buildPending=node('label-build-config').onchange({target:{files:[{name:'a.yml',size:1,arrayBuffer:()=>buildStale.promise}]}});node('label-build-clear').onclick();buildStale.resolve(new Uint8Array([97]).buffer);await buildPending;node('save-project').onclick();assert.equal(lastSnapshot.files.labelBuildInputs.config,null);
console.log('Label build UI: exact request, busy/stale-ID guards, cancellation retains received outputs without marking unfinished years built, exact ZIP, inert Save/Open, freshness and pre-read/stale clearing passed.');
}

{
const reader=sandbox.readConfigCatalog,loader=sandbox.prepareBuildCatalogJobs;
sandbox.readConfigCatalog=async()=>publishedFixture;
node('label-build-published').replaceChildren=()=>{};node('label-build-published').append=()=>{};
node('operation').value='run-label-build';node('operation').change();
const postsBefore=posted.length;await node('label-build-catalog').onclick();assert.equal(posted.length,postsBefore);assert.equal(node('label-build-prepare').disabled,false);
node('label-build-published').selectedOptions=[{value:'RI:2020'}];node('label-build-chamber').value='house';
const source={name:'published-house-jobs.json',base64:Buffer.from(JSON.stringify([{graph:{state:'RI',year:'2020'},districts:75,chamber:'house',demographics:null}])).toString('base64')};
const prepared={source,years:['2020'],selected:publishedFixture.entries};
sandbox.prepareBuildCatalogJobs=async(c,keys,chamber,inputs)=>{assert.deepEqual(Array.from(keys),['RI:2020']);assert.equal(chamber,'house');return {...prepared,inputs:{...inputs,jobs:source,settings:{...inputs.settings,years:prepared.years}}};};
await node('label-build-prepare').onclick();assert.equal(posted.length,postsBefore);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.labelBuildInputs.jobs,source);assert.deepEqual(Array.from(lastSnapshot.files.labelBuildInputs.settings.years),['2020']);assert.equal(node('label-build-years').value,'2020');
const old=lastSnapshot.files.labelBuildInputs.jobs,delayed=deferred();sandbox.prepareBuildCatalogJobs=()=>delayed.promise;
const pending=node('label-build-prepare').onclick();assert.equal(node('execute').disabled,true);assert.equal(node('label-build-chamber').disabled,true);node('execute').onclick();assert.equal(posted.length,postsBefore);
node('project-name').value='Changed while preparing';node('project-name').oninput();delayed.resolve({...prepared,source:{...source,name:'stale.json'}});await pending;node('save-project').onclick();assert.deepEqual(lastSnapshot.files.labelBuildInputs.jobs,old);assert.match(node('result-status').textContent,/Project changed/);
let preparationAborted=false;
sandbox.prepareBuildCatalogJobs=(c,keys,chamber,inputs,{signal})=>new Promise((resolve,reject)=>{signal.addEventListener('abort',()=>{preparationAborted=true;reject(new Error('Build job preparation cancelled.'));},{once:true});});
const cancelling=node('label-build-prepare').onclick();assert.equal(node('label-build-cancel-prepare').disabled,false);assert.equal(node('execute').disabled,true);node('label-build-cancel-prepare').onclick();await cancelling;assert.equal(preparationAborted,true);assert.equal(node('label-build-cancel-prepare').disabled,true);assert.equal(node('execute').disabled,false);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.labelBuildInputs.jobs,old);assert.equal(posted.length,postsBefore);
sandbox.readConfigCatalog=reader;sandbox.prepareBuildCatalogJobs=loader;
console.log('Build catalog UI: explicit worker preparation without engine execution, chamber/year/source retention, busy Run guard, cancellation cleanup and stale-load preservation passed.');
}

{
const archiveTools=await import('../../web/lab/label-build-archive.js');
const originalArchiveCreator=sandbox.serializeBuildArchive;sandbox.serializeBuildArchive=async v=>archiveTools.createBuildArchive(structuredClone(v));
const archiveSource=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const archiveInputs={config:archiveSource('archive.yml','name: archive\nalgorithm:\n  engine: metis-core\n'),jobs:archiveSource('jobs.json',JSON.stringify([{graph:{state:'RI',year:'2020'},districts:2,chamber:'congressional',demographics:null}])),registry:null,settings:{label:'archive',profile:'open',years:['2020'],force:false,created_at:'2026-10-07T00:00:00Z'}};
const archiveResult={schema_version:'bisect-label-build-v1',status:'built',label:'archive',states:[],years:[],files:{'configs/archive.yml':'exact retained YAML'},scope:'Unverified UI fixture.'};
const archived=archiveTools.parseBuildArchive(archiveTools.createBuildArchive({name:'Archive',inputs:archiveInputs,result:archiveResult,lastOperation:'run-label-build'}));
const originalArchiveReader=sandbox.readBuildArchiveFile;sandbox.readBuildArchiveFile=async()=>archived;
const before=posted.length;await node('label-build-open-archive').onchange({target:{files:[{name:'build.bisect-build'}]}});assert.equal(posted.length,before);assert.match(node('result-status').textContent,/unverified/);assert.equal(node('download-label-build').disabled,true);
await node('label-build-save-archive').onclick();const exported=archiveTools.parseBuildArchive(await(await fetch(downloadedLink.href)).text());assert.deepEqual(exported.files,archived.files);assert.deepEqual(exported.result,archived.result);URL.revokeObjectURL(downloadedLink.href);
node('execute').onclick();assert.equal(node('label-build-save-archive').disabled,true);assert.equal(node('label-build-open-archive').disabled,true);await node('label-build-save-archive').onclick();assert.match(node('project-status').textContent,/Finish or cancel/);uiWorker.onmessage({data:{type:'result',result:archived.result}});
const delayed=deferred();sandbox.readBuildArchiveFile=()=>delayed.promise;const pending=node('label-build-open-archive').onchange({target:{files:[{name:'stale.bisect-build'}]}});node('project-name').value='Keep current edit';node('project-name').oninput();delayed.resolve({...archived,name:'Stale name'});await pending;assert.equal(node('project-name').value,'Keep current edit');assert.match(node('project-status').textContent,/changed/);
const saveDelayed=deferred(),previousDownload=downloadedLink;sandbox.serializeBuildArchive=()=>saveDelayed.promise;const pendingSave=node('label-build-save-archive').onclick();assert.equal(node('execute').disabled,true);node('project-name').value='Edited during save';node('project-name').oninput();saveDelayed.resolve(archiveTools.createBuildArchive({name:'Stale archive',inputs:archiveInputs,result:archiveResult,lastOperation:'run-label-build'}));await pendingSave;assert.equal(downloadedLink,previousDownload);assert.match(node('project-status').textContent,/changed/);
sandbox.readBuildArchiveFile=originalArchiveReader;sandbox.serializeBuildArchive=originalArchiveCreator;
console.log('Build archive UI: exact saved sources/results, inert Open, disabled imported ZIP, busy guards and stale import preservation passed.');
}

{
node('operation').value='assemble-commission-report';node('operation').change();node('commission-time').value='2026-10-07T01:00:00Z';node('commission-time').onchange();
const folder=[['manifest.json','{"label":"ui"}'],['analysis/summary.json','{"total":9007199254740993}'],['analysis/contiguity.json','{}'],['analysis/compactness.json','{}']].map(([p,t])=>({...jsonFile(p.split('/').at(-1),t),webkitRelativePath:'plan/'+p}));
await node('commission-folder').onchange({target:{files:folder}});assert.equal(node('commission-fields').hidden,false);assert.equal(node('plan-file').parentElement.hidden,true);node('execute').onclick();const request=structuredClone(posted.at(-1).request);assert.equal(request.operation,'assemble-commission-report');assert.equal(request.generated_at,'2026-10-07T01:00:00Z');assert.equal(Buffer.from(request.files['analysis/summary.json']).toString(),'{"total":9007199254740993}');assert.equal(node('commission-folder').disabled,true);
const native={schema_version:'bisect-commission-report-v1',report_json:'{"sections":{},"seed":9007199254740993}',report_html:'<!doctype html><title>Escaped &lt;script&gt;</title>',audit_json:'{"audit":{"seed":9007199254740993}}',label:'ui',state:'RI',year:'2020',source_boundary:'Unverified UI fixture'};uiWorker.onmessage({data:{type:'result',result:native}});assert.equal(node('download-commission-json').disabled,false);node('download-commission-json').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),native.report_json);URL.revokeObjectURL(downloadedLink.href);assert.equal(node('download-commission-html').disabled,false);node('download-commission-html').onclick();assert.equal(downloadedLink.download,'commission-report.html');const htmlBlob=await fetch(downloadedLink.href);assert.match(htmlBlob.headers.get('content-type'),/text\/html/);assert.equal(await htmlBlob.text(),native.report_html);URL.revokeObjectURL(downloadedLink.href);
assert.equal(node('download-commission-audit').disabled,false);node('download-commission-audit').onclick();assert.equal(downloadedLink.download,'audit.json');assert.equal(await(await fetch(downloadedLink.href)).text(),native.audit_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const inputs=structuredClone(lastSnapshot.files.commissionInputs);const project=createProject({...lastSnapshot,files:{commissionInputs:inputs}});importRead=()=>Promise.resolve(project);const before=posted.length;await node('open-project').onchange({target:{files:[{name:'commission.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-commission-json').disabled,true);assert.equal(node('download-commission-html').disabled,true);assert.equal(node('download-commission-audit').disabled,true);assert.equal(node('commission-time').value,request.generated_at);
node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),request);uiWorker.onmessage({data:{type:'result',result:native}});node('commission-time').value='changed';node('commission-time').oninput();assert.equal(node('download-commission-json').disabled,true);assert.equal(node('download-commission-html').disabled,true);
let hugeRead=false;await node('commission-folder').onchange({target:{files:[{name:'manifest.json',size:16*1024*1024+1,arrayBuffer:async()=>{hugeRead=true;}}]}});assert.equal(hugeRead,false);
const delayed=deferred();const pending=node('commission-folder').onchange({target:{files:[{name:'manifest.json',size:2,arrayBuffer:()=>delayed.promise}]}});node('commission-clear').onclick();delayed.resolve(Uint8Array.from([123,125]).buffer);await pending;node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.commissionInputs.files).length,0);assert.match(node('result-status').textContent,/Project changed/);
console.log('Commission report UI: exact source request/download, busy controls, inert Save/Open, unchanged rerun, timestamp freshness and pre-read/stale clearing passed.');
}

{
node('operation').value='assemble-commission-audit';node('operation').change();assert.equal(node('commission-audit-fields').hidden,false);assert.equal(node('commission-fields').hidden,true);assert.equal(node('plan-file').parentElement.hidden,true);
const manifest='{"label":"audit-ui","seed":9007199254740993,"rctx_path":"missing.rctx"}';
await node('commission-audit-source').onchange({target:{files:[jsonFile('manifest.json',manifest)]}});node('execute').onclick();const request=structuredClone(posted.at(-1).request);assert.deepEqual(request,{operation:'assemble-commission-audit',manifest_json:manifest});assert.equal(node('commission-audit-source').disabled,true);assert.equal(node('commission-audit-clear').disabled,true);
const output={schema_version:'bisect-commission-audit-v1',audit_json:'{"label":"audit-ui","audit":{"seed":9007199254740993}}',label:'audit-ui'};uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-commission-audit-only').disabled,false);assert.equal(node('download-commission-html').disabled,true);node('download-commission-audit-only').onclick();assert.equal(downloadedLink.download,'audit.json');assert.equal(await(await fetch(downloadedLink.href)).text(),output.audit_json);URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const source=structuredClone(lastSnapshot.files.commissionAuditSource);const project=createProject({...lastSnapshot,files:{commissionAuditSource:source}});importRead=()=>Promise.resolve(project);const before=posted.length;await node('open-project').onchange({target:{files:[{name:'audit.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-commission-audit-only').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),request);uiWorker.onmessage({data:{type:'result',result:output}});
node('commission-audit-clear').onclick();assert.equal(node('download-commission-audit-only').disabled,true);let read=false;await node('commission-audit-source').onchange({target:{files:[{name:'huge.json',size:16*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
const delayed=deferred();const pending=node('commission-audit-source').onchange({target:{files:[{name:'manifest.json',size:2,arrayBuffer:()=>delayed.promise}]}});node('commission-audit-clear').onclick();delayed.resolve(Uint8Array.from([123,125]).buffer);await pending;node('save-project').onclick();assert.equal(lastSnapshot.files.commissionAuditSource,undefined);assert.match(node('result-status').textContent,/Project changed/);
console.log('Commission audit-only UI: manifest-only request, exact audit download, busy guards, inert Open, identical rerun, freshness and pre-read/stale clearing passed.');
}
{
node('operation').value='assemble-commission-report';node('operation').change();node('commission-fallback-label').value='fallback-missing';node('commission-fallback-year').value='2020';node('commission-fallback-enabled').checked=true;node('commission-fallback-enabled').onchange();node('commission-time').value='2026-10-07T01:00:00Z';node('commission-time').onchange();
const entries=[['summary.json','{}'],['contiguity.json','{}'],['compactness.json','{}']].map(([name,text])=>({...jsonFile(name,text),webkitRelativePath:'legacy/analysis/'+name}));await node('commission-folder').onchange({target:{files:entries}});node('execute').onclick();const request=structuredClone(posted.at(-1).request);assert.deepEqual(request.fallback,{label:'fallback-missing',year:'2020'});assert.equal(request.files['manifest.json'],undefined);assert.equal(node('commission-fallback-enabled').disabled,true);
const output={schema_version:'bisect-commission-report-v1',report_json:'{"label":"fallback-missing","sections":{}}',report_html:'<!doctype html><title>Fallback</title>',audit_json:'{"label":"fallback-missing","audit":{"seed":null}}',manifest_resolution:{fallback_used:true,source_status:'missing'},label:'fallback-missing'};uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-commission-html').disabled,false);node('save-project').onclick();const savedInputs=structuredClone(lastSnapshot.files.commissionInputs);assert.equal(savedInputs.opaque_manifest,true);const project=createProject({...lastSnapshot,files:{commissionInputs:savedInputs,commissionFallbackSettings:structuredClone(lastSnapshot.files.commissionFallbackSettings)}});importRead=()=>Promise.resolve(project);const before=posted.length;await node('open-project').onchange({target:{files:[{name:'legacy.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('commission-fallback-enabled').checked,true);assert.equal(node('download-commission-html').disabled,true);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),request);uiWorker.onmessage({data:{type:'result',result:output}});
node('commission-fallback-enabled').checked=false;node('commission-fallback-enabled').onchange();assert.equal(node('download-commission-html').disabled,true);node('save-project').onclick();assert.deepEqual(structuredClone(lastSnapshot.files.commissionInputs),savedInputs);const count=posted.length;node('execute').onclick();assert.equal(posted.length,count);assert.match(node('result-status').textContent,/explicit fallback/);
node('operation').value='assemble-commission-audit';node('operation').change();node('commission-audit-clear').onclick();node('commission-fallback-enabled').checked=true;node('commission-fallback-enabled').onchange();node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),{operation:'assemble-commission-audit',fallback:{label:'fallback-missing',year:'2020'}});const audit={schema_version:'bisect-commission-audit-v1',audit_json:'{"audit":{"seed":null}}',label:'fallback-missing',manifest_resolution:{fallback_used:true,source_status:'missing'}};uiWorker.onmessage({data:{type:'result',result:audit}});
await node('commission-audit-source').onchange({target:{files:[{name:'corrupt.json',size:1,arrayBuffer:async()=>Uint8Array.from([255]).buffer}]}});node('execute').onclick();const opaqueRequest=structuredClone(posted.at(-1).request);assert.deepEqual(opaqueRequest.manifest_bytes,[255]);assert.equal(opaqueRequest.manifest_json,undefined);uiWorker.onmessage({data:{type:'result',result:audit}});node('save-project').onclick();assert.equal(lastSnapshot.files.commissionAuditOpaqueSource.base64,'/w==');const opaqueSource=structuredClone(lastSnapshot.files.commissionAuditOpaqueSource);node('commission-fallback-enabled').checked=false;node('commission-fallback-enabled').onchange();node('save-project').onclick();assert.deepEqual(structuredClone(lastSnapshot.files.commissionAuditOpaqueSource),opaqueSource);node('execute').onclick();assert.equal(posted.at(-1).request.fallback,undefined);assert.deepEqual(structuredClone(posted.at(-1).request.manifest_bytes),[255]);uiWorker.onmessage({data:{type:'error',message:'Invalid manifest without fallback'}});
node('commission-fallback-enabled').checked=true;node('commission-fallback-enabled').onchange();const delayed=deferred();const pending=node('commission-audit-source').onchange({target:{files:[{name:'late.json',size:1,arrayBuffer:()=>delayed.promise}]}});node('commission-fallback-label').value='edited';node('commission-fallback-label').oninput();delayed.resolve(Uint8Array.from([1]).buffer);await pending;node('save-project').onclick();assert.deepEqual(structuredClone(lastSnapshot.files.commissionAuditOpaqueSource),opaqueSource);assert.match(node('result-status').textContent,/Project changed/);
console.log('Commission fallback UI: missing-manifest full run, exact opt-in request, inert settings/source Open, identical rerun, disabled fallback preserves bytes, audit without source, corrupt byte-vector request and stale-edit/busy/freshness guards passed.');
}
{
node('operation').value='assemble-commission-report';node('operation').change();node('commission-fallback-enabled').checked=false;node('commission-fallback-enabled').onchange();node('commission-delivery-label').value='delivery-ui';node('commission-delivery-html').checked=false;node('commission-delivery-json').checked=true;node('commission-delivery-audit').checked=false;node('commission-delivery-label').onchange();node('commission-time').value='2026-10-07T01:00:00Z';node('commission-time').onchange();
const entries=[['manifest.json','{"label":"ui"}'],['analysis/summary.json','{}'],['analysis/contiguity.json','{}'],['analysis/compactness.json','{}']].map(([p,text])=>({...jsonFile(p.split('/').at(-1),text),webkitRelativePath:'plan/'+p}));await node('commission-folder').onchange({target:{files:entries}});node('execute').onclick();const request=structuredClone(posted.at(-1).request);assert.deepEqual(request.delivery,{label:'delivery-ui',formats:['json'],audit_with_report:false});assert.equal(node('commission-delivery-html').disabled,true);assert.equal(node('download-commission-zip').disabled,true);
const text='{"seed":9007199254740993}',output={schema_version:'bisect-commission-report-v1',report_json:text,report_html:null,audit_json:'{"seed":null}',delivery_settings:request.delivery,report_files:{'delivery-ui_report.json':text},label:'ui'};uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-commission-html').disabled,true);assert.equal(node('download-commission-json').disabled,false);assert.equal(node('download-commission-audit').disabled,true);assert.equal(node('download-commission-zip').disabled,false);node('download-commission-json').onclick();assert.equal(downloadedLink.download,'delivery-ui_report.json');assert.equal(await(await fetch(downloadedLink.href)).text(),text);URL.revokeObjectURL(downloadedLink.href);node('download-commission-zip').onclick();assert.equal(downloadedLink.download,'commission-reports.zip');const response=await fetch(downloadedLink.href);assert.match(response.headers.get('content-type'),/application\/zip/);assert.deepEqual(new Uint8Array(await response.arrayBuffer()),sandbox.commissionReportZip(output));URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const project=createProject({...lastSnapshot,files:{commissionInputs:structuredClone(lastSnapshot.files.commissionInputs),commissionDeliverySettings:structuredClone(lastSnapshot.files.commissionDeliverySettings)}});importRead=()=>Promise.resolve(project);const before=posted.length;await node('open-project').onchange({target:{files:[{name:'delivery.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-commission-zip').disabled,true);assert.equal(node('commission-delivery-html').checked,false);assert.equal(node('commission-delivery-json').checked,true);assert.equal(node('commission-delivery-audit').checked,false);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),request);uiWorker.onmessage({data:{type:'result',result:output}});
node('commission-delivery-json').checked=false;node('commission-delivery-json').onchange();assert.equal(node('download-commission-zip').disabled,true);node('save-project').onclick();assert.deepEqual(structuredClone(lastSnapshot.files.commissionDeliverySettings.formats),[]);const count=posted.length;node('execute').onclick();assert.equal(posted.length,count);assert.match(node('result-status').textContent,/HTML\/JSON formats/);
console.log('Commission delivery UI: selected native formats/request, busy controls, exact ZIP Blob, excluded-format exports, inert Open and identical rerun, partial settings retention and no-format Run rejection passed.');
}

{
 const archive=JSON.parse(await readFile('target/label-build-wasm/browser-archive.bisect-build','utf8'));
 // Use the ordinary snapshot constructor rather than trusting archive result freshness.
 const project=createProject({name:archive.name,files:archive.files,operation:'run-label-build',constraints:[],result:archive.result,lastOperation:'run-label-build'});
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});
 await node('open-project').onchange({target:{files:[{name:'batch.bisect'}]}});
 assert.equal(node('use-label-build-plan').disabled,true);
 const before=posted.length;node('execute').onclick();assert.equal(node('use-label-build-plan').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:archive.result}});
 assert.equal(node('use-label-build-plan').disabled,false);
 const selected=labelBuildTools.labelBuildPlanChoices(archive.result)[0];node('label-build-plan').value=selected;node('label-build-next').value='audit-rplan-text';
 node('use-label-build-plan').onclick();assert.equal(posted.length,before+1);assert.equal(node('operation').value,'audit-rplan-text');
 node('save-project').onclick();assert.equal(Buffer.from(lastSnapshot.files.auditPlanText.base64,'base64').toString(),archive.result.files[selected]);
 assert.equal(Buffer.from(lastSnapshot.files.auditContextText.base64,'base64').toString(),archive.result.files[selected.replace(/plan\.rplan$/,'context.rctx')]);
 assert.equal(lastSnapshot.files.auditCertificateText,undefined);assert.equal(node('use-label-build-plan').disabled,true);
 console.log('Build practitioner handoff UI: inert Open, busy/freshness guard, exact paired sources, no automatic execution, stale certificate removal and project retention passed.');
}

{
 const project=parseProject(await readFile('target/analysis-collection-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/analysis-collection-wasm/statistics-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});
 await node('open-project').onchange({target:{files:[{name:'analyses.bisect'}]}});
 assert.equal(node('retain-analysis').disabled,true);assert.equal(node('analysis-to-commission').disabled,false);
 const count=posted.length;node('execute').onclick();assert.equal(node('retain-analysis').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('retain-analysis').disabled,false);
 await node('retain-analysis').onclick();assert.equal(posted.length,count+1);node('save-project').onclick();
 assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.analysisCollection)),project.files.analysisCollection);
 node('analysis-to-commission').onclick();assert.equal(posted.length,count+1);assert.equal(node('operation').value,'assemble-commission-report');
 node('save-project').onclick();assert.equal(lastSnapshot.files.commissionInputs.files['analysis/summary.json'].base64,project.files.analysisCollection.files['summary.json'].base64);
 assert.equal(lastSnapshot.files.commissionInputs.files['analysis/compactness.json'].base64,project.files.analysisCollection.files['compactness.json'].base64);
 assert.deepEqual(lastSnapshot.files.commissionInputs.files['analysis/partisan.json'],project.files.analysisCollection.files['partisan.json']);assert.equal(node('retain-analysis').disabled,true);
 node('clear-analysis-collection').onclick();assert.equal(node('analysis-to-commission').disabled,true);
 console.log('Analysis collection UI: inert Open, busy/freshness guards, exact retained sources, explicit report handoff without engine execution and clear passed.');
}

{
 const project=parseProject(await readFile('target/plan-contiguity-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/plan-contiguity-wasm/connected-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'contiguity.bisect'}]}});
 assert.equal(node('retain-analysis').disabled,true);assert.equal(node('auditContextText-file').parentElement.hidden,false);
 node('execute').onclick();const request=posted.at(-1).request;assert.equal(request.operation,'analyze-plan-contiguity');assert.equal(request.document_json,Buffer.from(project.files.auditPlanText.base64,'base64').toString());assert.equal(request.context_json,Buffer.from(project.files.auditContextText.base64,'base64').toString());
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});node('download-contiguity').onclick();assert.equal(downloadedLink.download,'contiguity.json');assert.equal(await(await fetch(downloadedLink.href)).text(),nativeResult.report_json);URL.revokeObjectURL(downloadedLink.href);await node('retain-analysis').onclick();node('analysis-to-commission').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.commissionInputs.files['analysis/contiguity.json'].base64,project.files.analysisCollection.files['contiguity.json'].base64);
 console.log('Native contiguity UI: matching context visible, exact request, inert Open/fresh retention and exact report-section handoff passed.');
}

{
 const project=parseProject(await readFile('target/plan-report-manifest-wasm/project-5.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/plan-report-manifest-wasm/result-5.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'manifest.bisect'}]}});
 assert.equal(node('use-plan-manifest').disabled,true);assert.equal(node('auditContextText-file').parentElement.hidden,false);
 node('execute').onclick();const request=posted.at(-1).request;assert.equal(request.operation,'prepare-plan-report-manifest');assert.equal(request.document_json,Buffer.from(project.files.auditPlanText.base64,'base64').toString());assert.equal(request.context_json,Buffer.from(project.files.auditContextText.base64,'base64').toString());
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});const n=posted.length;await node('use-plan-manifest').onclick();assert.equal(posted.length,n);assert.equal(node('operation').value,'assemble-commission-report');node('save-project').onclick();
 assert.equal(lastSnapshot.files.commissionInputs.files['manifest.json'].base64,Buffer.from(nativeResult.manifest_json).toString('base64'));assert.equal(lastSnapshot.files.commissionInputs.files['context.rctx'].base64,project.files.auditContextText.base64);
 for(const name of ['summary.json','compactness.json','contiguity.json'])assert.equal(lastSnapshot.files.commissionInputs.files['analysis/'+name].base64,project.files.analysisCollection.files[name].base64);
 assert.equal(node('use-plan-manifest').disabled,true);console.log('Native manifest UI: inert Open, exact plan/context request, fresh guarded package handoff, all required retained sections and no automatic report execution passed.');
}

{
 const project=parseProject(await readFile('target/plan-splits-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/plan-splits-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});
 const before=posted.length;await node('open-project').onchange({target:{files:[{name:'splits.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('download-splits').disabled,true);assert.equal(node('splits-fields').hidden,false);assert.equal(node('auditContextText-file').parentElement.hidden,false); // Custom county identity checks can consume a matching context.
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'analyze-plan-splits');assert.equal(posted.at(-1).request.document_json,Buffer.from(project.files.auditPlanText.base64,'base64').toString());assert.equal(posted.at(-1).request.geography_json,Buffer.from(project.files.splitsGeography.base64,'base64').toString());assert.equal(node('splits-geography').disabled,true);assert.equal(node('download-splits').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-splits').disabled,false);node('download-splits').onclick();assert.equal(downloadedLink.download,'splits.json');assert.equal(await(await fetch(downloadedLink.href)).text(),nativeResult.report_json);URL.revokeObjectURL(downloadedLink.href);
 await node('retain-analysis').onclick();node('analysis-to-commission').onclick();node('save-project').onclick();assert.equal(Buffer.from(lastSnapshot.files.commissionInputs.files['analysis/splits.json'].base64,'base64').toString(),nativeResult.report_json);
 node('operation').value='analyze-plan-splits';node('operation').change();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:nativeResult}});node('splits-clear').onclick();assert.equal(node('download-splits').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.splitsGeography,undefined);node('execute').onclick();assert.equal(posted.at(-1).request.geography_json,null);uiWorker.onmessage({data:{type:'error',message:'test complete'}});
 let oversizedRead=false;await node('splits-geography').onchange({target:{files:[{name:'large.json',size:3*1024*1024+1,arrayBuffer:async()=>{oversizedRead=true;return new ArrayBuffer(0);}}]}});assert.equal(oversizedRead,false);
 const duplicateLookup='{"place_to_tracts":{"x":[],"x":[]}}';await node('splits-geography').onchange({target:{files:[{name:'duplicate.json',size:duplicateLookup.length,arrayBuffer:async()=>new TextEncoder().encode(duplicateLookup).buffer}]}});node('save-project').onclick();assert.equal(lastSnapshot.files.splitsGeography,undefined);
 console.log('Splits UI: inert Open, exact optional geography/plan request, busy guards, native Blob download, retained report handoff and source-removal freshness passed.');
}

{
 const project=parseProject(await readFile('target/analysis-package-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/analysis-package-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'combined.bisect'}]}});
 assert.equal(posted.length,before);for(const id of ['analysis-package-fields','statistics-fields','election-analysis-fields','compactness-fields','splits-fields','text-audit-fields'])assert.equal(node(id).hidden,false,id);assert.equal(node('auditContextText-file').parentElement.hidden,false);assert.equal(node('download-analysis-package').disabled,true);assert.equal(node('use-analysis-package').disabled,true);
 node('execute').onclick();const req=posted.at(-1).request;assert.equal(req.operation,'analyze-plan-package');assert.equal(req.document_json,undefined);assert.equal(req.input.document_json,Buffer.from(project.files.auditPlanText.base64,'base64').toString());assert.equal(req.input.context_json,Buffer.from(project.files.auditContextText.base64,'base64').toString());assert.equal(req.input.election_settings.bootstrap_seed,'18446744073709551615');assert.equal(node('download-analysis-package').disabled,true);assert.equal(node('statistics-demographic').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-analysis-package').disabled,false);await node('download-analysis-package').onclick();assert.equal(downloadedLink.download,'plan-analysis.zip');assert.ok((await(await fetch(downloadedLink.href)).arrayBuffer()).byteLength>1000);URL.revokeObjectURL(downloadedLink.href);
 const prior=posted.length;await node('use-analysis-package').onclick();assert.equal(posted.length,prior);assert.equal(node('operation').value,'assemble-commission-report');node('save-project').onclick();assert.equal(Buffer.from(lastSnapshot.files.commissionInputs.files['analysis/partisan.json'].base64,'base64').toString(),nativeResult.files['partisan.json']);assert.equal(lastSnapshot.files.commissionInputs.files['analysis/partisan-metrics.json'],undefined);assert.equal(node('download-analysis-package').disabled,true);
 console.log('Combined analysis UI: all source panels, inert Open, exact seeded nested request, busy/freshness guard, ZIP Blob and explicit native report handoff without execution passed.');
}

{
 const project=parseProject(await readFile('target/label-analysis-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/label-analysis-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'batch.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('label-analysis-fields').hidden,false);assert.equal(node('download-label-analysis').disabled,true);assert.equal(node('label-analysis-created').value,project.files.labelAnalysisInputs.created);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'run-label-analysis');assert.equal(posted.at(-1).request.input.build_index_json,JSON.parse(Buffer.from(project.files.labelAnalysisInputs.source.base64,'base64').toString()).build_index_json);assert.equal(node('label-analysis-source').disabled,true);assert.equal(node('label-analysis-states').disabled,true);assert.equal(node('label-analysis-types').disabled,true);assert.equal(node('label-analysis-years').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-label-analysis').disabled,false);await node('download-label-analysis').onclick();assert.equal(downloadedLink.download,'analysis_fixture-2020-analysis.zip');assert.ok((await(await fetch(downloadedLink.href)).arrayBuffer()).byteLength>1000);URL.revokeObjectURL(downloadedLink.href);
 node('label-analysis-created').value='2026-10-08T00:00:00Z';node('label-analysis-created').onchange();assert.equal(node('download-label-analysis').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.labelAnalysisInputs.created,'2026-10-08T00:00:00Z');
 node('label-analysis-states').value='WASHINGTON';node('label-analysis-states').oninput();node('label-analysis-types').value='statistics, contiguity';node('label-analysis-types').oninput();node('label-analysis-created').value='2026-10-08T00:01:00Z';node('label-analysis-created').onchange();node('save-project').onclick();assert.deepEqual(Array.from(lastSnapshot.files.labelAnalysisInputs.states),['WASHINGTON']);assert.deepEqual(Array.from(lastSnapshot.files.labelAnalysisInputs.types),['statistics','contiguity']);assert.deepEqual(lastSnapshot.files.labelAnalysisInputs.source,project.files.labelAnalysisInputs.source);const selectedProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(selectedProject);const selectedBefore=posted.length;await node('open-project').onchange({target:{files:[{name:'selected.bisect'}]}});assert.equal(posted.length,selectedBefore);assert.equal(node('label-analysis-states').value,'WASHINGTON');assert.equal(node('label-analysis-types').value,'statistics, contiguity');assert.equal(node('download-label-analysis').disabled,true);node('execute').onclick();assert.deepEqual(Object.keys(posted.at(-1).request.input.states),['washington']);assert.deepEqual(Array.from(posted.at(-1).request.input.types),['statistics','contiguity']);uiWorker.onmessage({data:{type:'result',result:nativeResult}});
 node('label-analysis-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.labelAnalysisInputs.source,null);assert.equal(node('label-analysis-states').value,'');assert.equal(node('label-analysis-types').value,'');assert.equal(node('label-analysis-years').value,'');assert.equal(node('label-analysis-states').disabled,true);assert.equal(node('label-analysis-types').disabled,true);assert.equal(node('label-analysis-years').disabled,true);
 let read=false;await node('label-analysis-source').onchange({target:{files:[{size:25*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 const duplicate='{"build_index_json":"{}","created":"","states":{},"states":{}}';await node('label-analysis-source').onchange({target:{files:[{name:'duplicate.json',size:duplicate.length,arrayBuffer:async()=>new TextEncoder().encode(duplicate).buffer}]}});assert.match(node('result-status').textContent,/Duplicate/);
 console.log('Label analysis UI: inert Open, exact batch request, source-preserving state/type subset Save/Open and timestamp edits, busy/freshness guards, ZIP Blob, source clearing and pre-read/duplicate rejection passed.');
}

{
 const archive=JSON.parse(await readFile('target/label-build-wasm/browser-archive.bisect-build','utf8'));
 const project=createProject({name:'Build to state analyses',files:{labelBuildInputs:archive.inputs},operation:'run-label-build',constraints:[],result:archive.result,lastOperation:'run-label-build'});
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'build.bisect'}]}});assert.equal(node('label-build-analyze').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:archive.result}});assert.equal(node('label-build-analyze').disabled,false);const before=posted.length;node('label-build-analyze').onclick();assert.equal(posted.length,before);assert.equal(node('operation').value,'run-label-analysis');assert.equal(node('label-analysis-created').value,'');node('save-project').onclick();assert.ok(lastSnapshot.files.labelAnalysisInputs.source);assert.equal(lastSnapshot.files.labelAnalysisInputs.created,'');
 console.log('Build-to-analysis UI: imported output disabled, fresh completed-year handoff, exact sources retained and no automatic Run passed.');
 importRead=()=>Promise.resolve(project);await node('open-project').onchange({target:{files:[{name:'build.bisect'}]}});assert.equal(node('label-build-analyze-years').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:archive.result}});const yearsBefore=posted.length;node('label-build-analyze-years').onclick();assert.equal(posted.length,yearsBefore);node('save-project').onclick();const allYears=JSON.parse(Buffer.from(lastSnapshot.files.labelAnalysisInputs.source.base64,'base64').toString());assert.deepEqual(Object.keys(allYears.batches).sort(),['2010','2020']);assert.equal(node('label-analysis-years').value,'');assert.match(node('label-analysis-years-available').textContent,/2010, 2020/);
 node('label-analysis-years').value='2020';node('label-analysis-years').oninput();node('label-analysis-created').value='2026-10-12T20:00:00Z';node('label-analysis-created').onchange();node('save-project').onclick();const yearProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(yearProject);await node('open-project').onchange({target:{files:[{name:'years.bisect'}]}});assert.equal(posted.length,yearsBefore);assert.equal(node('label-analysis-years').value,'2020');assert.equal(node('download-label-analysis').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.operation,'run-label-analysis-years');assert.deepEqual(Object.keys(posted.at(-1).request.input.batches),['2020']);assert.equal(node('label-analysis-years').disabled,true);uiWorker.onmessage({data:{type:'error',error:'Fixture worker stopped after request validation.'}});
 console.log('Multi-year analysis UI: fresh-only all-year Build handoff, original year batches, timestamp/selection Save/Open, no automatic execution and selected-year worker request passed.');
 const analysisProject=parseProject(await readFile('target/label-analysis-years-wasm/project.bisect','utf8')),progress=JSON.parse(await readFile('target/label-analysis-years-wasm/checkpoint-events.json','utf8'));importRead=()=>Promise.resolve(analysisProject);await node('open-project').onchange({target:{files:[{name:'analysis-years.bisect'}]}});node('execute').onclick();const runningWorker=uiWorker,requestId=posted.at(-1).id,analysisBinding=await startLabelAnalysisYearsCheckpoint(posted.at(-1).request);runningWorker.onmessage({data:{type:'analysis-years-started',id:requestId,binding:analysisBinding}});runningWorker.onmessage({data:{type:'progress',id:requestId,progress:progress[0]}});node('cancel').onclick();runningWorker.onmessage({data:{type:'result',id:requestId,result:analysisProject.result}});uiWorker.onmessage({data:{type:'ready'}});node('save-project').onclick();assert.equal(lastSnapshot.result.execution_status,'cancelled');assert.deepEqual(Array.from(lastSnapshot.result.years,item=>item.year),['2010']);assert.deepEqual(Array.from(lastSnapshot.result.requested_years),['2010','2020']);assert.deepEqual(JSON.parse(lastSnapshot.result.registry_json).batch.analyzed,['2010']);assert.equal(node('download-label-analysis').disabled,false);await node('download-label-analysis').onclick();assert.equal(downloadedLink.download,'batch-years-analysis.zip');URL.revokeObjectURL(downloadedLink.href);const cancelledProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(cancelledProject);const cancelledBefore=posted.length;await node('open-project').onchange({target:{files:[{name:'cancelled-years.bisect'}]}});assert.equal(posted.length,cancelledBefore);assert.equal(node('download-label-analysis').disabled,true);assert.equal(node('label-analysis-report').disabled,true);
 console.log('Analysis-year cancellation UI: received native checkpoint retained, unfinished stages absent, late same-ID final result ignored, exact ZIP download, source-preserving inert Open and disabled imported exports passed.');
 const reportProject=parseProject(await readFile('target/label-report-years-wasm/project.bisect','utf8')),reportEvents=JSON.parse(await readFile('target/label-report-years-wasm/checkpoint-events.json','utf8'));importRead=()=>Promise.resolve(reportProject);const reportBefore=posted.length;await node('open-project').onchange({target:{files:[{name:'reports-years.bisect'}]}});assert.equal(posted.length,reportBefore);assert.equal(node('download-label-report').disabled,true);node('label-report-years').value='2020';node('label-report-years').oninput();node('label-report-created').value='2026-10-12T21:01:00Z';node('label-report-created').onchange();node('save-project').onclick();assert.deepEqual(Array.from(lastSnapshot.files.labelReportInputs.years),['2020']);assert.deepEqual(lastSnapshot.files.labelReportInputs.source,reportProject.files.labelReportInputs.source);const selectedReport=createProject(lastSnapshot);importRead=()=>Promise.resolve(selectedReport);await node('open-project').onchange({target:{files:[{name:'selected-reports.bisect'}]}});assert.equal(node('label-report-years').value,'2020');node('execute').onclick();assert.equal(posted.at(-1).request.operation,'run-label-report-years');assert.deepEqual(Object.keys(posted.at(-1).request.input.batches),['2020']);assert.equal(node('label-report-years').disabled,true);uiWorker.onmessage({data:{type:'error',error:'Fixture request stopped.'}});
 importRead=()=>Promise.resolve(reportProject);await node('open-project').onchange({target:{files:[{name:'all-reports.bisect'}]}});node('execute').onclick();const reportWorker=uiWorker,reportId=posted.at(-1).id,reportBinding=await startLabelReportYearsCheckpoint(posted.at(-1).request);reportWorker.onmessage({data:{type:'report-years-started',id:reportId,binding:reportBinding}});reportWorker.onmessage({data:{type:'progress',id:reportId,progress:reportEvents[0]}});node('cancel').onclick();reportWorker.onmessage({data:{type:'result',id:reportId,result:reportProject.result}});uiWorker.onmessage({data:{type:'ready'}});node('save-project').onclick();assert.equal(lastSnapshot.result.execution_status,'cancelled');assert.deepEqual(Array.from(lastSnapshot.result.years,item=>item.year),['2010']);assert.deepEqual(JSON.parse(lastSnapshot.result.registry_json).analysis_fixture.reported,['2010']);assert.equal(node('download-label-report').disabled,false);await node('download-label-report').onclick();assert.equal(downloadedLink.download,'analysis_fixture-years-reports.zip');URL.revokeObjectURL(downloadedLink.href);
 console.log('Multi-year report UI: inert source-preserving selected-year Save/Open, timestamp/busy guards, explicit native worker request, completed-year cancellation, missing unfinished report stages, late-result refusal and exact ZIP export passed.');



}

{
 const project=parseProject(await readFile('target/label-report-wasm/project.bisect','utf8'));
 const nativeResult=JSON.parse(await readFile('target/label-report-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'reports.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('label-report-fields').hidden,false);assert.equal(node('download-label-report').disabled,true);assert.equal(node('label-report-format').value,'json');assert.equal(node('label-report-audit').checked,true);assert.equal(node('label-report-created').value,project.files.labelReportInputs.settings.generated_at);
 node('execute').onclick();const req=posted.at(-1).request;assert.equal(req.operation,'run-label-report');assert.equal(req.input.format,'json');assert.equal(req.input.audit_with_report,true);assert.equal(req.input.analysis_index_json,JSON.parse(Buffer.from(project.files.labelReportInputs.source.base64,'base64').toString()).analysis_index_json);assert.equal(node('label-report-format').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-label-report').disabled,false);await node('download-label-report').onclick();assert.equal(downloadedLink.download,'analysis_fixture-2020-reports.zip');assert.ok((await(await fetch(downloadedLink.href)).arrayBuffer()).byteLength>1000);URL.revokeObjectURL(downloadedLink.href);
 node('label-report-audit').checked=false;node('label-report-audit').onchange();assert.equal(node('download-label-report').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.labelReportInputs.settings.audit_with_report,false);
 node('label-report-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.labelReportInputs.source,null);
 let oversizedRead=false;await node('label-report-source').onchange({target:{files:[{size:25*1024*1024+1,arrayBuffer:async()=>{oversizedRead=true;}}]}});assert.equal(oversizedRead,false);
 const duplicate='{"analysis_index_json":"{}","files":{},"files":{},"file_sha256":{}}';await node('label-report-source').onchange({target:{files:[{name:'bad.json',size:duplicate.length,arrayBuffer:async()=>new TextEncoder().encode(duplicate).buffer}]}});assert.match(node('result-status').textContent,/Duplicate/);
 console.log('Label report UI: inert Open, exact settings/source request, busy/freshness controls, saved settings, native ZIP Blob and pre-read/duplicate rejection passed.');
}
{
 const project=parseProject(await readFile('target/label-analysis-wasm/project.bisect','utf8'));const nativeResult=JSON.parse(await readFile('target/label-analysis-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'analyses.bisect'}]}});assert.equal(node('label-analysis-report').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('label-analysis-report').disabled,false);const before=posted.length;await node('label-analysis-report').onclick();assert.equal(posted.length,before);assert.equal(node('operation').value,'run-label-report');assert.equal(node('label-report-created').value,'');assert.equal(node('label-report-format').value,'html');node('save-project').onclick();const source=JSON.parse(Buffer.from(lastSnapshot.files.labelReportInputs.source.base64,'base64').toString());assert.equal(source.analysis_index_json,nativeResult.index_json);assert.deepEqual(source.files,nativeResult.files);assert.deepEqual(source.file_sha256,nativeResult.file_sha256);
 console.log('Analysis-to-report UI: fresh guarded handoff, exact native files and hashes retained, empty timestamp and no automatic execution passed.');
}

{
 const project=parseProject(await readFile('target/label-stage-wasm/project.bisect','utf8')),analysis=JSON.parse(await readFile('target/label-stage-wasm/analysis.json','utf8')),report=JSON.parse(await readFile('target/label-stage-wasm/report.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'stages.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-label-analysis').disabled,true);assert.equal(node('label-analysis-report').disabled,true);
 node('execute').onclick();assert.equal(typeof posted.at(-1).request.input.registry_json,'string');uiWorker.onmessage({data:{type:'result',result:analysis}});await node('label-analysis-report').onclick();node('save-project').onclick();const source=JSON.parse(Buffer.from(lastSnapshot.files.labelReportInputs.source.base64,'base64').toString());assert.equal(source.registry_json,analysis.registry_json);assert.equal(source.files['.bisect'],analysis.registry_json);
 node('label-report-created').value='2026-10-07T01:00:00Z';node('label-report-format').value='json';node('label-report-audit').checked=true;node('label-report-created').onchange();node('execute').onclick();assert.equal(posted.at(-1).request.input.registry_json,analysis.registry_json);uiWorker.onmessage({data:{type:'result',result:report}});await node('download-label-report').onclick();assert.equal(downloadedLink.download,'analysis_fixture-2020-reports.zip');assert.ok((await(await fetch(downloadedLink.href)).arrayBuffer()).byteLength>1000);URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.deepEqual(JSON.parse(lastSnapshot.result.registry_json).analysis_fixture.reported,['2010','2020']);
 console.log('Native registry UI: inert Open, exact supplied registry Run, analysis-to-report snapshot handoff, native recorded stages, saved project and fresh ZIP export passed.');
}

{
 const project=parseProject(await readFile('target/label-reconcile-wasm/project.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/label-reconcile-wasm/analysis-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'reconcile.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-label-reconcile').disabled,true);
 assert.equal(node('label-reconcile-currentRegistry-name').textContent,'.bisect');node('execute').onclick();const req=posted.at(-1).request;assert.equal(req.operation,'reconcile-label-registry');assert.equal(req.input.current_registry_json,Buffer.from(project.files.labelReconcileInputs.currentRegistry.base64,'base64').toString());assert.equal(node('label-reconcile-draft').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-label-reconcile').disabled,false);await node('download-label-reconcile').onclick();assert.equal(downloadedLink.download,'.bisect');assert.equal(await(await fetch(downloadedLink.href)).text(),nativeResult.registry_json);URL.revokeObjectURL(downloadedLink.href);
 node('label-reconcile-clear').onclick();assert.equal(node('download-label-reconcile').disabled,true);node('save-project').onclick();assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.labelReconcileInputs)),{draft:null,currentRegistry:null,upstreamIndex:null});
 let read=false;await node('label-reconcile-currentRegistry').onchange({target:{files:[{size:1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('Reconciliation UI: inert Open, exact source Run, busy/freshness controls, exact registry download and source limits passed.');
}
for(const [stage,path,resultPath]of [['analysis','project.bisect','analysis.json'],['report','report-project.bisect','report.json']]){
 const project=parseProject(await readFile('target/label-stage-wasm/'+path,'utf8')),nativeResult=JSON.parse(await readFile('target/label-stage-wasm/'+resultPath,'utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:stage+'.bisect'}]}});assert.equal(node('label-'+stage+'-reconcile').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('label-'+stage+'-reconcile').disabled,false);const before=posted.length;await node('label-'+stage+'-reconcile').onclick();assert.equal(posted.length,before);assert.equal(node('operation').value,'reconcile-label-registry');node('save-project').onclick();assert.equal(lastSnapshot.files.labelReconcileInputs.currentRegistry,null);assert.equal(lastSnapshot.files.labelReconcileInputs.upstreamIndex,null);const draft=JSON.parse(Buffer.from(lastSnapshot.files.labelReconcileInputs.draft.base64,'base64').toString());assert.equal(draft.proposed_registry_json,nativeResult.registry_json);assert.equal(draft.generated_index_json,nativeResult.index_json);
}
console.log('Stage reconciliation UI: fresh analysis/report drafts retain exact snapshots; current sources require selection and no automatic Run passed.');
{
 const project=parseProject(await readFile('target/label-package-merge-wasm/project.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/label-package-merge-wasm/report.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'repository.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-label-package-merge').disabled,true);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'reconcile-label-package');assert.equal(node('label-package-merge-source').disabled,true);assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request.input)),JSON.parse(Buffer.from(project.files.labelPackageMergeInputs.source.base64,'base64').toString()));
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-label-package-merge').disabled,false);await node('download-label-package-merge').onclick();assert.equal(downloadedLink.download,'analysis_fixture-2020-repository.zip');const actual=Buffer.from(await(await fetch(downloadedLink.href)).arrayBuffer());assert.deepEqual(actual,await readFile('target/label-package-merge-wasm/repository.zip'));URL.revokeObjectURL(downloadedLink.href);
 node('label-package-merge-clear').onclick();assert.equal(node('download-label-package-merge').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.labelPackageMergeInputs.source,null);
 let read=false;await node('label-package-merge-source').onchange({target:{files:[{size:25*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('Package merge UI: inert Open, exact snapshots/expectations, busy/fresh exports, exact repository ZIP Blob, saved sources and pre-read capacity rejection passed.');
}
{
 const project=parseProject(await readFile('target/ilp-audit-wasm/project.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/ilp-audit-wasm/result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'ilp-audit.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-ilp-audit').disabled,true);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'audit-ilp-artifacts');assert.equal(node('ilp-audit-reports').disabled,true);assert.equal(posted.at(-1).request.input.models_base64['node_root.lp'],project.files.ilpAuditInputs.models['node_root.lp'].base64);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-ilp-audit').disabled,false);node('download-ilp-audit').onclick();assert.equal(downloadedLink.download,'audit-summary.json');assert.equal(await(await fetch(downloadedLink.href)).text(),nativeResult.summary_json);URL.revokeObjectURL(downloadedLink.href);
 const summaryBytes=new TextEncoder().encode(nativeResult.summary_json);await node('ilp-audit-summary').onchange({target:{files:[{name:'audit-summary.json',size:summaryBytes.length,arrayBuffer:async()=>summaryBytes.buffer}]}});assert.equal(node('download-ilp-audit').disabled,true);node('execute').onclick();assert.equal(posted.at(-1).request.input.summary_json,nativeResult.summary_json);uiWorker.onmessage({data:{type:'result',result:{...nativeResult,supplied_summary_status:'match'}}});node('ilp-audit-remove-summary').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.ilpAuditInputs.summary,null);assert.ok(lastSnapshot.files.ilpAuditInputs.reports['node.json']);assert.equal(node('download-ilp-audit').disabled,true);
 const pending=deferred();const staleRead=node('ilp-audit-models').onchange({target:{files:[{name:'stale.lp',size:1,arrayBuffer:()=>pending.promise}]}});node('ilp-audit-clear').onclick();pending.resolve(Uint8Array.from([1]).buffer);await staleRead;node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.ilpAuditInputs.models).length,0);assert.match(node('result-status').textContent,/changed/);
 let read=false;await node('ilp-audit-models').onchange({target:{files:[{name:'oversized.lp',size:8*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('ILP audit UI: inert Open, exact binary requests and summary Blob, summary removal, busy/freshness controls, saved bytes, stale-read and pre-read size guards passed.');
 const conflictProject=parseProject(await readFile('target/ilp-audit-wasm/conflict-project.bisect','utf8')),conflictResult=JSON.parse(await readFile('target/ilp-audit-wasm/conflict-result.json','utf8'));importRead=()=>Promise.resolve(conflictProject);await node('open-project').onchange({target:{files:[{name:'conflict.bisect'}]}});assert.equal(node('download-ilp-audit').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:conflictResult}});assert.equal(node('download-ilp-audit').disabled,false);node('download-ilp-audit').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),conflictResult.summary_json);URL.revokeObjectURL(downloadedLink.href);
 console.log('ILP diagnostic export: fresh failed hash/stale-summary results export exact native failure summary bytes; imported failures remain inert.');
}

for(const name of ['project','recorded-fail-project','tampered-project']){
 const project=parseProject(await readFile('target/package-verification-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/package-verification-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('package-verification-fields').hidden,false);assert.equal(node('download-package-verification').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'verify-rplan-package',input:{manifest_json:Buffer.from(project.files.packageVerificationInputs.manifest.base64,'base64').toString('utf8'),files_base64:Object.fromEntries(Object.entries(project.files.packageVerificationInputs.files).map(([p,s])=>[p,s.base64]))}});
 assert.equal(node('package-verification-folder').disabled,true);uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-package-verification').disabled,false);
 node('download-package-verification').onclick();assert.equal(downloadedLink.download,'package-verification.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('package-verification-clear').onclick();assert.equal(node('download-package-verification').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.packageVerificationInputs.manifest,null);assert.equal(Object.keys(lastSnapshot.files.packageVerificationInputs.files).length,0);
}
{
 const pending=deferred();const staleRead=node('package-verification-files').onchange({target:{files:[{name:'stale.bin',size:1,arrayBuffer:()=>pending.promise}]}});node('package-verification-clear').onclick();pending.resolve(Uint8Array.from([1]).buffer);await staleRead;node('save-project').onclick();assert.equal(Object.keys(lastSnapshot.files.packageVerificationInputs.files).length,0);assert.match(node('result-status').textContent,/changed/);
 let read=false;await node('package-verification-files').onchange({target:{files:[{name:'large.bin',size:16*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('RPLAN verification UI: inert Open, exact requests, busy/freshness guards, saved sources, verified/recorded-fail/tampered receipt Blobs, stale-read and pre-read limits passed.');
}

for(const name of ['project','recorded-fail-project','tampered-project','not-recorded-project']){
 const project=parseProject(await readFile('target/manifest-certificate-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/manifest-certificate-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('package-verification-fields').hidden,false);assert.equal(node('download-package-verification').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'verify-manifest-certificate',input:{manifest_json:Buffer.from(project.files.packageVerificationInputs.manifest.base64,'base64').toString(),files_base64:Object.fromEntries(Object.entries(project.files.packageVerificationInputs.files).map(([p,s])=>[p,s.base64]))}});assert.equal(node('package-verification-manifest').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-package-verification').disabled,false);node('download-package-verification').onclick();assert.equal(downloadedLink.download,'manifest-certificate-verification.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('package-verification-clear').onclick();assert.equal(node('download-package-verification').disabled,true);
}
console.log('Manifest certificate UI: inert projects, exact requests, busy/fresh exports and distinct verified/recorded-fail/tampered/not-recorded receipts passed.');

for(const name of ['identical','relabelled','empty-original']){
 const project=parseProject(await readFile('target/assignment-verification-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/assignment-verification-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('assignment-verification-fields').hidden,false);assert.equal(node('download-assignment-verification').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'compare-verify-assignments',input:{original_json:Buffer.from(project.files.assignmentVerificationInputs.original.base64,'base64').toString(),reference_json:Buffer.from(project.files.assignmentVerificationInputs.reference.base64,'base64').toString(),min_similarity:project.files.assignmentVerificationInputs.minSimilarity}});assert.equal(node('assignment-verification-original').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-assignment-verification').disabled,false);node('download-assignment-verification').onclick();assert.equal(downloadedLink.download,'assignment-verification.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('assignment-verification-threshold').value='0.5';node('assignment-verification-threshold').onchange();assert.equal(node('download-assignment-verification').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.assignmentVerificationInputs.minSimilarity,.5);
 node('assignment-verification-threshold').value='';node('assignment-verification-threshold').oninput();assert.match(node('result-status').textContent,/finite/);assert.equal(node('execute').disabled,true);node('assignment-verification-threshold').value='-1';node('assignment-verification-threshold').oninput();assert.equal(node('execute').disabled,false);node('save-project').onclick();assert.equal(lastSnapshot.files.assignmentVerificationInputs.minSimilarity,-1);
 node('assignment-verification-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.assignmentVerificationInputs.original,null);
}
{
 const pending=deferred();const stale=node('assignment-verification-reference').onchange({target:{files:[{name:'stale.json',size:2,arrayBuffer:()=>pending.promise}]}});node('assignment-verification-clear').onclick();pending.resolve(new TextEncoder().encode('{}').buffer);await stale;node('save-project').onclick();assert.equal(lastSnapshot.files.assignmentVerificationInputs.reference,null);assert.match(node('result-status').textContent,/changed/);
 let read=false;await node('assignment-verification-original').onchange({target:{files:[{name:'large.json',size:8*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('Assignment comparison UI: inert Open, exact selected sources and thresholds, busy/freshness controls, saved threshold, matching/different/unavailable receipt Blobs, invalid threshold recovery, stale-read and pre-read limits passed.');
}

for(const name of ['valid','bad-digest','changed-model','not-recorded']){
 const project=parseProject(await readFile('target/manifest-prechecks-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/manifest-prechecks-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('package-verification-fields').hidden,false);assert.equal(node('download-package-verification').disabled,true);
 node('execute').onclick();const sources=project.files.packageVerificationInputs;assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'verify-manifest-prechecks',input:{manifest_json:Buffer.from(sources.manifest.base64,'base64').toString(),manifest_path:sources.manifest.name,files_base64:Object.fromEntries(Object.entries(sources.files).map(([p,s])=>[p,s.base64]))}});assert.equal(node('package-verification-files').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-package-verification').disabled,false);node('download-package-verification').onclick();assert.equal(downloadedLink.download,'manifest-prechecks.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('package-verification-clear').onclick();assert.equal(node('download-package-verification').disabled,true);
}
console.log('Manifest prechecks UI: inert Open, exact manifest path and binary inputs, busy/freshness controls, verified/failed/not-recorded receipt Blobs and Clear passed.');

for(const name of ['engine-42','different','empty-original','failed-execution']){
 const project=parseProject(await readFile('target/engine-replay-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/engine-replay-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('engine-replay-fields').hidden,false);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'replay-engine-run',input:{original_assignments_json:Buffer.from(project.files.engineReplayInputs.original.base64,'base64').toString(),run_json:Buffer.from(project.files.engineReplayInputs.run.base64,'base64').toString(),min_similarity:project.files.engineReplayInputs.minSimilarity}});assert.equal(node('engine-replay-original').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-engine-replay').disabled,false);node('download-engine-replay').onclick();assert.equal(downloadedLink.download,'engine-replay.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('engine-replay-threshold').value='0.5';node('engine-replay-threshold').onchange();assert.equal(node('download-engine-replay').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.minSimilarity,.5);
 node('engine-replay-threshold').value='';node('engine-replay-threshold').oninput();assert.match(node('result-status').textContent,/finite/);assert.equal(node('execute').disabled,true);node('engine-replay-threshold').value='-1';node('engine-replay-threshold').oninput();assert.equal(node('execute').disabled,false);node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.minSimilarity,-1);
 node('engine-replay-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.original,null);
}
{
 const pending=deferred();const stale=node('engine-replay-run').onchange({target:{files:[{name:'stale.json',size:30,arrayBuffer:()=>pending.promise}]}});node('engine-replay-clear').onclick();pending.resolve(new TextEncoder().encode('{"kind":"engine","request":{}}').buffer);await stale;node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.run,null);assert.match(node('result-status').textContent,/changed/);
 let read=false;await node('engine-replay-original').onchange({target:{files:[{name:'large.json',size:8*1024*1024+1,arrayBuffer:async()=>{read=true;}}]}});assert.equal(read,false);
 console.log('Engine replay UI: inert Open, exact selected sources and thresholds, busy/freshness controls, saved threshold, matching/different/unavailable/failed receipt Blobs, invalid threshold recovery, stale-read and pre-read limits passed.');
}

for(const name of ['configured','configured-build']){
 const project=parseProject(await readFile('target/engine-replay-wasm/handoff-'+name+'.bisect','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'handoff.bisect'}]}});
 assert.equal(node('prepare-engine-replay').disabled,true);const before=posted.length;node('prepare-engine-replay').onclick();assert.equal(posted.length,before);assert.match(node('result-status').textContent,/this device/);
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('prepare-engine-replay').disabled,false);
 node('prepare-engine-replay').onclick();assert.equal(posted.length,before);assert.equal(node('operation').value,'replay-engine-run');assert.equal(node('download-engine-replay').disabled,true);assert.equal(node('prepare-engine-replay').disabled,true);
 node('save-project').onclick();const sources=lastSnapshot.files.engineReplayInputs,artifact=JSON.parse(Buffer.from(sources.run.base64,'base64').toString());assert.deepEqual(artifact.request,project.result.engine_request);assert.equal(artifact.kind,'engine');assert.equal(sources.minSimilarity,1);
 node('execute').onclick();assert.equal(posted.length,before+1);assert.equal(posted.at(-1).request.operation,'replay-engine-run');assert.equal(node('prepare-engine-replay').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:JSON.parse(await readFile('target/engine-replay-wasm/engine-18446744073709551615-result.json','utf8'))}});
}
console.log('Configured replay handoff UI: imported-result rejection, fresh-only controls, exact accepted request, saved sources and explicit Run passed.');

for(const name of ['match','different','empty','unrecorded','bad-digest','changed-model','generation-failed']){
 const project=parseProject(await readFile('target/manifest-replay-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/manifest-replay-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('engine-replay-fields').hidden,false);assert.equal(node('package-verification-fields').hidden,false);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),(await import('../../web/lab/engine-replay-files.js')).manifestReplayRequest(project.files.packageVerificationInputs,project.files.engineReplayInputs));assert.equal(node('package-verification-files').disabled,true);assert.equal(node('engine-replay-run').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-engine-replay').disabled,false);node('download-engine-replay').onclick();assert.equal(downloadedLink.download,'manifest-replay-verification.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('engine-replay-threshold').value='0.5';node('engine-replay-threshold').oninput();assert.equal(node('download-engine-replay').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.minSimilarity,.5);assert.deepEqual(lastSnapshot.files.packageVerificationInputs,project.files.packageVerificationInputs);
 node('package-verification-clear').onclick();assert.equal(node('download-engine-replay').disabled,true);node('execute').onclick();assert.match(node('result-status').textContent,/manifest/);
}
console.log('Combined manifest/replay UI: inert Open, both source panels, exact stage requests, busy/fresh receipt guards, distinct failed/unavailable/matching receipts, saved sources and invalidation passed.');

for(const name of ['bound','original-plan','run-options','run-graph','no-recorded-options']){
 const project=parseProject(await readFile('target/replay-bindings-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/replay-bindings-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'verify-manifest-replay');uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-engine-replay').disabled,false);
 node('download-engine-replay').onclick();assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('engine-replay-threshold').value='0.9';node('engine-replay-threshold').oninput();assert.equal(node('download-engine-replay').disabled,true);
}
console.log('Replay binding UI: verified/partial/failed correspondence, inert Open, exact sources, fresh receipts and threshold invalidation passed.');

for(const name of ['demographics','combined','demographics-changed','character-relabelled','partisan-missing-provenance']){
 const project=parseProject(await readFile('target/replay-aux-bindings-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/replay-aux-bindings-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),(await import('../../web/lab/engine-replay-files.js')).manifestReplayRequest(project.files.packageVerificationInputs,project.files.engineReplayInputs));assert.equal(node('engine-replay-run').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-engine-replay').disabled,false);node('download-engine-replay').onclick();assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('engine-replay-threshold').value='0.5';node('engine-replay-threshold').oninput();assert.equal(node('download-engine-replay').disabled,true);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.packageVerificationInputs,project.files.packageVerificationInputs);
}
console.log('Auxiliary replay UI: inert Open, exact combined sources, busy/freshness, verified/failed/partial receipts and retained source bytes passed.');

for(const name of ['tract-county-fixed','bg-tract-adaptive','bg-tract-adaptive-physical-weight','bg-tract-adaptive-missing-boundaries','bg-county-fixed-build']){
 const project=parseProject(await readFile('target/replay-multiscale-bindings-wasm/'+name+'.bisect','utf8')),nativeResult=JSON.parse(await readFile('target/replay-multiscale-bindings-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),(await import('../../web/lab/engine-replay-files.js')).manifestReplayRequest(project.files.packageVerificationInputs,project.files.engineReplayInputs));assert.equal(node('engine-replay-run').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:nativeResult}});assert.equal(node('download-engine-replay').disabled,false);node('download-engine-replay').onclick();assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),nativeResult);URL.revokeObjectURL(downloadedLink.href);
 node('engine-replay-threshold').value='0.5';node('engine-replay-threshold').oninput();assert.equal(node('download-engine-replay').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.run.base64,project.files.engineReplayInputs.run.base64);
}
console.log('Multiscale replay UI: inert Open, exact physical-edge/source requests, busy/freshness, verified/failed/partial/build receipts and saved raw artifacts passed.');

{
 const tools=await import('../../web/lab/assignment-discovery-files.js');
 const discovery=JSON.parse(await readFile('target/assignment-discovery-ui/fixture.bisect','utf8')),output=JSON.parse(await readFile('target/assignment-discovery-ui/result.json','utf8'));
 importRead=()=>Promise.resolve(discovery);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'discovery.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('use-discovered-original').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.assignmentDiscoveryRequest(discovery.files.assignmentDiscoveryInputs));assert.equal(node('assignment-discovery-folder').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('use-discovered-original').disabled,false);
 const beforeHandoff=posted.length;node('use-discovered-original').onclick();assert.equal(posted.length,beforeHandoff);assert.equal(node('operation').value,'replay-engine-run');node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.original.base64,discovery.files.assignmentDiscoveryInputs.files[output.source_path].base64);assert.equal(lastSnapshot.files.engineReplayInputs.run.base64,discovery.files.engineReplayInputs.run.base64);
 await node('open-project').onchange({target:{files:[{name:'discovery.bisect'}]}});node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:output}});node('assignment-discovery-label').value='another';node('assignment-discovery-label').oninput();assert.equal(node('use-discovered-original').disabled,true);
 const pending=deferred();const reading=node('assignment-discovery-folder').onchange({target:{files:[{name:'a.json',webkitRelativePath:'outputs/a.json',size:2,arrayBuffer:()=>pending.promise}]}});node('assignment-discovery-clear').onclick();pending.resolve(new TextEncoder().encode('{}').buffer);await reading;node('save-project').onclick();assert.deepEqual(Object.keys(lastSnapshot.files.assignmentDiscoveryInputs.files),[]);
 console.log('Discovery UI: inert project Open, exact request, busy guard, fresh-only handoff, saved raw run/original, stale source rejection passed.');
}

for(const name of ['match','verified-sidecars','different','empty-reference','fallback','international','missing-reference','model-stop']){
 const project=parseProject(await readFile('target/verify-assignments-only-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/verify-assignments-only-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/verify-assignments-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-verify-assignments').disabled,true);assert.equal(node('assignment-discovery-manifest-label').hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.verifyAssignmentsRequest(project.files.packageVerificationInputs,project.files.assignmentDiscoveryInputs,project.files.verifyAssignmentsSettings));assert.equal(node('verify-assignments-reference').disabled,true);assert.equal(node('assignment-discovery-folder').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-verify-assignments').disabled,false);node('download-verify-assignments').onclick();assert.equal(downloadedLink.download,'assignments-only-verification.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),output);URL.revokeObjectURL(downloadedLink.href);
 node('verify-assignments-threshold').value='0.5';node('verify-assignments-threshold').oninput();assert.equal(node('download-verify-assignments').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.verifyAssignmentsSettings.minSimilarity,.5);
 node('verify-assignments-threshold').value='';node('verify-assignments-threshold').oninput();assert.equal(node('execute').disabled,true);node('verify-assignments-threshold').value='-1';node('verify-assignments-threshold').oninput();assert.equal(node('execute').disabled,false);
 node('verify-assignments-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.verifyAssignmentsSettings.reference,null);
}
{
 const pending=deferred();const reading=node('verify-assignments-reference').onchange({target:{files:[{name:'stale.json',size:2,arrayBuffer:()=>pending.promise}]}});node('verify-assignments-clear').onclick();pending.resolve(new TextEncoder().encode('{}').buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.verifyAssignmentsSettings.reference,null);
}
console.log('Assignments-only UI: inert Open, exact combined request, shared manifest authority, busy/fresh receipt guards, receipt Blobs, saved threshold, invalid threshold recovery and stale reference rejection passed.');

{
 const artifact={sha256:'a'.repeat(64),byte_length:12345};uiWorker.onmessage({data:{type:'ready',runtimeArtifact:artifact}});assert.match(node('runtime-artifact').textContent,/a{64}/);assert.match(node('runtime-artifact').textContent,/12345 bytes/);
 uiWorker.onmessage({data:{type:'ready'}});assert.equal(node('runtime-artifact').textContent,'Loaded WASM fingerprint unavailable.');
 console.log('Runtime fingerprint UI: worker Ready identity and unavailable identity remain explicit.');
}

for(const name of ['default','reserved-run','maximum','maximum-range']){
 const project=parseProject(await readFile('target/sweep-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/sweep-wasm/'+name+'-result.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-sweep-commands').disabled,true);assert.equal(node('sweep-seed_start').value,project.files.sweepSettings.seed_start);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),{operation:'plan-sweep',settings:project.files.sweepSettings});assert.equal(node('sweep-seed_start').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-sweep-commands').disabled,false);node('download-sweep-commands').onclick();assert.equal(downloadedLink.download,'native-sweep-commands.txt');assert.equal(await(await fetch(downloadedLink.href)).text(),output.plan.commands_stdout);URL.revokeObjectURL(downloadedLink.href);
 node('sweep-seed_start').value='42';node('sweep-seed_start').oninput();assert.equal(node('download-sweep-commands').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.sweepSettings.seed_start,'42');
 node('sweep-seeds').value='';node('sweep-seeds').oninput();assert.equal(node('execute').disabled,true);node('sweep-seeds').value='10';node('sweep-seeds').oninput();assert.equal(node('execute').disabled,false);
}
console.log('Sweep UI: inert Open, exact seed/settings, busy/fresh command downloads, native command Blob bytes, saved edits and invalid count recovery passed.');

for(const name of ['valid','duplicate','malformed-fips','short']){
 const project=parseProject(await readFile('target/acs-housing-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/acs-housing-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/acs-housing-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-acs-housing-csv').disabled,true);assert.equal(node('download-acs-housing-character').disabled,true);assert.equal(node('plan-file').parentElement.hidden,true);assert.equal(node('count-demo').hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.acsHousingRequest(project.files.acsHousingInputs));assert.equal(node('acs-housing-source').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-acs-housing-csv').disabled,false);assert.equal(node('download-acs-housing-character').disabled,output.character_status!=='ready');
 node('download-acs-housing-csv').onclick();assert.equal(downloadedLink.download,'acs-housing.csv');assert.equal(await(await fetch(downloadedLink.href)).text(),output.csv_text);URL.revokeObjectURL(downloadedLink.href);
 if(output.character_status==='ready'){node('download-acs-housing-character').onclick();assert.equal(downloadedLink.download,'housing-character-input.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),output.character_input);URL.revokeObjectURL(downloadedLink.href);}
 node('acs-housing-data_year').value='2019';node('acs-housing-data_year').oninput();assert.equal(node('download-acs-housing-csv').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.acsHousingInputs.settings.data_year,'2019');assert.deepEqual(lastSnapshot.files.acsHousingInputs.source,project.files.acsHousingInputs.source);
 node('acs-housing-data_year').value='';node('acs-housing-data_year').oninput();assert.equal(node('execute').disabled,true);node('acs-housing-data_year').value='2020';node('acs-housing-data_year').oninput();assert.equal(node('execute').disabled,false);
}
{
 const pending=deferred();const reading=node('acs-housing-source').onchange({target:{files:[{name:'stale.json',size:2,arrayBuffer:()=>pending.promise}]}});node('acs-housing-clear').onclick();pending.resolve(new TextEncoder().encode('[]').buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.acsHousingInputs.source,null);
}
console.log('ACS housing UI: inert Open, exact raw source request, busy/fresh CSV and character downloads, native CSV Blob bytes, saved sources/metadata, invalid metadata recovery and stale source rejection passed.');

for(const name of ['workplace','foreign','commuting','short-od']){
 const project=parseProject(await readFile('target/lodes-source-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/lodes-source-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/lodes-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-lodes-csv').disabled,true);assert.equal(node('download-lodes-character').disabled,true);assert.equal(node('plan-file').parentElement.hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.lodesRequest(project.files.lodesInputs));assert.equal(node('lodes-source').disabled,true);assert.equal(node('lodes-kind').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-lodes-csv').disabled,false);assert.equal(node('download-lodes-character').disabled,output.character_status!=='ready');node('download-lodes-csv').onclick();assert.equal(downloadedLink.download,'lodes-'+output.kind+'-tract.csv');assert.equal(await(await fetch(downloadedLink.href)).text(),output.csv_text);URL.revokeObjectURL(downloadedLink.href);
 if(output.character_status==='ready'){node('download-lodes-character').onclick();assert.equal(downloadedLink.download,'economic-character-input.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),output.character_input);URL.revokeObjectURL(downloadedLink.href);}
 node('lodes-data_year').value='2019';node('lodes-data_year').oninput();assert.equal(node('download-lodes-csv').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.lodesInputs.settings.data_year,'2019');assert.deepEqual(lastSnapshot.files.lodesInputs.source,project.files.lodesInputs.source);
 node('lodes-kind').value='bad';node('lodes-kind').onchange();assert.equal(node('execute').disabled,true);node('lodes-kind').value=project.files.lodesInputs.settings.kind;node('lodes-kind').onchange();assert.equal(node('execute').disabled,false);
}
{
 const pending=deferred();const reading=node('lodes-source').onchange({target:{files:[{name:'stale.csv',size:2,arrayBuffer:()=>pending.promise}]}});node('lodes-clear').onclick();pending.resolve(new TextEncoder().encode('xx').buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.lodesInputs.source,null);
}
console.log('LODES UI: inert Open, exact WAC/OD raw sources, busy/fresh CSV and economic downloads, saved bytes/settings, invalid kind recovery and stale source rejection passed.');

for(const name of ['workplace','commuting','large-expanded']){
 const project=parseProject(await readFile('target/lodes-gzip-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/lodes-gzip-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/lodes-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-lodes-csv').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.lodesRequest(project.files.lodesInputs));assert.equal(posted.at(-1).request.input.csv_text,undefined);assert.equal(posted.at(-1).request.input.source_base64,project.files.lodesInputs.source.base64);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-lodes-csv').disabled,false);assert.equal(node('download-lodes-character').disabled,output.character_status!=='ready');node('download-lodes-csv').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),output.csv_text);URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.equal(lastSnapshot.files.lodesInputs.source.base64,project.files.lodesInputs.source.base64);
}
console.log('LODES gzip UI: inert Open, exact compressed requests without JS expansion, native CSV downloads, character availability and saved original compressed bytes passed.');

for(const name of ['default','optional','selected-inventory','elections']){
 const project=parseProject(await readFile('target/fetch-plan-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/fetch-plan-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/fetch-plan-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('plan-file').parentElement.hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.fetchPlanRequest(project.files.fetchPlanInputs));assert.equal(node('fetch-plan-source').disabled,true);assert.equal(node('fetch-plan-template').disabled,true);uiWorker.onmessage({data:{type:'result',result:output}});
 node('save-project').onclick();assert.deepEqual(lastSnapshot.files.fetchPlanInputs,project.files.fetchPlanInputs);node('fetch-plan-template').onclick();assert.equal(downloadedLink.download,'fetch-source-selection.json');assert.deepEqual(JSON.parse(await(await fetch(downloadedLink.href)).text()),tools.FETCH_PLAN_DEFAULTS);URL.revokeObjectURL(downloadedLink.href);
 node('fetch-plan-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.fetchPlanInputs.source,null);
}
{
 const tools=await import('../../web/lab/fetch-plan-files.js'),raw=JSON.stringify(tools.FETCH_PLAN_DEFAULTS),pending=deferred(),reading=node('fetch-plan-source').onchange({target:{files:[{name:'stale.json',size:new TextEncoder().encode(raw).length,arrayBuffer:()=>pending.promise}]}});node('fetch-plan-clear').onclick();pending.resolve(new TextEncoder().encode(raw).buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.fetchPlanInputs.source,null);
}
console.log('Fetch planning UI: inert Open, exact selections, busy controls, raw source persistence, template bytes, default reset and stale source rejection passed.');

const acquisitionProject=parseProject(await readFile('target/fetch-acquisition/plan.bisect','utf8')),acquisitionPlan=JSON.parse(await readFile('target/fetch-acquisition/plan-result.json','utf8'));
for(const kind of ['acs-housing','lodes-wac']){
 const acquired=JSON.parse(await readFile('target/fetch-acquisition/'+kind+'-receipt.json','utf8')),bytes=Uint8Array.from(atob(acquired.inputs.source.base64),c=>c.charCodeAt(0));let calls=0;
 acquisitionRead=async()=>{calls++;return new Response(bytes);};importRead=()=>Promise.resolve(acquisitionProject);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'plan.bisect'}]}});assert.equal(calls,0);assert.equal(node('fetch-acquire-source').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:acquisitionPlan}});assert.equal(node('fetch-acquire-source').disabled,false);node('fetch-acquire-target').value=String(acquisitionPlan.items.findIndex(v=>v.kind===kind));node('fetch-acquire-key').value='test-key';const before=posted.length;await node('fetch-acquire-source').onclick();assert.equal(node('fetch-acquire-key').value,'');assert.equal(calls,1);assert.equal(posted.length,before);assert.equal(node('operation').value,kind==='acs-housing'?'prepare-acs-housing-source':'prepare-lodes-source');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.fetchAcquisitionReceipt,acquired.receipt);assert.equal(JSON.stringify(lastSnapshot).includes('test-key'),false);assert.deepEqual(lastSnapshot.files[kind==='acs-housing'?'acsHousingInputs':'lodesInputs'],acquired.inputs);assert.equal(node('fetch-acquire-source').disabled,true);
}
for(const mode of ['cancel','stale','404']){
 importRead=()=>Promise.resolve(acquisitionProject);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'plan.bisect'}]}});node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:acquisitionPlan}});
 const pending=deferred();acquisitionRead=()=>pending.promise;node('fetch-acquire-key').value='test-key';const attempt=node('fetch-acquire-source').onclick();assert.equal(node('fetch-plan-source').disabled,true);assert.equal(node('execute').disabled,true);assert.equal(node('cancel').disabled,false);
 if(mode==='cancel')node('cancel').onclick();if(mode==='stale'){await node('open-project').onchange({target:{files:[{name:'other.bisect'}]}});assert.match(node('project-status').textContent,/Finish or cancel/);node('project-name').oninput();}pending.resolve(new Response(mode==='404'?'missing':'[]',{status:mode==='404'?404:200}));await attempt;node('save-project').onclick();assert.equal(lastSnapshot.files.acsHousingInputs,undefined);assert.equal(lastSnapshot.files.lodesInputs,undefined);assert.equal(node('execute').disabled,false);
 if(mode==='404')assert.equal(lastSnapshot.files.fetchAcquisitionReceipt.availability,'unavailable');else assert.equal(lastSnapshot.files.fetchAcquisitionReceipt,undefined);
}
console.log('Fetch acquisition UI: no Open/request side effects, fresh-only download, exact bytes and receipt, explicit preparation handoff without worker Run, cancellation, stale project rejection, 404 unavailability and busy guards passed.');

for(const name of ['housing','workplace','commuting']){
 const project=parseProject(await readFile('target/fetch-data-package-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/fetch-data-package-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/fetch-data-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-fetch-data').disabled,true);assert.equal(node('plan-file').parentElement.hidden,true);node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.fetchDataRequest(project.files));assert.equal(node('fetch-data-force').disabled,true);
 uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-fetch-data').disabled,false);await node('download-fetch-data').onclick();assert.equal(downloadedLink.download,'fetch-derived-data.zip');assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),await tools.fetchDataZip(output));URL.revokeObjectURL(downloadedLink.href);
 node('fetch-data-force').checked=true;node('fetch-data-force').onchange();assert.equal(node('download-fetch-data').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.fetchDataPackageSettings.force,true);
}
{
 const project=parseProject(await readFile('target/acs-housing-wasm/valid.bisect','utf8')),output=JSON.parse(await readFile('target/acs-housing-wasm/valid-result.json','utf8')),tools=await import('../../web/lab/fetch-data-files.js');importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'source.bisect'}]}});assert.equal(node('prepare-fetch-data').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('prepare-fetch-data').disabled,false);const before=posted.length;node('prepare-fetch-data').onclick();assert.equal(posted.length,before);assert.equal(node('operation').value,'prepare-fetch-data-package');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.fetchDataPackageSettings,tools.fetchDataSettingsFromPrepared(output,project.files));
}
console.log('Fetch data UI: inert Open, exact source/settings requests, fresh ZIP bytes, busy controls, force invalidation/persistence and fresh-only preparation handoff without automatic execution passed.');

for(const name of ['2020','2010','2000','exact-i64']){
 const project=parseProject(await readFile('target/pl94-source-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/pl94-source-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/pl94-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-pl94-csv').disabled,true);assert.equal(node('pl94-fields').hidden,false);assert.equal(node('plan-file').parentElement.hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.pl94Request(project.files.pl94Inputs));assert.equal(node('pl94-geography').disabled,true);assert.equal(node('pl94-year').disabled,true);uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-pl94-csv').disabled,false);
 node('download-pl94-csv').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),output.csv_text);URL.revokeObjectURL(downloadedLink.href);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.pl94Inputs,project.files.pl94Inputs);
 node('pl94-year').value='1990';node('pl94-year').onchange();assert.equal(node('download-pl94-csv').disabled,true);assert.equal(node('execute').disabled,true);node('pl94-year').value=String(project.files.pl94Inputs.year);node('pl94-year').onchange();assert.equal(node('execute').disabled,false);
 node('pl94-clear').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.pl94Inputs.geography,null);assert.equal(lastSnapshot.files.pl94Inputs.population,null);
}
{
 const pending=deferred(),reading=node('pl94-geography').onchange({target:{files:[{name:'stale.pl',size:1,arrayBuffer:()=>pending.promise}]}});node('pl94-clear').onclick();pending.resolve(new Uint8Array([65]).buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.pl94Inputs.geography,null);
}
console.log('PL94 UI: inert Open, exact source requests across three vintages, exact integer CSV Blob, busy/fresh guards, retained bytes, invalid year recovery, clearing and stale source rejection passed.');

for(const [folder,name] of [
 ...['tracts-tracts','groups-block-groups','blocks-2020-blocks','blocks-2000-block-centroids'].map(name=>['tiger-source-wasm',name]),
 ['tiger-gzip-wasm','tracts-tracts']
]){
 const project=parseProject(await readFile('target/'+folder+'/'+name+'.bisect','utf8')),output=folder==='tiger-gzip-wasm'?project.result:JSON.parse(await readFile('target/tiger-source-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/tiger-files.js');
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-tiger-records').disabled,true);assert.equal(node('tiger-fields').hidden,false);assert.equal(node('plan-file').parentElement.hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.tigerRequest(project.files.tigerInputs));assert.equal(node('tiger-shp').disabled,true);assert.equal(node('tiger-kind').disabled,true);uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-tiger-records').disabled,false);node('download-tiger-records').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),output.records_json);URL.revokeObjectURL(downloadedLink.href);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.tigerInputs,project.files.tigerInputs);
 node('tiger-kind').value='invalid';node('tiger-kind').onchange();assert.equal(node('download-tiger-records').disabled,true);assert.equal(node('execute').disabled,true);node('tiger-kind').value=project.files.tigerInputs.kind;node('tiger-kind').onchange();assert.equal(node('execute').disabled,false);node('tiger-clear').onclick();node('save-project').onclick();for(const part of ['shp','dbf','shx'])assert.equal(lastSnapshot.files.tigerInputs[part],null);
}
{
 const pending=deferred(),reading=node('tiger-shp').onchange({target:{files:[{name:'stale.shp',size:1,arrayBuffer:()=>pending.promise}]}});node('tiger-clear').onclick();pending.resolve(new Uint8Array([65]).buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.tigerInputs.shp,null);
}
console.log('TIGER UI: inert Open, exact binary sources across four modes, geometry-record Blob bytes, busy/fresh guards, saved parts, invalid mode recovery, clear and stale-source rejection passed.');

for(const name of ['2020','2010','2000','2020-gzip']){
 const project=parseProject(await readFile('target/tract-preparation-wasm/'+name+'.bisect','utf8')),output=JSON.parse(await readFile('target/tract-preparation-wasm/'+name+'-result.json','utf8')),tools=await import('../../web/lab/tract-input-files.js');
 project.files.configRunInputs={...project.files.configRunInputs,coi:{name:'coi.json',base64:Buffer.from('{"version":1,"communities":[]}').toString('base64')},demographics:{name:'demographics.json',base64:Buffer.from('{}').toString('base64')}};
 project.files.configRunSettings={...project.files.configRunSettings,seats_per_district:5,total_seats:20,native_build:true,capture_rounds:true};
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:name+'.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-tract-graph').disabled,true);assert.equal(node('use-tract-graph').disabled,true);assert.equal(node('tract-preparation-fields').hidden,false);assert.equal(node('plan-file').parentElement.hidden,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tools.tractPreparationRequest(project.files));assert.equal(node('tract-preparation-prj').disabled,true);uiWorker.onmessage({data:{type:'result',result:output}});assert.equal(node('download-tract-graph').disabled,false);for(const kind of ['graph','geometry']){node('download-tract-'+kind).onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),output[kind+'_json']);URL.revokeObjectURL(downloadedLink.href);}
 node('save-project').onclick();assert.deepEqual(lastSnapshot.files.tigerInputs,project.files.tigerInputs);assert.deepEqual(lastSnapshot.files.pl94Inputs,project.files.pl94Inputs);const posts=posted.length;node('use-tract-graph').onclick();assert.equal(posted.length,posts);assert.equal(node('operation').value,'run-algorithm-config');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.configRunInputs.graph,tools.preparedTractRunSources(output).graph);assert.deepEqual(lastSnapshot.files.planGeometry,tools.preparedTractRunSources(output).geometry);assert.equal(node('use-tract-graph').disabled,true);
 for(const kind of ['coi','demographics'])assert.deepEqual(lastSnapshot.files.configRunInputs[kind],project.files.configRunInputs[kind]);
 for(const key of ['seats_per_district','total_seats','native_build','capture_rounds'])assert.equal(lastSnapshot.files.configRunSettings[key],project.files.configRunSettings[key]);
 assert.equal(lastSnapshot.files.configRunSettings.districts,tools.preparedTractRunSources(output).settings.districts);
}
{
 const project=parseProject(await readFile('target/tract-preparation-wasm/2020.bisect','utf8'));importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});await node('open-project').onchange({target:{files:[{name:'sources.bisect'}]}});node('tract-preparation-state').value='bad';node('tract-preparation-state').oninput();assert.equal(node('execute').disabled,true);node('tract-preparation-state').value='RI';node('tract-preparation-state').oninput();assert.equal(node('execute').disabled,false);
 const pending=deferred(),reading=node('tract-preparation-prj').onchange({target:{files:[{name:'stale.prj',size:1,arrayBuffer:()=>pending.promise}]}});node('tract-preparation-clear').onclick();pending.resolve(new Uint8Array([65]).buffer);await reading;node('save-project').onclick();assert.equal(lastSnapshot.files.tractPreparationSettings.projection,null);
}
console.log('Tract preparation UI: inert Open, exact retained source requests, fresh graph/GeoJSON downloads, busy guards, source-preserving save, explicit run handoff without worker execution, state recovery and stale projection rejection passed.');
if(process.argv.includes('--local-ca')){
 const archiveTools=await import('../../web/lab/workbench-archive.js'),tractTools=await import('../../web/lab/tract-input-files.js');
 const project=archiveTools.parseWorkbenchArchive(await readFile('target/large-state-preparation-wasm/ca-prepared.bisect-workbench','utf8'));
 const oldRead=sandbox.readWorkbenchArchiveFile,oldSave=sandbox.serializeWorkbenchArchive;sandbox.readWorkbenchArchiveFile=async()=>project;
 uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-workbench-archive').onchange({target:{files:[{name:'ca-prepared.bisect-workbench'}]}});assert.equal(posted.length,before);assert.equal(node('use-tract-graph').disabled,true);
 node('execute').onclick();assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request)),tractTools.tractPreparationRequest(project.files));uiWorker.onmessage({data:{type:'result',result:project.result}});
 const freshPosts=posted.length;node('use-tract-graph').onclick();assert.equal(posted.length,freshPosts);assert.equal(node('operation').value,'run-algorithm-config');
 let handed;sandbox.serializeWorkbenchArchive=async data=>{handed=structuredClone(data);return archiveTools.createWorkbenchArchive(handed);};await node('save-workbench-archive').onclick();
 assert.deepEqual(handed.files.tigerInputs,project.files.tigerInputs);assert.deepEqual(handed.files.pl94Inputs,project.files.pl94Inputs);
 assert.equal(Buffer.from(handed.files.configRunInputs.graph.base64,'base64').toString(),project.result.graph_json);assert.equal(Buffer.from(handed.files.planGeometry.base64,'base64').toString(),project.result.geometry_json);assert.equal(handed.files.configRunSettings.districts,52);URL.revokeObjectURL(downloadedLink.href);
 sandbox.readWorkbenchArchiveFile=oldRead;sandbox.serializeWorkbenchArchive=oldSave;
 console.log('Actual California UI: inert full source/result archive Open, exact explicit preparation request, fresh large-map handoff, 52-seat settings and original-source archive export without automatic engine run passed.');
}
for(const kind of ['tiger','pl94171']){
 const tools=await import('../../web/lab/fetch-acquisition.js'),project=parseProject(await readFile('target/census-zip-acquisition/plan.bisect','utf8')),plan=JSON.parse(await readFile('target/census-zip-acquisition/plan-result.json','utf8')),acquired=JSON.parse(await readFile('target/census-zip-acquisition/'+kind+'-acquired.json','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'plan.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('fetch-acquire-source').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:plan}});node('fetch-acquire-target').value=String(tools.fetchAcquisitionChoices(plan).find(v=>v.item.kind===kind).index);
 const original=sandbox.acquireFetchSource;sandbox.acquireFetchSource=async()=>acquired;const count=posted.length;await node('fetch-acquire-source').onclick();assert.equal(posted.length,count);assert.equal(node('operation').value,kind==='tiger'?'prepare-tiger-source':'prepare-pl94-source');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.fetchAcquisitionReceipt,acquired.receipt);assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.fetchArchiveSource)),acquired.raw_archive_source);assert.equal(lastSnapshot.files[kind==='tiger'?'tigerInputs':'pl94Inputs'][kind==='tiger'?'shp':'geography'].base64,acquired.inputs.archive.sources[kind==='tiger'?'shp':'geography'].base64);assert.equal(node('fetch-acquire-source').disabled,true);sandbox.acquireFetchSource=original;
}
console.log('Census acquisition UI: inert saved plan, fresh-only download, atomic archive/source receipt handoff, saved selected bytes and no automatic WASM execution passed.');
{
 const archiveTools=await import('../../web/lab/census-archive.js'),realReader=sandbox.readCensusArchive;
 const tiger=await archiveTools.extractCensusZip(new Uint8Array(await readFile('target/census-archive-tests/deflated.zip')),{kind:'tiger',year:2020});
 const project=parseProject(await readFile('target/tract-preparation-wasm/2020.bisect','utf8'));importRead=()=>Promise.resolve(project);await node('open-project').onchange({target:{files:[{name:'sources.bisect'}]}});
 sandbox.readCensusArchive=async()=>tiger;const before=posted.length;await node('tiger-zip').onchange({target:{files:[{name:'tiger.zip'}]}});assert.match(node('project-status').textContent,/ZIP imported locally/);node('save-project').onclick();assert.equal(posted.length,before);assert.equal(lastSnapshot.files.tigerInputs.shp.base64,tiger.sources.shp.base64);assert.deepEqual(lastSnapshot.files.tractPreparationSettings.projection,tiger.sources.prj);
 const pl94=await archiveTools.extractCensusZip(new Uint8Array(await readFile('target/census-archive-tests/pl2020.zip')),{kind:'pl94',year:2020});sandbox.readCensusArchive=async()=>pl94;await node('pl94-zip').onchange({target:{files:[{name:'ri2020.pl.zip'}]}});node('save-project').onclick();assert.deepEqual(lastSnapshot.files.pl94Inputs.geography,pl94.sources.geography);assert.equal(posted.length,before);
 const keep=lastSnapshot.files.pl94Inputs;sandbox.readCensusArchive=async()=>{throw new Error('CRC mismatch');};await node('pl94-zip').onchange({target:{files:[{name:'bad.zip'}]}});node('save-project').onclick();assert.deepEqual(lastSnapshot.files.pl94Inputs,keep);
 const delayed=deferred();sandbox.readCensusArchive=()=>delayed.promise;const pending=node('pl94-zip').onchange({target:{files:[{name:'stale.zip'}]}});node('pl94-clear').onclick();delayed.resolve(pl94);await pending;node('save-project').onclick();assert.equal(lastSnapshot.files.pl94Inputs.geography,null);let wasCancelled=false;sandbox.readCensusArchive=(f,s,{signal})=>new Promise((resolve,reject)=>signal.addEventListener('abort',()=>{wasCancelled=true;reject(new Error('Archive import cancelled.'));}));const cancelling=node('pl94-zip').onchange({target:{files:[{name:'cancel.zip'}]}});assert.equal(node('pl94-zip-cancel').disabled,false);node('pl94-zip-cancel').onclick();await cancelling;assert.equal(wasCancelled,true);assert.equal(node('pl94-zip-cancel').disabled,true);sandbox.readCensusArchive=realReader;
 console.log('Census ZIP UI: atomic source/projection handoff, lossless saved parts, no engine execution, rejection preservation and stale import discard passed.');
}
{
 const tools=await import('../../web/lab/workbench-archive.js'),archive=tools.parseWorkbenchArchive(await readFile('target/workbench-archive-tests/large.bisect-workbench','utf8')),oldRead=sandbox.readWorkbenchArchiveFile,oldSave=sandbox.serializeWorkbenchArchive;
 sandbox.readWorkbenchArchiveFile=async()=>archive;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-workbench-archive').onchange({target:{files:[{name:'large.bisect-workbench'}]}});assert.equal(posted.length,before);assert.match(node('result-status').textContent,/Imported result is unverified/);assert.match(node('project-status').textContent,/Opened workbench archive/);assert.equal(node('tiger-shp-name').textContent,archive.files.tigerInputs.shp.name);
 sandbox.serializeWorkbenchArchive=async data=>tools.createWorkbenchArchive(structuredClone(data));await node('save-workbench-archive').onclick();assert.match(downloadedLink.download,/\.bisect-workbench$/);const saved=tools.parseWorkbenchArchive(await(await fetch(downloadedLink.href)).text());assert.deepEqual(saved.files,archive.files);assert.deepEqual(saved.result,archive.result);URL.revokeObjectURL(downloadedLink.href);assert.equal(posted.length,before);
 const pending=deferred();sandbox.readWorkbenchArchiveFile=()=>pending.promise;const reading=node('open-workbench-archive').onchange({target:{files:[{name:'stale.bisect-workbench'}]}});node('project-name').value='Keep edit';node('project-name').oninput();pending.resolve(archive);await reading;assert.equal(node('project-name').value,'Keep edit');assert.match(node('project-status').textContent,/changed/);
 const waiting=deferred(),prior=downloadedLink;sandbox.serializeWorkbenchArchive=()=>waiting.promise;const saving=node('save-workbench-archive').onclick();assert.equal(node('execute').disabled,true);node('project-name').value='New edit';node('project-name').oninput();waiting.resolve(tools.createWorkbenchArchive(archive));await saving;assert.equal(downloadedLink,prior);assert.match(node('project-status').textContent,/changed/);
 let cancelled=false;sandbox.serializeWorkbenchArchive=(data,{signal})=>new Promise((resolve,reject)=>signal.addEventListener('abort',()=>{cancelled=true;reject(new Error('Archive save cancelled.'));}));const cancelling=node('save-workbench-archive').onclick();node('cancel-workbench-archive').onclick();await cancelling;assert.equal(cancelled,true);assert.equal(node('cancel-workbench-archive').disabled,true);sandbox.readWorkbenchArchiveFile=oldRead;sandbox.serializeWorkbenchArchive=oldSave;
 console.log('Large workbench archive UI: inert above-25-MiB Open, exact retained source/result export, no engine execution, stale read/save discard and cancel worker save passed.');
}

{
 const cacheTools=await import('../../web/lab/fletch-cache-files.js');
 const cacheResult=JSON.parse(await readFile('target/fletch-cache-wasm/exact-u64-result.json','utf8'));
 const text='{"schema_version":"fletch.cache-manifest.v1","generated_by":"fixture","cache_root":"C:/inert","entries":[{"dataset_id":"bisect.2020.ri.tiger","source_url":"https://example.test","cache_key":"opaque","relative_path":"../inert","sha256":"sha256:'+'a'.repeat(64)+'","bytes":9007199254740993,"fetched_at_ms":1,"verified":true}]}';
 const bytes=new TextEncoder().encode(text),source=await cacheTools.readFletchCacheManifest({name:'cache.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});
 const project=createProject({name:'Cache evidence',files:{fletchCacheManifest:source,fetchPlanInputs:{source:null}},operation:'inspect-fletch-cache',constraints:[],result:cacheResult,lastOperation:'inspect-fletch-cache'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'cache.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('download-fletch-cache').disabled,true);
 assert.equal(node('fletch-cache-manifest-name').textContent,'cache.json');
 assert.match(node('result-status').textContent,/unverified/);
 node('execute').onclick();assert.equal(posted.length,before+1);
 assert.equal(posted.at(-1).request.operation,'inspect-fletch-cache');
 assert.equal(posted.at(-1).request.input.cache_manifest_json,text);
 node('fletch-cache-clear').onclick();assert.equal(node('fletch-cache-manifest-name').textContent,'cache.json');
 uiWorker.onmessage({data:{type:'result',result:cacheResult}});
 assert.equal(node('download-fletch-cache').disabled,false);
 node('download-fletch-cache').onclick();assert.equal(downloadedLink.download,'fletch-cache-index.json');
 const output=await(await fetch(downloadedLink.href)).text();assert.equal(output,cacheResult.report_json); // Actual CLI writer appends no newline.assert.ok(output.includes('9007199254740993'));URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();const snapshot=createProject(lastSnapshot);assert.deepEqual(snapshot.files.fletchCacheManifest,source);
 node('fetch-plan-clear').onclick();assert.equal(node('download-fletch-cache').disabled,true);
 const real=sandbox.readFletchCacheManifest,pending=deferred();sandbox.readFletchCacheManifest=()=>pending.promise;
 const reading=node('fletch-cache-manifest').onchange({target:{files:[{name:'stale.json'}]}});
 node('fletch-cache-clear').onclick();pending.resolve(source);await reading;
 node('save-project').onclick();assert.equal(lastSnapshot.files.fletchCacheManifest,undefined);sandbox.readFletchCacheManifest=real;
 const tables=(await import('../../web/lab/count-results.js')).countResultTables(cacheResult);
 assert.ok(tables[0].rows.some(row=>row[1]==='9007199254740993'));
 console.log('FLETCH cache UI: inert project Open, original manifest text, explicit worker Run, busy/edited/stale-read guards, exact native report Blob and source-preserving Save passed.');
}

{
 const tools=await import('../../web/lab/fletch-cache-files.js');
 const bytes=new TextEncoder().encode(JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'inert',entries:[]}));
 const source=await tools.readFletchCacheManifest({name:'cache.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});
 source.name='all-vintage-cache.json';
 const project=createProject({name:'All-vintage cache',files:{fletchCacheManifest:source,fletchCacheSettings:{year:'all'}},operation:'inspect-fletch-cache',constraints:[]});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'all.bisect'}]}});
 assert.equal(node('fletch-cache-year').value,'all');assert.equal(posted.length,before);
 node('execute').onclick();assert.equal(posted.at(-1).request.input.selection.year,'all');
 uiWorker.onmessage({data:{type:'result',result:JSON.parse(await readFile('target/fletch-cache-wasm/all-vintages-result.json','utf8'))}});
 node('fletch-cache-year').value='2010';node('fletch-cache-year').onchange();assert.equal(node('download-fletch-cache').disabled,true);
 node('save-project').onclick();assert.equal(lastSnapshot.files.fletchCacheSettings.year,'2010');
 assert.throws(()=>parseProject(JSON.stringify({...project,files:{...project.files,fletchCacheSettings:{year:'unknown'}}})));
 console.log('FLETCH all-vintage UI: inert Open restores vintage, explicit all-vintage request, edited-vintage freshness invalidation and settings retention passed.');
}

{
 const project=JSON.parse(await readFile('target/fletch-sources-wasm/all-states-vintages.bisect','utf8'));
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'sources.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('fletch-source-year').value,'all');assert.equal(node('download-fletch-source').disabled,true);assert.match(node('result-status').textContent,/unverified/);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'inspect-fletch-sources');assert.equal(posted.at(-1).request.input.year,'all');assert.deepEqual(JSON.parse(JSON.stringify(posted.at(-1).request.input.states)),[]);
 assert.equal(node('fletch-source-year').disabled,true);
 const selected=node('fletch-source-year').value;node('fletch-source-year').onchange();assert.equal(node('fletch-source-year').value,selected);
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('download-fletch-source').disabled,false);
 node('download-fletch-source').onclick();assert.equal(downloadedLink.download,'fletch-source-handoff.csv');assert.equal(await(await fetch(downloadedLink.href)).text(),project.result.report_csv);URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files)),project.files);
 node('fletch-source-year').value='2010';node('fletch-source-year').onchange();assert.equal(node('download-fletch-source').disabled,true);
 const tables=(await import('../../web/lab/count-results.js')).countResultTables(project.result);assert.ok(tables[1].rows.some(r=>r.includes('adapter-required')));
 console.log('FLETCH source UI: inert Open, all-state/all-vintage explicit request, busy/freshness guards, exact CSV Blob, source/settings preservation and adapter disclosure passed.');
}

{
 const project=JSON.parse(await readFile('target/fletch-verify-wasm/verified.bisect','utf8'));
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'cache-verify.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('download-fletch-verification').disabled,true);assert.match(node('fletch-verify-files-name').textContent,/No cache files/);
 const bytes=new TextEncoder().encode('selected object bytes'),file=new Blob([bytes]);Object.defineProperty(file,'name',{value:'object'});Object.defineProperty(file,'webkitRelativePath',{value:'cache/objects/object'});
 node('fletch-verify-folder').onchange({target:{files:[file]}});assert.match(node('fletch-verify-files-name').textContent,/1 cache/);assert.equal(posted.length,before);
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'verify-fletch-cache-files');assert.equal(posted.at(-1).request.files[0].path,'objects/object');assert.equal(posted.at(-1).request.files[0].file,file);assert.equal(node('fletch-verify-folder').disabled,true);
 node('fletch-verify-clear').onclick();assert.match(node('fletch-verify-files-name').textContent,/1 cache/);
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('download-fletch-verification').disabled,false);
 node('download-fletch-verification').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),project.result.report_json+'\n');URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files)),project.files);
 await node('open-project').onchange({target:{files:[{name:'reopen.bisect'}]}});assert.match(node('fletch-verify-files-name').textContent,/No cache files/);assert.equal(node('download-fletch-verification').disabled,true);
 console.log('FLETCH cache verification UI: inert Open, required file reselection, explicit worker hashing request, busy guards, exact native report Blob, saved original manifest and reopened-result invalidation passed.');
}

{
 const project=JSON.parse(await readFile('target/fletch-merge-wasm/exact-u64.bisect','utf8'));
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'merge.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('download-fletch-merged').disabled,true);assert.equal(node('use-fletch-merged').disabled,true);
 assert.equal(node('fletch-merge-manifest-name').textContent,'base.json');assert.equal(node('fletch-merge-updates-name').textContent,'updates.json');
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'merge-fletch-cache-manifest');
 const decode=source=>Buffer.from(source.base64,'base64').toString('utf8');
 assert.equal(posted.at(-1).request.cache_manifest_json,decode(project.files.fletchCacheManifest));assert.equal(posted.at(-1).request.entries_json,decode(project.files.fletchCacheUpdates));assert.match(posted.at(-1).request.entries_json,/18446744073709551615/);
 assert.equal(node('fletch-merge-updates').disabled,true);node('fletch-merge-clear').onclick();assert.equal(node('fletch-merge-manifest-name').textContent,'base.json');
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('download-fletch-merged').disabled,false);
 node('download-fletch-merged').onclick();assert.equal(downloadedLink.download,'cache-manifest.json');assert.equal(await(await fetch(downloadedLink.href)).text(),project.result.manifest_json);URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files)),project.files);
 await node('open-project').onchange({target:{files:[{name:'again.bisect'}]}});assert.equal(node('download-fletch-merged').disabled,true);
 node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:project.result}});const handoffCount=posted.length;
 node('fletch-verify-files').value='old-selection';node('fletch-verify-folder').value='old-folder';
 node('use-fletch-merged').onclick();assert.equal(node('fletch-verify-files').value,'');assert.equal(node('fletch-verify-folder').value,'');assert.equal(node('operation').value,'verify-fletch-cache-files');assert.equal(posted.length,handoffCount);assert.match(node('fletch-verify-files-name').textContent,/No cache files/);assert.equal(node('download-fletch-verification').disabled,true);
 node('save-project').onclick();assert.equal(decode(lastSnapshot.files.fletchCacheManifest),project.result.manifest_json);assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.fletchCacheUpdates)),project.files.fletchCacheUpdates);
 await node('open-project').onchange({target:{files:[{name:'again.bisect'}]}});
 const tools=await import('../../web/lab/fletch-merge-files.js');
 const bytes=Buffer.from(project.files.fletchCacheUpdates.base64,'base64');
 const selected={name:'new-updates.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)};
 await node('fletch-merge-updates').onchange({target:{files:[selected]}});node('save-project').onclick();assert.equal(lastSnapshot.files.fletchCacheUpdates.name,'new-updates.json');assert.equal(node('download-fletch-merged').disabled,true);
 let finish;const stale=node('fletch-merge-updates').onchange({target:{files:[{...selected,name:'stale.json',arrayBuffer:()=>new Promise(resolve=>{finish=resolve;})}]}});
 await node('fletch-merge-updates').onchange({target:{files:[selected]}});finish(await selected.arrayBuffer());await stale;node('save-project').onclick();assert.equal(lastSnapshot.files.fletchCacheUpdates.name,'new-updates.json');
 await assert.rejects(()=>tools.readFletchCacheUpdates({name:'large.json',size:8*1024*1024+1,arrayBuffer(){throw new Error('Must not read');}}),/8 MiB/);
 assert.throws(()=>tools.fletchMergedManifestSource({...project.result,manifest_json:' '.repeat(8*1024*1024+1)}));
 console.log('FLETCH merge UI: inert Open, exact-u64 requests and manifest Blob, busy/stale-read/freshness guards, original-source Save/Open and explicit nonexecuting file-verification handoff passed.');
}

{
 const project=JSON.parse(await readFile('target/fletch-stage-wasm/acquired-acs-housing.bisect','utf8'));
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'staging.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-fletch-staged').disabled,true);assert.equal(node('use-fletch-staged').disabled,true);
 const sourceName=project.files.fletchStageInputs.source.name;
 node('fletch-stage-acquired').onclick();assert.equal(posted.length,before);assert.equal(node('fletch-stage-source-name').textContent,sourceName);assert.equal(node('fletch-stage-dataset').value,'bisect.2020.ri.acs-housing');
 node('execute').onclick();assert.equal(posted.at(-1).request.operation,'stage-fletch-cache-source');assert.equal(posted.at(-1).request.input.source_base64,project.files.fletchStageInputs.source.base64);assert.equal(posted.at(-1).request.input.expected_source_sha256,project.files.fetchAcquisitionReceipt.source_sha256);
 assert.equal(node('fletch-stage-source').disabled,true);node('fletch-stage-clear').onclick();assert.equal(node('fletch-stage-source-name').textContent,sourceName);
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('download-fletch-staged').disabled,false);
 await node('download-fletch-staged').onclick();assert.equal(downloadedLink.download,'fletch-cache.zip');
 const tools=await import('../../web/lab/fletch-stage-files.js');assert.deepEqual(Buffer.from(await(await fetch(downloadedLink.href)).arrayBuffer()),Buffer.from(await tools.fletchStagedCacheZip(project.result,project.files.fletchStageInputs)));URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files)),project.files);
 const handoffCount=posted.length;node('use-fletch-staged').onclick();assert.equal(node('operation').value,'merge-fletch-cache-manifest');assert.equal(posted.length,handoffCount);assert.equal(node('download-fletch-merged').disabled,true);
 node('save-project').onclick();assert.equal(Buffer.from(lastSnapshot.files.fletchCacheUpdates.base64,'base64').toString('utf8'),project.result.entries_json);assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.fletchStageInputs)),project.files.fletchStageInputs);
 await node('open-project').onchange({target:{files:[{name:'again.bisect'}]}});assert.equal(node('download-fletch-staged').disabled,true);
 node('fletch-stage-time').value='18446744073709551615';node('fletch-stage-time').onchange();node('save-project').onclick();assert.equal(lastSnapshot.files.fletchStageInputs.fetched_at_ms,'18446744073709551615');node('execute').onclick();assert.equal(posted.at(-1).request.input.fetched_at_ms,'18446744073709551615');uiWorker.onmessage({data:{type:'error',error:'test rejection'}});
 let finish;const bytes=Uint8Array.of(1,255),selected={name:'replacement.bin',size:2,arrayBuffer:async()=>bytes.buffer};
 const stale=node('fletch-stage-source').onchange({target:{files:[{...selected,name:'stale.bin',arrayBuffer:()=>new Promise(resolve=>{finish=resolve;})}]}});
 await node('fletch-stage-source').onchange({target:{files:[selected]}});finish(bytes.buffer);await stale;node('save-project').onclick();assert.equal(lastSnapshot.files.fletchStageInputs.source.name,'replacement.bin');assert.equal(lastSnapshot.files.fletchStageInputs.expected_source_sha256,null);
 assert.equal(node('download-fletch-staged').disabled,true);
 console.log('Raw FLETCH staging UI: inert Open/acquisition selection, exact bytes/u64 requests, busy and stale-read guards, native cache ZIP Blob, saved sources and explicit nonexecuting merge handoff passed.');
}

for(const kind of ['tiger','pl94171']){
 const project=JSON.parse(await readFile('target/fletch-stage-wasm/acquired-'+kind+'.bisect','utf8'));
 importRead=async()=>project;uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;
 await node('open-project').onchange({target:{files:[{name:'raw-archive.bisect'}]}});assert.equal(posted.length,before);assert.equal(node('download-fletch-staged').disabled,true);
 node('fletch-stage-acquired').onclick();assert.equal(posted.length,before);assert.equal(node('fletch-stage-source-name').textContent,project.files.fetchArchiveSource.source.name);
 node('execute').onclick();assert.equal(posted.at(-1).request.input.source_base64,project.files.fetchArchiveSource.source.base64);assert.equal(posted.at(-1).request.input.expected_source_sha256,project.files.fetchAcquisitionReceipt.source_sha256);
 uiWorker.onmessage({data:{type:'result',result:project.result}});await node('download-fletch-staged').onclick();const tools=await import('../../web/lab/fletch-stage-files.js');assert.deepEqual(Buffer.from(await(await fetch(downloadedLink.href)).arrayBuffer()),Buffer.from(await tools.fletchStagedCacheZip(project.result,project.files.fletchStageInputs,project.files)));URL.revokeObjectURL(downloadedLink.href);
 node('save-project').onclick();assert.equal(lastSnapshot.files.fletchStageInputs.source,null);assert.equal(lastSnapshot.files.fletchStageInputs.source_ref,'fetchArchiveSource');assert.deepEqual(JSON.parse(JSON.stringify(lastSnapshot.files.fetchArchiveSource)),project.files.fetchArchiveSource);
 node('fletch-stage-time').value='123';node('fletch-stage-time').onchange();assert.equal(node('download-fletch-staged').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.fletchStageInputs.source_ref,'fetchArchiveSource');
 await node('open-project').onchange({target:{files:[{name:'raw-archive.bisect'}]}});assert.equal(posted.length,before+1);assert.equal(node('download-fletch-staged').disabled,true);
 // Replacing the acquisition retires references to its previous raw body.
 node('operation').value='plan-fetch-sources';node('operation').change();node('execute').onclick();const plan=JSON.parse(await readFile('target/census-zip-acquisition/plan-result.json','utf8'));uiWorker.onmessage({data:{type:'result',result:plan}});
 const nextKind=kind==='tiger'?'pl94171':'tiger',acquired=JSON.parse(await readFile('target/census-zip-acquisition/'+nextKind+'-acquired.json','utf8')),original=sandbox.acquireFetchSource;sandbox.acquireFetchSource=async()=>acquired;node('fetch-acquire-target').value=String(plan.items.findIndex(v=>v.kind===nextKind));
 await node('fetch-acquire-source').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.fletchStageInputs,undefined);assert.equal(lastSnapshot.files.fetchArchiveSource.kind,nextKind);sandbox.acquireFetchSource=original;
 await node('open-project').onchange({target:{files:[{name:'raw-archive.bisect'}]}});node('operation').value='plan-fetch-sources';node('operation').change();node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:plan}});
 const missing={...project.files.fetchAcquisitionReceipt,http_status:404,availability:'unavailable',byte_length:0,source_sha256:null};sandbox.acquireFetchSource=async()=>({receipt:missing,inputs:null});node('fetch-acquire-target').value=String(plan.items.findIndex(v=>v.kind===kind));await node('fetch-acquire-source').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.fetchArchiveSource,undefined);assert.equal(lastSnapshot.files.fletchStageInputs,undefined);assert.equal(lastSnapshot.files.fetchAcquisitionReceipt.availability,'unavailable');sandbox.acquireFetchSource=original;
}
console.log('Raw Census archive UI: inert reference restore, original ZIP request/download, single-copy saved source, metadata-edit freshness and acquisition replacement/404 retiring dependent references passed.');

const mapPreviewFixture={schema_version:"bisect-native-district-map-v1",width:1200,png_sha256:"hash",png_base64:"opaque-png",svg_text:"opaque-svg"};
assert.equal(resultPreview(mapPreviewFixture),JSON.stringify({schema_version:"bisect-native-district-map-v1",width:1200,png_sha256:"hash"},null,2));
assert.equal(mapPreviewFixture.png_base64,"opaque-png");
assert.equal(mapPreviewFixture.svg_text,"opaque-svg");

// Registry readiness is distinct from measuring a comparison.
node('operation').value='inspect-label-comparison';node('operation').change();
for(const [key,value]of Object.entries({label_a:'proposal_a',label_b:'proposal_b',year:'2020'}))node('label-compare-'+key).value=value;
const compareRegistryText=JSON.stringify({proposal_a:{built:['2020'],analyzed:['2020'],reported:[]},proposal_b:{built:['2020'],analyzed:['2020'],reported:[]}}),compareRegistryBytes=new TextEncoder().encode(compareRegistryText);
await node('label-compare-registry').onchange({target:{files:[{name:'.bisect',size:compareRegistryBytes.length,arrayBuffer:async()=>compareRegistryBytes.buffer}]}});
node('execute').onclick();const readinessRequest=structuredClone(posted.at(-1).request);assert.deepEqual(readinessRequest,{operation:'inspect-label-comparison',registry_json:compareRegistryText,label_a:'proposal_a',label_b:'proposal_b',year:'2020'});assert.equal(node('label-compare-clear').disabled,true);assert.equal(node('download-label-compare-json').disabled,true);
const readinessResult={schema_version:'bisect-label-comparison-readiness-v1',status:'ready-to-compare',label_a:'proposal_a',label_b:'proposal_b',year:'2020',native_text:'Native comparison instructions',native_json:'{"status":"delegated_to_compare"}'};uiWorker.onmessage({data:{type:'result',result:readinessResult}});
for(const kind of ['text','json']){assert.equal(node('download-label-compare-'+kind).disabled,false);node('download-label-compare-'+kind).onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),readinessResult['native_'+kind]);URL.revokeObjectURL(downloadedLink.href);}
node('save-project').onclick();const readinessProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(readinessProject);const beforeReadinessOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'readiness.bisect'}]}});assert.equal(posted.length,beforeReadinessOpen);assert.equal(node('download-label-compare-json').disabled,true);assert.equal(node('label-compare-label_a').value,'proposal_a');node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),readinessRequest);uiWorker.onmessage({data:{type:'result',result:readinessResult}});node('label-compare-label_b').value='missing';node('label-compare-label_b').oninput();assert.equal(node('download-label-compare-json').disabled,true);
let readinessHugeRead=false;await node('label-compare-registry').onchange({target:{files:[{name:'.bisect',size:1024*1024+1,arrayBuffer:async()=>{readinessHugeRead=true;}}]}});assert.equal(readinessHugeRead,false);
const readinessStale=deferred(),readinessPending=node('label-compare-registry').onchange({target:{files:[{name:'.bisect',size:2,arrayBuffer:()=>readinessStale.promise}]}});node('label-compare-clear').onclick();readinessStale.resolve(new Uint8Array([123,125]).buffer);await readinessPending;node('save-project').onclick();assert.equal(lastSnapshot.files.labelCompareInputs.registry,null);assert.equal(lastSnapshot.files.labelCompareInputs.label_b,'missing');
console.log('Label comparison readiness UI: explicit request, busy/export gates, exact native Blob downloads, inert Open/source retention, edits, pre-read size and stale-read clearing passed.');

// External label import must retain exact original sources and reject stale reads.
const importedGolden=JSON.parse(await readFile('target/label-import-wasm/csv.bisect','utf8')),importedInputs=importedGolden.files.labelImportInputs;
node('operation').value='import-label-assignments';node('operation').change();for(const [key,value]of Object.entries(importedInputs.settings))node('label-import-'+key).value=value;
for(const key of ['source','registry']){const selected=importedInputs[key],bytes=new Uint8Array(Buffer.from(selected.base64,'base64'));await node('label-import-'+key).onchange({target:{files:[{name:selected.name,size:bytes.length,arrayBuffer:async()=>bytes.buffer}]}});}
node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),labelImportTools.labelImportRequest(importedInputs));assert.equal(node('label-import-clear').disabled,true);assert.equal(node('label-import-source').disabled,true);assert.equal(node('download-label-import').disabled,true);uiWorker.onmessage({data:{type:'result',result:importedGolden.result}});assert.equal(node('download-label-import').disabled,false);await node('download-label-import').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),await labelImportTools.labelImportZip(importedGolden.result,importedInputs));URL.revokeObjectURL(downloadedLink.href);
node('save-project').onclick();const importedUiProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(importedUiProject);const beforeImportedOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'imported.bisect'}]}});assert.equal(posted.length,beforeImportedOpen);assert.equal(node('download-label-import').disabled,true);node('save-project').onclick();assert.deepEqual(lastSnapshot.files.labelImportInputs,importedInputs);node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),labelImportTools.labelImportRequest(importedInputs));uiWorker.onmessage({data:{type:'result',result:importedGolden.result}});node('label-import-created').value='2026-10-12T22:00:00Z';node('label-import-created').oninput();assert.equal(node('download-label-import').disabled,true);
let importHugeRead=false;await node('label-import-source').onchange({target:{files:[{name:'huge.csv',size:16*1024*1024+1,arrayBuffer:async()=>{importHugeRead=true;}}]}});assert.equal(importHugeRead,false);const importStale=deferred(),importPending=node('label-import-source').onchange({target:{files:[{name:'later.csv',size:2,arrayBuffer:()=>importStale.promise}]}});node('label-import-clear').onclick();importStale.resolve(new Uint8Array([123,125]).buffer);await importPending;node('save-project').onclick();for(const key of ['source','dbf','shx'])assert.equal(lastSnapshot.files.labelImportInputs[key],null);assert.deepEqual(lastSnapshot.files.labelImportInputs.registry,importedInputs.registry);node('label-import-clear-registry').onclick();node('save-project').onclick();assert.equal(lastSnapshot.files.labelImportInputs.registry,null);
console.log('External label import UI: explicit native request, exact ZIP Blob, busy/export gates, inert source-preserving Open, timestamp freshness, separate Clear and oversized/stale-read guards passed.');

// A restored imported/context pair is inert until independently reparsed by WASM.
const pairedGolden=JSON.parse(await readFile('target/label-import-plan-wasm/csv.bisect','utf8'));
importRead=()=>Promise.resolve(pairedGolden);let beforePairOpen=posted.length;
await node('open-project').onchange({target:{files:[{name:'paired.bisect'}]}});
assert.equal(posted.length,beforePairOpen);assert.equal(node('download-label-import-plan').disabled,true);assert.equal(node('analyze-label-import-plan').disabled,true);
node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),labelImportTools.labelImportPlanRequest(pairedGolden.files.labelImportInputs,pairedGolden.files.labelImportContext));assert.equal(node('label-import-context-file').disabled,true);assert.equal(node('label-import-context-basis').disabled,true);
uiWorker.onmessage({data:{type:'result',result:pairedGolden.result}});assert.equal(node('download-label-import-plan').disabled,false);await node('download-label-import-plan').onclick();assert.deepEqual(new Uint8Array(await(await fetch(downloadedLink.href)).arrayBuffer()),await labelImportTools.labelImportPlanZip(pairedGolden.result,pairedGolden.files.labelImportInputs,pairedGolden.files.labelImportContext));URL.revokeObjectURL(downloadedLink.href);
const beforePairAdopt=posted.length;node('analyze-label-import-plan').onclick();assert.equal(posted.length,beforePairAdopt);node('save-project').onclick();assert.equal(lastSnapshot.operation,'analyze-plan-package');assert.equal(lastSnapshot.files.auditPlanText.base64,Buffer.from(pairedGolden.result.document_json).toString('base64'));assert.equal(lastSnapshot.files.auditContextText.base64,pairedGolden.files.labelImportContext.context.base64);assert.equal(lastSnapshot.files.auditLineageText,undefined);assert.equal(lastSnapshot.files.auditCertificateText,undefined);
await node('open-project').onchange({target:{files:[{name:'paired.bisect'}]}});let hugePairRead=false;await node('label-import-context-file').onchange({target:{files:[{name:'huge.rctx',size:8*1024*1024+1,arrayBuffer:async()=>{hugePairRead=true;}}]}});assert.equal(hugePairRead,false);
const pairStale=deferred(),pairPending=node('label-import-context-file').onchange({target:{files:[{name:'later.rctx',size:2,arrayBuffer:()=>pairStale.promise}]}});node('label-import-context-clear').onclick();pairStale.resolve(new Uint8Array([123,125]).buffer);await pairPending;node('save-project').onclick();assert.equal(lastSnapshot.files.labelImportContext.context,null);beforePairOpen=posted.length;node('execute').onclick();assert.equal(posted.length,beforePairOpen);
console.log('Imported-plan context UI: exact native requests/ZIP Blob, busy controls, inert Open and analysis handoff, retired lineage/certificate, pre-read size and stale/missing-context guards passed.');

await node('open-project').onchange({target:{files:[{name:'paired.bisect'}]}});node('execute').onclick();assert.equal(node('use-label-import-batch').disabled,true);assert.equal(node('add-label-import-batch').disabled,true);uiWorker.onmessage({data:{type:'result',result:pairedGolden.result}});
let beforeExternalBatch=posted.length;node('use-label-import-batch').onclick();assert.equal(posted.length,beforeExternalBatch);assert.equal(node('operation').value,'run-label-analysis');node('save-project').onclick();const externalBatchInput=JSON.parse(Buffer.from(lastSnapshot.files.labelAnalysisInputs.source.base64,'base64').toString());assert.equal(externalBatchInput.build_index_json,pairedGolden.result.files['import-index.json']);assert.equal(externalBatchInput.external_import.input.source.base64,pairedGolden.files.labelImportInputs.source.base64);assert.equal(externalBatchInput.external_import.district_bases.washington,'one-based');assert.equal(externalBatchInput.states.washington.context_json,pairedGolden.result.context_json);
node('label-analysis-created').value='2026-10-12T23:30:00Z';node('label-analysis-types').value='manifest,contiguity,splits,statistics';node('execute').onclick();const externalGolden=JSON.parse(await readFile('target/external-label-analysis-wasm/csv-result.json','utf8'));assert.equal(posted.at(-1).request.input.external_import.input.source.base64,pairedGolden.files.labelImportInputs.source.base64);uiWorker.onmessage({data:{type:'result',result:externalGolden}});assert.equal(node('download-label-analysis').disabled,false);node('save-project').onclick();const externalUiProject=createProject(lastSnapshot);importRead=()=>Promise.resolve(externalUiProject);beforeExternalBatch=posted.length;await node('open-project').onchange({target:{files:[{name:'external-analysis.bisect'}]}});assert.equal(posted.length,beforeExternalBatch);assert.equal(node('download-label-analysis').disabled,true);
console.log('External label batch UI: explicit source-preserving handoff, busy gates, original index/context/basis requests, native result downloads and inert Open passed.');

const hubGolden=JSON.parse(await readFile('target/plan-hub-wasm/library-zeta.bisect','utf8'));importRead=()=>Promise.resolve(hubGolden);const beforeHubOpen=posted.length;await node('open-project').onchange({target:{files:[{name:'hub.bisect'}]}});assert.equal(posted.length,beforeHubOpen);assert.equal(node('open-plan-hub-target').disabled,true);assert.equal(node('download-plan-hub-text').disabled,true);
node('execute').onclick();assert.deepEqual(structuredClone(posted.at(-1).request),planHubTools.planHubRequest(hubGolden.files.planHubInputs,hubGolden.files.tuiLibraryInputs));assert.equal(node('plan-hub-configure').disabled,true);assert.equal(node('plan-hub-yaml').disabled,true);uiWorker.onmessage({data:{type:'result',result:hubGolden.result}});assert.equal(node('open-plan-hub-target').disabled,false);
node('download-plan-hub-text').onclick();assert.equal(await(await fetch(downloadedLink.href)).text(),hubGolden.result.native_fallback_text);URL.revokeObjectURL(downloadedLink.href);let beforeHubAdopt=posted.length;node('open-plan-hub-target').onclick();assert.equal(posted.length,beforeHubAdopt);assert.equal(node('operation').value,'inspect-tui-plan-library');node('save-project').onclick();assert.equal(lastSnapshot.files.tuiLibraryInputs.settings.selected_directory,'library/zeta');assert.equal(lastSnapshot.files.tuiLibraryInputs.settings.filter,'');
await node('open-project').onchange({target:{files:[{name:'hub.bisect'}]}});node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:hubGolden.result}});node('plan-hub-label').value='other';node('plan-hub-label').oninput();assert.equal(node('open-plan-hub-target').disabled,true);let hugeHubRead=false;await node('plan-hub-registry').onchange({target:{files:[{name:'.bisect',size:1024*1024+1,arrayBuffer:async()=>{hugeHubRead=true;}}]}});assert.equal(hugeHubRead,false);const hubStale=deferred(),hubPending=node('plan-hub-registry').onchange({target:{files:[{name:'.bisect',size:2,arrayBuffer:()=>hubStale.promise}]}});node('plan-hub-clear-registry').onclick();hubStale.resolve(new Uint8Array([123,125]).buffer);await hubPending;node('save-project').onclick();assert.equal(lastSnapshot.files.planHubInputs.registry,null);
const hubYamlGolden=JSON.parse(await readFile('target/plan-hub-wasm/local-yaml.bisect','utf8'));importRead=()=>Promise.resolve(hubYamlGolden);await node('open-project').onchange({target:{files:[{name:'hub-yaml.bisect'}]}});node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:hubYamlGolden.result}});beforeHubAdopt=posted.length;node('open-plan-hub-target').onclick();assert.equal(posted.length,beforeHubAdopt);assert.equal(node('operation').value,'validate-algorithm-config');node('save-project').onclick();assert.deepEqual(lastSnapshot.files.configSource,hubYamlGolden.files.planHubInputs.yaml);
const bomRegistry={name:'.bisect',base64:Buffer.from('\ufeff{}').toString('base64')};assert.equal(labelInventoryTools.labelInventoryRequest({label:'',registry:bomRegistry}).registry_json,'\ufeff{}');assert.equal(labelInventoryTools.labelCompareRequest({label_a:'a',label_b:'b',year:'2020',registry:bomRegistry}).registry_json,'\ufeff{}');assert.equal(labelImportTools.labelImportRequest({...importedInputs,registry:bomRegistry}).input.registry_json,'\ufeff{}');
console.log('Plan hub UI: exact native requests/text Blob, busy/freshness controls, inert Open/library/config handoffs, pre-read size/stale-read guards and raw BOM registry bytes across inventory/comparison/import passed.');

// Advisory binary setting is saved and edited without starting a worker.
{
 const project=parseProject(await readFile('target/verify-binary-wasm/skip.bisect','utf8'));
 importRead=()=>Promise.resolve(project);uiWorker.onmessage({data:{type:'ready'}});const before=posted.length;await node('open-project').onchange({target:{files:[{name:'binary-skip.bisect'}]}});
 assert.equal(posted.length,before);assert.equal(node('engine-replay-skip-binary').checked,true);assert.equal(node('engine-replay-binary-option').hidden,false);assert.equal(node('download-engine-replay').disabled,true);
 node('execute').onclick();assert.equal(posted.at(-1).request.skip_binary_check,true);assert.equal(node('engine-replay-skip-binary').disabled,true);node('engine-replay-skip-binary').checked=false;node('engine-replay-skip-binary').onchange();assert.equal(posted.length,before+1);
 uiWorker.onmessage({data:{type:'result',result:project.result}});assert.equal(node('download-engine-replay').disabled,false);node('engine-replay-skip-binary').checked=false;node('engine-replay-skip-binary').onchange();assert.equal(posted.length,before+1);assert.equal(node('download-engine-replay').disabled,true);node('save-project').onclick();assert.equal(lastSnapshot.files.engineReplayInputs.skipBinaryCheck,false);
 node('engine-replay-clear').onclick();assert.equal(node('engine-replay-skip-binary').checked,false);
}
console.log('Composed binary advisory UI: inert Open, retained skip, exact explicit request, busy/edit guards, source retention and Clear default passed.');

// Hash verification is asynchronous. A selection change or Close must retire
// its authority before any HTML frame can be created, even if hashing finishes.
{
 const project=parseProject(await readFile('target/label-report-wasm/project.bisect','utf8'));
 project.files.labelReportInputs.settings.format='html';
 const native=JSON.parse(await readFile('target/label-report-wasm/html-result.json','utf8'));
 importRead=()=>Promise.resolve(project);await node('open-project').onchange({target:{files:[{name:'reports.bisect'}]}});
 assert.equal(node('preview-commission').disabled,true);node('execute').onclick();uiWorker.onmessage({data:{type:'result',result:native}});assert.equal(node('preview-commission').disabled,false);
 const original=sandbox.generatedReportText,before=posted.length;
 try{
  for(const cancel of [()=>node('generated-report-select').onchange(),()=>node('close-commission-preview').onclick()]){
   const pending=deferred();sandbox.generatedReportText=()=>pending.promise;
   const previewing=node('preview-commission').onclick();assert.equal(node('preview-commission').disabled,true);assert.equal(node('print-commission').disabled,true);cancel();
   pending.resolve({name:'report.html',text:'<p>Late hash result</p>',sha256:'0'.repeat(64)});await previewing;
   assert.equal(node('print-commission').disabled,true);assert.equal(node('commission-preview-panel').hidden,true);assert.equal(node('preview-commission').disabled,false);
  }
 }finally{sandbox.generatedReportText=original;}
 assert.equal(posted.length,before);
}
console.log('Generated report UI: imported reports stay inactive; selection/Close retire pending hash verification before frame creation without an engine run.');

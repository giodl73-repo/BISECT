import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const {createProject,parseProject,validateProject,readProjectFile,parseSafeJson,readFileTextStrict,MAX_PROJECT_BYTES}=await import('../../web/lab/project.js');
const {resultPreview,MAX_PREVIEW_CHARS}=await import('../../web/lab/result-preview.js');
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
  if(!nodes.has(id))nodes.set(id,{value:id==='operation'?'audit-plan':id==='project-name'?'Current project':'',disabled:false,textContent:'',addEventListener(type,handler){this[type]=handler;}});
  return nodes.get(id);
}
let restoreRead=()=>Promise.resolve(null), importRead=()=>Promise.resolve(saved);
let lastSnapshot;
let uiWorker;const posted=[];
const packageTools=await import('../../web/lab/package-files.js');
const sandbox={
  document:{getElementById:node,querySelectorAll:()=>[]},
  Worker:class {constructor(){uiWorker=this;}postMessage(value){posted.push(value);}terminate(){}}, URL, AbortController,structuredClone,TextEncoder,
  setTimeout:()=>1,clearTimeout(){},
  renderCountResult(){},resultPreview,...packageTools,
  createProject(data){lastSnapshot=data;return data;},readProjectFile:(...args)=>importRead(...args),
  readJsonFile:async file=>parseSafeJson(await file.text()),
  readAutosave:()=>restoreRead(),writeAutosave:()=>Promise.resolve(),
};
runInNewContext(ui.replace(/^import .*;\r?\n/gm,'').replaceAll('import.meta.url',JSON.stringify(new URL('../../web/lab/workbench.js',import.meta.url).href)),sandbox);
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
  runInNewContext(source.replace(/^import .*;\r?\n/gm,''),{self,readFileTextStrict,parseProject:parser,parseLabProject:parser,parseSafeJson:parser});
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

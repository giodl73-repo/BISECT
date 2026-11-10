import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive,createCompressedWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {validateFreshNativeStateSession,retainFreshNativeStateRun,freshNativeStateRetainedText} from '../../web/lab/fresh-native-state-session.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {loadNativeUnitIdentityCorpus} from './fixtures/native-unit-identity.mjs';
import {freshNativeStateInput} from './fixtures/fresh-native-state.mjs';
const argv=process.argv.slice(2),arg=name=>{const i=argv.indexOf(name);assert(i>=0,'Supply '+name);return path.resolve(argv[i+1]);};
const parent=arg('--output');await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,'fresh-project-'));
const bytes=await fs.readFile(arg('--wasm')),engine=await instantiateEngine(bytes),rows=await loadNativeUnitIdentityCorpus();
const evidence={status:'running',cases:[],refusals:[],scope:'Existing .bisect and uncompressed/compressed workbench archive persistence for twelve actual fresh WASM runs. Retained result is encoded data and is not an execution grant. No capacity or full parity claim.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
function session(input,result=null){
 const args=input.arguments.filter((value,i,all)=>!['--adjacency','--coi-weights'].includes(value)&&!['--adjacency','--coi-weights'].includes(all[i-1]));
 const sources={};for(const [kind,source]of Object.entries(input.source.sources))if(source)sources[kind]=source;
 if(input.auxiliary_json){const source=JSON.parse(input.auxiliary_json).native_auxiliary.coi;sources.coi={name:source.source_name,base64:source.bytes_base64};}
 return validateFreshNativeStateSession({schema_version:'bisect-fresh-native-state-session-v1',fields:{arguments_json:JSON.stringify(args),state:input.source.state,year:input.source.year,label:input.label,chamber:input.chamber,created_at:input.created_at},sources,display_geometry_json:null,retained_run:result?retainFreshNativeStateRun(result):null,runtime_artifact:result?engine.artifactIdentity:null});
}
const project=value=>createProject({name:'Fresh native State project',operation:'run-tui-configured',constraints:[],files:{freshNativeStateSession:value},result:null,lastOperation:null});
try{
 for(const row of rows){
  const input=freshNativeStateInput(row),result=engine.execute({operation:'run-fresh-native-state',input});assert.equal(result.export_error,null);
  const value=session(input,result);
  // Exercise the archive's existing >=64 KiB shared-string compression path
  // with exact valid GeoJSON text, including its harmless leading whitespace.
  // Native graph size, assignments and seed remain the original fixed cases.
  const geometry={type:'FeatureCollection',features:JSON.parse(row.document_json).plan.units.unit_ids.map((id,i)=>({type:'Feature',properties:{GEOID:id},geometry:{type:'Polygon',coordinates:[[[i,0],[i+.5,0],[i+.5,.5],[i,.5],[i,0]]]}}))};
  value.display_geometry_json=' '.repeat(64*1024)+JSON.stringify(geometry);
  const original=project(value),text=JSON.stringify(original),parsed=parseProject(text);assert.deepEqual(parsed.files.freshNativeStateSession,value);
  const archived=parseWorkbenchArchive(createWorkbenchArchive(original));assert.deepEqual(archived.files.freshNativeStateSession,value);
  const compressed=await createCompressedWorkbenchArchive(original),expanded=await parseWorkbenchArchiveBytes(compressed);assert.deepEqual(expanded.files.freshNativeStateSession,value);
  assert.deepEqual(JSON.parse(freshNativeStateRetainedText(value.retained_run)).export.document.plan,JSON.parse(row.document_json).plan);
  await fs.writeFile(path.join(root,row.name+'.bisect'),text);await fs.writeFile(path.join(root,row.name+'.bisect-workbench.gz'),compressed);
  evidence.cases.push({name:row.name,status:'passed',ordinary_project_exact:true,workbench_archive_exact:true,compressed_archive_exact:true,retained_native_plan_exact:true});await save();
 }
 const draft=session(freshNativeStateInput(rows.find(row=>row.name==='tract')));draft.fields.arguments_json='[ unfinished draft';assert.deepEqual(parseProject(JSON.stringify(project(draft))).files.freshNativeStateSession,draft);
 for(const [name,edit]of [
  ['unknown-session-field',v=>v.context_source={}],['unknown-draft-field',v=>v.fields.operation='run-fresh-native-state'],
  ['unknown-source',v=>v.sources.executable={name:'run.js',base64:'e30='}],['invalid-source-base64',v=>v.sources.adjacency.base64='bad'],
  ['noncanonical-pad-bits',v=>v.sources.adjacency.base64='AB=='],['invalid-runtime-sha',v=>v.runtime_artifact={sha256:'bad',byte_length:1}],
  ['invalid-runtime-size',v=>v.runtime_artifact={sha256:'a'.repeat(64),byte_length:-1}],['invalid-retained-encoding',v=>v.retained_run={name:'saved.json',base64:'not base64'}],
  ['unknown-runtime-field',v=>v.runtime_artifact={sha256:'a'.repeat(64),byte_length:1,verified:true}],['oversized-draft',v=>v.fields.arguments_json='x'.repeat(256*1024+1)]
 ]){
  const value=structuredClone(draft);edit(value);assert.throws(()=>project(value),undefined,name);evidence.refusals.push(name);
 }
 // The envelope may retain hostile bytes only as inert source data; the
 // explicit saved-result viewer applies its own safe JSON parser later.
 const inert=structuredClone(draft);inert.retained_run={name:'saved.json',base64:Buffer.from('<script>globalThis.executed=true</script>').toString('base64')};assert.equal(parseProject(JSON.stringify(project(inert))).files.freshNativeStateSession.retained_run.base64,inert.retained_run.base64);assert.equal(globalThis.executed,undefined);
 evidence.invalid_argument_draft_preserved=true;evidence.hostile_retained_bytes_inert=true;evidence.status='passed-twelve-fresh-project-and-archive-roundtrips';await save();
}catch(error){evidence.status='failed';evidence.error={message:error.message,stack:error.stack};await save();throw error;}
console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,refusals:evidence.refusals.length}));

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {readElectionSource,validateImportSettings,validatePackageArchive,decodePackageFiles} from '../../web/lab/package-files.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/election-import-fixtures';await fs.mkdir(root,{recursive:true});
const metadata={country:'US',state:'RI',county:'Synthetic County',date:'2024-11-05',election_type:'general',scope:'county',status:'canvassed'};
const header='contest_id,contest_title,vote_for,selection_id,selection_label,selection_kind,reporting_unit_id,reporting_unit_kind,parent_jurisdiction,status,votes,undervotes,overvotes,blank_contests,counted_ballots';
function csv(count){return header+'\r\n'+`mayor,Synthetic Mayor,1,a,Candidate A,candidate,p1,precinct,county,canvassed,${count},0,0,0,${count}\r\n`+`mayor,Synthetic Mayor,1,a,Candidate A,candidate,total,jurisdiction-total,county,canvassed,${count},0,0,0,${count}\r\n`;}
function nist(count){return `{"ElectionReport":{"ResultsStatus":"canvassed","GpUnit":[{"@id":"p1","Type":"precinct"},{"@id":"total","Type":"county"}],"Election":[{"Contest":[{"@id":"mayor","Name":{"Text":[{"Value":"Synthetic Mayor"}]},"NumberElected":1,"ContestSelection":[{"@id":"a","Name":{"Text":[{"Value":"Candidate A"}]},"VoteCounts":[{"GpUnitId":"p1","Count":${count}},{"GpUnitId":"total","Count":${count}}]}],"OtherCounts":[{"GpUnitId":"p1","Undervotes":0,"Overvotes":0,"BlankVotes":0},{"GpUnitId":"total","Undervotes":0,"Overvotes":0,"BlankVotes":0}]}]}]}}\n`;}
let sequence=0;
async function native(request){const file=path.join(root,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));return spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});}
let passed=0;
for(const [adapter,operation,sourcePath,make,extension] of [
  ['statement-csv','import-statement-csv','sources/statement-of-votes.csv',csv,'csv'],
  ['nist-cdf-json','import-nist-cdf-json','sources/nist-cdf-results.json',nist,'json'],
])for(const count of ['80','9007199254740993']){
  const text=make(count),bytes=Buffer.from(text),source_base64=bytes.toString('base64');
  const file=Object.assign(new Blob([bytes]),{name:`${adapter}-${count}.${extension}`});
  const sourceFile=await readElectionSource(file);assert.equal(sourceFile.base64,source_base64);
  await fs.writeFile(path.join(root,file.name),bytes);
  const request={operation,source_base64,metadata},reference=await native(request);
  assert.equal(reference.status,0,reference.stderr);
  const result=engine.execute(request);assert.deepEqual(result,JSON.parse(reference.stdout));
  assert.equal(result.adapter,adapter);assert.equal(result.verification.status,'pass');
  assert.equal(result.package_files[sourcePath],source_base64);
  assert.ok(Object.values(result.package_files).some(value=>Buffer.from(value,'base64').toString().includes(count)));
  assert.equal(result.manifest.jurisdiction.county,metadata.county);
  const packageFiles=validatePackageArchive(JSON.parse(JSON.stringify({schema_version:'bisect-rcount-files-v1',files:result.package_files})));
  assert.equal(engine.execute({operation:'verify-count-files',files:decodePackageFiles(packageFiles)}).status,'pass');
  const project=createProject({name:`${adapter} exact counts`,files:{sourceFile,importSettings:metadata,packageFiles},operation, constraints:[],result,lastOperation:operation});
  const reopened=parseProject(JSON.stringify(project));assert.deepEqual(reopened,project);
  assert.equal(reopened.files.sourceFile.base64,source_base64);
  assert.equal(engine.execute({operation:'verify-count-files',files:decodePackageFiles(reopened.files.packageFiles)}).status,'pass');
  const tampered=structuredClone(packageFiles);const altered=Buffer.from(tampered[sourcePath],'base64');altered[0]^=1;tampered[sourcePath]=altered.toString('base64');
  assert.equal(engine.execute({operation:'verify-count-files',files:decodePackageFiles(tampered)}).status,'fail');
  await fs.writeFile(path.join(root,`${adapter}-${count}.bisect`),JSON.stringify(project));passed++;
}
for(const request of [
  {operation:'import-statement-csv',source_base64:'',metadata},
  {operation:'import-nist-cdf-json',source_base64:'not base64',metadata},
  {operation:'import-statement-csv',source_base64:Buffer.from(csv('80')).toString('base64'),metadata:{...metadata,county:''}},
  {operation:'import-nist-cdf-json',source_base64:Buffer.from('{broken').toString('base64'),metadata},
  {operation:'import-statement-csv',source_base64:Buffer.from('wrong,headers\n1,2').toString('base64'),metadata},
]){assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);}
const draft=Object.fromEntries(Object.keys(metadata).map(key=>[key,'']));
assert.throws(()=>validateImportSettings(draft));validateImportSettings(draft,{requireComplete:false});
assert.deepEqual(parseProject(JSON.stringify(createProject({name:'Metadata draft',files:{importSettings:draft},operation:'import-statement-csv',constraints:[]}))).files.importSettings,draft);
for(const value of [{...metadata,unexpected:'x'},{...metadata,county:'é'.repeat(101)}])assert.throws(()=>validateImportSettings(value));
await assert.rejects(readElectionSource({size:1,name:'x.csv',arrayBuffer:async()=>new ArrayBuffer(8*1024*1024+1)}),/8 MiB/);
console.log(`Election import: ${passed} native/WASM exact-result comparisons, both source adapters, counts beyond 2^53, exact source bytes, metadata/project/archive roundtrip, package verification and source tamper rejection passed; five invalid requests rejected by both engines.`);

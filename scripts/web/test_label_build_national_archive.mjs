import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createBuildArchive,parseBuildArchive} from '../../web/lab/label-build-archive.js';
const root='target/label-build-national';
execFileSync('python',['-c',"import json,zipfile;from pathlib import Path;p=Path('target/label-build-national');z=zipfile.ZipFile(p/'completed.zip');(p/'archive-files.json').write_text(json.dumps({n:z.read(n).decode('utf-8') for n in z.namelist()},ensure_ascii=False,separators=(',',':')),encoding='utf-8')"]);
const project=JSON.parse(await fs.readFile(root+'/inputs.bisect','utf8'));
const files=JSON.parse(await fs.readFile(root+'/archive-files.json','utf8'));
const evidence=JSON.parse(await fs.readFile(root+'/evidence.json','utf8'));
// Exact genuine generated file bytes, with intentionally minimal imported display metadata.
// This fixture does not reconstruct the engine's full per-state metrics/progress transcript.
const result={schema_version:'bisect-label-build-v1',status:'built',label:'national_capacity',states:evidence.states.map(s=>({state:s.state,year:'2020',status:s.status,build_report:{attempts:[{seed:s.accepted_seed}]}})),years:[{year:'2020',status:'built',summary:evidence.summary}],files,invalidated_downstream:[],output_hashes:[],scope:'Capacity fixture retaining exact genuine national generated files, with minimal unverified display metadata.'};
const text=createBuildArchive({name:'Fifty-state archive capacity',inputs:project.files.labelBuildInputs,result,lastOperation:'run-label-build'});
assert.ok(Buffer.byteLength(text)>25*1024*1024);const opened=parseBuildArchive(text);
assert.deepEqual(opened.files,project.files);assert.equal(Object.keys(opened.result.files).length,153);
for(const [path,bytes]of Object.entries(files))assert.equal(opened.result.files[path],bytes);
assert.deepEqual(opened.result.years[0].summary,{total:50,succeeded:50,failed:0});
await fs.writeFile(root+'/full-build.bisect-build',text);await fs.writeFile(root+'/archive-evidence.json',JSON.stringify({archive_bytes:Buffer.byteLength(text),exact_sources:true,exact_generated_files:153,inert_codec:true,metadata_scope:'Minimal unverified display rows, not full engine metrics/transcript.',ordinary_project_limit_exceeded:true},null,2));
console.log('National build archive: '+Buffer.byteLength(text)+' bytes; all exact source bytes and 153 genuine generated files roundtrip above the ordinary project limit.');

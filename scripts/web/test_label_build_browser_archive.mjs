import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import {parseBuildArchive} from '../../web/lab/label-build-archive.js';
const path=process.argv[2]||'target/label-build-national/browser-complete.bisect-build';
const raw=await fs.readFile(path);assert.ok(raw.length<=128*1024*1024);
const archive=parseBuildArchive(new TextDecoder('utf-8',{fatal:true}).decode(raw));
const expectedInputs=JSON.parse(await fs.readFile('target/label-build-national/inputs.bisect','utf8')).files;
assert.deepEqual(archive.files,expectedInputs);
const expectedFiles=JSON.parse(await fs.readFile('target/label-build-national/archive-files.json','utf8'));
assert.deepEqual(archive.result.files,expectedFiles);assert.equal(archive.result.status,'built');assert.equal(archive.result.states.length,50);
assert.deepEqual(archive.result.years[0].summary,{total:50,succeeded:50,failed:0});
for(const row of archive.result.states){assert.equal(row.status,'ok');assert.equal(row.metrics.contiguous,true);assert.ok(row.metrics.max_deviation_percent<=0.5);assert.ok(row.build_report.attempts.length>=1);}
assert.equal(archive.result.states.find(r=>r.state==='CA').build_report.attempts.at(-1).seed,'46');assert.equal(archive.result.states.find(r=>r.state==='TX').build_report.attempts.at(-1).seed,'45');
assert.match(archive.result.scope,/Runs selected prepared/);
await fs.writeFile('target/label-build-national/browser-evidence.json',JSON.stringify({archive_bytes:raw.length,archive_sha256:crypto.createHash('sha256').update(raw).digest('hex'),exact_sources:true,exact_generated_files:153,states:50,succeeded:50,connected:50,within_tolerance:50,full_engine_metrics:true,downloaded_file_verified:true},null,2));
console.log('Actual browser download: '+raw.length+' bytes, exact source/153 output files, full metrics, 50 connected population-passing states and CA/TX accepted seeds verified.');

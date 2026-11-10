// Integration fixtures are explicitly produced by the registered gate runner.
// A caller may also select a retained, hash-bound foundation for local replay.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
export async function nativeReportReference(profile='default'){
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 let evidencePath=process.env.BISECT_REPORT_SOURCE_EVIDENCE;
 if(!evidencePath){
  const foundation=JSON.parse(await fs.readFile('target/state-report-sources/evidence.json','utf8'));
  const prior=foundation.profiles[profile].source_preparation;
  assert.equal(sha(await fs.readFile(prior.path)),prior.sha256,'Retained source-preparation evidence changed');evidencePath=prior.path;
 }
 const preparationBytes=await fs.readFile(evidencePath),preparation=JSON.parse(preparationBytes);
 assert.equal(preparation.status,'passed-source-snapshot-preparation');
 const reference=preparation.native_reference,bytes=await fs.readFile(reference.path);
 assert.equal(sha(bytes),reference.sha256,'Native report reference changed');
 const native=JSON.parse(bytes);assert.equal(sha(await fs.readFile(native.native.path)),native.native.sha256,'Native report executable changed');
 for(const row of native.outcomes)for(const ref of row.files)assert.equal(sha(await fs.readFile(ref.path)),ref.sha256,ref.path);
 const relative=path.relative(process.cwd(),path.resolve(evidencePath)).replaceAll('\\','/');assert(!relative.startsWith('../')&&!path.isAbsolute(relative),'Evidence must remain within the workspace');
 return {evidencePath,sourceRoot:path.dirname(evidencePath),reference,native,sourcePreparation:{path:relative,sha256:sha(preparationBytes),byte_length:preparationBytes.length}};
}

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {parseProject} from '../../web/lab/project.js';
import {decodePackageFiles,validateAggregationSettings} from '../../web/lab/package-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const [root,projectPath,transcriptPath]=process.argv.slice(2);
if(!root||!projectPath||!transcriptPath)throw new Error('Usage: verify_aggregation_project.mjs <preview> <downloaded-project> <downloaded-transcript>');
const project=parseProject(await fs.readFile(projectPath,'utf8'));
assert.equal(project.operation,'aggregate-districts');
assert.equal(project.lastOperation,project.operation);
const module=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
const engine=await instantiateEngine(module);
const request={operation:project.operation,
  ...validateAggregationSettings(project.files.aggregationSettings),
  document:project.files.plan,context:project.files.context??null,
  files:decodePackageFiles(project.files.packageFiles),
  crosswalk_base64:project.files.crosswalkFile?.base64??null};
const rerun=engine.execute(request);
assert.deepEqual(project.result,rerun);
// Compare raw bytes: JSON.parse would round native transcript integer counts.
const transcript=await fs.readFile(transcriptPath);
assert.deepEqual(transcript,Buffer.from(rerun.transcript_base64,'base64'));
console.log(JSON.stringify({project:projectPath,transcript:transcriptPath,
  wasm_sha256:createHash('sha256').update(module).digest('hex'),
  transcript_sha256:createHash('sha256').update(transcript).digest('hex'),
  status:rerun.status,output_format:rerun.output_format,
  districts:rerun.district_totals.map(d=>({district:d.district_label,counted_ballots:d.counted_ballots})),
  exact_rerun_match:true,exact_transcript_bytes:true},null,2));

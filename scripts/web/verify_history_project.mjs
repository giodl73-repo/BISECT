import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {parseProject,parseSafeJson} from '../../web/lab/project.js';
import {decodePackageFiles} from '../../web/lab/package-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const [root,projectPath,transcriptPath]=process.argv.slice(2);
if(!root||!projectPath||!transcriptPath)throw new Error('Usage: verify_history_project.mjs <preview> <downloaded-project> <downloaded-transcript>');
const project=parseProject(await fs.readFile(projectPath,'utf8'));
assert.equal(project.operation,'verify-history-files');assert.equal(project.lastOperation,project.operation);
const module=await fs.readFile(path.join(root,'bisect_wasm.wasm')),engine=await instantiateEngine(module);
const rerun=engine.execute({operation:project.operation,files:decodePackageFiles(project.files.packageFiles)});
assert.deepEqual(rerun,project.result);assert.deepEqual(rerun,parseSafeJson(await fs.readFile(transcriptPath,'utf8')));
console.log(JSON.stringify({project:projectPath,transcript:transcriptPath,status:rerun.status,
  package_id:rerun.package_id,package_content_hash:rerun.package_content_hash,checks:rerun.checks,
  wasm_sha256:createHash('sha256').update(module).digest('hex'),exact_rerun_match:true,exact_transcript_match:true},null,2));

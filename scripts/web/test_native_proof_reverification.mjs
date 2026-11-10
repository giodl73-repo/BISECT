import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifySavedNativeProofRun} from '../../web/lab/native-proof-pipeline.js';
import {sha256} from '../../web/lab/proof-solver-core.js';
const root = 'target/native-proof-reverification'; await fs.mkdir(root, {recursive: true});
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base = 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';
const options = {engine, checkerWasm: await fs.readFile(base + 'bisect_proof_checker.wasm'), checkerReceipt: JSON.parse(await fs.readFile(base + 'proof-checker-build.json', 'utf8'))};
const evidence = [];
for (const key of ['two-optimal', 'three-optimal']) {
  const saved = JSON.parse(await fs.readFile('target/native-proof-pipeline-tests/' + key + '.json', 'utf8'));
  const progress = [], result = await verifySavedNativeProofRun({...options, saved, onProgress: item => progress.push(item)});
  assert.equal(result.status, 'all-native-decisions-rechecked'); assert.equal(result.decisions.length, saved.decisions.length);
  assert.equal(progress.length, saved.decisions.length); assert.deepEqual(result.decisions, saved.decisions);
  await fs.writeFile(root + '/' + key + '.json', JSON.stringify(result, null, 2)); evidence.push({key, decisions: result.decisions.length, all_proofs_rechecked: true});
}
const original = JSON.parse(await fs.readFile('target/native-proof-pipeline-tests/two-optimal.json', 'utf8'));
async function resign(saved) {
  for (const [name, text] of Object.entries(saved.files)) if (name !== 'checked-native-proof-package.json') saved.file_sha256[name] = await sha256(Buffer.from(text));
  const {files, ...report} = saved; files['checked-native-proof-package.json'] = JSON.stringify(report, null, 2) + '\n';
  return saved;
}
// Recomputed public hashes do not make an invalid proof authoritative.
const invalidProof = structuredClone(original);
invalidProof.files['01-population.pbp'] = invalidProof.files['01-population.pbp'].replace(/conclusion UNSAT : \d+/, 'conclusion UNSAT : 999999999');
invalidProof.decisions[0].proof_sha256 = await sha256(Buffer.from(invalidProof.files['01-population.pbp']));
await assert.rejects(async () => verifySavedNativeProofRun({...options, saved: await resign(invalidProof)}));
const invalidBinding = structuredClone(original); invalidBinding.decisions[0].request_id = 'sha256:' + '0'.repeat(64);
await assert.rejects(async () => verifySavedNativeProofRun({...options, saved: await resign(invalidBinding)}), /binding/);
const alteredModel = structuredClone(original); alteredModel.files['01-population.opb'] += ' ';
await assert.rejects(async () => verifySavedNativeProofRun({...options, saved: await resign(alteredModel)}), /differ/);
const alteredRequest = structuredClone(original); alteredRequest.files['01-population.request.json'] = alteredRequest.files['02-boundary.request.json'];
await assert.rejects(async () => verifySavedNativeProofRun({...options, saved: await resign(alteredRequest)}), /differ/);
const incomplete = JSON.parse(await fs.readFile('target/native-proof-pipeline-tests/three-discovery-counterexample.json', 'utf8'));
await assert.rejects(() => verifySavedNativeProofRun({...options, saved: incomplete}), /Complete/);
await fs.writeFile(root + '/evidence.json', JSON.stringify({cases: evidence, rehashed_invalid_proof_refused: true, forged_native_identity_refused: true, rehashed_model_request_refused: true, incomplete_counterexample_refused: true, native_recursive_certificate_adoption: false}, null, 2));
console.log('Saved native proof packages: eight actual proof rechecks, forged hashes/identities/models/requests and incomplete packages refused.');

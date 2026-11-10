import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {checkProof} from '../../web/lab/proof-checker-core.js';
import {sha256} from '../../web/lab/proof-solver-core.js';
const root = path.resolve('target/proof-checker-tests'); await fs.mkdir(root, {recursive: true});
const build = 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';
const receipt = JSON.parse(await fs.readFile(build + 'proof-checker-build.json', 'utf8'));
const wasmBinary = new Uint8Array(await fs.readFile(build + 'bisect_proof_checker.wasm'));
assert.deepEqual(WebAssembly.Module.imports(new WebAssembly.Module(wasmBinary)), []);
const native = path.resolve('target/proof-checker-build/release/bisect-proof-checker.exe');
const upstream = process.env.BISECT_VERIPB; assert.ok(upstream, 'Set BISECT_VERIPB to the independent unmodified upstream executable.');
const makeRequest = async (opb, proof, requireUnsat = true) => ({opb, proof, opb_sha256: await sha256(Buffer.from(opb)), proof_sha256: await sha256(Buffer.from(proof)), require_unsat: requireUnsat});
const nativeCheck = request => {
  const run = spawnSync(native, [], {input: JSON.stringify(request), encoding: 'utf8', timeout: 60000, maxBuffer: 2 * 1024 * 1024});
  assert.equal(run.status, 0, run.stderr); return JSON.parse(run.stdout);
};
const evidence = [], cases = ['smoke-population', 'smoke-compact-boundary', 'fresh-01-population', 'fresh-02-boundary', 'fresh-03-canonical', 'sat'];
for (const name of cases) {
  const opb = await fs.readFile(`target/roundingsat-wasm-tests/${name}.opb`, 'utf8'), proof = await fs.readFile(`target/roundingsat-wasm-tests/${name}.pbp`, 'utf8');
  const request = await makeRequest(opb, proof, name !== 'sat'), expected = nativeCheck(request); assert.equal(expected.ok, true);
  const result = await checkProof({opb, proof, requireUnsat: request.require_unsat, wasmBinary, receipt});
  const {checker_wasm_sha256, ...report} = result; assert.deepEqual(report, expected.result); assert.equal(checker_wasm_sha256, receipt.wasm_sha256);
  const independent = spawnSync(upstream, [`target/roundingsat-wasm-tests/${name}.opb`, `target/roundingsat-wasm-tests/${name}.pbp`], {encoding: 'utf8', timeout: 60000});
  assert.equal(independent.status, 0, independent.stderr);
  assert.match(independent.stdout, new RegExp('VERIFIED ' + (name === 'sat' ? 'SATISFIABLE' : 'UNSATISFIABLE')));
  await fs.writeFile(path.join(root, name + '.json'), JSON.stringify(result, null, 2));
  const truncated = proof.replace(/conclusion[^\n]*\nend pseudo-Boolean proof\s*$/, '');
  assert.equal(nativeCheck(await makeRequest(opb, truncated, false)).ok, false);
  await assert.rejects(() => checkProof({opb, proof: truncated, requireUnsat: false, wasmBinary, receipt}));
  evidence.push({name, conclusion: result.conclusion, whole_native_match: true, independent_upstream_acceptance: true, truncated_proof_refused: true});
}
const opb = await fs.readFile('target/roundingsat-wasm-tests/smoke-population.opb', 'utf8'), proof = await fs.readFile('target/roundingsat-wasm-tests/smoke-population.pbp', 'utf8');
const satOpb = await fs.readFile('target/roundingsat-wasm-tests/sat.opb', 'utf8'), satProof = await fs.readFile('target/roundingsat-wasm-tests/sat.pbp', 'utf8');
const negative = [
  {name: 'mismatched-formula', opb: satOpb, proof},
  {name: 'invalid-reference', opb, proof: proof.replace(/conclusion UNSAT : \d+/, 'conclusion UNSAT : 999999999')},
  {name: 'missing-contradiction', opb, proof: 'pseudo-Boolean proof version 2.0\nf 281\noutput NONE\nconclusion UNSAT : 1\nend pseudo-Boolean proof\n'},
  {name: 'trailing-junk', opb, proof: proof + 'nonsense after end\n'},
  {name: 'external-output', opb, proof: proof.replace('output NONE', 'output EQUIOPTIMAL FILE'), guarded: true},
  {name: 'timer-rule', opb, proof: proof.replace('output NONE', 'start_time hostile\noutput NONE'), guarded: true},
  {name: 'unchecked-assertion', opb, proof: proof.replace('output NONE', 'a >= 0 ;\noutput NONE')},
  {name: 'sat-is-not-unsat', opb: satOpb, proof: satProof},
];
for (const item of negative) {
  const refused = nativeCheck(await makeRequest(item.opb, item.proof));
  assert.equal(refused.ok, false, item.name);
  if (item.guarded) assert.match(refused.error, /External output and timer rules/);
  await assert.rejects(() => checkProof({...item, wasmBinary, receipt}), item.guarded ? /External output and timer rules/ : undefined, item.name);
}
// A valid formula above the former 3 MiB boundary must still receive a real proof check.
const largeOpb=opb+'\n* '+ 'padding '.repeat(524288)+'\n',largeExpected=nativeCheck(await makeRequest(largeOpb,proof));assert.equal(largeExpected.ok,true,largeExpected.error);
const largeChecked=await checkProof({opb:largeOpb,proof,wasmBinary,receipt});const {checker_wasm_sha256:largeArtifact,...largeReport}=largeChecked;assert.deepEqual(largeReport,largeExpected.result);
const largeModelPath=path.join(root,'valid-over-three-mib.opb'),largeProofPath=path.join(root,'valid-over-three-mib.pbp');await fs.writeFile(largeModelPath,largeOpb);await fs.writeFile(largeProofPath,proof);const largeUpstream=spawnSync(upstream,[largeModelPath,largeProofPath],{encoding:'utf8',timeout:60000});assert.equal(largeUpstream.status,0,largeUpstream.stderr);evidence.push({name:'valid-formula-over-former-three-mib-limit',model_bytes:Buffer.byteLength(largeOpb),whole_native_match:true,independent_upstream_acceptance:true});
const valid = await makeRequest(opb, proof);
assert.equal(nativeCheck({...valid, opb_sha256: '0'.repeat(64)}).ok, false);
assert.equal(nativeCheck({...valid, unexpected: 'injected'}).ok, false);
await assert.rejects(() => checkProof({opb, proof, wasmBinary: new Uint8Array([0]), receipt}), /identity/);
await assert.rejects(() => checkProof({opb, proof, wasmBinary, receipt: {...receipt, vendor_provenance_sha256: '0'.repeat(64)}}), /binding/);
await assert.rejects(() => checkProof({opb: 'x'.repeat(8 * 1024 * 1024 + 1), proof, wasmBinary, receipt}), /limits/);
// Optional unmodified upstream v3 conformance cases exercise the second parser.
const version3 = ['conclusion_UNSAT_hint', 'conclusion_UNSAT_no_hint', 'miniProof_polishnotation', 'miniProof_polishnotation_2'];
for (const name of version3) {
  const base = `target/proof-toolchain/veripb-current/tests/instances/correct/version3/${name}`;
  const opb = await fs.readFile(base + '.opb', 'utf8'), proof = await fs.readFile(base + '.pbp', 'utf8');
  const expected = nativeCheck(await makeRequest(opb, proof, false)); assert.equal(expected.ok, true, name);
  const result = await checkProof({opb, proof, requireUnsat: false, wasmBinary, receipt});
  const {checker_wasm_sha256, ...report} = result; assert.deepEqual(report, expected.result);
  assert.equal(result.proof_version, '3.0');
  const checked = spawnSync(upstream, [base + '.opb', base + '.pbp'], {encoding: 'utf8', timeout: 60000}); assert.equal(checked.status, 0, checked.stderr);
  evidence.push({name: 'upstream-v3-' + name, conclusion: result.conclusion, whole_native_match: true, independent_upstream_acceptance: true});
}
await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({build: receipt, native_adapter_sha256: await sha256(await fs.readFile(native)), upstream_native_sha256: await sha256(await fs.readFile(upstream)), cases: evidence, paired_refusals: negative.map(item => item.name), imports: [], native_request_binding: false, full_plan_certification: false}, null, 2));
console.log(`VeriPB WASM: ${evidence.length} whole native matches with independent upstream acceptance, six truncated proofs and ${negative.length} paired negative cases refused; identity/input refusals passed.`);

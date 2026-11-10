import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync, spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {sha256} from '../../web/lab/proof-solver-core.js';
import {checkProof} from '../../web/lab/proof-checker-core.js';
import {runNativeProofPipeline, nativeProofRunZip, compileNativeProofModels, bindNativeDecision} from '../../web/lab/native-proof-pipeline.js';
const root = path.resolve('target/native-proof-pipeline-tests'); await fs.mkdir(root, {recursive: true});
const engineBytes = await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'), engine = await instantiateEngine(engineBytes);
const nativeHost = path.resolve('target/release/examples/execute_tool_request.exe'), upstream = process.env.BISECT_VERIPB; assert.ok(upstream, 'Set BISECT_VERIPB.');
const solverBase = 'target/roundingsat-wasm/', checkerBase = 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';
const {default: solverFactory} = await import(pathToFileURL(path.resolve(solverBase + 'roundingsat.mjs')).href);
const options = {engine, solverFactory, solverWasm: await fs.readFile(solverBase + 'roundingsat.wasm'), solverReceipt: JSON.parse(await fs.readFile(solverBase + 'build-receipt.json', 'utf8')),
  checkerWasm: await fs.readFile(checkerBase + 'bisect_proof_checker.wasm'), checkerReceipt: JSON.parse(await fs.readFile(checkerBase + 'proof-checker-build.json', 'utf8')), seconds: 3};
const source = async key => ({instanceJson: await fs.readFile('target/proof-models-wasm/' + key + '-instance.json', 'utf8'), discoveryJson: await fs.readFile('target/proof-models-wasm/' + key + '-discovery.json', 'utf8')});
const equal = await source('2-full-42'), unequal = await source('3-nrs-v0-3-42');
// Build a five-decision fixture from the actual native bounded exhaustive
// oracle, retaining the original instance. This oracle is used only in tests.
const solveRequest = {operation: 'solve-certified-split', instance_json: unequal.instanceJson};
const nativeRequestPath = path.join(root, 'native-request.json'); await fs.writeFile(nativeRequestPath, JSON.stringify(solveRequest));
const exact = JSON.parse(execFileSync(nativeHost, [nativeRequestPath], {encoding: 'utf8', maxBuffer: 8 * 1024 * 1024}));
assert.deepEqual(engine.execute(solveRequest), exact);
const discovery = JSON.parse(unequal.discoveryJson), artifacts = JSON.parse(exact.artifacts_json);
discovery.objective = artifacts.certificate.result.objective; discovery.solver_name = 'native bounded exhaustive oracle (test fixture)'; discovery.solver_version = null; discovery.method = 'fixture-from-native-solve-certified-split';
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const projection = {...discovery}; delete projection.discovery_id;
discovery.discovery_id = 'sha256:' + await sha256(Buffer.from(JSON.stringify(canonical(projection))));
const optimalUnequal = {...unequal, discoveryJson: JSON.stringify(discovery, null, 2)};
const evidence = [];
for (const [key, inputs, expectedStatus, expectedCount] of [['two-optimal', equal, 'all-native-decisions-checked', 3], ['three-optimal', optimalUnequal, 'all-native-decisions-checked', 5], ['three-discovery-counterexample', unequal, 'checked-counterexample', 5]]) {
  const phases = [], result = await runNativeProofPipeline({...options, ...inputs, onProgress: progress => phases.push(progress)});
  assert.equal(result.status, expectedStatus); assert.equal(result.required_decisions.length, expectedCount); assert.equal(result.decisions.length, expectedCount);
  await fs.writeFile(path.join(root, key + '-instance.json'), inputs.instanceJson); await fs.writeFile(path.join(root, key + '-discovery.json'), inputs.discoveryJson);
  await fs.writeFile(path.join(root, key + '.json'), JSON.stringify(result));
  // Compare every freshly compiled native OPB/request byte to the independent
  // native compiler host before checking the generated proof with upstream.
  const compilerRequest = {operation: 'compile-certified-proof-models', instance_json: inputs.instanceJson, discovery_json: inputs.discoveryJson};
  await fs.writeFile(nativeRequestPath, JSON.stringify(compilerRequest));
  const nativeCompiled = JSON.parse(execFileSync(nativeHost, [nativeRequestPath], {encoding: 'utf8', maxBuffer: 32 * 1024 * 1024}));
  for (const decision of result.decisions) {
    const stem = decision.stem;
    assert.equal(result.files[stem + '.opb'], nativeCompiled.files[stem + '.opb']); assert.equal(result.files[stem + '.request.json'], nativeCompiled.files[stem + '.request.json']);
    const native = JSON.parse(nativeCompiled.files[stem + '.request.json']); assert.equal(decision.request_id, native.request_id); assert.equal(decision.opb_sha256, native.opb_sha256.slice(7));
    const opb = path.join(root, key + '-' + stem + '.opb'), proof = path.join(root, key + '-' + stem + '.pbp'); await fs.writeFile(opb, result.files[stem + '.opb']); await fs.writeFile(proof, result.files[stem + '.pbp']);
    const verified = spawnSync(upstream, [opb, proof], {encoding: 'utf8', timeout: 60000}); assert.equal(verified.status, 0, verified.stderr);
    assert.match(verified.stdout, new RegExp('VERIFIED ' + (decision.proof_status === 'checked-sat' ? 'SATISFIABLE' : 'UNSATISFIABLE')));
    const checked = JSON.parse(result.files[stem + '.checked.json']); assert.deepEqual(checked.native_decision, await bindNativeDecision(nativeCompiled, stem, nativeCompiled.files[stem + '.opb'], nativeCompiled.files[stem + '.request.json'], engine.artifactIdentity));
  }
  if (expectedStatus === 'all-native-decisions-checked') {
    await fs.writeFile(path.join(root, key + '.zip'), await nativeProofRunZip(result));
    await assert.rejects(() => nativeProofRunZip({...result, files: {...result.files, '01-population.pbp': result.files['01-population.pbp'] + ' '}}), /changed/);
    await assert.rejects(() => nativeProofRunZip({...result, required_decisions: result.required_decisions.slice(1)}));
  } else {
    assert.equal(result.decisions.at(-1).stem, '03-canonical'); assert.equal(result.decisions.at(-1).proof_status, 'checked-sat');
    await assert.rejects(() => nativeProofRunZip(result), /Complete/);
  }
  assert.ok(phases.some(item => item.phase === 'checking'));
  evidence.push({key, status: result.status, decisions: result.decisions.map(item => ({stem: item.stem, status: item.proof_status})), native_files_match: true, native_ids_bound_before_proof_check: true, independent_upstream_checked: true});
}
const grid = await source('grid100'), timeoutStart = performance.now();
const timeout = await runNativeProofPipeline({...options, ...grid, seconds: 1});
const elapsed = performance.now() - timeoutStart;
assert.equal(timeout.status, 'proof-run-incomplete'); assert.equal(timeout.decisions.at(-1).solver_status, 'solver-reported-time-limit'); assert.equal(timeout.decisions.at(-1).proof_status, 'unverified');
assert.ok(elapsed >= 900 && elapsed < 15000); assert.equal(timeout.decisions[0].proof_status, 'checked-unsat');
await assert.rejects(() => nativeProofRunZip(timeout), /Complete/);
const stoppedStem = timeout.decisions.at(-1).stem;
await assert.rejects(() => checkProof({opb: timeout.files[stoppedStem + '.opb'], proof: timeout.files[stoppedStem + '.pbp'], requireUnsat: true, wasmBinary: options.checkerWasm, receipt: options.checkerReceipt}));
await fs.writeFile(path.join(root, 'grid100-timeout-instance.json'), grid.instanceJson); await fs.writeFile(path.join(root, 'grid100-timeout-discovery.json'), grid.discoveryJson); await fs.writeFile(path.join(root, 'grid100-timeout.json'), JSON.stringify(timeout));
evidence.push({key: 'grid100-timeout', status: timeout.status, unit_count: 100, elapsed_ms: elapsed, solver_time_limit_seconds: 1, completed_prefix_retained: true, incomplete_proof_cannot_certify_unsat: true, complete_package_export_refused: true});
// Emscripten mirrors its intentional timeout exit code onto the Node process.
process.exitCode = 0;
const compiled = await compileNativeProofModels(engine, equal.instanceJson, equal.discoveryJson), stem = '01-population';
await assert.rejects(() => bindNativeDecision(compiled, stem, compiled.files[stem + '.opb'] + ' ', compiled.files[stem + '.request.json'], engine.artifactIdentity), /differ/);
await assert.rejects(() => bindNativeDecision(compiled, stem, compiled.files[stem + '.opb'], compiled.files['02-boundary.request.json'], engine.artifactIdentity), /differ/);
const forged = JSON.parse(compiled.files[stem + '.request.json']); forged.opb_sha256 = 'sha256:' + '0'.repeat(64);
await assert.rejects(() => bindNativeDecision(compiled, stem, compiled.files[stem + '.opb'], JSON.stringify(forged, null, 2), engine.artifactIdentity), /differ/);
const missing = structuredClone(await compileNativeProofModels(engine, optimalUnequal.instanceJson, optimalUnequal.discoveryJson)); delete missing.artifacts[Object.keys(missing.artifacts).find(key => key.startsWith('02-low'))];
await assert.rejects(() => compileNativeProofModels({execute: () => missing}, optimalUnequal.instanceJson, optimalUnequal.discoveryJson), /inventory/);
let solverCalls = 0;
await assert.rejects(() => runNativeProofPipeline({...options, ...equal, discoveryJson: equal.discoveryJson.replace(/"discovery_id":\s*"[^"]+"/, '"discovery_id":"sha256:' + '0'.repeat(64) + '"'), solverFactory: config => { solverCalls++; return solverFactory(config); }})); assert.equal(solverCalls, 0);
await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({engine_wasm_sha256: await sha256(engineBytes), native_host_sha256: await sha256(await fs.readFile(nativeHost)), solver_wasm_sha256: options.solverReceipt.files['roundingsat.wasm'].sha256, checker_wasm_sha256: options.checkerReceipt.wasm_sha256, cases: evidence, changed_opb_request_refused: true, missing_branch_refused: true, invalid_sources_refused_before_solver: true, full_plan_certificate_adoption: false}, null, 2));
console.log('Native proof pipeline: three/five decisions complete, canonical SAT counterexample checked, actual 100-unit timeout retains only checked prefix; native file/identity binding, strict ZIP and source/request/branch refusals passed.');

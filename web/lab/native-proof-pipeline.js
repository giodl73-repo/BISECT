import {sha256, runRoundingSat, MAX_OPB_BYTES} from './proof-solver-core.js';
import {checkProof} from './proof-checker-core.js';
import {storedZipBytes} from './reproducibility-files.js';
const encoder = new TextEncoder();
const digest = text => sha256(encoder.encode(text));
const stemPattern = /^(01-population|02-boundary|03-canonical|02-(low|high)-boundary-right-\d+)$/;
const MAX_PACKAGE_BYTES = 128 * 1024 * 1024;
export function recordedDiscoverySeed(discoveryJson) {
  const method = JSON.parse(discoveryJson).method;
  if (typeof method !== 'string') return null;
  const matches = Array.from(method.matchAll(/(?:^|[; ])seed=(0|[1-9]\d{0,19})(?=;|$)/g));
  if (matches.length !== 1 || BigInt(matches[0][1]) > (1n << 64n) - 1n) return null;
  return matches[0][1];
}

export function validateNativeProofSources(instanceJson, discoveryJson, seconds) {
  for (const text of [instanceJson, discoveryJson]) if (typeof text !== 'string' || !text.trim() || text.includes('\0') || encoder.encode(text).length > 3 * 1024 * 1024) throw new Error('Select nonempty UTF-8 instance/discovery JSON, at most 3 MiB each.');
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 120) throw new Error('Each solver time limit must be 1–120 seconds.');
}

export async function compileNativeProofModels(engine, instanceJson, discoveryJson) {
  validateNativeProofSources(instanceJson, discoveryJson, 30);
  const compiled = engine.execute({operation: 'compile-certified-proof-models', instance_json: instanceJson, discovery_json: discoveryJson});
  if (compiled?.schema_version !== 'bisect-certified-proof-models-v1' || compiled.status !== 'models-compiled' || compiled.proof_status !== 'not-generated' || !compiled.artifacts || !compiled.files) throw new Error('Invalid native compiler response.');
  if (compiled.instance_source_sha256 !== await digest(instanceJson) || compiled.discovery_source_sha256 !== await digest(discoveryJson)) throw new Error('Native compiler source binding mismatch.');
  const stems = Object.keys(compiled.artifacts).sort(), names = ['manifest.json'];
  for (const stem of stems) {
    const artifact = compiled.artifacts[stem];
    if (!stemPattern.test(stem) || artifact.path !== stem + '.opb' || artifact.request_path !== stem + '.request.json') throw new Error('Invalid native model artifact inventory.');
    names.push(artifact.path, artifact.request_path);
  }
  if (![3, 5].includes(stems.length) || !['01-population', '02-boundary', '03-canonical'].every(stem => stems.includes(stem)) || Object.keys(compiled.files).sort().join('\n') !== names.sort().join('\n')) throw new Error('Incomplete native decision model inventory.');
  const discovery = JSON.parse(discoveryJson);
  const branching = BigInt(discovery.objective.primary.max_population_deviation_scaled) > 0n;
  if (stems.length !== (branching ? 5 : 3) || (branching && !['low', 'high'].every(side => stems.some(stem => stem.startsWith('02-' + side + '-boundary-right-'))))) throw new Error('Required native population branches are missing.');
  for (const name of names) if (typeof compiled.files[name] !== 'string' || await digest(compiled.files[name]) !== compiled.file_sha256?.[name]) throw new Error('Native compiled model file hash mismatch.');
  return compiled;
}

// Requests are accepted as the exact freshly compiled native file bytes. This
// closes unknown/duplicate JSON fields and binds command text without executing it.
export async function bindNativeDecision(compiled, stem, opb, requestText, engineIdentity) {
  const artifact = compiled.artifacts?.[stem];
  if (!stemPattern.test(stem) || !artifact || opb !== compiled.files?.[stem + '.opb'] || requestText !== compiled.files?.[stem + '.request.json']) throw new Error('Decision sources differ from the freshly compiled native model/request.');
  const request = JSON.parse(requestText), opbSha = await digest(opb), requestSha = await digest(requestText);
  if (request.schema_version !== 'certified-split-proof-request-v5' || request.instance_hash !== compiled.instance_hash || request.discovery_id !== compiled.discovery_id || request.opb_sha256 !== 'sha256:' + opbSha || artifact.sha256 !== opbSha || artifact.request_sha256 !== requestSha || request.status !== 'proof-required-unclassified' || request.proof_format !== 'veripb' || request.proof_status !== 'not-generated') throw new Error('Native request identity/hash mismatch.');
  if (!/^sha256:[a-f0-9]{64}$/.test(request.request_id) || !/^[a-f0-9]{64}$/.test(engineIdentity?.sha256)) throw new Error('Missing native request or engine artifact identity.');
  const exactPopulation = /"exact_right_population"\s*:\s*(null|-?\d+)/.exec(requestText)?.[1];
  if (exactPopulation === undefined) throw new Error('Missing exact population branch metadata.');
  return {schema: 'bisect-native-decision-binding-v1', stem, request_id: request.request_id,
    instance_hash: request.instance_hash, discovery_id: request.discovery_id, stage: request.stage,
    connectivity_encoding: request.connectivity_encoding, exact_right_population: exactPopulation === 'null' ? null : exactPopulation,
    opb_sha256: opbSha, request_source_sha256: requestSha,
    instance_source_sha256: compiled.instance_source_sha256, discovery_source_sha256: compiled.discovery_source_sha256,
    compiler_source_sha256: compiled.compiler_sha256, engine_wasm_sha256: engineIdentity.sha256};
}

export async function runNativeProofPipeline({engine, instanceJson, discoveryJson, seconds = 30, solverFactory, solverWasm, solverReceipt, checkerWasm, checkerReceipt, onProgress = () => {}}) {
  validateNativeProofSources(instanceJson, discoveryJson, seconds);
  const compiled = await compileNativeProofModels(engine, instanceJson, discoveryJson), stems = Object.keys(compiled.artifacts).sort();
  const discoverySeed = recordedDiscoverySeed(discoveryJson);
  const files = {'certified-split-instance.json': instanceJson, 'certified-discovery.json': discoveryJson, ...compiled.files};
  const decisions = [];
  const progress = (phase, stem = null) => onProgress({phase, stem, completed: decisions.filter(item => item.proof_status === 'checked-unsat').length, total: stems.length});
  const addFile = (name, text) => {
    files[name] = text;
    if (Object.values(files).reduce((sum, text) => sum + encoder.encode(text).length, 0) > MAX_PACKAGE_BYTES) throw new Error('Native proof evidence package exceeds 128 MiB.');
  };
  progress('compiled');
  for (const stem of stems) {
    const opb = compiled.files[stem + '.opb'], requestText = compiled.files[stem + '.request.json'];
    // Recompilation and native identity binding happen before proof generation
    // or verification, as required by the native proof contract.
    const binding = await bindNativeDecision(compiled, stem, opb, requestText, engine.artifactIdentity);
    progress('solving', stem);
    const generated = await runRoundingSat({opb, seconds, factory: solverFactory, wasmBinary: solverWasm, receipt: solverReceipt});
    if (generated.opb_sha256 !== binding.opb_sha256) throw new Error('Generated proof refers to a different native formula.');
    addFile(stem + '.pbp', generated.proof);
    const {proof, log, ...solverReport} = generated;
    addFile(stem + '.solver.json', JSON.stringify(solverReport, null, 2) + '\n');
    if (generated.status !== 'solver-reported-unsat') {
      let counterexample = null;
      if (generated.status === 'solver-reported-sat') {
        progress('checking-counterexample', stem);
        counterexample = await checkProof({opb, proof: generated.proof, requireUnsat: false, wasmBinary: checkerWasm, receipt: checkerReceipt});
        if (counterexample.conclusion !== 'SAT' || counterexample.opb_sha256 !== binding.opb_sha256 || counterexample.proof_sha256 !== generated.proof_sha256) throw new Error('Counterexample proof identity/conclusion mismatch.');
        addFile(stem + '.checked.json', JSON.stringify({...counterexample, native_decision: binding}, null, 2) + '\n');
      }
      decisions.push({...binding, proof_status: counterexample ? 'checked-sat' : 'unverified', solver_status: generated.status, proof_sha256: generated.proof_sha256,
        ...(counterexample ? {checker_wasm_sha256: counterexample.checker_wasm_sha256} : {})});
      progress('incomplete', stem);
      return {schema: 'bisect-native-proof-run-v1', status: counterexample ? 'checked-counterexample' : 'proof-run-incomplete',
        proof_status: 'incomplete', instance_hash: compiled.instance_hash, discovery_id: compiled.discovery_id, recorded_discovery_seed: discoverySeed,
        required_decisions: stems, decisions, files,
        scope: 'A native decision did not produce checked UNSAT. The incumbent remains unproved; complete decision-package export is unavailable.'};
    }
    progress('checking', stem);
    const checked = await checkProof({opb, proof: generated.proof, requireUnsat: true, wasmBinary: checkerWasm, receipt: checkerReceipt});
    if (checked.opb_sha256 !== binding.opb_sha256 || checked.proof_sha256 !== generated.proof_sha256 || checked.conclusion !== 'UNSAT') throw new Error('Checked proof identity/conclusion mismatch.');
    const decision = {...binding, proof_status: 'checked-unsat', proof_sha256: checked.proof_sha256, checker_wasm_sha256: checked.checker_wasm_sha256};
    decisions.push(decision);
    addFile(stem + '.checked.json', JSON.stringify({...checked, native_decision: binding}, null, 2) + '\n');
    progress('checked', stem);
  }
  const fileHashes = Object.fromEntries(await Promise.all(Object.entries(files).map(async ([name, text]) => [name, await digest(text)])));
  const report = {schema: 'bisect-native-proof-run-v1', status: 'all-native-decisions-checked', proof_status: 'checked-unsat',
    instance_hash: compiled.instance_hash, discovery_id: compiled.discovery_id, recorded_discovery_seed: discoverySeed, required_decisions: stems, decisions,
    instance_source_sha256: compiled.instance_source_sha256, discovery_source_sha256: compiled.discovery_source_sha256,
    compiler_source_sha256: compiled.compiler_sha256, engine_wasm_sha256: engine.artifactIdentity.sha256,
    solver_wasm_sha256: solverReceipt.files['roundingsat.wasm'].sha256, checker_wasm_sha256: checkerReceipt.wasm_sha256,
    file_sha256: fileHashes,
    scope: 'Every decision in the freshly compiled native compact package has a checked UNSAT proof for its exact native request and formula bytes. This is evidence for this split instance; recursive tree certificate adoption, global plan optimality, source truth and legal compliance are not established.'};
  addFile('checked-native-proof-package.json', JSON.stringify(report, null, 2) + '\n');
  progress('complete');
  return {...report, files};
}

async function validatedNativeProofFiles(result, collectEntries = false) {
  if (result?.schema !== 'bisect-native-proof-run-v1' || result.status !== 'all-native-decisions-checked' || result.proof_status !== 'checked-unsat') throw new Error('Complete freshly checked native decision package required.');
  if (!Array.isArray(result.required_decisions) || ![3, 5].includes(result.required_decisions.length) || !Array.isArray(result.decisions) || result.decisions.length !== result.required_decisions.length || result.decisions.some((item, index) => item.stem !== result.required_decisions[index] || item.proof_status !== 'checked-unsat')) throw new Error('Incomplete checked decision inventory.');
  const names = ['certified-split-instance.json', 'certified-discovery.json', 'manifest.json', 'checked-native-proof-package.json'];
  for (const stem of result.required_decisions) {
    if (!stemPattern.test(stem)) throw new Error('Invalid checked decision filename.');
    names.push(...['.opb', '.request.json', '.pbp', '.solver.json', '.checked.json'].map(suffix => stem + suffix));
  }
  if (Object.keys(result.files ?? {}).sort().join('\n') !== names.sort().join('\n')) throw new Error('Checked package file inventory mismatch.');
  const {files, ...report} = result;
  if (files['checked-native-proof-package.json'] !== JSON.stringify(report, null, 2) + '\n') throw new Error('Checked package report changed.');
  const entries = []; let total = 0;
  for (const name of names) {
    if (typeof files[name] !== 'string') throw new Error('Invalid package file bytes.');
    const bytes = encoder.encode(files[name]); total += bytes.length;
    if (total > MAX_PACKAGE_BYTES) throw new Error('Native proof evidence package exceeds 128 MiB.');
    if (name !== 'checked-native-proof-package.json' && await digest(files[name]) !== result.file_sha256?.[name]) throw new Error('Checked package file bytes changed.');
    if (collectEntries) entries.push([name, bytes]);
  }
  return entries;
}
export async function nativeProofRunZip(result) {
  return storedZipBytes(await validatedNativeProofFiles(result, true), {maxSourceBytes: MAX_PACKAGE_BYTES});
}

// Imported reports carry no live verification authority. Rebuild the native
// queries and check the actual proof bytes before returning a fresh receipt.
export async function verifySavedNativeProofRun({engine, saved, checkerWasm, checkerReceipt, onProgress = () => {}}) {
  saved = structuredClone(saved);
  // Integrity/inventory checks are only preflight; they never replace VeriPB.
  await validatedNativeProofFiles(saved);
  const instanceJson = saved.files['certified-split-instance.json'], discoveryJson = saved.files['certified-discovery.json'];
  const compiled = await compileNativeProofModels(engine, instanceJson, discoveryJson);
  const stems = Object.keys(compiled.artifacts).sort();
  if (JSON.stringify(stems) !== JSON.stringify(saved.required_decisions) || saved.instance_hash !== compiled.instance_hash || saved.discovery_id !== compiled.discovery_id || saved.compiler_source_sha256 !== compiled.compiler_sha256) throw new Error('Saved proof package differs from the freshly compiled native instance/requests.');
  const decisions = [];
  for (const stem of stems) {
    const binding = await bindNativeDecision(compiled, stem, saved.files[stem + '.opb'], saved.files[stem + '.request.json'], engine.artifactIdentity);
    const previous = saved.decisions.find(item => item.stem === stem);
    for (const field of ['request_id', 'instance_hash', 'discovery_id', 'stage', 'connectivity_encoding', 'exact_right_population', 'opb_sha256', 'request_source_sha256', 'instance_source_sha256', 'discovery_source_sha256', 'compiler_source_sha256']) {
      if (previous?.[field] !== binding[field]) throw new Error('Saved native decision binding changed: ' + field);
    }
    onProgress({phase: 'rechecking', stem, completed: decisions.length, total: stems.length});
    const checked = await checkProof({opb: saved.files[stem + '.opb'], proof: saved.files[stem + '.pbp'], requireUnsat: true, wasmBinary: checkerWasm, receipt: checkerReceipt});
    if (checked.proof_sha256 !== previous.proof_sha256) throw new Error('Saved proof identity changed.');
    decisions.push({...binding, proof_status: 'checked-unsat', proof_sha256: checked.proof_sha256, checker_wasm_sha256: checked.checker_wasm_sha256});
  }
  return {schema: 'bisect-native-proof-reverification-v1', status: 'all-native-decisions-rechecked',
    instance_hash: compiled.instance_hash, discovery_id: compiled.discovery_id, required_decisions: stems, decisions,
    input_manifest_sha256: await digest(saved.files['checked-native-proof-package.json']),
    engine_wasm_sha256: engine.artifactIdentity.sha256, checker_wasm_sha256: checkerReceipt.wasm_sha256,
    scope: 'Every saved proof was freshly checked against freshly compiled native requests. Imported receipts were not trusted. This does not adopt a native recursive tree certificate or establish global plan optimality.'};
}

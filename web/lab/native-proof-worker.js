import {instantiateEngine} from './wasm-engine.js';
import {runNativeProofPipeline, verifySavedNativeProofRun, validateNativeProofSources} from './native-proof-pipeline.js';
import {sha256} from './proof-solver-core.js';
import {adoptNativeExternalSplit} from './native-external-certificate.js';
let started = false;
self.onmessage = async ({data}) => {
  if (started) return;
  started = true;
  try {
    validateNativeProofSources(data?.instanceJson, data?.discoveryJson, data?.seconds);
    if (data.mode === 'issue-certificate') {
      const [metadata, artifact] = await Promise.all([fetch(new URL('./proof-checker/proof-checker-build.json', import.meta.url)), fetch(new URL('./proof-checker/bisect_proof_checker.wasm', import.meta.url))]);
      if (!metadata.ok || !artifact.ok) throw new Error('Native certificate checker assets are unavailable.');
      const receipt = await metadata.json(), wasmBinary = new Uint8Array(await artifact.arrayBuffer());
      const proofs = Object.fromEntries(data.saved.required_decisions.map(stem => [stem, data.saved.files[stem + '.pbp']]));
      const certificate = await adoptNativeExternalSplit({instanceJson: data.instanceJson, discoveryJson: data.discoveryJson, proofs, wasmBinary, receipt});
      self.postMessage({certificate}); return;
    }
    const paths = ['./bisect_wasm.wasm', './roundingsat/build-receipt.json', './roundingsat/roundingsat.wasm', './roundingsat/roundingsat.mjs', './proof-checker/proof-checker-build.json', './proof-checker/bisect_proof_checker.wasm'];
    const responses = await Promise.all(paths.map(path => fetch(new URL(path, import.meta.url))));
    if (!responses.every(response => response.ok)) throw new Error('Native proof pipeline assets have not been installed in this preview.');
    const [engineBytes, solverReceipt, solverWasm, solverGlue, checkerReceipt, checkerWasm] = await Promise.all(responses.map((response, index) => [1, 4].includes(index) ? response.json() : response.arrayBuffer().then(bytes => new Uint8Array(bytes))));
    if (await sha256(solverGlue) !== solverReceipt.files?.['roundingsat.mjs']?.sha256) throw new Error('Solver JavaScript checksum mismatch.');
    const engine = await instantiateEngine(engineBytes);
    const {default: solverFactory} = await import(new URL('./roundingsat/roundingsat.mjs', import.meta.url).href);
    const result = data.mode === 'verify-saved' ? await verifySavedNativeProofRun({engine, saved: data.saved, checkerWasm, checkerReceipt, onProgress: progress => self.postMessage({progress})}) : await runNativeProofPipeline({engine, instanceJson: data.instanceJson, discoveryJson: data.discoveryJson, seconds: data.seconds,
      solverFactory, solverReceipt, solverWasm, checkerReceipt, checkerWasm, onProgress: progress => self.postMessage({progress})});
    self.postMessage({result});
  } catch (error) { self.postMessage({error: String(error?.message ?? error)}); }
};

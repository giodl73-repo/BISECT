import {checkProof} from './proof-checker-core.js';
let started = false;
self.onmessage = async ({data}) => {
  if (started) return;
  started = true;
  try {
    const base = new URL('./proof-checker/', import.meta.url);
    const [metadata, artifact] = await Promise.all([fetch(new URL('proof-checker-build.json', base)), fetch(new URL('bisect_proof_checker.wasm', base))]);
    if (!metadata.ok || !artifact.ok) throw new Error('Proof checker assets have not been installed in this preview.');
    const receipt = await metadata.json(), wasmBinary = new Uint8Array(await artifact.arrayBuffer());
    self.postMessage({result: await checkProof({opb: data.opb, proof: data.proof, requireUnsat: data.requireUnsat, receipt, wasmBinary})});
  } catch (error) { self.postMessage({error: String(error?.message ?? error)}); }
};

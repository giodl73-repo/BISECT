import {runRoundingSat, validateSolverInput, sha256} from './proof-solver-core.js';
let started = false;
self.onmessage = async ({data}) => {
  if (started) return;
  started = true;
  try {
    validateSolverInput(data?.opb, data?.seconds);
    const base = new URL('./roundingsat/', import.meta.url);
    const [receiptResponse, wasmResponse, glueResponse] = await Promise.all([
      fetch(new URL('build-receipt.json', base)), fetch(new URL('roundingsat.wasm', base)), fetch(new URL('roundingsat.mjs', base))]);
    if (![receiptResponse, wasmResponse, glueResponse].every(response => response.ok)) throw new Error('The external solver assets have not been installed in this preview.');
    const receipt = await receiptResponse.json(), wasmBinary = new Uint8Array(await wasmResponse.arrayBuffer()), glue = new Uint8Array(await glueResponse.arrayBuffer());
    if (await sha256(glue) !== receipt.files?.['roundingsat.mjs']?.sha256) throw new Error('Solver JavaScript checksum mismatch.');
    const {default: factory} = await import(new URL('roundingsat.mjs', base).href);
    const result = await runRoundingSat({opb: data.opb, seconds: data.seconds, factory, wasmBinary, receipt});
    self.postMessage({result});
  } catch (error) { self.postMessage({error: String(error?.message ?? error)}); }
};

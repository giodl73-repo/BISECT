import {sha256, MAX_OPB_BYTES, MAX_PROOF_BYTES} from './proof-solver-core.js';
const encoder = new TextEncoder();
export async function checkProof({opb, proof, requireUnsat = true, wasmBinary, receipt}) {
  if (typeof opb !== 'string' || typeof proof !== 'string' || !opb.trim() || !proof.trim() || opb.includes('\0') || proof.includes('\0') || encoder.encode(opb).length > MAX_OPB_BYTES || encoder.encode(proof).length > MAX_PROOF_BYTES) throw new Error('Nonempty UTF-8 model/proof required; limits are 8 MiB and 64 MiB.');
  if (typeof requireUnsat !== 'boolean') throw new Error('An explicit boolean UNSAT requirement is needed.');
  if (receipt?.schema !== 'bisect-veripb-wasm-build-v1' || receipt.upstream_commit !== 'a50f0483d65aa019918d03ccc590b71dc23c666c' || await sha256(wasmBinary) !== receipt.wasm_sha256 || wasmBinary.byteLength !== receipt.wasm_bytes) throw new Error('Proof checker artifact identity mismatch.');
  const request = {opb, proof, opb_sha256: await sha256(encoder.encode(opb)), proof_sha256: await sha256(encoder.encode(proof)), require_unsat: requireUnsat};
  const bytes = encoder.encode(JSON.stringify(request));
  if (bytes.length > 192 * 1024 * 1024) throw new Error('Encoded checker request exceeds 192 MiB.');
  const {instance} = await WebAssembly.instantiate(wasmBinary, {}), api = instance.exports;
  const pointer = api.proof_alloc(bytes.length);
  if (!pointer) throw new Error('Proof checker refused its input allocation.');
  let response;
  try {
    new Uint8Array(api.memory.buffer, pointer, bytes.length).set(bytes);
    const output = api.proof_execute(pointer, bytes.length), length = api.proof_response_length();
    if (length > 1024 * 1024) throw new Error('Proof checker response exceeds 1 MiB.');
    response = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(new Uint8Array(api.memory.buffer, output, length)));
  } finally { api.proof_free(pointer, bytes.length); }
  if (!response.ok) throw new Error(response.error || 'Proof refused.');
  const result = response.result;
  if (result?.schema !== 'bisect-external-proof-check-v1' || result.status !== 'proof-checked' || result.opb_sha256 !== request.opb_sha256 || result.proof_sha256 !== request.proof_sha256 || result.vendor_provenance_sha256 !== receipt.vendor_provenance_sha256 || (requireUnsat && result.conclusion !== 'UNSAT')) throw new Error('Proof checker response binding mismatch.');
  return {...result, checker_wasm_sha256: receipt.wasm_sha256};
}

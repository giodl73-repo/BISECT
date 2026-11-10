import {sha256} from './proof-solver-core.js';
export async function adoptNativeExternalSplit({instanceJson, discoveryJson, proofs, expectedCertificate = null, wasmBinary, receipt}) {
  if (receipt?.schema !== 'bisect-veripb-wasm-build-v1' || receipt.upstream_commit !== 'a50f0483d65aa019918d03ccc590b71dc23c666c' || await sha256(wasmBinary) !== receipt.wasm_sha256 || wasmBinary.byteLength !== receipt.wasm_bytes) throw new Error('Native certificate checker identity mismatch.');
  const request = {operation: 'adopt-native-external-split', instance_json: instanceJson, discovery_json: discoveryJson, proofs, expected_certificate: expectedCertificate};
  const bytes = new TextEncoder().encode(JSON.stringify(request));
  if (bytes.length > 192 * 1024 * 1024) throw new Error('Native certificate request exceeds 192 MiB.');
  const {instance} = await WebAssembly.instantiate(wasmBinary, {}), api = instance.exports;
  const pointer = api.proof_alloc(bytes.length); if (!pointer) throw new Error('Native certificate allocation refused.');
  let response;
  try {
    new Uint8Array(api.memory.buffer, pointer, bytes.length).set(bytes);
    const output = api.proof_execute(pointer, bytes.length), length = api.proof_response_length();
    if (length > 3 * 1024 * 1024) throw new Error('Native certificate response exceeds 3 MiB.');
    response = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(new Uint8Array(api.memory.buffer, output, length)));
  } finally { api.proof_free(pointer, bytes.length); }
  if (!response.ok) throw new Error(response.error || 'Native certificate refused.');
  const certificate = response.result;
  if (certificate?.schema_version !== 'certified-recursive-bisection-external-split-certificate-v1' || certificate.proof_kind !== 'veripb-compact-decisions' || certificate.result?.result !== 'optimal' || certificate.instance_source_sha256 !== await sha256(new TextEncoder().encode(instanceJson)) || certificate.discovery_source_sha256 !== await sha256(new TextEncoder().encode(discoveryJson))) throw new Error('Native certificate response binding mismatch.');
  return certificate;
}

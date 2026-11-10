import {sha256} from './proof-solver-core.js';
export async function adoptNativeExternalTree({rootInstanceJson, nodes, expectedTree = null, wasmBinary, receipt}) {
  if (receipt?.schema !== 'bisect-veripb-wasm-build-v1' || receipt.upstream_commit !== 'a50f0483d65aa019918d03ccc590b71dc23c666c' || await sha256(wasmBinary) !== receipt.wasm_sha256 || wasmBinary.byteLength !== receipt.wasm_bytes) throw new Error('External tree checker artifact identity mismatch.');
  const bytes = new TextEncoder().encode(JSON.stringify({operation: 'adopt-native-external-tree', root_instance_json: rootInstanceJson, nodes, expected_tree: expectedTree}));
  if (bytes.length > 192 * 1024 * 1024) throw new Error('External tree request exceeds 192 MiB.');
  const {instance} = await WebAssembly.instantiate(wasmBinary, {}), api = instance.exports;
  const pointer = api.proof_alloc(bytes.length); if (!pointer) throw new Error('Tree input allocation refused.');
  let response;
  try {
    new Uint8Array(api.memory.buffer, pointer, bytes.length).set(bytes);
    const output = api.proof_execute(pointer, bytes.length), length = api.proof_response_length();
    if (length > 24 * 1024 * 1024) throw new Error('External tree response exceeds 24 MiB.');
    response = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(new Uint8Array(api.memory.buffer, output, length)));
  } finally { api.proof_free(pointer, bytes.length); }
  if (!response.ok) throw new Error(response.error || 'External tree refused.');
  if (response.result?.schema_version !== 'certified-recursive-bisection-external-tree-v1') throw new Error('Unexpected external tree response.');
  return response.result;
}

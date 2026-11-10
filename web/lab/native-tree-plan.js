import {sha256} from './proof-solver-core.js';
import {storedZipBytes} from './reproducibility-files.js';
export async function exportNativeTreePlan({treeRequest, contextJson, profileJson, constraints = ['plan-shape', 'population', 'contiguity'], label, createdAt, wasmBinary, receipt}) {
  if (receipt?.schema !== 'bisect-veripb-wasm-build-v1' || receipt.upstream_commit !== 'a50f0483d65aa019918d03ccc590b71dc23c666c' || await sha256(wasmBinary) !== receipt.wasm_sha256 || wasmBinary.byteLength !== receipt.wasm_bytes) throw new Error('Native plan verifier artifact identity mismatch.');
  const bytes = new TextEncoder().encode(JSON.stringify({operation: 'export-native-external-tree-plan', tree_request: treeRequest, context_json: contextJson, profile_json: profileJson, constraints, label, created_at: createdAt}));
  if (bytes.length > 192 * 1024 * 1024) throw new Error('Native tree plan request exceeds 192 MiB.');
  const {instance} = await WebAssembly.instantiate(wasmBinary, {}), api = instance.exports;
  const pointer = api.proof_alloc(bytes.length); if (!pointer) throw new Error('Plan export allocation refused.');
  let response;
  try { new Uint8Array(api.memory.buffer, pointer, bytes.length).set(bytes); const output = api.proof_execute(pointer, bytes.length), length = api.proof_response_length(); if (length > 64 * 1024 * 1024) throw new Error('Plan export response exceeds 64 MiB.'); response = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(new Uint8Array(api.memory.buffer, output, length))); }
  finally { api.proof_free(pointer, bytes.length); }
  if (!response.ok) throw new Error(response.error || 'Native plan export refused.');
  const result = response.result, hashText = text => sha256(new TextEncoder().encode(text));
  if (result?.schema_version !== 'bisect-external-tree-plan-output-v1' || result.status !== 'completed' || result.context_source_sha256 !== await hashText(contextJson) || result.profile_source_sha256 !== await hashText(profileJson)) throw new Error('Native plan export source binding mismatch.');
  return result;
}
export async function nativeTreePlanZip(result) {
  if (result?.schema_version !== 'bisect-external-tree-plan-output-v1' || result.status !== 'completed' || Object.keys(result.files ?? {}).sort().join() !== 'audit-certificate.json,exact.rctx,exact.rplan,external-tree.json,manifest.json') throw new Error('Fresh complete native tree plan package required.');
  const entries = [];
  for (const [name, text] of Object.entries(result.files)) {
    if (typeof text !== 'string') throw new Error('Invalid native plan file.'); const bytes = new TextEncoder().encode(text);
    if (await sha256(bytes) !== result.file_sha256?.[name]) throw new Error('Native plan file bytes changed.'); entries.push([name, bytes]);
  }
  return storedZipBytes(entries, {maxSourceBytes: 128 * 1024 * 1024});
}

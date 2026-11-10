import {validateNativeProofSources} from './native-proof-pipeline.js';
export const MAX_PROOF_SESSION_BYTES = 192 * 1024 * 1024;
export function validateProofSession(value) {
  const pending = [[value, 0]]; let nodes = 0;
  while (pending.length) {
    const [item, depth] = pending.pop();
    if (++nodes > 10000 || depth > 20) throw new Error('Proof session structure exceeds its limit.');
    if (item && typeof item === 'object') {
      for (const [key, child] of Object.entries(item)) {
        if (['__proto__', 'constructor', 'prototype'].includes(key)) throw new Error('Unsafe proof session key.');
        pending.push([child, depth + 1]);
      }
    }
  }
  if (!value || Object.keys(value).sort().join() !== 'discovery_json,instance_json,result,schema,seconds' || value.schema !== 'bisect-native-proof-session-v1') throw new Error('Unsupported proof session.');
  validateNativeProofSources(value.instance_json, value.discovery_json, value.seconds);
  if (value.result !== null) {
    const result = value.result;
    if (result?.schema !== 'bisect-native-proof-run-v1' || !['all-native-decisions-checked', 'checked-counterexample', 'proof-run-incomplete'].includes(result.status) || !result.files || typeof result.files !== 'object' || Array.isArray(result.files) || !Array.isArray(result.decisions) || result.decisions.length > 5) throw new Error('Invalid saved proof run.');
    const names = Object.keys(result.files);
    if (names.length > 29 || names.some(name => !/^(certified-split-instance\.json|certified-discovery\.json|manifest\.json|checked-native-proof-package\.json|(01-population|02-boundary|03-canonical|02-(low|high)-boundary-right-\d+)\.(opb|request\.json|pbp|solver\.json|checked\.json))$/.test(name))) throw new Error('Invalid saved proof file inventory.');
    let total = 0;
    for (const text of Object.values(result.files)) {
      if (typeof text !== 'string') throw new Error('Saved proof files must be text.');
      total += new TextEncoder().encode(text).length;
      if (total > 128 * 1024 * 1024) throw new Error('Saved proof files exceed 128 MiB.');
    }
    if (result.files['certified-split-instance.json'] !== value.instance_json || result.files['certified-discovery.json'] !== value.discovery_json) throw new Error('Session sources disagree with the retained proof run.');
  }
  return value;
}
export function parseProofSession(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.length > MAX_PROOF_SESSION_BYTES) throw new Error('Proof session exceeds 192 MiB.');
  return validateProofSession(JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes)));
}
export function encodeProofSession(value) {
  validateProofSession(value);
  const bytes = new TextEncoder().encode(JSON.stringify(value));
  if (bytes.length > MAX_PROOF_SESSION_BYTES) throw new Error('Proof session exceeds 192 MiB.');
  return bytes;
}

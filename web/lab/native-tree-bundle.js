export const MAX_TREE_BUNDLE_BYTES = 192 * 1024 * 1024;
const encoder = new TextEncoder();
export function parseNativeTreeBundle(bytes) {
  if (!(bytes instanceof Uint8Array) || !bytes.length || bytes.length > MAX_TREE_BUNDLE_BYTES) throw new Error('Tree proof bundle must be at most 192 MiB.');
  const value = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));
  const pending = [[value, 0]]; let count = 0;
  while (pending.length) {
    const [item, depth] = pending.pop(); if (++count > 500000 || depth > 20) throw new Error('Tree proof bundle structure exceeds its limit.');
    if (item && typeof item === 'object') for (const [key, child] of Object.entries(item)) {
      if (['__proto__', 'constructor', 'prototype'].includes(key)) throw new Error('Unsafe tree bundle key.');
      pending.push([child, depth + 1]);
    }
  }
  const keys = (object, required, optional = []) => {
    if (!object || typeof object !== 'object' || Array.isArray(object) || required.some(key => !Object.hasOwn(object, key)) || Object.keys(object).some(key => !required.includes(key) && !optional.includes(key))) throw new Error('Unsupported tree bundle fields.');
  };
  const source = text => { if (typeof text !== 'string' || !text.trim() || text.includes('\0') || encoder.encode(text).length > 3 * 1024 * 1024) throw new Error('Tree source must be nonempty UTF-8 text of at most 3 MiB.'); };
  keys(value, ['operation', 'root_instance_json', 'nodes'], ['expected_tree']);
  if (value.operation !== 'adopt-native-external-tree' || !Array.isArray(value.nodes) || !value.nodes.length || value.nodes.length > 1023) throw new Error('Unsupported tree operation or node count.');
  source(value.root_instance_json); let total = 0;
  for (const node of value.nodes) {
    keys(node, ['operation', 'instance_json', 'discovery_json', 'proofs'], ['expected_certificate']);
    if (node.operation !== 'adopt-native-external-split') throw new Error('Unsupported tree node operation.');
    source(node.instance_json); source(node.discovery_json);
    if (!node.proofs || typeof node.proofs !== 'object' || Array.isArray(node.proofs) || ![3, 5].includes(Object.keys(node.proofs).length)) throw new Error('Incomplete node proof inventory.');
    for (const [stem, text] of Object.entries(node.proofs)) {
      if (!/^(01-population|02-boundary|03-canonical|02-(low|high)-boundary-right-\d+)$/.test(stem) || typeof text !== 'string' || !text.trim() || text.includes('\0') || encoder.encode(text).length > 64 * 1024 * 1024) throw new Error('Invalid tree proof name or text.');
      total += encoder.encode(text).length; if (total > 128 * 1024 * 1024) throw new Error('Tree proofs exceed 128 MiB.');
    }
  }
  return value;
}

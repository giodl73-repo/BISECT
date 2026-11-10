import assert from 'node:assert/strict';

// Explicit operation/schema/path rules; every other field compares exactly.
const rules = new Map();
for (const kind of ['run', 'state', 'states']) {
  const schema = `bisect-native-${kind}-command-v1`;
  rules.set(`inspect-native-${kind}-command`, {path: [], schema});
  for (const suffix of ['graph-run', 'run'])
    rules.set(`prepare-native-${kind}-${suffix}`, {path: ['native_command'], schema});
}
rules.set('prepare-native-adjacency-state-run', {
  path: ['native_command'], schema: 'bisect-native-state-command-v1',
});

// Test-owned composition selectors; never sent to the Rust engine.
rules.set('compare-prepared-native-state-job', {
  path:['native_state','compilation','native_command'],schema:'bisect-native-state-command-v1',
  envelopes:[{path:[],schema:'bisect-prepared-state-run-v1'}],
});
rules.set('compare-source-native-state-job', {
  path:['result','native_state','compilation','native_command'],schema:'bisect-native-state-command-v1',
  envelopes:[{path:[],schema:'bisect-native-source-flags-state-run-v1'},
             {path:['result'],schema:'bisect-prepared-state-run-v1'}],
});

export function compareCommandReplies(request, wasm, native, nativeWidth) {
  assert([32, 64].includes(nativeWidth), 'Independently establish native target width');
  const actual = structuredClone(wasm), expected = structuredClone(native);
  const rule = rules.get(request.operation);
  if (rule) {
    for (const envelope of rule.envelopes ?? []) for (const reply of [actual,expected]) {
      let node=reply;
      for (const key of envelope.path) {
        assert(node && Object.hasOwn(node,key), 'Missing composed reply envelope');
        node=node[key];
      }
      assert.equal(node?.schema_version,envelope.schema,'Unexpected composed reply schema');
    }
    let a = actual, n = expected;
    for (const key of rule.path) {
      assert(a && Object.hasOwn(a, key) && n && Object.hasOwn(n, key), 'Missing command node');
      a = a[key]; n = n[key];
    }
    for (const node of [a, n]) {
      assert(node && !Array.isArray(node), 'Expected command object');
      assert.equal(node.schema_version, rule.schema, 'Unexpected command schema');
      assert(Object.hasOwn(node, 'target_pointer_width'), 'Missing target width');
    }
    assert.equal(a.target_pointer_width, 32, 'Unexpected WASM target width');
    assert.equal(n.target_pointer_width, nativeWidth, 'Unexpected native target width');
    a.target_pointer_width = n.target_pointer_width;
  }
  assert.deepEqual(actual, expected);
}


import {readFileSync,writeFileSync,mkdirSync,mkdtempSync} from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

// Current native test executables are Windows PE files. Refuse unknown targets.
export function createNativeCommandComparator(root, wasmFile, {nativeExecutable='target/release/examples/execute_tool_request.exe'}={}) {
  const executable = nativeExecutable;
  const bytes = readFileSync(executable);
  assert.equal(bytes.toString('ascii',0,2), 'MZ');
  const offset = bytes.readUInt32LE(0x3c);
  assert.equal(bytes.toString('ascii',offset,offset+4), 'PE\0\0');
  assert.equal(bytes.readUInt16LE(offset+4), 0x8664, 'Expected AMD64 native test executable');
  assert.equal(bytes.readUInt16LE(offset+24), 0x20b, 'Expected PE32+ native test executable');
  mkdirSync(root,{recursive:true});
  const directory = mkdtempSync(path.join(root,'command-comparisons-'));
  const receipt = {schema_version:'bisect-native-command-comparisons-v1',
    scope:'Shared native Rust replies or explicitly typed browser composition over native Rust replies versus WASM; not desktop CLI orchestration or full browser acceptance',
    native_executable:executable,native_sha256:sha256(bytes),native_pointer_width:64,
    native_width_evidence:'AMD64 PE32+ executable header',
    wasm_file:wasmFile,wasm_sha256:sha256(readFileSync(wasmFile)),
    comparator_sha256:sha256(readFileSync(new URL(import.meta.url))),
    node_version:process.version,pairs:[]};
  const save = () => writeFileSync(path.join(directory,'evidence.json'),JSON.stringify(receipt,null,2)+'\n');
  save();
  const compare = (request,wasm,native) => {
    const file = `pair-${receipt.pairs.length}.json`;
    const raw = JSON.stringify({request,wasm,native});
    writeFileSync(path.join(directory,file),raw);
    const pair = {file,sha256:sha256(raw),operation:request.operation,status:'pending'};
    receipt.pairs.push(pair); save();
    try { compareCommandReplies(request,wasm,native,64); pair.status='passed'; }
    catch(error) { pair.status='failed'; pair.error=String(error); throw error; }
    finally { save(); }
  };
  return {compare,directory};
}

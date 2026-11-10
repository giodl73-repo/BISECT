import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {parseProofSession, encodeProofSession} from '../../web/lab/native-proof-session.js';
const result = JSON.parse(await fs.readFile('target/native-proof-pipeline-tests/two-optimal.json', 'utf8'));
const session = {schema: 'bisect-native-proof-session-v1', instance_json: result.files['certified-split-instance.json'], discovery_json: result.files['certified-discovery.json'], seconds: 3, result};
assert.deepEqual(parseProofSession(encodeProofSession(session)), session);
assert.deepEqual(parseProofSession(encodeProofSession({...session, result: null})), {...session, result: null});
assert.throws(() => encodeProofSession({...session, injected: true}));
assert.throws(() => encodeProofSession({...session, instance_json: '{}'}), /disagree/);
const unsafe = structuredClone(session); unsafe.result.files['../attack.js'] = 'bad'; assert.throws(() => encodeProofSession(unsafe));
assert.throws(() => parseProofSession(new Uint8Array([0xff])), /encoded|encoding/);
const nested = structuredClone(session); let current = nested.result; for (let i = 0; i < 25; i++) { current.deep = {}; current = current.deep; } assert.throws(() => encodeProofSession(nested), /structure/);
const forged = structuredClone(session); forged.result.decisions[0].request_id = 'sha256:' + '0'.repeat(64);
// Parsing is inert structural validation, never proof acceptance.
assert.deepEqual(parseProofSession(encodeProofSession(forged)), forged);
console.log('Proof sessions: exact/source-only roundtrip, closed keys, source mismatch, unsafe paths, malformed UTF-8 and depth bounds passed; forged claims remain unverified data.');

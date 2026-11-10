import assert from 'node:assert/strict';
import {parseStoredInstallationOutcome,readInstallationOutcome,saveInstallationOutcome} from '../../web/lab/installation-outcome-store.js';
const record={schema_version:'bisect-local-installation-outcome-v1',id:'12345678-1234-1234-1234-123456789abc',outcome:{schema_version:'bisect-browser-installation-outcome-v1',status:'in-progress',package_sha256:'a'.repeat(64),target_namespace:'v1 · 3 state targets',recovery:false,receipt:null,scope:'Passive observation, no folder authority.'}};
assert.deepEqual(parseStoredInstallationOutcome(JSON.stringify(record)),record);
const invalid=[
 {...record,root:{}},
 {...record,id:'../outside'},
 {...record,outcome:{...record.outcome,root:{}}},
 {...record,outcome:{...record.outcome,status:'verified'}},
 {...record,outcome:{...record.outcome,package_sha256:'bad'}},
 {...record,outcome:{...record.outcome,receipt:{schema_version:'bisect-browser-native-install-v1'}}},
 {...record,outcome:{...record.outcome,status:'completed',receipt:{schema_version:'executable'}}},
 {...record,outcome:{...record.outcome,error:'x'.repeat(2001)}},
];
for(const value of invalid)assert.throws(()=>parseStoredInstallationOutcome(JSON.stringify(value)));
assert.throws(()=>parseStoredInstallationOutcome('{"__proto__":{}}'));
assert.throws(()=>parseStoredInstallationOutcome('{"schema_version":"x","schema_version":"y"}'));
assert.throws(()=>parseStoredInstallationOutcome(' '.repeat(1024*1024+1)));
assert.throws(()=>parseStoredInstallationOutcome(JSON.stringify({...record,outcome:{...record.outcome,scope:'💾'.repeat(300000)}})));
await assert.rejects(readInstallationOutcome({idb:null}),/unavailable/);
await assert.rejects(saveInstallationOutcome(record,{idb:{open(){throw new Error('quota refusal');}}}),/quota refusal/);
console.log('PASS stored outcome: bounded passive schema, hostile/oversized records and explicit storage failures.');

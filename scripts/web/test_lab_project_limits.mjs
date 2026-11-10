import assert from 'node:assert/strict';
import {MAX_PROJECT_BYTES,parseSafeJson} from '../../web/lab/project.js';
import {MAX_LAB_PROJECT_VALUES,parseLabProject} from '../../web/lab/laboratory-project.js';

// The measured full 50-state economic project has 1,274,114 values in 19 MB.
// Test token limits independently of its schema; real projects are exercised
// by test_character_national.mjs, including inert restore and data tampering.
assert.equal(MAX_LAB_PROJECT_VALUES,2000000);
const arrayText=values=>'['+'0,'.repeat(values-1)+'0]';
assert.throws(()=>parseSafeJson(arrayText(500000)),/500,000 values/);
assert.throws(()=>parseLabProject(arrayText(500000)),/Invalid laboratory project format/);
assert.throws(()=>parseLabProject(arrayText(MAX_LAB_PROJECT_VALUES-1)),/Invalid laboratory project format/);
assert.throws(()=>parseLabProject(arrayText(MAX_LAB_PROJECT_VALUES)),/2,000,000 values/);
assert.throws(()=>parseLabProject(' '.repeat(MAX_PROJECT_BYTES+1)),/25 MB import limit/);
assert.throws(()=>parseLabProject(JSON.stringify('é'.repeat(Math.floor(MAX_PROJECT_BYTES/2)))),/25 MB import limit/);
for(const text of ['{"__proto__":{}}','{"constructor":0}','{"n":9007199254740992}','['.repeat(65)+'0'+']'.repeat(65)])assert.throws(()=>parseLabProject(text),/Unsafe|exact JavaScript|maximum 64 levels/);
console.log('PASS laboratory project limits: separate 2-million-value budget, inclusive boundary, ordinary 500k unchanged, UTF-8/25-MiB, depth, unsafe-key and exact-integer refusals.');

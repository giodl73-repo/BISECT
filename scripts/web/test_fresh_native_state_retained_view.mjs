import assert from 'node:assert/strict';
import {parseSafeJson} from '../../web/lab/project.js';
import {parseFreshNativeStateRetainedRun} from '../../web/lab/fresh-native-state-retained-view.js';
const file=text=>({name:'saved-run.json',base64:Buffer.from(text).toString('base64')});
// Artificial display data tests the parser budget, not a completed engine run.
const saved={schema_version:'bisect-fresh-native-state-run-v1',result:{display_samples:Array(500001).fill(0)}};
const text=JSON.stringify(saved);assert.throws(()=>parseSafeJson(text),/too complex/);
assert.deepEqual(parseFreshNativeStateRetainedRun(file(text)),saved);
for(const [bad,pattern]of [
 ['{"a":1,"a":2}',/duplicate/i],
 ['{"__proto__":{"polluted":true}}',/unsafe/i],
 ['{"constructor":{}}',/unsafe/i],
 ['{"n":9007199254740993}',/exact JavaScript range/],
 ['['.repeat(66)+'0'+']'.repeat(66),/too complex/],
 ['<script>globalThis.retainedExecuted=true</script>',/JSON|Unexpected/],
 ['['+'0,'.repeat(4000000)+'0]',/too complex/],
 ])assert.throws(()=>parseFreshNativeStateRetainedRun(file(bad)),pattern);
assert.throws(()=>parseFreshNativeStateRetainedRun({name:'saved.json',base64:'AB=='}),/canonical/);
assert.equal(globalThis.retainedExecuted,undefined);assert.equal({}.polluted,undefined);
console.log('Fresh retained viewer: bounded >500,000-value data parses inertly; eight malformed/security/capacity controls refuse. Ordinary project budgets unchanged.');

import assert from 'node:assert/strict';
import {engineReplayRequest} from '../../web/lab/engine-replay-files.js';
import {isBase64Encoding} from '../../web/lab/package-files.js';
for(const value of ['', 'AA==', 'AAA=', 'AAAA', 'QUJD'.repeat(2000000)])assert(isBase64Encoding(value));
for(const value of ['A', 'A===', '=AAA', 'AA=A', 'AAAA\n', 'AA==AAAA', '😀AAA'])assert.equal(isBase64Encoding(value),false);
const source=text=>({name:'selected.json',base64:Buffer.from(text).toString('base64')});
const request=text=>engineReplayRequest({run:source(text),original:source('{}'),minSimilarity:1});
const raw='{"kind":"engine","request":{"seed":18446744073709551615}}';
assert.equal(request(raw).input.run_json,raw);
for(const text of [
 '{"kind":"engine","request":{"seed":1,"\\u0073eed":2}}',
 '{"kind":"engine","request":{"__proto__":{}}}',
 '{"kind":"engine","request":{"padding":'+'['.repeat(65)+'0'+']'.repeat(65)+'}}',
 '{"kind":"engine","request":{"padding":['+'0,'.repeat(4000000)+'0]}}',
])assert.throws(()=>request(text),/Duplicate|Unsafe|too complex/);
assert.throws(()=>request(' '.repeat(16*1024*1024+1)),/byte limit|Invalid bounded/);
console.log('Replay source bounds: exact numeric-u64 bytes retained; duplicate/escaped keys, unsafe keys, depth, four-million-value budget and byte cap refuse before execution.');

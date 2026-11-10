import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const current=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),before=await instantiateEngine(await fs.readFile('target/native-state-vra-summary/before/engine.wasm'));
const records=JSON.parse(await fs.readFile('target/native-signal-presets/records.json','utf8'));let enriched=0;
for(const r of records){const actual=current.execute(r.request),old=before.execute(r.request);if(actual.metrics.native_state_vra){delete actual.metrics.native_state_vra;enriched++;}assert.deepEqual(actual,old);}
await fs.writeFile('target/native-state-vra-summary/compatibility.json',JSON.stringify({wasm:current.artifactIdentity,previous_wasm:before.artifactIdentity,compared:records.length,enriched_only:enriched,existing_assignments_options_and_other_metrics_unchanged:true},null,2)+'\n');console.log(`PASS ${records.length} prior-WASM comparisons; only ${enriched} eligible native State summaries added.`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
const coverage=JSON.parse(await fs.readFile('docs/specs/wasm-engine-composition-coverage.json','utf8'));
const catalog=JSON.parse(await fs.readFile((process.argv[2]||'dist/wasm-lab-build')+'/catalog.json','utf8')).catalog;
const expected=Object.entries(catalog.search_compatibility).flatMap(([structure,searches])=>searches.map(search=>structure+'/'+search)).sort();
const actual=coverage.rows.filter(r=>r.weights==='partisan').map(r=>r.structure+'/'+r.search).sort();
assert.equal(new Set(actual).size,actual.length,'Duplicate partisan composition');assert.deepEqual(actual,expected,'Catalog compositions and scoped partisan evidence disagree');
for(const row of coverage.rows){const group=coverage.evidence_groups[row.evidence_group];assert(group,'Missing evidence group');assert.match(row.verified_wasm_sha256,/^[a-f0-9]{64}$/);await fs.access(group.script);const receipt=JSON.parse(await fs.readFile(group.receipt,'utf8'));assert.equal(receipt[group.cases_key],group.expected,group.receipt);}
console.log(`Composition declaration gate: ${actual.length} scoped partisan rows match the current catalog; referenced scripts and case counts checked. This does not establish fresh artifact execution, arbitrary parameter capacity or full parity.`);

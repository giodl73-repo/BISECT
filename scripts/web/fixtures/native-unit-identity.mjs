import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
// Frozen actual desktop outputs and selected bytes; names/paths confer no IO authority.
export async function loadNativeUnitIdentityCorpus(){
 const raw=await fs.readFile(new URL('./native-imported-recorded-replay.json',import.meta.url));
 assert.equal(createHash('sha256').update(raw).digest('hex'),"948f5595578d8551cc3b328ebc191ad54c7bb6127b62425f9b5ae12404673410");
 const rows=JSON.parse(raw);assert.equal(rows.length,12);
 assert.deepEqual(rows.map(row=>row.name).sort(),["absent-index", "absent-index-coi", "block", "block-group-native-spelling", "county-ids", "foreign-prefix-tract", "mixed-imported", "no-prefix-imported", "prefix-imported", "sparse-index", "sparse-index-coi", "tract"]);
 for(const row of rows){
  assert.equal(row.request.operation,'run-native-recorded-replay');
  const manifest=JSON.parse(row.request.input.sources.prechecks.manifest_json);
  assert.equal(manifest.native_replay_recipe.state.algorithm.metis.seed,'18446744073709551615');
  assert.deepEqual(manifest.native_replay_recipe.state.algorithm.seeds,{Multi:{seeds:50}});
  const context=JSON.parse(row.context_json),document=JSON.parse(row.document_json);
  assert.equal(context.units.unit_ids.length,4);assert.equal(document.plan.k,2);
  assert.deepEqual(document.plan.units,context.units);
  assert.deepEqual(context.populations,[100,100,100,100]);
  assert.equal(row.request.input.sources.prechecks.manifest_path,'plans/original/manifest.json');
  assert.equal(Buffer.from(row.request.input.sources.prechecks.files_base64['plans/original/context.rctx'],'base64').toString('utf8'),row.context_json);
 }
 return rows;
}

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

// The baseline executable was retained before changing the native repair
// enumerator; its result exercises the old full 3^16 assignment search.
const root = 'target/partisan-capacity';
const source = JSON.parse(await fs.readFile(root + '/baseline-grid-request.json', 'utf8'));
const original = JSON.parse(await fs.readFile(root + '/original-grid-result.json', 'utf8'));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const partisan = {schema_version: 'bisect-partisan-shares-v1', state: source.graph.state, year: source.graph.year, source_label: 'SYNTHETIC unchanged-native repair comparison', dem_shares: Object.fromEntries(source.graph.geoids.map(id => [id, .5]))};
const request = {graph: source.graph, partisan, options: {structure: 'capacity-clustering', weights: 'partisan', search: 'single', districts: 3, seed: null, seeds: 1, steps: 20, percentile: 0, alpha_county: 0, balance_tolerance: 25, area_swing: 1.1, iterations: 10, dem_threshold: .55, rep_threshold: .45}};
const actual = engine.execute(request);
assert.deepEqual(actual.assignments, original.assignments);
assert.deepEqual(actual.metrics.structure_evidence, original.summary);
assert.equal(actual.metrics.weighted_boundary, original.weighted_boundary);
assert.equal(actual.metrics.structure_evidence.repair_method, 'exhaustive-small');
await fs.writeFile(root + '/repair-baseline-evidence.json', JSON.stringify({unchanged_native_exhaustive_assignments: true, unchanged_native_summary_and_cut: true, original_reference_sha256: createHash('sha256').update(await fs.readFile(root + '/original-reference.exe')).digest('hex'), original_result_sha256: createHash('sha256').update(await fs.readFile(root + '/original-grid-result.json')).digest('hex'), wasm_sha256: engine.artifactIdentity.sha256}, null, 2));
console.log('Exact repair pruning: 16-unit/three-district assignments, summary and weighted cut match the retained original exhaustive native reference.');

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';

const root = 'target/partisan-prime-reference', source = 'dist/wasm-lab-build';
await fs.mkdir(root, {recursive: true});
const manifest = JSON.parse(await fs.readFile(source + '/catalog.json', 'utf8'));
const reference = 'target/release/examples/partisan_structure_reference.exe';
const n = 120;
const line = {schema_version: 1, state: 'IA', year: '2020', geoids: Array.from({length: n}, (_, i) => '19001' + String(i).padStart(6, '0')),
  adjacency: Array.from({length: n}, (_, i) => [i - 1, i + 1].filter(j => j >= 0 && j < n)), population: Array(n).fill(100),
  edges: Array.from({length: n - 1}, (_, i) => [i, i + 1, 100]), areas: Array(n).fill(10000), exterior_perimeters: Array(n).fill(0),
  centroids: Array.from({length: n}, (_, i) => [-93 + i / 10000, 42])};
const graphs = [{label: 'synthetic-line', graph: line}];
for (const code of ['RI', 'IA']) graphs.push({label: code, graph: JSON.parse(await fs.readFile(source + '/' + manifest.graphs[code + ':2020'].graph_ref, 'utf8'))});
let cases = 0, successes = 0, failures = 0;
const records = [];
for (const {label, graph} of graphs) for (const profile of ['uniform', 'stripe', 'threshold-boundary']) {
  for (const seed of [null, '18446744073709551615']) for (const districts of [1, 2, 3, 5]) {
    const shares = graph.geoids.map((_, i) => profile === 'uniform' ? .5 : profile === 'stripe' ? (i % 4 < 2 ? .8 : .2) : (i % 3 === 0 ? .55 : .45));
    const partisan = {schema_version: 'bisect-partisan-shares-v1', state: graph.state, year: graph.year,
      source_label: 'SYNTHETIC prime-factor parity shares', dem_shares: Object.fromEntries(graph.geoids.map((id, i) => [id, shares[i]]))};
    const tsv = root + '/shares-' + cases + '.tsv';
    await fs.writeFile(tsv, 'geoid\tdem_share\n' + graph.geoids.map((id, i) => id + '\t' + shares[i]).join('\n') + '\n');
    const state_args = ['--state', graph.state, '--year', graph.year, '--partition-mode', 'partisan-weighted', '--structure', 'prime-factor',
      '--partisan-shares', path.resolve(tsv), '--districts', String(districts), '--search', 'single', '--niter', '10', '--balance-tolerance', '5',
      '--alpha-county', '0', '--dem-threshold', '.55', '--rep-threshold', '.45', '--metis-engine', 'metis-core', ...(seed === null ? [] : ['--seed', seed])];
    const request = {graph, partisan, options: {structure: 'prime-factor', weights: 'partisan', search: 'single', districts, seed, seeds: 1, steps: 20,
      percentile: 0, alpha_county: 0, balance_tolerance: 5, area_swing: 1.1, iterations: 10, dem_threshold: .55, rep_threshold: .45}};
    const file = root + '/case-' + cases + '.json';
    await fs.writeFile(file, JSON.stringify({graph, state_args}));
    await fs.writeFile(root + '/engine-' + cases + '.json', JSON.stringify(request));
    const native = spawnSync(reference, [file], {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
    assert.equal(native.error, undefined);
    const record = {case: cases, label, profile, seed, districts};
    if (native.status === 0) {
      const result = JSON.parse(native.stdout), ids = new Set(graph.geoids);
      assert.equal(Object.keys(result.assignments).length, ids.size);
      for (const [id, d] of Object.entries(result.assignments)) assert(ids.has(id) && Number.isInteger(d) && d >= 1 && d <= districts);
      const total = graph.population.reduce((a, b) => a + b, 0), pops = Array(districts).fill(0);
      graph.geoids.forEach((id, i) => {pops[result.assignments[id] - 1] += graph.population[i];});
      const deviation = Math.max(...pops.map(p => Math.abs(p / (total / districts) - 1) * 100));
      assert(deviation <= 3 + 1e-9);
      let weighted = 0;
      for (const [u, v] of graph.edges) if (result.assignments[graph.geoids[u]] !== result.assignments[graph.geoids[v]]) {
        const boosted = shares[u] >= .55 && shares[v] >= .55 || shares[u] <= .45 && shares[v] <= .45;
        weighted += boosted ? 3 : 1;
      }
      assert(Math.abs(weighted - result.weighted_boundary) < 1e-9);
      assert.equal(result.summary.seed, seed);
      await fs.writeFile(root + '/result-' + cases + '.json', JSON.stringify(result));
      record.status = 'native-completed'; record.max_deviation_percent = deviation; successes++;
    } else {
      assert.match(native.stderr, /apportion-regions/);
      await fs.writeFile(root + '/error-' + cases + '.txt', native.stderr);
      record.status = 'native-rejected'; failures++;
    }
    records.push(record); cases++;
  }
}
assert.equal(cases, 72); assert(successes > 0); assert(failures > 0);
await fs.writeFile(root + '/evidence.json', JSON.stringify({native_state_tsv_cases: cases, native_successes: successes, native_failures: failures,
  exact_max_u64_optional_seed_retained: true, independent_coverage_balance_and_sparse_cut_checks: successes,
  scope: 'Native reference preparation only. Browser prime-factor/partisan execution is still blocked and has not been validated by this script.',
  native_reference_sha256: createHash('sha256').update(await fs.readFile(reference)).digest('hex'), records}, null, 2));
console.log(`Prime-factor native preparation: ${cases} State/TSV cases, ${successes} complete and ${failures} native rejections; independent coverage/3% balance/cut checks. Browser execution remains pending.`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root = 'target/partisan-annealing';
await fs.mkdir(root, {recursive: true});
const source = 'dist/wasm-lab-build';
const manifest = JSON.parse(await fs.readFile(source + '/catalog.json', 'utf8'));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const reference = 'target/release/examples/partisan_structure_reference.exe';
const graphs = [JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json', 'utf8')).graph];
for (const code of ['RI', 'IA']) {
  graphs.push(JSON.parse(await fs.readFile(source + '/' + manifest.graphs[code + ':2020'].graph_ref, 'utf8')));
}
const partisanFor = (graph, profile) => ({
  schema_version: 'bisect-partisan-shares-v1', state: graph.state, year: graph.year,
  source_label: 'SYNTHETIC annealing parity shares',
  dem_shares: Object.fromEntries(graph.geoids.map((id, i) => [id, profile === 'uniform' ? .5 : profile === 'stripe' ? (i % 4 < 2 ? .8 : .2) : (i % 3 === 0 ? .55 : .45)])),
});
const baseOptions = {
  structure: 'simulated-annealing', search: 'single', weights: 'partisan', districts: 3,
  seed: null, seeds: 1, steps: 20, percentile: 0, alpha_county: 0,
  balance_tolerance: 25, area_swing: 1.1, iterations: 10,
  dem_threshold: .55, rep_threshold: .45, sa_steps_per_tract: 2, sa_t0_factor: .01, sa_t_final: .0001,
};
let cases = 0, failures = 0, empty = 0, coiCases = 0;
const records = [];
for (const graph of graphs) for (const profile of ['uniform', 'stripe', 'threshold-boundary']) {
  for (const seed of [null, '18446744073709551615']) for (const k of [1, 3]) for (const steps of [0, 2, 10]) {
    const partisan = partisanFor(graph, profile), tsv = root + '/shares-' + cases + '.tsv';
    await fs.writeFile(tsv, 'geoid\tdem_share\n' + graph.geoids.map(id => id + '\t' + partisan.dem_shares[id]).join('\n') + '\n');
    // SA is a native preset; its weight override supplies the partisan source.
    const tokens = ['--state', graph.state, '--year', graph.year, '--partition-mode', 'simulated-annealing',
      '--weights-override', 'proportional', '--partisan-shares', path.resolve(tsv), '--districts', String(k),
      '--search', 'single', '--sa-steps-per-tract', String(steps), '--sa-t0-factor', '.01', '--sa-t-final', '.0001',
      '--niter', '10', '--balance-tolerance', '25', '--metis-engine', 'metis-core', ...(seed === null ? [] : ['--seed', seed])];
    const file = root + '/' + cases + '.json';
    await fs.writeFile(file, JSON.stringify({graph, state_args: tokens}));
    const native = spawnSync(reference, [file], {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
    assert.equal(native.error, undefined);
    const options = {...baseOptions, districts: k, seed, sa_steps_per_tract: steps};
    let actual, error;
    try {actual = engine.execute({graph, options, partisan});} catch (e) {error = e;}
    assert.equal(Boolean(actual), native.status === 0, `${graph.state}/${profile}/k${k}/steps${steps}: ${native.stderr}; ${error?.message}`);
    if (actual) {
      const n = JSON.parse(native.stdout);
      assert.deepEqual(actual.assignments, n.assignments);
      assert(Math.abs(actual.metrics.weighted_boundary - n.weighted_boundary) < Math.max(1, n.weighted_boundary) * 1e-12);
      assert.equal(n.dem_threshold, .55); assert.equal(n.rep_threshold, .45);
      if (k > 1) {
        assert.equal(actual.metrics.structure_evidence.objective, 'unweighted-edge-cut');
        assert.equal(actual.metrics.structure_evidence.initial_refinement_iterations, 100);
        assert.equal(actual.metrics.structure_evidence.steps_per_tract, steps);
      }
      if (profile === 'uniform') empty++;
      if (profile === 'stripe' && k === 3 && steps === 2) {
        const weights_json = JSON.stringify({[graph.geoids[0]]: 9, [graph.geoids[1]]: 0, 'unknown-parity-unit': 3}) + '\n';
        const coi = {source_name: 'SYNTHETIC annealing COI.json', weights_json};
        const coiPath = root + '/coi-' + cases + '.json';
        await fs.writeFile(coiPath, weights_json);
        const coiFile = root + '/coi-request-' + cases + '.json';
        await fs.writeFile(coiFile, JSON.stringify({graph, state_args: tokens, coi_path: path.resolve(coiPath)}));
        const coiNative = spawnSync(reference, [coiFile], {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
        assert.equal(coiNative.status, 0, coiNative.stderr);
        const coiOut = engine.execute({graph, options, partisan, coi}), cn = JSON.parse(coiNative.stdout);
        assert.deepEqual(coiOut.assignments, cn.assignments);
        assert(Math.abs(coiOut.metrics.weighted_boundary - cn.weighted_boundary) < Math.max(1, cn.weighted_boundary) * 1e-12);
        assert.equal(coiOut.metrics.coi_weighting_evidence.source_sha256, createHash('sha256').update(weights_json).digest('hex'));
        coiCases++;
      }
      records.push({state: graph.state, profile, seed, k, steps, balance_passed: actual.metrics.within_requested_tolerance, contiguous: actual.metrics.contiguous});
    } else {
      assert(native.stderr.includes(error.message)); failures++;
      records.push({state: graph.state, profile, seed, k, steps, error: error.message});
    }
    cases++;
  }
}

class Worker {
  postMessage(m) {queueMicrotask(() => {try {
    this.onmessage({data: m.type === 'initialize' ? {type: 'ready'} : {type: 'result', id: m.id, result: engine.execute(m.request)}});
  } catch (e) {this.onmessage({data: {type: 'error', id: m.id, error: e.message}});}});}
  terminate() {}
}
const fetcher = async ref => {const b = await fs.readFile(source + '/' + ref); return {ok: true, arrayBuffer: async () => b.buffer.slice(b.byteOffset, b.byteOffset + b.length)};};
const base = JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect', 'utf8')).experiment.config;
let projects = 0, refusals = 0;
for (const code of ['RI', 'IA']) for (const steps of [0, 10]) {
  const graph = graphs.find(g => g.state === code), partisan = partisanFor(graph, 'stripe');
  const config = {...base, ...baseOptions, states: [code], districts: 3, seed: '18446744073709551615', balance_tolerance: 50, sa_steps_per_tract: steps, partisans: {[code]: partisan}};
  const lab = new WasmCatalog(manifest, fetcher, () => new Worker()), job = lab.submit(config);
  while (lab.active) await new Promise(r => setTimeout(r, 5));
  assert.equal(lab.jobs.get(job.id).status, 'completed', lab.jobs.get(job.id).logs.join('\n'));
  const project = await createLabProject(lab, job.id);
  const restore = new WasmCatalog(manifest, fetcher, () => {throw new Error('Open must not execute');});
  await restore.restoreProject(project);
  for (const mutate of [
    p => {p.experiment.config.partisans[code].dem_shares[graph.geoids[0]] = .6;},
    p => {p.experiment.states[0].metrics.structure_evidence.objective = 'weighted';},
    p => {p.experiment.states[0].metrics.structure_evidence.steps_per_tract++;},
  ]) {
    const changed = structuredClone(project); mutate(changed);
    await assert.rejects(() => restore.restoreProject(changed)); refusals++;
  }
  const exported = await lab.exportPractitionerProject(job.id, code);
  assert.deepEqual(exported.exported.document.provenance.producer.partisan_input, partisan);
  if (code === 'RI' && steps === 10) await fs.writeFile(root + '/browser.bisect', JSON.stringify(project));
  projects++;
}
await fs.writeFile(root + '/evidence.json', JSON.stringify({native_state_tsv_cases: cases, native_coi_composition_cases: coiCases, matching_failures: failures, empty_sparse_cases: empty, real_state_projects_and_exports: projects, altered_source_or_summary_refusals: refusals, inert_open: true, weighted_initialization_unweighted_annealing: true, records, wasm_sha256: engine.artifactIdentity.sha256, native_reference_sha256: createHash('sha256').update(await fs.readFile(reference)).digest('hex')}, null, 2));
console.log(`Partisan annealing: ${cases} native State/TSV cases, ${coiCases} COI compositions, ${failures} matching failures, ${empty} empty maps; ${projects} projects/exports and ${refusals} altered-source/objective/steps refusals.`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root = 'target/partisan-capacity', source = 'dist/wasm-lab-build';
await fs.mkdir(root, {recursive: true});
const manifest = JSON.parse(await fs.readFile(source + '/catalog.json', 'utf8'));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const reference = 'target/release/examples/partisan_structure_reference.exe';
const graphs = [JSON.parse(await fs.readFile('target/ilp-balance-contract/branch-and-cut.json', 'utf8')).graph];
for (const code of ['RI', 'IA']) graphs.push(JSON.parse(await fs.readFile(source + '/' + manifest.graphs[code + ':2020'].graph_ref, 'utf8')));
const variants = ['capacity-clustering','regionalization','flow-construction'].map(structure=>({structure,options:{},args:[]}));
const sharesFor = (g, profile) => ({schema_version: 'bisect-partisan-shares-v1', state: g.state, year: g.year, source_label: 'SYNTHETIC construction parity shares', dem_shares: Object.fromEntries(g.geoids.map((id, i) => [id, profile === 'uniform' ? .5 : profile === 'stripe' ? (i % 4 < 2 ? .8 : .2) : (i % 3 === 0 ? .55 : .45)]))});
const base = {search: 'single', weights: 'partisan', districts: 3, seed: null, seeds: 1, steps: 20, percentile: 0, alpha_county: 0, balance_tolerance: 25, area_swing: 1.1, iterations: 10, dem_threshold: .55, rep_threshold: .45};
let cases = 0, failures = 0, empty = 0, invariant = 0, coiCases = 0;
const records = [];
for (const graph of graphs) for (const variant of variants) for (const profile of ['uniform', 'stripe', 'threshold-boundary']) {
  for (const seed of [null, '18446744073709551615']) for (const k of [1, 3]) {
    const partisan = sharesFor(graph, profile), tsv = root + '/shares-' + cases + '.tsv';
    await fs.writeFile(tsv, 'geoid\tdem_share\n' + graph.geoids.map(id => id + '\t' + partisan.dem_shares[id]).join('\n') + '\n');
    const tokens = ['--state', graph.state, '--year', graph.year, '--partition-mode', 'partisan-weighted', '--structure', variant.structure,
      '--partisan-shares', path.resolve(tsv), '--districts', String(k), '--search', 'single', '--alpha-county', '0',
      '--dem-threshold', '.55', '--rep-threshold', '.45', '--balance-tolerance', '25', ...variant.args, ...(seed === null ? [] : ['--seed', seed])];
    const file = root + '/' + cases + '.json';
    await fs.writeFile(file, JSON.stringify({graph, state_args: tokens}));
    const native = spawnSync(reference, [file], {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
    assert.equal(native.error, undefined);
    const options = {...base, structure: variant.structure, ...variant.options, districts: k, seed};
    let actual, error;
    try {actual = engine.execute({graph, options, partisan});} catch (e) {error = e;}
    assert.equal(Boolean(actual), native.status === 0, `${graph.state}/${variant.structure}/${profile}/k${k}: ${native.stderr}; ${error?.message}`);
    if (actual) {
      const n = JSON.parse(native.stdout);
      assert.deepEqual(actual.assignments, n.assignments);assert.deepEqual(actual.metrics.structure_evidence,n.summary);
      assert(Math.abs(actual.metrics.weighted_boundary - n.weighted_boundary) < Math.max(1, n.weighted_boundary) * 1e-12);
      const unweightedOptions = {...options, weights: 'unweighted'};
      delete unweightedOptions.dem_threshold; delete unweightedOptions.rep_threshold;
      const unweighted = engine.execute({graph, options: unweightedOptions});
      assert.deepEqual(actual.assignments, unweighted.assignments);
      assert.deepEqual(actual.metrics.structure_evidence, unweighted.metrics.structure_evidence);
      invariant++;
      if (profile === 'uniform') empty++;
      if (profile === 'stripe' && k === 3) {
        const weights_json = JSON.stringify({[graph.geoids[0]]: 9, [graph.geoids[1]]: 0, 'unknown-parity-unit': 3}) + '\n';
        const coiPath = root + '/coi-' + cases + '.json'; await fs.writeFile(coiPath, weights_json);
        const coiFile = root + '/coi-request-' + cases + '.json'; await fs.writeFile(coiFile, JSON.stringify({graph, state_args: tokens, coi_path: path.resolve(coiPath)}));
        const cn = spawnSync(reference, [coiFile], {encoding: 'utf8', maxBuffer: 16 * 1024 * 1024}); assert.equal(cn.status, 0, cn.stderr);
        const out = engine.execute({graph, options, partisan, coi: {source_name: 'SYNTHETIC COI.json', weights_json}}), expected = JSON.parse(cn.stdout);
        assert.deepEqual(out.assignments, actual.assignments); assert.deepEqual(out.assignments, expected.assignments);
        assert(Math.abs(out.metrics.weighted_boundary - expected.weighted_boundary) < Math.max(1, expected.weighted_boundary) * 1e-12);
        assert.equal(out.metrics.coi_weighting_evidence.source_sha256, createHash('sha256').update(weights_json).digest('hex')); coiCases++;
      }
      records.push({state: graph.state, structure: variant.structure, parameters: variant.options, profile, seed, k, balance_passed: actual.metrics.within_requested_tolerance, contiguous: actual.metrics.contiguous});
    } else {assert(native.stderr.includes(error.message)); failures++; records.push({state: graph.state, structure: variant.structure, profile, seed, k, error: error.message});}
    cases++;
  }
}
class Worker {
  postMessage(m) {queueMicrotask(() => {try {this.onmessage({data: m.type === 'initialize' ? {type: 'ready'} : {type: 'result', id: m.id, result: engine.execute(m.request)}});} catch (e) {this.onmessage({data: {type: 'error', id: m.id, error: e.message}});}});}
  terminate() {}
}
const fetcher = async ref => {const b = await fs.readFile(source + '/' + ref); return {ok: true, arrayBuffer: async () => b.buffer.slice(b.byteOffset, b.byteOffset + b.length)};};
const configBase = JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect', 'utf8')).experiment.config;
let projects = 0, refusals = 0, historicalProjects = 0, failedProjectRuns = 0;
for (const code of ['RI', 'IA']) for (const variant of variants) for(const districts of [1,2]) {
  const graph = graphs.find(g => g.state === code), partisan = sharesFor(graph, 'stripe');
  const config = {...configBase, ...base, structure: variant.structure, ...variant.options, states: [code], districts, seed: '18446744073709551615', balance_tolerance: 50, partisans: {[code]: partisan}};
  const lab = new WasmCatalog(manifest, fetcher, () => new Worker()), job = lab.submit(config);
  while (lab.active) await new Promise(r => setTimeout(r, 5));
  if(lab.jobs.get(job.id).status==='failed'){
    const tsv=root+'/project-shares-'+failedProjectRuns+'.tsv';await fs.writeFile(tsv,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
    const file=root+'/project-failure-'+failedProjectRuns+'.json';await fs.writeFile(file,JSON.stringify({graph,state_args:['--state',code,'--year','2020','--partition-mode','partisan-weighted','--structure',variant.structure,'--partisan-shares',path.resolve(tsv),'--districts',String(districts),'--search','single','--balance-tolerance','50','--seed','18446744073709551615']}));
    const native=spawnSync(reference,[file],{encoding:'utf8'});assert.notEqual(native.status,0);assert(lab.jobs.get(job.id).logs.some(line=>line.includes('[ALGO]')&&native.stderr.includes(line.slice(line.indexOf('[ALGO]')))));
    failedProjectRuns++;continue;
  }
  assert.equal(lab.jobs.get(job.id).status, 'completed', lab.jobs.get(job.id).logs.join('\n'));
  const project = await createLabProject(lab, job.id), restore = new WasmCatalog(manifest, fetcher, () => {throw new Error('Open must not execute');});
  await restore.restoreProject(project);
  for (const mutate of [p => {p.experiment.config.partisans[code].dem_shares[graph.geoids[0]] = .6;}, p => {p.experiment.states[0].metrics.weighted_boundary++;}]) {
    const changed = structuredClone(project); mutate(changed); await assert.rejects(() => restore.restoreProject(changed)); refusals++;
  }
  const forged=structuredClone(project),evidence=forged.experiment.states[0].metrics.structure_evidence;
  if(variant.structure==='regionalization')evidence.summary.edge_cut++;else evidence.edge_cut++;
  await assert.rejects(()=>restore.restoreProject(forged));refusals++;
  if(districts===1){const historical=structuredClone(project);historical.experiment.states[0].metrics.structure_evidence=null;await restore.restoreProject(historical);historicalProjects++;}
  const exported = await lab.exportPractitionerProject(job.id, code);
  assert.deepEqual(exported.exported.document.provenance.producer.partisan_input, partisan);
  if (code === 'RI' && variant.structure === 'capacity-clustering'&&districts===1) await fs.writeFile(root + '/browser.bisect', JSON.stringify(project));
  projects++;
}
await fs.writeFile(root + '/evidence.json', JSON.stringify({native_state_tsv_cases: cases, matching_failures: failures, empty_sparse_cases: empty, assignment_invariance_checks: invariant, native_coi_composition_cases: coiCases, real_state_projects_and_exports: projects,failed_project_runs_match_native:failedProjectRuns,historical_one_district_imports:historicalProjects, altered_source_or_cost_refusals: refusals, inert_open: true, reported_weighting_only: true,native_one_district_summaries:true, records, wasm_sha256: engine.artifactIdentity.sha256, native_reference_sha256: createHash('sha256').update(await fs.readFile(reference)).digest('hex')}, null, 2));
console.log(`Partisan capacity/flow construction: ${cases} native State/TSV cases, ${failures} matching failures, ${invariant} assignment invariance checks, ${coiCases} COI comparisons; ${projects} projects/exports, ${refusals} source/cost refusals.`);

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
class Worker {
  postMessage(m) {queueMicrotask(() => {try {this.onmessage({data: m.type === 'initialize' ? {type: 'ready'} : {type: 'result', id: m.id, result: engine.execute(m.request)}});} catch (e) {this.onmessage({data: {type: 'error', id: m.id, error: e.message}});}});}
  terminate() {}
}
const fetcher = async ref => {const b = await fs.readFile(source + '/' + ref); return {ok: true, arrayBuffer: async () => b.buffer.slice(b.byteOffset, b.byteOffset + b.length)};};
const configBase = JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect', 'utf8')).experiment.config;
let projects = 0, refusals = 0, historicalProjects = 0;
for (const code of ['RI', 'IA']) for (const variant of variants) for(const districts of [1]) {
  const graph = graphs.find(g => g.state === code), partisan = sharesFor(graph, 'stripe');
  const config = {...configBase, ...base, structure: variant.structure, ...variant.options, states: [code], districts, seed: '18446744073709551615', balance_tolerance: 50, partisans: {[code]: partisan}};
  const lab = new WasmCatalog(manifest, fetcher, () => new Worker()), job = lab.submit(config);
  while (lab.active) await new Promise(r => setTimeout(r, 5));
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
console.log(`Native one-district constructors: ${projects} source-preserving projects/exports, ${refusals} altered-source/cost/summary refusals, ${historicalProjects} historical null-summary imports; browser fixture prepared.`);

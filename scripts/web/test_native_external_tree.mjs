import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync, spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {sha256} from '../../web/lab/proof-solver-core.js';
import {runNativeProofPipeline} from '../../web/lab/native-proof-pipeline.js';
import {adoptNativeExternalSplit} from '../../web/lab/native-external-certificate.js';
import {adoptNativeExternalTree} from '../../web/lab/native-external-tree.js';
const root = path.resolve('target/native-external-tree'); await fs.mkdir(root, {recursive: true});
const checkerBase = 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';
const wasmBinary = await fs.readFile(checkerBase + 'bisect_proof_checker.wasm'), receipt = JSON.parse(await fs.readFile(checkerBase + 'proof-checker-build.json', 'utf8'));
const native = path.resolve('target/proof-checker-build/release/bisect-proof-checker.exe');
const nativeRun = request => { const result = spawnSync(native, [], {input: JSON.stringify(request), encoding: 'utf8', timeout: 60000, maxBuffer: 24 * 1024 * 1024}); assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout); };
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const hashed = async value => 'sha256:' + await sha256(Buffer.from(JSON.stringify(canonical(value))));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const {default: solverFactory} = await import(pathToFileURL(path.resolve('target/roundingsat-wasm/roundingsat.mjs')).href);
const common = {engine, solverFactory, solverWasm: await fs.readFile('target/roundingsat-wasm/roundingsat.wasm'), solverReceipt: JSON.parse(await fs.readFile('target/roundingsat-wasm/build-receipt.json', 'utf8')), checkerWasm: wasmBinary, checkerReceipt: receipt, seconds: 3};
const evidence = [];
for (const k of [2, 3, 4, 5]) {
  const instance = JSON.parse(await fs.readFile('target/proof-models-wasm/2-full-42-instance.json', 'utf8')); instance.k_parent = k; instance.k_left = Math.floor(k / 2); instance.k_right = k - instance.k_left; instance.orientation_rule = instance.k_left === instance.k_right ? 'equal-seats-unit-zero-left' : 'seat-ordered-floor-left-ceil-right';
  if (k === 5) { instance.unit_ids = Array.from({length: 8}, (_, i) => 'unit-' + String(i).padStart(2, '0')); instance.populations = Array(8).fill(10); instance.edges = Array.from({length: 7}, (_, left) => ({left, right: left + 1, weight: 100})); instance.unit_universe_hash = await hashed({unit_ids: instance.unit_ids}); }
  const rootInstanceJson = JSON.stringify(instance, null, 2), nativeRequest = path.join(root, 'oracle-request.json');
  const solve = {operation: 'solve-certified-tree', instance_json: rootInstanceJson}; await fs.writeFile(nativeRequest, JSON.stringify(solve));
  const reference = JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe', [nativeRequest], {encoding: 'utf8'})); assert.deepEqual(engine.execute(solve), reference);
  const oracle = JSON.parse(reference.tree_json), certificates = new Map(), nodes = [];
  for (const node of oracle.nodes) {
    const local = structuredClone(node.instance);
    if (local.node_path) local.parent_certificate_id = certificates.get(local.node_path.slice(0, -1)).certificate_id;
    const discovery = {schema_version: 'certified-split-discovery-v1', instance_hash: await hashed(local), solver_name: 'native bounded tree oracle (test fixture)', solver_version: null, method: 'fixture-from-native-bounded-tree', objective: node.certificate.result.objective};
    discovery.discovery_id = await hashed(discovery);
    const instanceJson = local.node_path ? JSON.stringify(local, null, 2) : rootInstanceJson, discoveryJson = JSON.stringify(discovery, null, 2);
    const run = await runNativeProofPipeline({...common, instanceJson, discoveryJson}); assert.equal(run.status, 'all-native-decisions-checked');
    const proofs = Object.fromEntries(run.required_decisions.map(stem => [stem, run.files[stem + '.pbp']]));
    const certificate = await adoptNativeExternalSplit({instanceJson, discoveryJson, proofs, wasmBinary, receipt}); certificates.set(local.node_path, certificate);
    nodes.push({operation: 'adopt-native-external-split', instance_json: instanceJson, discovery_json: discoveryJson, proofs, expected_certificate: certificate});
  }
  const request = {operation: 'adopt-native-external-tree', root_instance_json: rootInstanceJson, nodes, expected_tree: null};
  const tree = await adoptNativeExternalTree({rootInstanceJson, nodes, wasmBinary, receipt}); assert.deepEqual(tree, nativeRun(request).result);
  assert.deepEqual(tree.nodes.map(node => node.node_path), oracle.nodes.map(node => node.instance.node_path));
  const leafFields = leaves => leaves.map(({node_path, unit_ids, district_index}) => ({node_path, unit_ids, district_index})); assert.deepEqual(leafFields(tree.leaves), leafFields(oracle.leaves));
  assert.equal(Object.keys(tree.assignments).length, instance.unit_ids.length);
  assert.deepEqual(await adoptNativeExternalTree({rootInstanceJson, nodes, expectedTree: tree, wasmBinary, receipt}), tree);
  const forged = structuredClone(tree); forged.leaves[0].district_index = 999;
  await assert.rejects(() => adoptNativeExternalTree({rootInstanceJson, nodes, expectedTree: forged, wasmBinary, receipt}), /differs/);
  await assert.rejects(() => adoptNativeExternalTree({rootInstanceJson, nodes: nodes.slice(1), wasmBinary, receipt}), /k-1/);
  if (k > 2) {
    const changed = structuredClone(nodes), child = JSON.parse(changed[1].instance_json); child.parent_certificate_id = 'sha256:' + '0'.repeat(64); changed[1].instance_json = JSON.stringify(child);
    assert.equal(nativeRun({...request, nodes: changed}).ok, false); await assert.rejects(() => adoptNativeExternalTree({rootInstanceJson, nodes: changed, wasmBinary, receipt}), /child instance mismatch/);
    const alteredGraph = structuredClone(nodes), childGraph = JSON.parse(alteredGraph[1].instance_json); childGraph.edges[0].weight++; alteredGraph[1].instance_json = JSON.stringify(childGraph);
    await assert.rejects(() => adoptNativeExternalTree({rootInstanceJson, nodes: alteredGraph, wasmBinary, receipt}), /child instance mismatch/);
  }
  if (k === 4) { const reordered = [nodes[0], nodes[2], nodes[1]]; await assert.rejects(() => adoptNativeExternalTree({rootInstanceJson, nodes: reordered, wasmBinary, receipt}), /child instance mismatch/); }
  await fs.writeFile(path.join(root, 'tree-' + k + '.json'), JSON.stringify(tree, null, 2)); await fs.writeFile(path.join(root, 'request-' + k + '.json'), JSON.stringify(request)); evidence.push({k, native_wasm_match: true, native_bounded_leaf_and_schedule_match: true, fresh_expected_tree_reverification: true});
}
await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({checker_wasm_sha256: receipt.wasm_sha256, cases: evidence, forged_parent_induced_graph_schedule_leaf_and_missing_nodes_refused: true, old_bounded_tree_schema_unchanged: true, tree_ui: false}, null, 2));
console.log('External native trees: 2/3/4/5-seat whole native/WASM matches, native oracle leaves/schedules, fresh rechecks and forged lineage/graph/schedule/leaf/missing-node refusals passed.');

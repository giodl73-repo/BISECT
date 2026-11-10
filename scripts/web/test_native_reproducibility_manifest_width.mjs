// Actual desktop Export versus direct and ordered-folder WASM package export.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import { instantiateEngine } from '../../web/lab/wasm-engine.js';

const arg = name => {
  const i = process.argv.indexOf(name);
  assert(i >= 0 && process.argv[i + 1], `Supply ${name}`);
  return process.argv[i + 1];
};
const cli = await fs.realpath(arg('--cli'));
const wasm = await fs.readFile(arg('--wasm'));
const output = path.resolve(arg('--output'));
const legacy = process.argv.includes('--expect-legacy-width-failure');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const engine = await instantiateEngine(wasm);
const label = 'WidthExport';
const document_json = await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan', 'utf8');
const manifest = {label: 'WidthRecorded', state_code: 'WA', year: '2020', chamber: 'house', num_districts: 2,
  population_source: 'fixture', partition_mode: 'fixture', seed: 42, binary_version: 'fixture', binary_sha256: '',
  binary_download_url: '', adjacency_file: 'fixture.adj.bin', adjacency_sha256: '', adjacency_build_command: '',
  adjacency_build_version: '', tiger_source_url: '', created_at: '2026-11-06T00:00:00Z',
  balance_tolerance_pct: 0.5, population_balance_valid: true};
const fields = ['num_districts', 'seats_per_district', 'total_seats', 'split_seeds', 'spectral_iters',
  'ilp_solve_report_count', 'flip_steps', 'flip_visited_count', 'flip_selected_plan_rank', 'burst_length',
  'n_bursts', 'selected_burst_idx', 'n_units'];
const max = '18446744073709551615';
const count = (field, token) => JSON.stringify({...manifest, [field]: '__COUNT__'}).replace('"__COUNT__"', token);
const cases = [['baseline', JSON.stringify(manifest), false], ...fields.map(field => ['wide-' + field, count(field, max), false]),
  ...[['negative', '-1'], ['fraction', '1.5'], ['overflow', '18446744073709551616'], ['string', '"2"']]
    .map(([id, token]) => ['invalid-' + id, count('num_districts', token), true])];
const formats = ['csv', 'reproducibility-package', 'gerrychain'];
const analysis_files = {'population.json': '{ "source": 9007199254740993 }\n'};
const observed = request => {
  try { return {result: engine.execute(request)}; }
  catch (error) { return {error: error.message}; }
};
async function filesAt(directory, prefix = '') {
  const found = {};
  for (const item of await fs.readdir(directory, {withFileTypes: true})) {
    const file = path.join(directory, item.name), name = prefix + item.name;
    if (item.isDirectory()) Object.assign(found, await filesAt(file, name + '/'));
    else found[name] = await fs.readFile(file, 'utf8');
  }
  return found;
}

await fs.mkdir(output);
const records = [];
for (const [id, manifest_json, invalid] of cases) {
  const root = path.join(output, id), base = path.join(root, 'inputs');
  const plan = path.join(base, 'v1/2020/plans', label), out = path.join(root, 'output');
  await fs.mkdir(path.join(plan, 'analysis'), {recursive: true});
  const sources = {'manifest.json': manifest_json, [label + '.rplan']: document_json, 'analysis/population.json': analysis_files['population.json']};
  for (const [name, text] of Object.entries(sources)) await fs.writeFile(path.join(plan, name), text);
  const argv = ['export', '--label', label, '--output-base', base, '--out', out, '--format', ...formats];
  const native = spawnSync(cli, argv, {cwd: root, encoding: 'utf8', windowsHide: true, timeout: 25000});
  assert.ifError(native.error);
  await fs.writeFile(path.join(root, 'native.stderr'), native.stderr);
  await fs.writeFile(path.join(root, 'native.stdout'), native.stdout);
  assert.equal(native.status, invalid ? 1 : 0, native.stderr);
  const nativeFiles = await filesAt(out);
  const requests = {
    direct: {operation: 'export-reproducibility-package', label, document_json, manifest_json, analysis_files},
    folder: {operation: 'export-native-plan-folder', input: {label, formats, files: Object.fromEntries(
      Object.entries(sources).map(([name, text]) => [name, Buffer.from(text).toString('base64')]))}},
  };
  const direct = observed(requests.direct), folder = observed(requests.folder);
  await fs.writeFile(path.join(root, 'requests.json'), JSON.stringify(requests));
  await fs.writeFile(path.join(root, 'result.json'), JSON.stringify({direct, folder}));
  const prefix = label + '_reproducibility/';
  const wasmRejected = invalid || (legacy && id.startsWith('wide-'));
  assert.ok(folder.result, folder.error);
  assert.equal(folder.result.selected_source, label + '.rplan');
  assert.deepEqual(folder.result.source_sha256, Object.fromEntries(Object.entries(sources).map(([k, v]) => [k, sha(v)])));
  assert.equal(folder.result.status, wasmRejected ? 'partial' : 'completed');
  assert.deepEqual(folder.result.stages.map(row => [row.format, row.status]), wasmRejected ?
    [[formats[0], 'completed'], [formats[1], 'failed']] : formats.map(format => [format, 'completed']));
  if (wasmRejected) {
    const diagnostic = legacy ? /expected usize/ : /expected u64/;
    assert.match(direct.error, diagnostic);
    assert.match(folder.result.stages[1].error, diagnostic);
    assert.deepEqual(folder.result.files, {[label + '_tracts.csv']: nativeFiles[label + '_tracts.csv']});
  } else {
    const packageFiles = Object.fromEntries(Object.entries(nativeFiles).filter(([k]) => k.startsWith(prefix))
      .map(([k, v]) => [k.slice(prefix.length), v]));
    assert.ok(direct.result, direct.error);
    assert.deepEqual(direct.result.files, packageFiles);
    assert.deepEqual(folder.result.files, nativeFiles);
    for (const result of [direct.result, folder.result]) {
      assert.deepEqual(result.file_sha256, Object.fromEntries(Object.entries(result.files).map(([k, v]) => [k, sha(v)])));
    }
  }
  if (!invalid) {
    assert.equal(nativeFiles[prefix + 'manifest.json'], manifest_json);
    assert.equal(nativeFiles[prefix + 'analysis/population.json'], analysis_files['population.json']);
    if (id === 'wide-num_districts') assert.match(nativeFiles[prefix + 'verify.sh'], new RegExp('--districts ' + max));
  } else {
    assert.deepEqual(Object.keys(nativeFiles), [label + '_tracts.csv']);
  }
  records.push({id, argv, native_exit: native.status, wasm_rejected: wasmRejected, native_outputs: Object.keys(nativeFiles).sort()});
}
const status = legacy ? 'reproduced-native64-reproducibility-width-gap' : 'passed-native-reproducibility-manifest-width';
await fs.writeFile(path.join(output, 'evidence.json'), JSON.stringify({status, native: {path: cli, sha256: sha(await fs.readFile(cli))},
  wasm_sha256: sha(wasm), records, full_parity: false}, null, 2));
console.log(`${status}: ${records.length} actual CLI, direct WASM and ordered folder cases.`);

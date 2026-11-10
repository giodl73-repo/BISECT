import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

// Run after prime-factor/partisan exposure is implemented. Retained native
// outputs come from actual State conversion/TSV loading and strict 3% checks.
const root = 'target/partisan-prime-reference';
const native = JSON.parse(await fs.readFile(root + '/evidence.json', 'utf8'));
const engine = await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let cases = 0, failures = 0, hostCases = 0;
for (const record of native.records) {
  const request = JSON.parse(await fs.readFile(root + '/engine-' + record.case + '.json', 'utf8'));
  let actual, error;
  try {actual = engine.execute(request);} catch (e) {error = e;}
  if(process.argv.includes('--host')){
    const host=spawnSync('target/release/examples/execute_request.exe',[root+'/engine-'+record.case+'.json'],{encoding:'utf8',maxBuffer:16*1024*1024});
    assert.equal(host.error,undefined);
    if(actual){assert.equal(host.status,0,host.stderr);assert.deepEqual(actual,JSON.parse(host.stdout));}
    else{assert.notEqual(host.status,0);assert(host.stderr.includes(error.message));}
    hostCases++;
  }
  if (record.status === 'native-rejected') {
    assert(error);
    const nativeError = await fs.readFile(root + '/error-' + record.case + '.txt', 'utf8');
    assert(nativeError.includes(error.message), error.message);
    failures++;
  } else {
    assert(actual, error?.message);
    const expected = JSON.parse(await fs.readFile(root + '/result-' + record.case + '.json', 'utf8'));
    assert.deepEqual(actual.assignments, expected.assignments);
    assert(Math.abs(actual.metrics.weighted_boundary - expected.weighted_boundary) < 1e-9);
    if (request.options.districts > 1) assert.deepEqual(actual.metrics.structure_evidence, expected.summary);
    else assert.equal(actual.metrics.structure_evidence, null);
    assert(actual.metrics.max_deviation_percent <= 3 + 1e-9);
  }
  cases++;
}
assert.equal(cases, 72); assert.equal(failures, native.native_failures);
await fs.writeFile(root + '/wasm-evidence.json', JSON.stringify({native_wasm_cases: cases, matching_failures: failures,
  exact_native_host_cases:hostCases,native_3_percent_limit_retained: true, factor_schedule_and_exact_optional_seed: true, wasm_sha256: engine.artifactIdentity.sha256}, null, 2));
console.log(`Partisan prime factor: ${cases} native State/TSV comparisons, ${failures} matching failures; native factor schedule, cut and exact optional seed retained.`);

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
import {runRoundingSat, validateSolverInput, sha256} from '../../web/lab/proof-solver-core.js';

const root = path.resolve('target/roundingsat-wasm-tests');
await fs.mkdir(root, {recursive: true});
const build = path.resolve('target/roundingsat-wasm');
const receipt = JSON.parse(await fs.readFile(path.join(build, 'build-receipt.json'), 'utf8'));
const wasmBinary = new Uint8Array(await fs.readFile(path.join(build, 'roundingsat.wasm')));
const {default: factory} = await import(pathToFileURL(path.join(build, 'roundingsat.mjs')).href);
assert.equal(receipt.clock, 'monotonic-elapsed-time');
const clockProbe = await factory({wasmBinary, noInitialRun: true, print: () => {}, printErr: () => {}});
const beforeClock = clockProbe._bisect_solver_elapsed_seconds();
await new Promise(resolve => setTimeout(resolve, 30));
assert.ok(clockProbe._bisect_solver_elapsed_seconds() - beforeClock >= 0.02, 'WASM timeout clock must advance.');
const checker = process.env.BISECT_VERIPB;
assert.ok(checker, 'Set BISECT_VERIPB to the independent native checker executable.');
const cases = [];
for (const name of ['population', 'compact-boundary']) cases.push({name: 'smoke-' + name, opb: await fs.readFile(`docs/examples/proof-toolchain-smoke/${name}.opb`, 'utf8'), expected: 'unsat'});
const project = JSON.parse(await fs.readFile('target/proof-models-wasm/models-2-full-42.bisect', 'utf8'));
for (const [name, opb] of Object.entries(project.result.files)) if (name.endsWith('.opb')) cases.push({name: 'fresh-' + name.replace('.opb', ''), opb, expected: 'unsat'});
cases.push({name: 'sat', opb: '* #variable= 1 #constraint= 1 #equal= 0 intsize= 128\n+1 x1 >= 1 ;\n', expected: 'sat'});
const evidence = [];
for (const item of cases) {
  const result = await runRoundingSat({opb: item.opb, seconds: 30, factory, wasmBinary, receipt});
  assert.equal(result.status, 'solver-reported-' + item.expected, item.name);
  assert.equal(result.proof_status, 'unverified');
  assert.equal(result.solver.program_name, 'roundingsat');
  assert.deepEqual(result.solver.arguments, ['--lp=0', '--proof-log=/proof.pbp', '--time-limit=30', '/input.opb']);
  const opbPath = path.join(root, item.name + '.opb'), proofPath = path.join(root, item.name + '.pbp');
  await fs.writeFile(opbPath, item.opb); await fs.writeFile(proofPath, result.proof);
  const checked = spawnSync(checker, [opbPath, proofPath], {encoding: 'utf8', timeout: 60000});
  await fs.writeFile(path.join(root, item.name + '-checker.log'), (checked.stdout ?? '') + (checked.stderr ?? ''));
  assert.equal(checked.status, 0, `${item.name}: ${checked.error ?? checked.stderr ?? checked.stdout}`);
  assert.match(checked.stdout, new RegExp(item.expected === 'unsat' ? 'UNSATISFIABLE' : 'SATISFIABLE'));
  const badPath = path.join(root, item.name + '-truncated.pbp');
  await fs.writeFile(badPath, result.proof.replace(/conclusion[^\n]*\nend pseudo-Boolean proof\s*$/, ''));
  const refused = spawnSync(checker, [opbPath, badPath], {encoding: 'utf8', timeout: 60000});
  assert.equal(refused.error, undefined);
  assert.notEqual(refused.status, 0, 'Truncated proof must be rejected.');
  evidence.push({name: item.name, status: result.status, opb_sha256: result.opb_sha256, proof_sha256: result.proof_sha256, independent_native_checker: true, truncated_proof_refused: true});
}
// Reject a valid UNSAT proof paired with a different satisfiable formula.
const mismatched = spawnSync(checker, [path.join(root, 'sat.opb'), path.join(root, 'smoke-population.pbp')], {encoding: 'utf8', timeout: 60000});
assert.equal(mismatched.error, undefined); assert.notEqual(mismatched.status, 0);
const invalidReference = path.join(root, 'invalid-reference.pbp');
await fs.writeFile(invalidReference, (await fs.readFile(path.join(root, 'smoke-population.pbp'), 'utf8')).replace(/conclusion UNSAT : \d+/, 'conclusion UNSAT : 999999999'));
const invalidChecked = spawnSync(checker, [path.join(root, 'smoke-population.opb'), invalidReference], {encoding: 'utf8', timeout: 60000});
assert.equal(invalidChecked.error, undefined); assert.notEqual(invalidChecked.status, 0);
// Pairwise pigeonhole encoding has no satisfying assignment and exercises the
// actual solver timeout, rather than merely inspecting the exported clock.
const pigeons = 25, holes = 24, clauses = [];
const variable = (pigeon, hole) => pigeon * holes + hole + 1;
for (let p = 0; p < pigeons; p++) clauses.push(Array.from({length: holes}, (_, h) => `+1 x${variable(p, h)}`).join(' ') + ' >= 1 ;');
for (let h = 0; h < holes; h++) for (let p = 0; p < pigeons; p++) for (let q = p + 1; q < pigeons; q++) clauses.push(`-1 x${variable(p, h)} -1 x${variable(q, h)} >= -1 ;`);
const timeoutModel = `* #variable= ${pigeons * holes} #constraint= ${clauses.length} #equal= 0 intsize= 128\n` + clauses.join('\n') + '\n';
const timeoutStart = performance.now();
const timeoutResult = await runRoundingSat({opb: timeoutModel, seconds: 1, factory, wasmBinary, receipt});
const timeoutElapsed = performance.now() - timeoutStart;
assert.equal(timeoutResult.status, 'solver-reported-time-limit');
assert.equal(timeoutResult.proof_status, 'unverified');
const limited=await runRoundingSat({opb:timeoutModel,seconds:30,factory,wasmBinary,receipt,proofByteLimit:16384});assert.equal(limited.status,'solver-proof-output-limit');assert.equal(limited.exit_code,null);assert.equal(limited.proof_complete,false);assert.equal(limited.proof_truncated,true);assert(Buffer.byteLength(limited.proof)<=16384);assert.equal(limited.proof_bytes,Buffer.byteLength(limited.proof));assert.equal(limited.proof_limit_enforced_during_write,true);
await fs.writeFile(path.join(root,'output-limited.opb'),timeoutModel);await fs.writeFile(path.join(root,'output-limited.pbp'),limited.proof);const limitedCheck=spawnSync(checker,[path.join(root,'output-limited.opb'),path.join(root,'output-limited.pbp')],{encoding:'utf8',timeout:60000});assert.notEqual(limitedCheck.status,0);
await assert.rejects(()=>runRoundingSat({opb:timeoutModel,factory,wasmBinary,receipt,proofByteLimit:64*1024*1024+1}),/budget/);
const freshAfterLimit=await runRoundingSat({opb:cases[0].opb,factory,wasmBinary,receipt});assert.equal(freshAfterLimit.status,'solver-reported-unsat');

assert.ok(timeoutElapsed >= 900 && timeoutElapsed < 15000, 'Actual solver must honor the elapsed-time limit.');
await assert.rejects(() => runRoundingSat({opb: cases[0].opb, factory, wasmBinary: new Uint8Array([0]), receipt}), /checksum/);
assert.throws(() => validateSolverInput('x'.repeat(8 * 1024 * 1024 + 1), 30), /8 MiB/);
assert.throws(() => validateSolverInput(cases[0].opb, 121), /120/);
await assert.rejects(() => runRoundingSat({opb: 'invalid model', factory, wasmBinary, receipt}), /exited/);
// Emscripten mirrors a native exit(1) onto Node's process.exitCode even when
// the test deliberately catches its ExitStatus. Browser workers have no process.
process.exitCode = 0;
const checkerIdentity = spawnSync(checker, ['--version'], {encoding: 'utf8'}).stdout.trim();
const checkerCommit = spawnSync('git', ['-C', 'target/proof-toolchain/veripb-current', 'rev-parse', 'HEAD'], {encoding: 'utf8'}).stdout.trim();
assert.equal(checkerCommit, 'a50f0483d65aa019918d03ccc590b71dc23c666c');
await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({solver_build: receipt, checker: checkerIdentity,
  checker_upstream_commit: checkerCommit, checker_executable_sha256: await sha256(await fs.readFile(checker)),
  cases: evidence, monotonic_clock_advances: true, proof_byte_limit_enforced_during_write:true, output_limit_stops_without_complete_proof:true, fresh_module_after_output_limit:true, actual_solver_timeout_ms: timeoutElapsed, mismatched_formula_refused: true, invalid_conclusion_reference_refused: true, wasm_checker: false, certification: false}, null, 2));
console.log(`RoundingSat WASM: ${cases.length} fresh-module solves independently checked, truncated proofs refused, input/checksum/parser refusals passed.`);

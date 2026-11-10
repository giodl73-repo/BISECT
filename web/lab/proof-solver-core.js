// External solver results are unverified until an independent checker accepts them.
export const MAX_OPB_BYTES = 8 * 1024 * 1024;
export const MAX_PROOF_BYTES = 64 * 1024 * 1024;
const encoder = new TextEncoder();
export async function sha256(bytes) {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), b => b.toString(16).padStart(2, '0')).join('');
}
export function validateSolverInput(opb, seconds) {
  if (typeof opb !== 'string' || !opb.trim() || encoder.encode(opb).length > MAX_OPB_BYTES || opb.includes('\0')) throw new Error('Choose a nonempty UTF-8 OPB model of at most 8 MiB.');
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 120) throw new Error('Time limit must be an integer from 1 to 120 seconds.');
}
export async function runRoundingSat({opb, seconds = 30, factory, wasmBinary, receipt, proofByteLimit = MAX_PROOF_BYTES}) {
  validateSolverInput(opb, seconds);
  if (!Number.isSafeInteger(proofByteLimit) || proofByteLimit < 1 || proofByteLimit > MAX_PROOF_BYTES) throw new Error('Proof byte budget must be 1 byte–64 MiB.');
  if (receipt?.schema !== 'bisect-roundingsat-wasm-build-v1' || receipt.commit !== 'd4edbf7908a9bb951fd181940919e0f3ac7ab1ee' || receipt.source_patches?.length !== 0 || receipt.lp !== false) throw new Error('Unsupported solver build receipt.');
  const solverSha = await sha256(wasmBinary);
  if (solverSha !== receipt.files?.['roundingsat.wasm']?.sha256 || wasmBinary.byteLength !== receipt.files?.['roundingsat.wasm']?.bytes) throw new Error('Solver WASM checksum mismatch.');
  let log = '', status = 'unknown';
  const capture = line => {
    if (line === 's UNSATISFIABLE') status = 'unsat';
    if (line === 's SATISFIABLE') status = 'sat';
    if (line === 's TIMELIMIT') status = 'time-limit';
    if (log.length < 1024 * 1024) log += String(line).slice(0, 8192) + '\n';
  };
  // Upstream has process-global solver state. Never reuse this module instance.
  const module = await factory({wasmBinary, thisProgram: 'roundingsat', noInitialRun: true, print: capture, printErr: capture});
  module.FS.writeFile('/input.opb', encoder.encode(opb));
  const args = ['--lp=0', '--proof-log=/proof.pbp', `--time-limit=${seconds}`, '/input.opb'];
  // MEMFS grows independently of WASM linear memory. Stop before a proof write
  // crosses its budget, not after the whole log has already accumulated.
  const proofLimit = new Error('Proof output byte budget reached.');
  const originalWrite = module.FS.write;
  module.FS.write = function(stream, buffer, offset, length, position, ...rest) {
    if (stream.path === '/proof.pbp' && (position ?? stream.position) + length > proofByteLimit) throw proofLimit;
    return originalWrite.call(this, stream, buffer, offset, length, position, ...rest);
  };
  let interrupted = false, exitCode = 0;
  // Emscripten prepends argv[0] in place. Preserve the recorded option list and
  // use a fixed program name rather than leaking a host script path into it.
  try { exitCode = module.callMain([...args]) ?? 0; }
  catch (error) {
    if (error === proofLimit) { interrupted = true; status = 'proof-output-limit'; exitCode = null; }
    else { if (error?.name !== 'ExitStatus' || !Number.isInteger(error.status)) throw error; exitCode = error.status; }
  } finally { module.FS.write = originalWrite; }
  // Upstream uses exit code 4 for its caught timeout interrupt.
  if (!interrupted && exitCode !== 0 && !(exitCode === 4 && status === 'time-limit')) throw new Error(`RoundingSat exited with ${exitCode}. ${log.slice(-1024)}`);
  const stat = module.FS.stat('/proof.pbp');
  if ((!stat.size && !interrupted) || stat.size > proofByteLimit) throw new Error('Missing proof log or proof exceeded its write budget.');
  const proof = module.FS.readFile('/proof.pbp', {encoding: 'utf8'});
  return {schema: 'bisect-external-proof-solver-v1', status: interrupted ? 'solver-proof-output-limit' : `solver-reported-${status}`, exit_code: exitCode, proof_status: 'unverified',
    proof_complete: !interrupted && ['unsat', 'sat'].includes(status), proof_truncated: interrupted, proof_bytes: stat.size, proof_byte_limit: proofByteLimit, proof_limit_enforced_during_write: true,
    opb_sha256: await sha256(encoder.encode(opb)), proof_sha256: await sha256(encoder.encode(proof)),
    solver: {name: 'RoundingSat', program_name: 'roundingsat', commit: receipt.commit, wasm_sha256: solverSha, arguments: args,
      integer_backend: 'boost-multiprecision', lp: false, seed: null},
    scope: 'One supplied OPB decision model. This result is not a checked proof or a district-plan optimality certificate.', proof, log};
}

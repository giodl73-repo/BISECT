import {MAX_OPB_BYTES, MAX_PROOF_BYTES} from './proof-solver-core.js';
const get = id => document.getElementById(id);
let model = null, proof = null, result = null, worker = null, timer = null, revision = 0;
const reads = {model: 0, proof: 0};
function stop() { worker?.terminate(); worker = null; clearTimeout(timer); timer = null; get('cancel').disabled = true; get('check').disabled = !(model && proof); }
function invalidate() { revision++; stop(); result = null; get('save').disabled = true; get('details').textContent = ''; }
async function select(kind) {
  invalidate();
  const token = ++reads[kind];
  if (kind === 'model') model = null; else proof = null;
  stop();
  const file = get(kind).files[0]; get(kind + '-name').textContent = file?.name ?? 'No file loaded.';
  get('status').textContent = 'Choose a model and proof. Selecting files does not execute the checker.';
  if (!file) return;
  try {
    if (file.size > (kind === 'model' ? MAX_OPB_BYTES : MAX_PROOF_BYTES)) throw new Error('File exceeds the ' + (kind === 'model' ? '3' : '64') + ' MiB limit.');
    const text = new TextDecoder('utf-8', {fatal: true}).decode(await file.arrayBuffer());
    if (token !== reads[kind]) return;
    if (!text.trim() || text.includes('\0')) throw new Error('A nonempty NUL-free UTF-8 file is required.');
    if (kind === 'model') model = text; else proof = text;
    stop(); get('status').textContent = model && proof ? 'Files loaded. Press Check proof to execute.' : 'Choose the remaining file.';
  } catch (error) { if (token === reads[kind]) get('status').textContent = error.message; }
}
get('model').onchange = () => select('model'); get('proof').onchange = () => select('proof');
get('require-unsat').onchange = () => { invalidate(); get('status').textContent = 'Conclusion requirement changed. Check again for a fresh receipt.'; };
get('check').onclick = () => {
  invalidate(); if (!(model && proof)) return;
  const current = revision;
  worker = new Worker(new URL('./proof-checker-worker.js', import.meta.url), {type: 'module'});
  get('check').disabled = true; get('cancel').disabled = false; get('status').textContent = 'Checking locally with VeriPB…';
  worker.onmessage = ({data}) => {
    if (current !== revision) return;
    stop();
    if (data.error) { get('status').textContent = 'Proof refused: ' + data.error; return; }
    result = data.result; get('status').textContent = ['SAT', 'UNSAT'].includes(result.conclusion) ? `Verified ${result.conclusion} for this model.` : `Proof checked. Recorded conclusion: ${result.conclusion}.`;
    get('details').textContent = JSON.stringify(result, null, 2); get('save').disabled = false;
  };
  worker.onerror = event => { if (current !== revision) return; stop(); get('status').textContent = 'Checker failed: ' + (event.message || 'No verified result.'); };
  timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Watchdog stopped the checker. No verified result.'; }, 60000);
  worker.postMessage({opb: model, proof, requireUnsat: get('require-unsat').checked});
};
get('cancel').onclick = () => { invalidate(); get('status').textContent = 'Cancelled. No checked receipt retained.'; };
get('save').onclick = () => {
  if (!result || worker) return;
  const url = URL.createObjectURL(new Blob([JSON.stringify(result, null, 2) + '\n'], {type: 'application/json'})), link = document.createElement('a');
  link.href = url; link.download = 'checked-proof.json'; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
};

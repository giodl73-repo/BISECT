import {MAX_OPB_BYTES, validateSolverInput} from './proof-solver-core.js';
const get = id => document.getElementById(id);
let opb = null, worker = null, result = null, timer = null, revision = 0;
function stop() { worker?.terminate(); worker = null; clearTimeout(timer); timer = null; get('cancel').disabled = true; get('run').disabled = !opb; }
function invalidate() { revision++; stop(); result = null; get('proof').disabled = get('receipt').disabled = true; get('details').textContent = ''; }
get('model').onchange = async () => {
  invalidate(); opb = null; get('run').disabled = true;
  const file = get('model').files[0], current = revision;
  get('filename').textContent = file?.name ?? 'No model loaded.';
  get('status').textContent = 'Choose an OPB model.';
  if (!file) return;
  try {
    if (file.size > MAX_OPB_BYTES) throw new Error('Model exceeds 8 MiB.');
    const text = new TextDecoder('utf-8', {fatal: true}).decode(await file.arrayBuffer());
    if (current !== revision) return;
    validateSolverInput(text, Number(get('seconds').value)); opb = text;
    get('run').disabled = false; get('status').textContent = 'Model loaded. Press Run solver to execute.';
  } catch (error) { if (current === revision) get('status').textContent = error.message; }
};
get('seconds').oninput = () => { invalidate(); get('status').textContent = opb ? 'Settings changed. Run again to generate a fresh proof log.' : 'Choose an OPB model.'; };
get('run').onclick = () => {
  invalidate();
  try { validateSolverInput(opb, Number(get('seconds').value)); }
  catch (error) { get('status').textContent = error.message; return; }
  const current = revision, seconds = Number(get('seconds').value);
  worker = new Worker(new URL('./proof-solver-worker.js', import.meta.url), {type: 'module'});
  get('run').disabled = true; get('cancel').disabled = false; get('status').textContent = 'Running RoundingSat locally…';
  worker.onmessage = ({data}) => {
    if (current !== revision) return;
    stop();
    if (data.error) { get('status').textContent = data.error; return; }
    result = data.result; get('status').textContent = `${result.status} · proof unverified`;
    get('details').textContent = `Model SHA-256: ${result.opb_sha256}\nProof SHA-256: ${result.proof_sha256}\nSeed: not used by this solver\n\n${result.log}`;
    get('proof').disabled = get('receipt').disabled = false;
  };
  worker.onerror = event => { if (current !== revision) return; stop(); get('status').textContent = event.message || 'Solver worker failed.'; };
  timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Elapsed-time watchdog stopped the worker. No verified result.'; }, (seconds + 30) * 1000);
  worker.postMessage({opb, seconds});
};
get('cancel').onclick = () => { invalidate(); get('status').textContent = 'Cancelled. No result retained.'; };
function download(name, text) { const url = URL.createObjectURL(new Blob([text], {type: 'text/plain;charset=utf-8'})), link = document.createElement('a'); link.href = url; link.download = name; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); }
get('proof').onclick = () => { if (result && !worker) download('proof.pbp', result.proof); };
get('receipt').onclick = () => { if (result && !worker) { const {proof, log, ...receipt} = result; download('proof-run.json', JSON.stringify(receipt, null, 2) + '\n'); } };

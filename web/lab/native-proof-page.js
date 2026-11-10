import {MAX_OPB_BYTES} from './proof-solver-core.js';
import {nativeProofRunZip, validateNativeProofSources, recordedDiscoverySeed} from './native-proof-pipeline.js';
import {MAX_PROOF_SESSION_BYTES, encodeProofSession} from './native-proof-session.js';
const get = id => document.getElementById(id);
let instanceJson = null, discoveryJson = null, result = null, importedRun = null, worker = null, timer = null, revision = 0;
let nativeCertificate = null;
const reads = {instance: 0, discovery: 0};
function stop() { worker?.terminate(); worker = null; clearTimeout(timer); timer = null; get('cancel').disabled = true; get('run').disabled = get('save-session').disabled = !(instanceJson && discoveryJson); get('verify-session').disabled = importedRun?.status !== 'all-native-decisions-checked'; }
function invalidate() { revision++; importedRun = null; nativeCertificate = null; stop(); result = null; get('save').disabled = get('package').disabled = get('issue-certificate').disabled = get('save-certificate').disabled = true; get('details').textContent = ''; get('decisions').replaceChildren(); }
async function select(kind) {
  get('open-session').value = '';
  invalidate(); const token = ++reads[kind]; if (kind === 'instance') instanceJson = null; else { discoveryJson = null; get('seed').textContent = 'Recorded discovery seed: unavailable. RoundingSat does not use a selectable seed.'; } stop();
  const file = get(kind).files[0]; get(kind + '-name').textContent = file?.name ?? 'No file loaded.';
  get('status').textContent = 'Choose instance and discovery files. Selecting files does not execute the tools.';
  if (!file) return;
  try {
    if (file.size > 3 * 1024 * 1024) throw new Error('Source exceeds 3 MiB.');
    const text = new TextDecoder('utf-8', {fatal: true}).decode(await file.arrayBuffer()); if (token !== reads[kind]) return;
    if (!text.trim() || text.includes('\0')) throw new Error('Nonempty NUL-free UTF-8 JSON required.');
    if (kind === 'instance') instanceJson = text; else {
      const seed = recordedDiscoverySeed(text);
      discoveryJson = text; get('seed').textContent = `Recorded discovery seed: ${seed ?? 'unavailable'}. RoundingSat does not use a selectable seed.`;
    }
    stop(); get('status').textContent = instanceJson && discoveryJson ? 'Sources loaded. Press Generate and check to execute.' : 'Choose the remaining source.';
  } catch (error) { if (token === reads[kind]) get('status').textContent = error.message; }
}
get('instance').onchange = () => select('instance'); get('discovery').onchange = () => select('discovery');
get('seconds').oninput = () => { invalidate(); get('status').textContent = 'Time limit changed. Run again for fresh evidence.'; };
function start(saved = null) {
  invalidate(); const seconds = Number(get('seconds').value);
  try { validateNativeProofSources(instanceJson, discoveryJson, seconds); } catch (error) { get('status').textContent = error.message; return; }
  const current = revision;
  const watchdog = milliseconds => { clearTimeout(timer); timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Elapsed watchdog stopped the worker. No checked package.'; }, milliseconds); };
  worker = new Worker(new URL('./native-proof-worker.js', import.meta.url), {type: 'module'});
  get('run').disabled = get('save-session').disabled = get('verify-session').disabled = true; get('cancel').disabled = false; get('status').textContent = 'Compiling native decision models…';
  worker.onmessage = ({data}) => {
    if (current !== revision) return;
    if (data.progress) { const p = data.progress; watchdog(p.phase === 'solving' ? (seconds + 30) * 1000 : 60000); get('status').textContent = `${p.phase}${p.stem ? ': ' + p.stem : ''} · ${p.completed}/${p.total} UNSAT proofs checked`; return; }
    stop();
    if (data.error) { get('status').textContent = 'Proof run failed: ' + data.error; return; }
    result = data.result; if (saved) { importedRun = saved; get('verify-session').disabled = false; }
    get('status').textContent = result.status === 'all-native-decisions-rechecked' ? `All ${result.decisions.length} saved proofs freshly rechecked.` : result.status === 'all-native-decisions-checked' ? `All ${result.decisions.length} native decision proofs checked.` : result.status === 'checked-counterexample' ? 'Checked SAT counterexample. The incumbent remains unproved.' : 'Proof run incomplete. The incumbent remains unproved.';
    for (const decision of result.decisions) { const row = document.createElement('p'); row.textContent = `${decision.stem}: ${decision.proof_status} · ${decision.request_id}`; get('decisions').append(row); }
    const {files, ...receipt} = result; get('details').textContent = JSON.stringify(receipt, null, 2);
    get('save').disabled = false; get('package').disabled = result.status !== 'all-native-decisions-checked';
    get('issue-certificate').disabled = !['all-native-decisions-checked', 'all-native-decisions-rechecked'].includes(result.status);
  };
  worker.onerror = event => { if (current !== revision) return; stop(); get('status').textContent = 'Proof worker failed: ' + (event.message || 'No checked package.'); };
  watchdog(60000);
  worker.postMessage({instanceJson, discoveryJson, seconds, ...(saved ? {mode: 'verify-saved', saved} : {})});
}
get('run').onclick = () => start();
get('verify-session').onclick = () => { if (importedRun && !worker) start(importedRun); };
get('issue-certificate').onclick = () => {
  if (worker || !['all-native-decisions-checked', 'all-native-decisions-rechecked'].includes(result?.status)) return;
  const saved = result.files ? result : importedRun;
  if (!saved) return;
  const current = ++revision; nativeCertificate = null;
  get('save-certificate').disabled = get('issue-certificate').disabled = get('run').disabled = get('save-session').disabled = get('verify-session').disabled = true;
  get('save').disabled = get('package').disabled = true; get('cancel').disabled = false;
  get('status').textContent = 'Native Rust is recompiling and checking every proof for certificate issuance…';
  worker = new Worker(new URL('./native-proof-worker.js', import.meta.url), {type: 'module'});
  timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Certificate watchdog stopped the worker. No certificate.'; }, 60000);
  worker.onmessage = ({data}) => {
    if (current !== revision) return;
    stop();
    if (data.error) { result = null; get('status').textContent = 'Native certificate refused: ' + data.error; return; }
    nativeCertificate = data.certificate;
    get('status').textContent = 'Native external split certificate issued: ' + nativeCertificate.certificate_id;
    get('details').textContent = JSON.stringify(nativeCertificate, null, 2);
    get('save-certificate').disabled = false;
    get('save').disabled = false; get('package').disabled = result.status !== 'all-native-decisions-checked';
    get('issue-certificate').disabled = false;
  };
  worker.onerror = event => { if (current !== revision) return; invalidate(); get('status').textContent = event.message || 'Certificate worker failed.'; };
  worker.postMessage({mode: 'issue-certificate', instanceJson, discoveryJson, seconds: Number(get('seconds').value), saved});
};
get('open-session').onchange = async () => {
  invalidate(); reads.instance++; reads.discovery++;
  instanceJson = discoveryJson = null; stop();
  get('instance-name').textContent = 'No instance loaded.'; get('discovery-name').textContent = 'No discovery loaded.';
  get('seed').textContent = 'Recorded discovery seed: unavailable. RoundingSat does not use a selectable seed.';
  const file = get('open-session').files[0], current = revision; if (!file) return;
  get('status').textContent = 'Reading proof session…';
  try {
    if (file.size > MAX_PROOF_SESSION_BYTES) throw new Error('Proof session exceeds 192 MiB.');
    const bytes = await file.arrayBuffer(); if (current !== revision) return;
    worker = new Worker(new URL('./native-proof-session-worker.js', import.meta.url), {type: 'module'});
    get('cancel').disabled = false;
    timer = setTimeout(() => { if (current !== revision) return; invalidate(); get('status').textContent = 'Session parsing watchdog stopped the worker.'; }, 30000);
    worker.onmessage = ({data}) => {
      if (current !== revision) return;
      stop();
      if (data.error) { get('status').textContent = data.error; return; }
      const session = data.session;
      instanceJson = session.instance_json; discoveryJson = session.discovery_json; importedRun = session.result;
      get('seconds').value = String(session.seconds); get('instance').value = get('discovery').value = '';
      get('instance-name').textContent = 'Restored native split instance'; get('discovery-name').textContent = 'Restored native discovery';
      get('seed').textContent = `Recorded discovery seed: ${recordedDiscoverySeed(discoveryJson) ?? 'unavailable'}. RoundingSat does not use a selectable seed.`;
      stop(); get('status').textContent = 'Imported session is unverified. Press Verify saved proofs or generate a fresh run.';
    };
    worker.onerror = event => { if (current !== revision) return; invalidate(); get('status').textContent = event.message || 'Session parser failed.'; };
    worker.postMessage({bytes}, [bytes]);
  } catch (error) { if (current === revision) get('status').textContent = error.message; }
};
get('cancel').onclick = () => { invalidate(); get('status').textContent = 'Cancelled. No run result retained.'; };
function download(name, bytes, type) { const url = URL.createObjectURL(new Blob([bytes], {type})), link = document.createElement('a'); link.href = url; link.download = name; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); }
get('save-certificate').onclick = () => { if (nativeCertificate && !worker) download('native-external-split-certificate.json', JSON.stringify(nativeCertificate, null, 2) + '\n', 'application/json'); };
get('save-session').onclick = () => {
  if (worker || !(instanceJson && discoveryJson)) return;
  try { download('proof-session.native-proof.json', encodeProofSession({schema: 'bisect-native-proof-session-v1', instance_json: instanceJson, discovery_json: discoveryJson, seconds: Number(get('seconds').value), result: result?.files ? result : importedRun}), 'application/json'); }
  catch (error) { get('status').textContent = error.message; }
};
get('save').onclick = () => { if (!result || worker) return; const {files, ...receipt} = result; download('native-proof-run.json', JSON.stringify(receipt, null, 2) + '\n', 'application/json'); };
get('package').onclick = async () => {
  if (!result || worker) return; const selected = result, current = revision;
  try { const zip = await nativeProofRunZip(selected); if (current !== revision || result !== selected || worker) return; download('checked-native-proofs.zip', zip, 'application/zip'); }
  catch (error) { if (current === revision) get('status').textContent = error.message; }
};

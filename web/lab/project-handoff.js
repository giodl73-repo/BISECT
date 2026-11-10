import {MAX_PROJECT_BYTES, validateProject} from './project.js';

const TOKEN = /^[a-f0-9]{32}$/;
const TTL = 15 * 60 * 1000;
const KEY = 'pending';

// One bounded, short-lived transfer per origin, separate from saved projects.
function transaction(action) {
  return new Promise((resolve, reject) => {
    let db, tx, value, settled = false;
    const finish = error => {
      if (settled) return;
      settled = true; clearTimeout(timer); db?.close();
      error ? reject(error) : resolve(value);
    };
    const timer = setTimeout(() => {
      tx?.abort(); finish(new Error('Project transfer timed out. Download the project instead.'));
    }, 10000);
    let open;
    try { open = indexedDB.open('bisect-project-handoff', 1); }
    catch (error) { finish(error); return; }
    open.onupgradeneeded = () => open.result.createObjectStore('transfers');
    open.onerror = () => finish(open.error);
    open.onblocked = () => finish(new Error('Project transfer storage is unavailable. Download the project instead.'));
    open.onsuccess = () => {
      db = open.result;
      if (settled) { db.close(); return; }
      try {
        tx = db.transaction('transfers', 'readwrite');
        tx.oncomplete = () => finish();
        tx.onabort = tx.onerror = () => finish(tx.error || new Error('Project transfer failed.'));
        action(tx.objectStore('transfers'), result => { value = result; });
      } catch (error) { tx?.abort(); finish(error); }
    };
  });
}

export async function queueProjectHandoff(project) {
  validateProject(project);
  const blob = new Blob([JSON.stringify(project)], {type: 'application/json'});
  if (blob.size > MAX_PROJECT_BYTES) throw new Error('Project exceeds the 25 MB transfer limit. Download a workbench archive instead.');
  const token = Array.from(crypto.getRandomValues(new Uint8Array(16)), byte => byte.toString(16).padStart(2, '0')).join('');
  await transaction((store, done) => { store.put({token, expires: Date.now() + TTL, blob}, KEY); done(token); });
  return token;
}

export function handoffToken(hash) {
  const match = /^#project=([a-f0-9]{32})$/.exec(hash || '');
  return match?.[1] || null;
}

export async function takeProjectHandoff(token) {
  if (!TOKEN.test(token || '')) throw new Error('Invalid project transfer token.');
  return transaction((store, done) => {
    const request = store.get(KEY);
    request.onsuccess = () => {
      const record = request.result;
      const valid = record && Number.isSafeInteger(record.expires) && record.expires > Date.now() && record.expires <= Date.now() + TTL && record.blob instanceof Blob && record.blob.size <= MAX_PROJECT_BYTES;
      if (record && !valid) store.delete(KEY);
      if (!valid || record.token !== token) { done(null); return; }
      // Consume before parsing: replay and malformed transfers cannot linger.
      store.delete(KEY); done(record.blob);
    };
  });
}

// Exercise the actual packaged replay worker over localhost workerd, online and offline.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [address, packageDir, nativeDir, fixtureDir, output, profile] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
assert(['core', 'c'].includes(profile));
const withC = profile === 'c';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const read = async file => JSON.parse(await fs.readFile(file, 'utf8'));
const nativePath = path.join(nativeDir, 'evidence.json');
const native = await read(nativePath);
const fixture = path.join(fixtureDir, 'actual-ri.bisect');
const prior = await read(path.join(fixtureDir, 'evidence.json'));
const expected = await read(path.join(fixtureDir, 'current-wasm.json'));
const hostingPath = path.join(packageDir, 'hosting-manifest.json');
const offlinePath = path.join(packageDir, 'assets', 'offline-manifest.json');
const hosting = await read(hostingPath), offline = await read(offlinePath);
const engine = hosting.gzip_routes['toolkit/bisect_wasm.wasm'].decoded;
assert.deepEqual(engine, offline.assets['toolkit/bisect_wasm.wasm']);
assert.equal(native.status, 'passed-actual-state-and-replay-native-on-original-ri-tracts');
assert.equal(native.effective_backend, 'c-ffi');
assert.equal(native.seed, '42');
assert.equal(prior.actual_native_cli, native.native.sha256);
assert.equal(prior.original_manifest_sha256, native.original_manifest_sha256);
assert.deepEqual(expected.comparison.regenerated_assignments, native.assignments);
const project = await read(fixture);
assert.equal(sha(Buffer.from(project.files.packageVerificationInputs.manifest.base64, 'base64')), native.original_manifest_sha256);
const sidecarPrefix = '/toolkit/c-metis/';
const sidecarUrls = ['metis-sidecar.json', 'metis-sidecar.mjs', 'metis-sidecar.wasm'].map(name => sidecarPrefix + name);
for (const name of sidecarUrls) assert.equal(offline.shell.includes(name.slice(1)), withC);
const sidecarDirs = withC ? ['toolkit/c-metis/'] : [];
if (withC && offline.shell.includes('c-metis/metis-sidecar.json')) sidecarDirs.push('c-metis/');
const noticePaths = sidecarDirs.flatMap(dir => ['LICENSE-METIS.txt', 'LICENSE-GKlib.txt'].map(name => dir + name));
for (const name of noticePaths) assert(offline.shell.includes(name));
if (withC) assert.equal(prior.wasm_sha256, engine.sha256);

const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES, 'package.json'))('playwright');
const browser = await chromium.launch({headless: true,
  executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args: ['--no-first-run', '--disable-background-networking', '--disable-extensions']});
const timeout = 120000;
try {
  const context = await browser.newContext({acceptDownloads: true});
  const errors = [], sidecarRequests = [], sidecarResponses = [];
  context.on('request', request => {
    const url = new URL(request.url());
    if (url.origin === origin && sidecarUrls.includes(url.pathname)) sidecarRequests.push(url.pathname);
  });
  context.on('response', response => {
    const url = new URL(response.url());
    if (url.origin === origin && sidecarUrls.includes(url.pathname)) {
      sidecarResponses.push({path: url.pathname, status: response.status(), cached: response.fromServiceWorker()});
    }
  });
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await context.addInitScript(() => {
    window.engineRuns = []; window.engineReplies = []; window.pickerCalls = 0;
    window.showDirectoryPicker = () => { pickerCalls++; throw Error('Recorded replay must not request folder authority.'); };
    const BrowserWorker = Worker;
    window.Worker = class extends BrowserWorker {
      constructor(url, ...args) {
        super(url, ...args);
        this.engine = new URL(url, location.href).pathname === '/toolkit/wasm-worker.js';
        if (this.engine) this.addEventListener('message', ({data}) => {
          if (['result', 'error'].includes(data?.type)) engineReplies.push(data);
        });
      }
      postMessage(message, ...rest) {
        if (this.engine && message?.type === 'run') engineRuns.push(message);
        return super.postMessage(message, ...rest);
      }
    };
  });
  let page;
  const ui = id => page.locator('#' + id);
  const count = () => page.evaluate(() => engineRuns.length);
  async function visit() {
    page = await context.newPage();
    page.on('pageerror', e => errors.push(e.message));
    await page.goto(origin + '/toolkit/');
    await page.waitForFunction(hash => !document.querySelector('#execute').disabled &&
      document.querySelector('#runtime-artifact').textContent.includes(hash), engine.sha256, {timeout});
  }
  async function open(filename) {
    const runs = await count(), fetches = sidecarRequests.length;
    await openWorkbenchProject(page, filename, {timeout});
    assert.equal(await count(), runs);
    assert.equal(sidecarRequests.length, fetches);
    assert(!(await ui('c-metis-runtime').isChecked()));
    assert(await ui('recorded-replay-plan').isDisabled());
  }
  async function run(selected) {
    const i = await count();
    await ui('execute').click();
    await page.waitForFunction(i => engineReplies.length > i && !document.querySelector('#execute').disabled, i, {timeout});
    const {message, reply} = await page.evaluate(i => ({message: engineRuns[i], reply: engineReplies[i]}), i);
    assert.equal(message.cMetis ?? false, selected);
    assert.equal(reply.type, 'result', reply.error);
    return reply.result;
  }
  async function replay(disconnected) {
    await visit();
    const requests = sidecarRequests.length, responses = sidecarResponses.length;
    await open(fixture);
    const refusal = await run(false);
    assert.equal(refusal.status, 'preparation-failed');
    assert.equal(refusal.execution_performed, false);
    assert.equal(refusal.recorded_paths_opened, false);
    assert.equal(sidecarRequests.length, requests);
    if (!withC) {
      assert(await ui('c-metis-runtime-control').isHidden());
      return {refusal: refusal.error, executed: false};
    }
    await ui('c-metis-runtime').check();
    assert.equal(await count(), 1);
    assert.equal(sidecarRequests.length, requests);
    const result = await run(true);
    assert.deepEqual(result, expected);
    assert.equal(result.execution_performed, true);
    assert.equal(result.recorded_paths_opened, false);
    assert.deepEqual(result.comparison.regenerated_assignments, native.assignments);
    assert.deepEqual(result.runtime_artifacts.metis_sidecar, prior.sidecar_identity);
    const delivered = sidecarResponses.slice(responses);
    assert.deepEqual(delivered.map(r => r.path).sort(), [...sidecarUrls].sort());
    assert(delivered.every(r => r.status === 200 && r.cached === disconnected));
    const exports = {};
    for (const [control, field] of [['recorded-replay-plan', 'document_json'], ['recorded-replay-context', 'context_json']]) {
      const waiting = page.waitForEvent('download', {timeout});
      await ui(control).click();
      const bytes = await fs.readFile(await (await waiting).path());
      assert.equal(bytes.toString('utf8'), expected.replay.export[field]);
      exports[field] = sha(bytes);
    }
    await open(fixture);
    assert.deepEqual(await run(false), refusal);
    assert.equal(await page.evaluate(() => pickerCalls), 0);
    return {refusal: refusal.error, executed: true, delivered, exports, runtime: result.runtime_artifacts};
  }
  const online = await replay(false);
  await ui('offline-package').evaluate(el => el.open = true);
  await ui('offline-install').click({timeout});
  await page.waitForFunction(() => !document.querySelector('#offline-install').disabled, null, {timeout});
  const installed = await ui('offline-status').textContent();
  assert.match(installed, /^Installed .*application only/);
  await page.waitForFunction(() => !!navigator.serviceWorker.controller, null, {timeout});
  await page.close();
  await context.setOffline(true);
  const disconnected = await replay(true);
  assert.equal(online.executed, disconnected.executed);
  assert.deepEqual(online.exports, disconnected.exports);
  for (const name of noticePaths) {
    const decoded = await page.evaluate(async name => {
      const response = await fetch('/' + name, {redirect: 'error'});
      if (!response.ok) throw Error('Offline notice unavailable: ' + name);
      const bytes = await response.arrayBuffer();
      const hash = await crypto.subtle.digest('SHA-256', bytes);
      return {bytes: bytes.byteLength, sha256: [...new Uint8Array(hash)].map(x => x.toString(16).padStart(2, '0')).join('')};
    }, name);
    assert.deepEqual(decoded, offline.assets[name]);
  }
  assert.deepEqual(errors, []);
  await fs.writeFile(output, JSON.stringify({status: 'passed-local-cloudflare-recorded-c-replay',
    profile, browser: browser.version(), units: Object.keys(native.assignments).length, seed: native.seed,
    native: native.native.sha256, native_evidence: sha(await fs.readFile(nativePath)),
    fixture: sha(await fs.readFile(fixture)), hosting_manifest: sha(await fs.readFile(hostingPath)),
    offline_manifest: sha(await fs.readFile(offlinePath)), engine, online, disconnected, installed,
    verified_offline_notices: noticePaths, deployed: false,
  }, null, 2) + '\n', {flag: 'wx'});
} finally {
  await browser.close();
}

// Verify a locally served Wrangler package using a previously verified browser recipe.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import http from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {gunzipSync} from 'node:zlib';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [address, packageDir, fixtureDir, receipt, ...scopes] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
const read = async name => JSON.parse(await fs.readFile(name, 'utf8'));
const manifest = await read(path.join(packageDir, 'hosting-manifest.json'));
const offlineManifest = await read(path.join(packageDir, 'assets', 'offline-manifest.json'));
assert(scopes.every(scope => Object.hasOwn(offlineManifest.states, scope)));
assert.equal(scopes.length, new Set(scopes).size);
const reference = await read(path.join(fixtureDir, 'evidence.json'));
const fixture = 'disconnected-one-based';
const expectedZip = reference.cases.find(c => c.id === fixture).zip_sha256;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const pin = bytes => ({bytes: bytes.length, sha256: sha(bytes)});
function raw(url, options) {
  return new Promise((resolve, reject) => {
    const req = http.request(new URL(url, origin), options, response => {
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('error', reject);
      response.on('end', () => resolve({status: response.statusCode, headers: response.headers, body: Buffer.concat(chunks)}));
    });
    req.setTimeout(60000, () => req.destroy(new Error('Asset response timed out')));
    req.on('error', reject);
    req.end();
  });
}
const routes = {};
for (const [name, asset] of Object.entries(manifest.gzip_routes)) {
  assert.equal(asset.decoded.sha256, reference.wasm_sha256);
  const headers = {'Accept-Encoding': 'gzip'};
  const response = await raw('/' + name + '?probe=1', {headers: {...headers, Range: 'bytes=0-99'}});
  assert.equal(response.status, 200);
  assert.equal(response.headers['content-type'], 'application/wasm');
  assert.equal(response.headers['content-encoding'], 'gzip');
  assert.deepEqual(pin(response.body), asset.gzip);
  assert.deepEqual(pin(gunzipSync(response.body)), asset.decoded);
  const head = await raw('/' + name, {method: 'HEAD', headers});
  assert.equal(head.status, 200);
  assert.equal(head.body.length, 0);
  assert.equal((await raw('/' + name, {method: 'POST', headers})).status, 405);
  routes[name] = asset;
}
assert.equal(Object.keys(routes).length, 2);
assert.equal(new Set(Object.values(routes).map(r => r.asset)).size, 1);
assert.equal((await raw('/api/source')).status, 400);
assert.equal((await raw('/missing-test-asset.bin')).status, 404);

const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES, 'package.json'))('playwright');
const browser = await chromium.launch({headless: true,
  executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args: ['--no-first-run', '--disable-background-networking', '--disable-extensions']});
const timeout = 240000;
try {
  const context = await browser.newContext({acceptDownloads: true});
  await context.addInitScript(() => {
    window.executions = [];
    window.stateRequests = [];
    window.stateReplies = [];
    window.showDirectoryPicker = async () => { throw Error('Local analysis requested a directory'); };
    const BrowserWorker = Worker;
    window.Worker = class extends BrowserWorker {
      constructor(url, ...rest) {
        super(url, ...rest);
        if (new URL(url, location.href).pathname === '/wasm-worker.js') {
          this.stateEngine = true;
          this.addEventListener('message', ({data}) => {
            if (data?.type === 'result') stateReplies.push(structuredClone(data.result));
          });
        }
      }
      postMessage(message, ...rest) {
        if (message?.type === 'run') {
          executions.push(message.request.operation);
          if (this.stateEngine) stateRequests.push(structuredClone(message.request));
        }
        return super.postMessage(message, ...rest);
      }
    };
  });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  async function openAndRun(workbenchPath) {
    await page.goto(origin + workbenchPath);
    await page.waitForFunction(() => !document.querySelector('#execute').disabled, null, {timeout});
    assert.deepEqual(await page.evaluate(() => executions), []);
    await openWorkbenchProject(page, path.join(fixtureDir, fixture + '.bisect'), {timeout});
    assert.deepEqual(await page.evaluate(() => executions), []);
    await page.locator('#execute').click({timeout});
    await page.waitForFunction(() => !document.querySelector('#execute').disabled &&
      !document.querySelector('#download-label-analysis').disabled, null, {timeout});
    assert.deepEqual(await page.evaluate(() => executions), ['run-label-analysis']);
    const waiting = page.waitForEvent('download', {timeout});
    await page.locator('#download-label-analysis').click({timeout});
    const download = await waiting;
    const actual = sha(await fs.readFile(await download.path()));
    assert.equal(actual, expectedZip);
    return actual;
  }
  const online = await openAndRun(scopes.length ? '/toolkit/' : '/');
  async function stateRuns() {
    await page.goto(origin + '/');
    await page.waitForFunction(() => document.querySelector('#connection').textContent.includes('WASM'), null, {timeout});
    const rows = {};
    const seed = '18446744073709551615';
    for (const [index, scope] of scopes.entries()) {
      const [state, year] = scope.split(':');
      await page.locator('#year').selectOption(year);
      await page.locator('#state').selectOption(state);
      await page.locator('#seed').fill(seed);
      await page.locator('#run').click({timeout});
      await page.waitForFunction(n => stateReplies.length === n &&
        document.querySelector('#run-status').textContent === 'completed' &&
        document.querySelector('#map-empty').hidden && !document.querySelector('#download-map').disabled,
      index + 1, {timeout});
      const [request, reply] = await page.evaluate(i => [stateRequests[i], stateReplies[i]], index);
      assert(request && reply);
      assert.equal(request.options.seed, seed);
      const waiting = page.waitForEvent('download', {timeout});
      await page.locator('#download-map').click({timeout});
      const map = await fs.readFile(await (await waiting).path());
      const geometry = JSON.parse(map);
      assert.equal(geometry.state, state);
      assert.equal(geometry.year, year);
      assert(geometry.features.length > 0);
      const rendered = Object.fromEntries(geometry.features.map(({properties: p}) => [p.geoid, p.district]));
      assert.equal(Object.keys(rendered).length, geometry.features.length);
      assert.deepEqual(rendered, reply.assignments);
      rows[scope] = {request, reply, geojson: {...pin(map), features: geometry.features.length}, seed};
    }
    return rows;
  }
  const onlineStates = scopes.length ? await stateRuns() : {};
  await page.locator('#offline-package').evaluate(el => el.open = true);
  if (scopes.length) await page.locator('#offline-states').selectOption(scopes);
  await page.locator('#offline-install').click({timeout});
  await page.waitForFunction(() => !document.querySelector('#offline-install').disabled, null, {timeout});
  const installed = await page.locator('#offline-status').textContent();
  assert.match(installed, /^Installed /);
  for (const scope of scopes.length ? scopes : ['application only']) assert(installed.includes(scope), installed);
  await page.waitForFunction(() => !!navigator.serviceWorker.controller);
  const cached = await page.evaluate(async () => {
    const names = await caches.keys();
    const cache = await caches.open(names.find(name => !name.endsWith('pointer')));
    return (await cache.keys()).map(request => new URL(request.url).pathname.slice(1)).sort();
  });
  const expectedCache = new Set([...offlineManifest.shell,
    ...scopes.flatMap(scope => offlineManifest.states[scope]), 'offline-manifest.json']);
  assert.deepEqual(cached, [...expectedCache].sort());
  await context.setOffline(true);
  const offline = await openAndRun('/toolkit/');
  const offlineStates = scopes.length ? await stateRuns() : {};
  assert.deepEqual(offlineStates, onlineStates);
  const stateRunsEvidence = Object.fromEntries(Object.entries(offlineStates).map(([scope, row]) => [scope, {
    seed: row.seed, request_sha256: sha(JSON.stringify(row.request)), reply_sha256: sha(JSON.stringify(row.reply)),
    geojson: row.geojson, installed_assets: offlineManifest.states[scope],
  }]));
  assert.deepEqual(errors, []);
  await fs.writeFile(receipt, JSON.stringify({status: 'passed-local-workerd-browser-transport',
    browser: browser.version(), routes, online, offline, installed, state_runs: stateRunsEvidence,
    catalog_scope_count: Object.keys(offlineManifest.states).length, verified_cache_entries: cached.length,
    deployed: false}, null, 2) + '\n', {flag: 'wx'});
} finally {
  await browser.close();
}

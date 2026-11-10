// Full retained 2020 national run through the actual locally packaged application.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';

const [address, packageDir, destination] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const assets = path.join(packageDir, 'assets');
const manifestBytes = await fs.readFile(path.join(assets, 'offline-manifest.json'));
const manifest = JSON.parse(manifestBytes);
const catalogBytes = await fs.readFile(path.join(assets, 'catalog.json'));
const catalog = JSON.parse(catalogBytes);
const scopes = Object.keys(manifest.states).sort();
assert.equal(scopes.length, 50);
assert(scopes.every(scope => /^[A-Z]{2}:2020$/.test(scope)));
const codes = scopes.map(scope => scope.slice(0, 2));
const wanted = new Set([...manifest.shell, ...scopes.flatMap(scope => manifest.states[scope]), 'offline-manifest.json']);
const expectedUnits = {};
for (const scope of scopes) {
  const [graph, geometry] = manifest.states[scope];
  assert.equal(graph, catalog.graphs[scope].graph_ref);
  assert.equal(geometry, catalog.geometries[scope]);
  const graphBytes = await fs.readFile(path.join(assets, graph));
  assert.equal(sha(graphBytes), catalog.assets[graph]);
  expectedUnits[scope.slice(0, 2)] = JSON.parse(graphBytes).geoids.length;
}
const sum = values => values.reduce((a, b) => a + b, 0);
assert.equal(sum(Object.values(expectedUnits)), 84208);
await fs.mkdir(destination);
const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES, 'package.json'))('playwright');
const browser = await chromium.launch({headless: true,
  executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args: ['--no-first-run', '--disable-background-networking', '--disable-extensions']});
const timeout = 360000;
let page;
try {
  const context = await browser.newContext({acceptDownloads: true});
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  const errors = [];
  await context.addInitScript(() => {
    window.nationalRuns = []; window.nationalReplies = [];
    const BrowserWorker = Worker;
    window.Worker = class extends BrowserWorker {
      constructor(url, ...args) {
        super(url, ...args);
        if (new URL(url, location.href).pathname !== '/wasm-worker.js') return;
        this.engine = true;
        this.addEventListener('message', ({data}) => {
          if (data?.type === 'error') nationalReplies.push({error: data.error});
          if (data?.type === 'result') nationalReplies.push({assignments: data.result.assignments,
            metrics: {districts: data.result.metrics.districts, contiguous: data.result.metrics.contiguous,
              balanced: data.result.metrics.within_requested_tolerance}, runtime: data.runtimeArtifact});
        });
      }
      postMessage(message, ...args) {
        if (this.engine && message?.type === 'run') nationalRuns.push({state: message.request.graph.state,
          year: message.request.graph.year, units: message.request.graph.geoids.length, options: message.request.options});
        return super.postMessage(message, ...args);
      }
    };
  });
  async function visit() {
    page = await context.newPage();
    page.on('pageerror', e => errors.push(e.message));
    await page.goto(origin + '/');
    await page.waitForFunction(() => document.querySelector('#connection').textContent.includes('WASM'), null, {timeout});
  }
  const el = id => page.locator('#' + id);
  await visit();
  await el('offline-package').evaluate(section => section.open = true);
  await el('offline-states').selectOption(scopes);
  await el('offline-install').click({timeout});
  await page.waitForFunction(() => !document.querySelector('#offline-install').disabled, null, {timeout});
  const installed = await el('offline-status').textContent();
  assert.match(installed, /^Installed /);
  for (const scope of scopes) assert(installed.includes(scope));
  await page.waitForFunction(() => !!navigator.serviceWorker.controller, null, {timeout});
  const cached = await page.evaluate(async () => {
    const key = (await caches.keys()).find(name => !name.endsWith('pointer'));
    return (await (await caches.open(key)).keys()).map(req => new URL(req.url).pathname.slice(1)).sort();
  });
  assert.deepEqual(cached, [...wanted].sort());
  await page.close();
  await context.setOffline(true);
  await visit();
  console.log('Verified offline install for all 50 state packs. Starting national run.');
  await page.locator('[data-mode="national"]').click();
  await el('year').selectOption('2020');
  await el('chamber').selectOption('congressional');
  // The published Laboratory applies these defaults. Some shells hide the
  // underlying selects behind presets, so confirm rather than force them.
  for (const [control, expected] of [['structure','standard-bisect'], ['weights','geographic'], ['search','single']]) {
    assert.equal(await el(control).inputValue(), expected);
  }
  await page.locator('[data-batch="all"]').click();
  const seed = '18446744073709551615';
  await el('seed').fill(seed);
  await el('run').click({timeout});
  await page.waitForFunction(() => ['completed','partial','failed','cancelled'].includes(
    document.querySelector('#run-status').textContent), null, {timeout});
  assert.equal(await el('run-status').textContent(), 'completed', await el('log').textContent());
  const {runs, replies} = await page.evaluate(() => ({runs: nationalRuns, replies: nationalReplies}));
  assert.deepEqual(runs.map(r => r.state).sort(), codes);
  assert.equal(replies.length, 50);
  const states = {};
  for (const [i, req] of runs.entries()) {
    const result = replies[i];
    assert.equal(req.year, '2020');
    assert.equal(req.options.seed, seed);
    assert.equal(req.units, expectedUnits[req.state]);
    assert.equal(Object.keys(result.assignments).length, req.units);
    assert.equal(result.runtime.sha256, catalog.wasm_sha256);
    states[req.state] = {units: req.units, ...result.metrics, assignments_sha256: sha(JSON.stringify(result.assignments))};
  }
  assert.equal(sum(Object.values(states).map(state => state.districts)), 435);
  await el('national-map-view').click();
  await page.waitForFunction(() => document.querySelector('#map-loading').hidden &&
    !document.querySelector('#download-map').disabled && document.querySelector('#map-caption').textContent.includes('Albers projection'), null, {timeout});
  assert.doesNotMatch(await el('map-caption').textContent(), /unavailable:/);
  async function download(id) {
    const waiting = page.waitForEvent('download', {timeout});
    await el(id).click();
    return await (await waiting).path();
  }
  const mapBytes = await fs.readFile(await download('download-map'));
  const map = JSON.parse(mapBytes);
  assert.equal(map.state, 'US'); assert.equal(map.year, '2020'); assert.equal(map.features.length, 84208);
  const assignments = Object.fromEntries(runs.map((request, i) => [request.state, replies[i].assignments]));
  const seen = new Set();
  for (const {properties: p} of map.features) {
    assert(!seen.has(p.geoid)); seen.add(p.geoid);
    assert(Object.hasOwn(assignments[p.state], p.geoid));
    assert.equal(p.district, assignments[p.state][p.geoid]);
  }
  const projectPath = path.resolve(destination, 'national.bisect');
  await fs.copyFile(await download('save-lab-project'), projectPath);
  const project = JSON.parse(await fs.readFile(projectPath));
  assert.deepEqual(project.assignments, assignments);
  await el('open-lab-project').setInputFiles(projectPath);
  await page.waitForFunction(() => document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'), null, {timeout});
  assert.equal(await page.evaluate(() => nationalRuns.length), 50);
  assert.deepEqual(errors, []);
  await fs.writeFile(path.join(destination, 'evidence.json'), JSON.stringify({status: 'passed-local-cloudflare-offline-national-run',
    browser: browser.version(), seed, installed, cached_assets: cached.length, manifest_sha256: sha(manifestBytes),
    catalog_sha256: sha(catalogBytes), wasm_sha256: catalog.wasm_sha256, states, units: map.features.length,
    districts: 435, geojson: {bytes: mapBytes.length, sha256: sha(mapBytes)},
    project_sha256: sha(await fs.readFile(projectPath)), open_inert: true, deployed: false}, null, 2) + '\n', {flag: 'wx'});
} catch (error) {
  const ui = await page?.evaluate(() => ({connection: document.querySelector('#connection')?.textContent,
    run: document.querySelector('#run-status')?.textContent, project: document.querySelector('#lab-project-status')?.textContent,
    offline: document.querySelector('#offline-status')?.textContent, runs: window.nationalRuns?.length,
    replies: window.nationalReplies?.length})).catch(() => null);
  await fs.writeFile(path.join(destination, 'failure.json'), JSON.stringify({failure: String(error.stack), ui}, null, 2) + '\n');
  throw error;
} finally {
  await browser.close();
}

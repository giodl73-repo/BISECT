// Reopen a verified national project, lose one map request, and retry the full map.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createReadStream} from 'node:fs';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';

const [address, packageDir, referenceDir, receipt] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
const read = async file => JSON.parse(await fs.readFile(file));
const catalog = await read(path.join(packageDir, 'assets', 'catalog.json'));
const reference = await read(path.join(referenceDir, 'evidence.json'));
assert.equal(reference.status, 'passed-local-cloudflare-offline-national-run');
assert.equal(reference.wasm_sha256, catalog.wasm_sha256);
const project = path.resolve(referenceDir, 'national.bisect');
assert.equal(createHash('sha256').update(await fs.readFile(project)).digest('hex'), reference.project_sha256);
const state = 'UT', asset = '/' + catalog.geometries[`${state}:2020`];
const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES, 'package.json'))('playwright');
const browser = await chromium.launch({headless: true,
  executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args: ['--no-first-run','--disable-background-networking','--disable-extensions']});
const timeout = 180000;
try {
  const context = await browser.newContext({acceptDownloads: true});
  let targetRequests = 0;
  await context.route('**/*', route => {
    const url = new URL(route.request().url());
    if (url.origin !== origin) return route.abort();
    if (url.pathname === asset && ++targetRequests === 1) return route.abort('failed');
    return route.continue();
  });
  await context.addInitScript(() => {
    window.runs = 0;
    const BrowserWorker = Worker;
    window.Worker = class extends BrowserWorker {
      postMessage(message,...args) {
        if (message?.type === 'run') runs++;
        return super.postMessage(message,...args);
      }
    };
  });
  const page = await context.newPage(), warnings = [], errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {if (message.type() === 'warning') warnings.push(message.text());});
  await page.goto(origin + '/');
  await page.waitForFunction(() => document.querySelector('#connection').textContent.includes('WASM'), null, {timeout});
  await page.locator('#open-lab-project').setInputFiles(project);
  await page.waitForFunction(() => document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'), null, {timeout});
  assert.equal(await page.evaluate(() => runs), 0);
  assert.equal(targetRequests, 0);
  async function loadMap() {
    await page.locator('#national-map-view').click();
    await page.waitForFunction(() => document.querySelector('#map-loading').hidden &&
      !document.querySelector('#download-map').disabled, null, {timeout});
    return await page.locator('#map-caption').textContent();
  }
  assert.match(await loadMap(), /unavailable: UT$/);
  assert(warnings.some(message => message.includes('National map geometry unavailable for UT')));
  assert.equal(targetRequests, 1);
  await page.locator('#national-view').click();
  assert.doesNotMatch(await loadMap(), /unavailable:/);
  assert.equal(targetRequests, 2);
  const waiting = page.waitForEvent('download', {timeout});
  await page.locator('#download-map').click({timeout});
  const file = await (await waiting).path();
  const hash = createHash('sha256'); let bytes = 0;
  for await (const chunk of createReadStream(file)) {hash.update(chunk);bytes += chunk.length;}
  const map = {bytes, sha256: hash.digest('hex')};
  assert.deepEqual(map, reference.geojson);
  assert.equal(await page.evaluate(() => runs), 0);
  assert.deepEqual(errors, []);
  await fs.writeFile(receipt, JSON.stringify({status:'passed-national-map-transient-fetch-recovery', state,
    targetRequests, warnings, map, open_and_map_retry_engine_runs:0, browser:browser.version(), deployed:false}, null, 2) + '\n', {flag:'wx'});
} finally {
  await browser.close();
}

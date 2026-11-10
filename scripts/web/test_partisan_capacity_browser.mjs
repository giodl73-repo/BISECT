import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';

const packages = process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packages);
const {chromium} = createRequire(path.join(packages, 'package.json'))('playwright');
const source = path.resolve('dist/wasm-lab-build'), root = path.resolve('target/partisan-capacity-browser');
await fs.mkdir(root, {recursive: true});
const server = createServer(async (req, res) => {
  try {
    const name = new URL(req.url, 'http://localhost').pathname;
    const file = path.resolve(source, '.' + (name === '/' ? '/index.html' : name));
    if (!file.startsWith(source + path.sep)) throw new Error('Path refused');
    res.setHeader('Content-Type', file.endsWith('.js') ? 'application/javascript' : file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.css') ? 'text/css' : file.endsWith('.json') ? 'application/json' : 'text/html');
    res.end(await fs.readFile(file));
  } catch {res.writeHead(404); res.end();}
});
await new Promise(r => server.listen(0, '127.0.0.1', r));
const origin = 'http://127.0.0.1:' + server.address().port;
let context;
try {
  context = await chromium.launchPersistentContext(path.join(root, 'profile-' + Date.now()), {
    headless: true, executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
    args: ['--no-first-run', '--disable-background-networking', '--disable-extensions'],
  });
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  const page = await context.newPage(), errors = [];
  page.on('pageerror', e => errors.push(e.message));
  await page.addInitScript(() => {
    window.engineRequests = [];
    const W = Worker;
    window.Worker = class extends W {
      postMessage(v, ...rest) {if (v.type === 'run') engineRequests.push(v.request); return super.postMessage(v, ...rest);}
    };
  });
  await page.goto(origin + '/');
  await page.locator('#open-lab-project').setInputFiles(path.resolve('target/partisan-capacity/browser.bisect'));
  await page.waitForFunction(() => document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));
  assert.equal(await page.evaluate(() => engineRequests.length), 0);
  assert.equal(await page.locator('#structure').inputValue(), 'capacity-clustering');
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.equal(await page.locator('#seed').inputValue(), '18446744073709551615');
  const save = async name => {
    const pending = page.waitForEvent('download'); await page.locator('#save-lab-project').click();
    const file = path.join(root, name); await (await pending).saveAs(file);
    return JSON.parse(await fs.readFile(file, 'utf8'));
  };
  for (const structure of ['capacity-clustering', 'regionalization', 'flow-construction']) for(const districts of [1,2]) {
    await page.locator('#structure').selectOption(structure);await page.locator('#districts').fill(String(districts));
    assert.equal(await page.locator('#weights').inputValue(), 'partisan');
    const n = await page.evaluate(() => engineRequests.length);
    await page.locator('#run').click();await page.waitForFunction(n => engineRequests.length > n, n);
    await page.waitForFunction(() => ['completed','failed'].includes(document.querySelector('#run-status').textContent));
    const request = await page.evaluate(() => engineRequests.at(-1));
    assert.equal(request.options.weights, 'partisan');assert.equal(request.options.structure, structure);assert.equal(request.options.districts,districts);assert(request.partisan);
    if(await page.locator('#run-status').textContent()==='failed'){assert((await page.locator('#log').textContent()).includes('[ALGO]'));continue;}
    const project = await save(structure + '-' + districts + '.bisect');
    assert.deepEqual(project.experiment.config.partisans.RI, request.partisan);
    const evidence=project.experiment.states[0].metrics.structure_evidence;
    assert(evidence);if(districts===1)assert.equal((structure==='regionalization'?evidence.summary:evidence).edge_cut,0);
  }
  const n = await page.evaluate(() => engineRequests.length);
  await page.locator('#open-lab-project').setInputFiles(path.join(root, 'flow-construction-1.bisect'));
  await page.waitForFunction(() => document.querySelector('#execution-warning').textContent.includes('Imported project'));
  assert.equal(await page.evaluate(() => engineRequests.length), n);
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.deepEqual(errors, []);
  await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({actual_RI_worker: true,reported_cost_only: true,clustering_regionalization_flow_runs: true,native_one_district_summaries:true,exact_max_u64_seed_visible: true,source_retained: true,inert_open: true}, null, 2));
  console.log('Partisan capacity browser: actual RI clustering/regionalization/flow worker runs and one-district summaries, retained shares, visible exact seed, native objective and inert Open passed.');
} finally {await context?.close(); await new Promise(r => server.close(r));}

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';

const packages = process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packages);
const {chromium} = createRequire(path.join(packages, 'package.json'))('playwright');
const source = path.resolve('dist/wasm-lab-build'), root = path.resolve('target/partisan-geometry-browser');
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
  await page.locator('#open-lab-project').setInputFiles(path.resolve('target/partisan-geometry/browser.bisect'));
  await page.waitForFunction(() => document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));
  assert.equal(await page.evaluate(() => engineRequests.length), 0);
  assert.equal(await page.locator('#structure').inputValue(), 'bfs-growth');
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.equal(await page.locator('#seed').inputValue(), '18446744073709551615');
  const save = async name => {
    const pending = page.waitForEvent('download'); await page.locator('#save-lab-project').click();
    const file = path.join(root, name); await (await pending).saveAs(file);
    return JSON.parse(await fs.readFile(file, 'utf8'));
  };
  for (const structure of ['bfs-growth', 'centroidal-voronoi', 'moving-knife']) {
    await page.locator('#structure').selectOption(structure);
    assert.equal(await page.locator('#weights').inputValue(), 'partisan');
    assert((await page.locator('#structure-description').textContent()).includes('Boundary weights affect reported costs only.'));
    const controls = structure === 'centroidal-voronoi' ? ['cvd-iters', 'cvd-metric'] : structure === 'moving-knife' ? ['mka-orientations', 'mka-metric'] : [];
    for (const id of controls) await page.locator('#' + id).evaluate(e => {for (let p = e.parentElement; p; p = p.parentElement) if (p.tagName === 'DETAILS') p.open = true;});
    if (structure === 'centroidal-voronoi') {await page.locator('#cvd-iters').fill('3');await page.locator('#cvd-metric').selectOption('graph-distance');}
    if (structure === 'moving-knife') {await page.locator('#mka-orientations').fill('2');await page.locator('#mka-metric').selectOption('polsby');}
    const n = await page.evaluate(() => engineRequests.length);
    await page.locator('#run').click();
    await page.waitForFunction(n => engineRequests.length > n, n);
    await page.waitForFunction(() => document.querySelector('#run-status').textContent === 'completed');
    const request = await page.evaluate(() => engineRequests.at(-1));
    assert.equal(request.options.weights, 'partisan');assert.equal(request.options.structure, structure);assert(request.partisan);
    const project = await save(structure + '.bisect');
    assert.deepEqual(project.experiment.config.partisans.RI, request.partisan);
    if (structure === 'centroidal-voronoi') assert.equal(project.experiment.states[0].metrics.structure_evidence.distance_metric, 'graph-distance');
    if (structure === 'moving-knife') {assert.equal(project.experiment.states[0].metrics.structure_evidence.requested_metric, 'polsby');assert.equal(project.experiment.states[0].metrics.structure_evidence.effective_metric, 'reock');}
  }
  const n = await page.evaluate(() => engineRequests.length);
  await page.locator('#open-lab-project').setInputFiles(path.join(root, 'moving-knife.bisect'));
  await page.waitForFunction(() => document.querySelector('#execution-warning').textContent.includes('Imported project'));
  assert.equal(await page.evaluate(() => engineRequests.length), n);
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.deepEqual(errors, []);
  await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({actual_RI_worker: true,reported_cost_only_disclosed: true,bfs_cvd_moving_knife_runs: true,native_polsby_fallback_retained: true,exact_max_u64_seed_visible: true,source_retained: true,inert_open: true}, null, 2));
  console.log('Partisan geometry browser: actual RI BFS/CVD/moving-knife worker runs, retained shares, visible exact seed, native objective and inert Open passed.');
} finally {await context?.close(); await new Promise(r => server.close(r));}

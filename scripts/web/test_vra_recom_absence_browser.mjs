import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';

const packages = process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packages);
const {chromium} = createRequire(path.join(packages, 'package.json'))('playwright');
const source = path.resolve('dist/wasm-lab-build'), root = path.resolve('target/vra-recom-absence-browser');
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
  await page.locator('#open-lab-project').setInputFiles(path.resolve('target/vra-recom-absence/browser.bisect'));
  await page.waitForFunction(() => document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));
  assert.equal(await page.evaluate(() => engineRequests.length), 0);
  assert.equal(await page.locator('#structure').inputValue(), 'standard-bisect');
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.equal(await page.locator('#seed').inputValue(), '18446744073709551615');assert.equal(await page.locator('#seed').isDisabled(),false);
  const save = async name => {
    const pending = page.waitForEvent('download'); await page.locator('#save-lab-project').click();
    const file = path.join(root, name); await (await pending).saveAs(file);
    return JSON.parse(await fs.readFile(file, 'utf8'));
  };
  let runs=0;
  for(const search of ['vra-recom']){
    await page.locator('#search').selectOption(search);
    assert.equal(await page.locator('#weights').inputValue(),'partisan');
    assert.equal(await page.locator('#vra-absence').isChecked(),true);assert((await page.locator('#demographic-help').textContent()).includes('zero fractions'));
    const before=await page.evaluate(()=>engineRequests.length);
    await page.locator('#run').click();await page.waitForFunction(n=>engineRequests.length>n,before);
    await page.waitForFunction(()=>document.querySelector('#run-status').textContent==='completed');
    const request=await page.evaluate(()=>engineRequests.at(-1));
    assert.equal(request.options.weights,'partisan');assert.equal(request.options.search,search);assert(request.partisan);
    const project=await save(search+'.bisect');assert.deepEqual(project.experiment.config.partisans[request.graph.state],request.partisan);
    assert.equal(project.experiment.states[0].metrics.structure_evidence.threshold,request.options.vra_threshold);assert.equal(request.options.vra_absent_file_fallback,true);assert.equal(Object.hasOwn(request,'demographics'),false);assert.equal(Object.hasOwn(project.experiment.config,'demographics'),false);assert.equal(project.experiment.states[0].metrics.structure_evidence.source_policy,'explicit-native-absent-file-zero-fractions');runs++;
  }
  await fs.copyFile(path.join(root,'vra-recom.bisect'),path.join(root,'bisect.bisect'));
  const n = await page.evaluate(() => engineRequests.length);
  await page.locator('#open-lab-project').setInputFiles(path.join(root, 'bisect.bisect'));
  await page.waitForFunction(() => document.querySelector('#execution-warning').textContent.includes('Imported project'));
  assert.equal(await page.evaluate(() => engineRequests.length), n);
  assert.equal(await page.locator('#weights').inputValue(), 'partisan');
  assert.deepEqual(errors, []);
  await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({actual_state_worker_runs:runs,source_retained:true,exact_max_u64_seed_visible:true,inert_open:true},null,2));
  console.log('VRA ReCom explicit absence browser: one actual-state Worker runs, retained shares, visible exact seed and inert Open passed.');
} finally {await context?.close(); await new Promise(r => server.close(r));}

// Uses a private Chrome profile and local server, never the user's browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
const packageRoot = process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert.ok(packageRoot, 'Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium} = createRequire(path.join(packageRoot, 'package.json'))('playwright');
const root = path.resolve('target/roundingsat-browser'), source = path.resolve('web/lab'), assets = path.resolve('target/roundingsat-wasm');
await fs.mkdir(root, {recursive: true});
const server = createServer(async (req, res) => {
  try {
    const name = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    const base = name.startsWith('/roundingsat/') ? assets : source;
    const file = path.resolve(base, '.' + (base === assets ? name.slice('/roundingsat'.length) : name));
    if (!file.startsWith(base + path.sep)) throw new Error('Path refused');
    res.setHeader('Content-Type', /\.(mjs|js)$/.test(file) ? 'application/javascript' : file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.json') ? 'application/json' : 'text/html');
    res.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'");
    res.end(await fs.readFile(file));
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = 'http://127.0.0.1:' + server.address().port;
let context;
try {
  context = await chromium.launchPersistentContext(path.join(root, 'profile-' + Date.now()), {headless: true, executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe', args: ['--disable-extensions', '--disable-background-networking', '--no-first-run']});
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  const page = await context.newPage(), errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(() => { window.solves = 0; const Original = Worker; window.Worker = class extends Original { postMessage(data, ...rest) { window.solves++; return super.postMessage(data, ...rest); } }; });
  await page.goto(origin + '/proof-solver.html');
  const evidence = [];
  for (const name of ['smoke-population', 'smoke-compact-boundary', 'fresh-01-population', 'sat']) {
    const model = path.resolve('target/roundingsat-wasm-tests', name + '.opb');
    const before = await page.evaluate(() => window.solves);
    await page.locator('#model').setInputFiles(model);
    await page.waitForFunction(() => !document.querySelector('#run').disabled);
    assert.equal(await page.evaluate(() => window.solves), before);
    assert.equal(await page.locator('#proof').isDisabled(), true);
    await page.locator('#run').click();
    await page.waitForFunction(() => !document.querySelector('#proof').disabled, {timeout: 60000});
    assert.match(await page.locator('#status').innerText(), /proof unverified/);
    const proofWait = page.waitForEvent('download'); await page.locator('#proof').click();
    const proofPath = path.join(root, name + '.pbp'); await (await proofWait).saveAs(proofPath);
    assert.equal(await fs.readFile(proofPath, 'utf8'), await fs.readFile(path.resolve('target/roundingsat-wasm-tests', name + '.pbp'), 'utf8'));
    const receiptWait = page.waitForEvent('download'); await page.locator('#receipt').click();
    const receiptPath = path.join(root, name + '.json'); await (await receiptWait).saveAs(receiptPath);
    const receipt = JSON.parse(await fs.readFile(receiptPath, 'utf8'));
    assert.equal(receipt.proof_status, 'unverified'); assert.equal(receipt.solver.seed, null);
    const checked = spawnSync(process.env.BISECT_VERIPB, [model, proofPath], {encoding: 'utf8', timeout: 60000});
    assert.equal(checked.status, 0, checked.stderr);
    evidence.push({name, status: receipt.status, proof_sha256: receipt.proof_sha256, independently_checked: true});
  }
  await page.locator('#seconds').fill('5'); assert.equal(await page.locator('#proof').isDisabled(), true);
  await page.evaluate(() => { document.querySelector('#run').click(); document.querySelector('#cancel').click(); });
  assert.equal(await page.locator('#proof').isDisabled(), true); assert.equal(await page.locator('#cancel').isDisabled(), true);
  assert.match(await page.locator('#status').innerText(), /Cancelled/);
  await page.locator('#run').click(); await page.waitForFunction(() => !document.querySelector('#proof').disabled);
  assert.match(await page.locator('#status').innerText(), /solver-reported-sat/);
  await page.screenshot({path: path.join(root, 'preview.png'), fullPage: true});
  assert.deepEqual(errors, []);
  await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({cases: evidence, inert_file_selection: true, settings_freshness: true, cancel_terminates_worker: true, fresh_run_after_cancel: true, strict_script_csp: true, wasm_checker: false}, null, 2));
  console.log('RoundingSat Chrome: four independently checked proof downloads, inert selection, fresh settings, cancellation, fresh run after cancel and CSP passed.');
} finally { await context?.close(); await new Promise(resolve => server.close(resolve)); }

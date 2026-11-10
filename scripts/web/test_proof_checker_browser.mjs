// Own test server/profile only; never attach to the user's browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot = process.env.BISECT_PLAYWRIGHT_PACKAGES; assert.ok(packageRoot, 'Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium} = createRequire(path.join(packageRoot, 'package.json'))('playwright');
const root = path.resolve('target/proof-checker-browser'), source = path.resolve('web/lab'), assets = path.resolve('target/proof-checker-wasm-build/wasm32-unknown-unknown/release');
await fs.mkdir(root, {recursive: true});
const server = createServer(async (req, res) => {
  try {
    const name = decodeURIComponent(new URL(req.url, 'http://localhost').pathname), base = name.startsWith('/proof-checker/') ? assets : source;
    const file = path.resolve(base, '.' + (base === assets ? name.slice('/proof-checker'.length) : name));
    if (!file.startsWith(base + path.sep)) throw new Error('Path refused');
    res.setHeader('Content-Type', file.endsWith('.js') ? 'application/javascript' : file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.json') ? 'application/json' : 'text/html');
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
  const page = await context.newPage(), errors = []; page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(() => { window.checks = 0; const Original = Worker; window.Worker = class extends Original { postMessage(data, ...rest) { window.checks++; return super.postMessage(data, ...rest); } }; });
  await page.goto(origin + '/proof-checker.html');
  const evidence = [];
  for (const name of ['smoke-population', 'fresh-02-boundary', 'sat']) {
    const before = await page.evaluate(() => window.checks);
    await page.locator('#model').setInputFiles(path.resolve('target/roundingsat-wasm-tests', name + '.opb'));
    await page.locator('#proof').setInputFiles(path.resolve('target/roundingsat-wasm-tests', name + '.pbp'));
    await page.waitForFunction(() => !document.querySelector('#check').disabled);
    assert.equal(await page.evaluate(() => window.checks), before);
    assert.equal(await page.locator('#save').isDisabled(), true);
    await page.locator('#check').click();
    if (name === 'sat') {
      await page.waitForFunction(() => document.querySelector('#status').textContent.startsWith('Proof refused:'));
      assert.equal(await page.locator('#save').isDisabled(), true);
      await page.locator('#require-unsat').uncheck(); await page.locator('#check').click();
    }
    await page.waitForFunction(() => !document.querySelector('#save').disabled, null, {timeout: 60000});
    assert.equal(await page.locator('#status').innerText(), `Verified ${name === 'sat' ? 'SAT' : 'UNSAT'} for this model.`);
    const wait = page.waitForEvent('download'); await page.locator('#save').click();
    const downloaded = path.join(root, name + '.json'); await (await wait).saveAs(downloaded);
    assert.deepEqual(JSON.parse(await fs.readFile(downloaded, 'utf8')), JSON.parse(await fs.readFile(path.resolve('target/proof-checker-tests', name + '.json'), 'utf8')));
    evidence.push({name, exact_native_receipt: true});
  }
  await page.locator('#require-unsat').check(); assert.equal(await page.locator('#save').isDisabled(), true);
  await page.locator('#model').setInputFiles(path.resolve('target/roundingsat-wasm-tests/smoke-population.opb'));
  await page.locator('#proof').setInputFiles(path.resolve('target/roundingsat-wasm-tests/smoke-population-truncated.pbp'));
  await page.waitForFunction(() => !document.querySelector('#check').disabled); await page.locator('#check').click();
  await page.waitForFunction(() => document.querySelector('#status').textContent.startsWith('Proof refused:'));
  assert.equal(await page.locator('#save').isDisabled(), true);
  await page.locator('#proof').setInputFiles(path.resolve('target/roundingsat-wasm-tests/smoke-population.pbp'));
  await page.waitForFunction(() => !document.querySelector('#check').disabled);
  await page.evaluate(() => { document.querySelector('#check').click(); document.querySelector('#cancel').click(); });
  assert.equal(await page.locator('#save').isDisabled(), true); assert.match(await page.locator('#status').innerText(), /Cancelled/);
  await page.locator('#check').click(); await page.waitForFunction(() => !document.querySelector('#save').disabled);
  assert.equal(await page.locator('#status').innerText(), 'Verified UNSAT for this model.');
  await page.screenshot({path: path.join(root, 'preview.png'), fullPage: true}); assert.deepEqual(errors, []);
  await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({cases: evidence, inert_selection: true, strict_unsat_policy: true, truncated_proof_refused: true, settings_freshness: true, cancel: true, fresh_run_after_cancel: true, strict_script_csp: true, full_plan_certification: false}, null, 2));
  console.log('VeriPB Chrome: three exact native receipts, inert file selection, strict UNSAT policy, truncated proof refusal, export freshness, cancellation, fresh verification after cancel and CSP passed.');
} finally { await context?.close(); await new Promise(resolve => server.close(resolve)); }

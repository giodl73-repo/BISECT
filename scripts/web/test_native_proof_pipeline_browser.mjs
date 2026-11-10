// Isolated browser profile/server; never touch the user's browser session.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages = process.env.BISECT_PLAYWRIGHT_PACKAGES; assert.ok(packages, 'Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium} = createRequire(path.join(packages, 'package.json'))('playwright');
const root = path.resolve('target/native-proof-pipeline-browser'), source = path.resolve('web/lab'), solver = path.resolve('target/roundingsat-wasm'), checker = path.resolve('target/proof-checker-wasm-build/wasm32-unknown-unknown/release'), engine = path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
await fs.mkdir(root, {recursive: true});
const server = createServer(async (req, res) => {
  try {
    const name = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    const base = name.startsWith('/roundingsat/') ? solver : name.startsWith('/proof-checker/') ? checker : source;
    const relative = base === solver ? name.slice('/roundingsat'.length) : base === checker ? name.slice('/proof-checker'.length) : name;
    const file = name === '/bisect_wasm.wasm' ? engine : path.resolve(base, '.' + relative);
    if (file !== engine && !file.startsWith(base + path.sep)) throw new Error('Path refused');
    res.setHeader('Content-Type', /\.(mjs|js)$/.test(file) ? 'application/javascript' : file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.json') ? 'application/json' : 'text/html');
    res.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'");
    res.end(await fs.readFile(file));
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = 'http://127.0.0.1:' + server.address().port; let context;
try {
  context = await chromium.launchPersistentContext(path.join(root, 'profile-' + Date.now()), {headless: true, executablePath: process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe', args: ['--no-first-run', '--disable-extensions', '--disable-background-networking']});
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  const page = await context.newPage(), errors = []; page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(() => { window.proofRuns = 0; window.nativeCalls = 0; const Original = Worker; window.Worker = class extends Original { constructor(...args) { super(...args); this.nativeTool = String(args[0]).endsWith('/native-proof-worker.js'); this.addEventListener('message', event => { if (event.data?.result) window.latestProofRun = event.data.result; }); } postMessage(data, ...rest) { window.proofRuns++; if (this.nativeTool) window.nativeCalls++; return super.postMessage(data, ...rest); } }; });
  await page.goto(origin + '/native-proof.html'); await page.locator('#seconds').fill('3');
  const cases = ['two-optimal', 'three-optimal', 'three-discovery-counterexample'];
  for (const key of cases) {
    const before = await page.evaluate(() => window.proofRuns);
    await page.locator('#instance').setInputFiles(path.resolve('target/native-proof-pipeline-tests', key + '-instance.json'));
    await page.locator('#discovery').setInputFiles(path.resolve('target/native-proof-pipeline-tests', key + '-discovery.json'));
    await page.waitForFunction(() => !document.querySelector('#run').disabled);
    assert.equal(await page.evaluate(() => window.proofRuns), before); assert.equal(await page.locator('#package').isDisabled(), true);
    assert.ok((await page.locator('#seed').innerText()).includes(key === 'three-optimal' ? 'unavailable' : '42'));
    await page.locator('#run').click(); await page.waitForFunction(() => !document.querySelector('#save').disabled, null, {timeout: 60000});
    const expected = JSON.parse(await fs.readFile(path.resolve('target/native-proof-pipeline-tests', key + '.json'), 'utf8'));
    const actual = await page.evaluate(() => window.latestProofRun); await fs.writeFile(path.join(root, key + '-whole-result.json'), JSON.stringify(actual));
    assert.deepEqual(actual, expected);
    const {files, ...receipt} = expected;
    const wait = page.waitForEvent('download'); await page.locator('#save').click(); const saved = path.join(root, key + '.json'); await (await wait).saveAs(saved);
    assert.deepEqual(JSON.parse(await fs.readFile(saved, 'utf8')), receipt);
    if (key === 'three-discovery-counterexample') {
      assert.equal(await page.locator('#issue-certificate').isDisabled(), true); assert.equal(await page.locator('#save-certificate').isDisabled(), true);
      assert.match(await page.locator('#status').innerText(), /Checked SAT counterexample/); assert.equal(await page.locator('#package').isDisabled(), true); assert.match(await page.locator('#decisions').innerText(), /03-canonical: checked-sat/);
    } else {
      assert.equal(await page.locator('#package').isDisabled(), false);
      const zipWait = page.waitForEvent('download'); await page.locator('#package').click(); const zip = path.join(root, key + '.zip'); await (await zipWait).saveAs(zip);
      assert.deepEqual(await fs.readFile(zip), await fs.readFile(path.resolve('target/native-proof-pipeline-tests', key + '.zip')));
      const sessionWait = page.waitForEvent('download'); await page.locator('#save-session').click(); const sessionFile = path.join(root, key + '-session.json'); await (await sessionWait).saveAs(sessionFile);
      const session = JSON.parse(await fs.readFile(sessionFile, 'utf8')); assert.deepEqual(session.result, expected);
      await page.reload(); await page.locator('#open-session').setInputFiles(sessionFile);
      await page.waitForFunction(() => document.querySelector('#status').textContent.startsWith('Imported session is unverified'));
      assert.equal(await page.evaluate(() => window.nativeCalls), 0); assert.equal(await page.locator('#save').isDisabled(), true); assert.equal(await page.locator('#package').isDisabled(), true);
      assert.equal(await page.locator('#issue-certificate').isDisabled(), true); assert.equal(await page.locator('#save-certificate').isDisabled(), true);
      await page.locator('#verify-session').click(); await page.waitForFunction(() => !document.querySelector('#save').disabled, null, {timeout: 60000});
      assert.match(await page.locator('#status').innerText(), /saved proofs freshly rechecked/); assert.equal(await page.locator('#package').isDisabled(), true);
      assert.deepEqual(await page.evaluate(() => window.latestProofRun), JSON.parse(await fs.readFile(path.resolve('target/native-proof-reverification', key + '.json'), 'utf8')));
      assert.equal(await page.locator('#issue-certificate').isDisabled(), false);
      await page.locator('#issue-certificate').click(); await page.waitForFunction(() => !document.querySelector('#save-certificate').disabled, null, {timeout: 60000});
      const certWait = page.waitForEvent('download'); await page.locator('#save-certificate').click(); const certificateFile = path.join(root, key + '-certificate.json'); await (await certWait).saveAs(certificateFile);
      assert.deepEqual(JSON.parse(await fs.readFile(certificateFile, 'utf8')), JSON.parse(await fs.readFile(path.resolve('target/native-external-certificate', key + '.json'), 'utf8')));
      await page.locator('#seconds').fill('4'); assert.equal(await page.locator('#save-certificate').isDisabled(), true); assert.equal(await page.locator('#issue-certificate').isDisabled(), true);
      await page.locator('#seconds').fill('3');
    }
  }
  const forgedSession = JSON.parse(await fs.readFile(path.join(root, 'two-optimal-session.json'), 'utf8'));
  forgedSession.result.decisions[0].request_id = 'sha256:' + '0'.repeat(64);
  const {files: forgedFiles, ...forgedReport} = forgedSession.result;
  forgedFiles['checked-native-proof-package.json'] = JSON.stringify(forgedReport, null, 2) + '\n';
  const forgedPath = path.join(root, 'forged-session.json'); await fs.writeFile(forgedPath, JSON.stringify(forgedSession));
  const beforeOpen = await page.evaluate(() => window.nativeCalls); await page.locator('#open-session').setInputFiles(forgedPath);
  await page.waitForFunction(() => document.querySelector('#status').textContent.startsWith('Imported session is unverified'));
  assert.equal(await page.evaluate(() => window.nativeCalls), beforeOpen); assert.equal(await page.locator('#save').isDisabled(), true);
  await page.locator('#verify-session').click(); await page.waitForFunction(() => document.querySelector('#status').textContent.startsWith('Proof run failed:'));
  assert.equal(await page.locator('#save').isDisabled(), true); assert.equal(await page.locator('#package').isDisabled(), true);
  await page.locator('#seconds').fill('1');
  await page.locator('#instance').setInputFiles(path.resolve('target/native-proof-pipeline-tests/grid100-timeout-instance.json'));
  await page.locator('#discovery').setInputFiles(path.resolve('target/native-proof-pipeline-tests/grid100-timeout-discovery.json'));
  await page.waitForFunction(() => !document.querySelector('#run').disabled); await page.locator('#run').click(); await page.waitForFunction(() => !document.querySelector('#save').disabled, null, {timeout: 60000});
  assert.match(await page.locator('#status').innerText(), /Proof run incomplete/); assert.equal(await page.locator('#package').isDisabled(), true);
  assert.equal(await page.locator('#issue-certificate').isDisabled(), true); assert.equal(await page.locator('#save-certificate').isDisabled(), true);
  const timedOut = await page.evaluate(() => window.latestProofRun); assert.equal(timedOut.decisions.at(-1).solver_status, 'solver-reported-time-limit'); assert.equal(timedOut.decisions[0].proof_status, 'checked-unsat');
  await page.locator('#seconds').fill('4'); assert.equal(await page.locator('#save').isDisabled(), true); assert.equal(await page.locator('#package').isDisabled(), true);
  await page.locator('#instance').setInputFiles(path.resolve('target/native-proof-pipeline-tests/two-optimal-instance.json'));
  await page.locator('#discovery').setInputFiles(path.resolve('target/native-proof-pipeline-tests/two-optimal-discovery.json'));
  await page.waitForFunction(() => !document.querySelector('#run').disabled);
  await page.evaluate(() => { document.querySelector('#run').click(); document.querySelector('#cancel').click(); });
  assert.match(await page.locator('#status').innerText(), /Cancelled/); assert.equal(await page.locator('#package').isDisabled(), true);
  await page.locator('#seconds').fill('3'); await page.locator('#run').click(); await page.waitForFunction(() => !document.querySelector('#package').disabled, null, {timeout: 60000});
  await page.evaluate(() => { document.querySelector('#issue-certificate').click(); document.querySelector('#cancel').click(); });
  assert.match(await page.locator('#status').innerText(), /Cancelled/); assert.equal(await page.locator('#save-certificate').isDisabled(), true);
  await page.locator('#run').click(); await page.waitForFunction(() => !document.querySelector('#package').disabled, null, {timeout: 60000});
  await page.locator('#issue-certificate').click(); await page.waitForFunction(() => !document.querySelector('#save-certificate').disabled, null, {timeout: 60000});
  await page.screenshot({path: path.join(root, 'preview.png'), fullPage: true}); assert.deepEqual(errors, []);
  await page.locator('#instance').setInputFiles(path.resolve('target/native-proof-pipeline-tests/three-optimal-instance.json'));
  assert.equal(await page.locator('#save-certificate').isDisabled(), true); assert.equal(await page.locator('#issue-certificate').isDisabled(), true);
  await fs.writeFile(path.join(root, 'evidence.json'), JSON.stringify({cases, session_roundtrip: true, reload_open_inert_and_unverified: true, fresh_saved_proof_reverification: true, rehashed_forged_claim_refused_after_inert_open: true, whole_results_match_node_wasm: true, exact_native_receipts: true, exact_checked_zip_downloads: true, checked_sat_counterexample_blocks_complete_export: true, actual_100_unit_timeout_blocks_complete_export: true, inert_selection: true, visible_discovery_seed: true, source_setting_freshness: true, cancel: true, fresh_run_after_cancel: true, strict_script_csp: true, tree_certificate_adoption: false}, null, 2));
  console.log('Native proof Chrome: exact receipts/ZIPs for three/five decisions, checked SAT stops complete export, inert selection, visible seeds, source/settings freshness, cancellation and fresh rerun passed.');
} finally { await context?.close(); await new Promise(resolve => server.close(resolve)); }

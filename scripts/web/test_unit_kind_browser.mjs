// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/unit-kind-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}


try{
 context=await launch();const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto(url);
 const save=async(name)=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const download=await wait,file=path.join(root,name);await download.saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 for(const kind of ['county','tract','block-group','block']){
  const file=path.resolve('target/unit-kind-wasm/'+kind+'.bisect'),expected=JSON.parse(await fs.readFile(file,'utf8'));await page.locator('#open-project').setInputFiles(file);await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'),expected.name);assert.equal(await page.locator('#download-tui-run').isDisabled(),true);assert.equal(await page.locator('#tui-run-resolution').inputValue(),expected.files.tuiRunSettings.form.resolution);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-run').disabled);let saved=await save(kind+'-run.bisect');assert.deepEqual(saved.result,expected.result);assert.deepEqual(saved.files.configRunInputs,expected.files.configRunInputs);assert.equal(await page.locator('#plan-map svg').isVisible(),true);assert.equal(await page.locator('#plan-map path').count(),36);
  await page.locator('#tui-run-map').click();assert.equal(await page.locator('#download-native-map-svg').isDisabled(),true);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-svg').disabled);saved=await save(kind+'-map.bisect');assert.equal(saved.result.status,'rendered');assert.deepEqual(saved.files.nativeMapInputs.geometry,expected.files.planGeometry);
 }
 assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,'block-map-preview.png'),fullPage:true});console.log('Prepared unit kind browser: actual Rust worker county/tract/block-group/block runs, typed native export/full result equality, original graph/geometry retention, visible accepted maps, explicit native district map rendering for all four kinds, inert Open and fresh export guards passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}

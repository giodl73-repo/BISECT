// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/multiscale-seed-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}




try{
 context=await launch();const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto(url);
 for(const name of ['tract-county-false-null-5','bg-tract-true-18446744073709551615-0-build','accepted-retry-18446744073709551615']){
  const file=path.resolve('target/multiscale-seed-wasm/'+name+'.bisect'),expected=JSON.parse(await fs.readFile(file,'utf8'));
  await page.locator('#open-project').setInputFiles(file);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));assert.equal(await page.locator('#download-engine-replay').isDisabled(),true);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-engine-replay').disabled);
  const text=await page.locator('#result-tables').innerText();assert(text.includes('Effective seed'));assert(text.includes('zero when no seed'));if(expected.result.replayed_seed!==null)assert(text.includes(String(expected.result.replayed_seed)));
  const wait=page.waitForEvent('download');await page.locator('#save-project').click();const download=await wait,target=path.join(root,name+'.bisect');await download.saveAs(target);const saved=JSON.parse(await fs.readFile(target,'utf8'));assert.deepEqual(saved.result,expected.result);assert.deepEqual(saved.files.engineReplayInputs,expected.files.engineReplayInputs);
  const receiptWait=page.waitForEvent('download');await page.locator('#download-engine-replay').click();const receipt=await receiptWait,receiptPath=path.join(root,name+'-receipt.json');await receipt.saveAs(receiptPath);assert.deepEqual(JSON.parse(await fs.readFile(receiptPath,'utf8')),expected.result);
 }
 assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,'seed-display-preview.png'),fullPage:true});console.log('Multiscale seed browser: actual worker receipt equality, visible exact max-u64 and effective-zero policy, declared null preservation, inert Open and exact saved sources/replay receipt downloads passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}

// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/native-adjacency-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}
try{
 context=await launch();const page=await context.newPage();await page.goto(url);
 await page.locator('#operation').selectOption('inspect-native-adjacency');await page.waitForFunction(()=>!document.querySelector('#native-adjacency-fields').hidden&&!document.querySelector('#execute').disabled);
 for(const [kind,file]of [['adjacency','v3-companions.adj.bin'],['geoids','v3-companions_geoids.json'],['centroids','v3-companions_centroids.json']])await page.locator('#native-adjacency-'+kind).setInputFiles(path.resolve('target/native-adjacency-wasm/'+file));
 await page.waitForFunction(()=>document.querySelector('#native-adjacency-sources').textContent.includes('centroids: v3-companions_centroids.json'));assert.equal(await page.locator('#download-native-adjacency').isDisabled(),true);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-adjacency').disabled);
 const download=page.waitForEvent('download');await page.locator('#download-native-adjacency').click();await(await download).saveAs(path.join(root,'native-adjacency.json'));const expected=JSON.parse(await fs.readFile('target/native-adjacency-wasm/v3-companions.bisect','utf8')).result;assert.equal(await fs.readFile(path.join(root,'native-adjacency.json'),'utf8'),expected.loaded_graph_json);
 const save=async name=>{const d=page.waitForEvent('download');await page.locator('#save-project').click();await(await d).saveAs(path.join(root,name));return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));};const snapshot=await save('native.bisect');assert.deepEqual(snapshot.files.nativeAdjacencyInputs,JSON.parse(await fs.readFile('target/native-adjacency-wasm/v3-companions.bisect','utf8')).files.nativeAdjacencyInputs);
 await page.locator('#native-adjacency-clear').click();assert.equal(await page.locator('#download-native-adjacency').isDisabled(),true);
 await page.locator('#open-project').setInputFiles(path.join(root,'native.bisect'));await page.waitForFunction(()=>document.querySelector('#native-adjacency-sources').textContent.includes('v3-companions.adj.bin'));assert.equal(await page.locator('#download-native-adjacency').isDisabled(),true);for(const kind of ['adjacency','geoids','centroids'])assert.equal(await page.locator('#native-adjacency-'+kind).inputValue(),'');
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-adjacency').disabled);
 await fs.writeFile(path.join(root,'duplicate.json'),'{"0":"first","0":"second"}');await page.locator('#native-adjacency-geoids').setInputFiles(path.join(root,'duplicate.json'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.toLowerCase().includes('duplicate'));const rejected=await save('rejected.bisect');assert.deepEqual(rejected.files.nativeAdjacencyInputs,snapshot.files.nativeAdjacencyInputs);assert.equal(await page.locator('#download-native-adjacency').isDisabled(),true);
 await page.screenshot({path:path.join(root,'native-adjacency.png'),fullPage:true});console.log('Real browser native adjacency: three local source selections, actual WASM Run, exact i64-safe JSON download, source-preserving Save/inert Open/re-Run, clear freshness and duplicate-companion preservation passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}

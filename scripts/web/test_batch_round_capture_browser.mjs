// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/batch-round-capture-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}

try{
 const expected=JSON.parse(await fs.readFile('target/batch-round-capture-wasm/captured.bisect','utf8'));
 await fs.writeFile(path.join(root,'captured.bisect'),JSON.stringify(expected));
 context=await launch();const page=await context.newPage();await page.goto(url);
 await page.locator('#open-project').setInputFiles(path.join(root,'captured.bisect'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.locator('#label-build-capture_rounds').isChecked(),true);
 assert.equal(await page.locator('#download-label-build').isDisabled(),true);
 const save=async(name,button='#save-project')=>{const wait=page.waitForEvent('download');await page.locator(button).click();const d=await wait;const file=path.join(root,name);await d.saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);
 const saved=await save('run.bisect');assert.deepEqual(saved.result,expected.result);assert.equal(saved.files.labelBuildInputs.settings.capture_rounds,true);
 assert((await page.locator('#result-tables').innerText()).includes('Captured events'));
 let download=page.waitForEvent('download');await page.locator('#download-label-build').click();let file=await download;await file.saveAs(path.join(root,'captured.zip'));assert.deepEqual(await fs.readFile(path.join(root,'captured.zip')),await fs.readFile('target/batch-round-capture-wasm/captured.zip'));
 const archive=await save('run.bisect-build','#label-build-save-archive');assert.deepEqual(archive.result,expected.result);assert.equal(archive.files.labelBuildInputs.settings.capture_rounds,true);

 const row=expected.result.states.find(r=>r.year==='2020');await page.locator('#label-build-plan').selectOption(row.plan_path);await page.waitForFunction(()=>!document.querySelector('#download-captured-round').disabled);
 const capture=JSON.parse(expected.result.files[row.round_capture.path]);assert.equal(await page.locator('#use-captured-round-map').isDisabled(),false);
 download=page.waitForEvent('download');await page.locator('#download-captured-round').click();file=await download;await file.saveAs(path.join(root,'round.json'));assert.equal(await fs.readFile(path.join(root,'round.json'),'utf8'),capture.events[0].assignments_json);
 await page.screenshot({path:path.join(root,'batch-capture-preview.png'),fullPage:true});
 await page.locator('#use-captured-round-map').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='render-native-district-map');assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);
 const handoff=await save('handoff.bisect');assert.equal(handoff.files.nativeMapInputs.geometry,null);assert.equal(handoff.files.nativeMapInputs.round_key_basis,'native-index-map');assert.equal(Buffer.from(handoff.files.nativeMapInputs.round_assignments.base64,'base64').toString(),capture.events[0].assignments_json);
 const graph=JSON.parse(Buffer.from(expected.files.labelBuildInputs.jobs.base64,'base64').toString())[0].graph,x=Array.from({length:5},(_,i)=>-71.5+i*.01),y=Array.from({length:4},(_,i)=>41.5+i*.01);
 const geometry={type:'FeatureCollection',state:'RI',year:2020,features:graph.geoids.map((GEOID,i)=>{const c=i%4,r=Math.floor(i/4);return {type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[[[x[c],y[r]],[x[c+1],y[r]],[x[c+1],y[r+1]],[x[c],y[r+1]],[x[c],y[r]]] ]}};})};
 await fs.writeFile(path.join(root,'geometry.json'),JSON.stringify(geometry));await page.locator('#native-map-geometry').setInputFiles(path.join(root,'geometry.json'));await page.waitForFunction(()=>document.querySelector('#native-map-sources').textContent.includes('geometry: geometry.json'));await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);await page.waitForFunction(()=>{const img=document.querySelector('#native-map-preview img');return img?.complete&&img.naturalWidth>0;});assert.equal((await save('mapped.bisect')).result.map_type,'rounds');

 await page.locator('#label-build-open-archive').setInputFiles(path.join(root,'run.bisect-build'));
 await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Opened build archive'));
 assert.equal(await page.locator('#download-label-build').isDisabled(),true);assert.equal(await page.locator('#label-build-capture_rounds').isChecked(),true);
 await page.locator('#label-build-capture_rounds').uncheck();assert.equal(await page.locator('#download-label-build').isDisabled(),true);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);
 const plain=await save('plain.bisect');assert.equal(plain.files.labelBuildInputs.settings.capture_rounds,undefined);assert.equal(Object.keys(plain.result.files).some(p=>p.endsWith('/round_capture.json')),false);
 for(const row of plain.result.states){const captured=expected.result.states.find(s=>s.state===row.state&&s.year===row.year);assert.deepEqual(JSON.parse(plain.result.files[row.plan_path]).plan.assignment,JSON.parse(expected.result.files[captured.plan_path]).plan.assignment);assert.deepEqual(row.metrics,captured.metrics);}

 const maxYaml=Buffer.from(expected.files.labelBuildInputs.config.base64,'base64').toString().replace('seed: 42','seed: 18446744073709551615');await fs.writeFile(path.join(root,'max-seed.yml'),maxYaml);await page.locator('#label-build-config').setInputFiles(path.join(root,'max-seed.yml'));await page.waitForFunction(()=>document.querySelector('#label-build-names').textContent.includes('max-seed.yml'));await page.locator('#label-build-capture_rounds').check();await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);
 const maximum=await save('max-seed.bisect'),maxRow=maximum.result.states.find(r=>r.year==='2020');assert.equal(maxRow.build_report.attempts.at(-1).seed,'18446744073709551615');assert(maximum.result.files[maxRow.plan_path].includes('18446744073709551615'));await page.locator('#label-build-plan').selectOption(maxRow.plan_path);await page.waitForFunction(()=>!document.querySelector('#use-captured-round-map').disabled);await page.locator('#use-captured-round-map').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='render-native-district-map');const maxHandoff=await save('max-seed-handoff.bisect');assert.equal(Buffer.from(maxHandoff.files.nativeMapInputs.plan.base64,'base64').toString(),maximum.result.files[maxRow.plan_path]);
 await page.locator('#native-map-geometry').setInputFiles(path.join(root,'geometry.json'));await page.waitForFunction(()=>document.querySelector('#native-map-sources').textContent.includes('geometry: geometry.json'));await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);await page.waitForFunction(()=>{const img=document.querySelector('#native-map-preview img');return img?.complete&&img.naturalWidth>0;});assert.equal((await save('max-seed-mapped.bisect')).result.map_type,'rounds');
 console.log('Batch capture browser: real WASM two-year build, readable event disclosure, exact ZIP, retained flag and snapshots in ordinary/build archive disk downloads, exact selected-event download and nonexecuting map handoff/explicit geometry/render/decoded PNG, inert archive Open, change invalidation and capture-off rerun parity and max-u64 native seed original-source preservation through actual map rendering passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}

// Validate same-tab Open after a full national export, using the real offline app.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createReadStream} from 'node:fs';
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import path from 'node:path';

const [address, packageDir, projectPath, referencePath, receipt] = process.argv.slice(2);
const origin = new URL(address).origin;
assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/);
const read = async file => JSON.parse(await fs.readFile(file));
const manifest = await read(path.join(packageDir, 'assets/offline-manifest.json'));
const reference = await read(referencePath);
const projectBytes = await fs.readFile(projectPath);
assert.equal(createHash('sha256').update(projectBytes).digest('hex'), reference.project_sha256);
const project = JSON.parse(projectBytes);
assert.equal(project.experiment.states.length, 50);
assert.equal(reference.wasm_sha256, manifest.assets['bisect_wasm.wasm'].sha256);
const {chromium} = createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES, 'package.json'))('playwright');
const browser = await chromium.launch({headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER || 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']});
const timeout = 180000;
try {
  const context = await browser.newContext({acceptDownloads:true});
  await context.addInitScript(() => {
    window.runs=0;window.failParser=false;window.parserStarts=[];window.parserErrors=0;
    const OriginalWorker=Worker;
    window.Worker=class extends OriginalWorker {
      constructor(url,...args) {
        const target=new URL(url,location.href),parser=target.pathname==='/laboratory-project-worker.js';
        if(parser){
          const grid=document.querySelector('#national-grid'),map=document.querySelector('#map-container');
          parserStarts.push({gridVisible:!grid.hidden,mapHidden:map.hidden,injected:failParser});
          if(failParser){target.pathname='/test-missing-laboratory-project-worker.js';failParser=false;}
        }
        super(target,...args);
        if(parser)this.addEventListener('error',()=>parserErrors++);
      }
      postMessage(message,...args){if(message?.type==='run')runs++;return super.postMessage(message,...args);}
    };
  });
  let page=await context.newPage();
  const el=id=>page.locator('#'+id);
  await page.goto(origin+'/');
  await el('offline-package').evaluate(section=>section.open=true);
  await el('offline-states').selectOption(Object.keys(manifest.states));
  await el('offline-install').click({timeout});
  await page.waitForFunction(()=>document.querySelector('#offline-status').textContent.startsWith('Installed '),null,{timeout});
  await page.waitForFunction(()=>!!navigator.serviceWorker.controller,null,{timeout});
  await page.close();await context.setOffline(true);
  page=await context.newPage();
  const pageErrors=[];page.on('pageerror',e=>pageErrors.push(e.message));
  await page.goto(origin+'/');
  await page.waitForFunction(()=>document.querySelector('#connection').textContent.includes('WASM'),null,{timeout});
  async function open(buffer,name,expected) {
    await el('open-lab-project').setInputFiles({buffer,name,mimeType:'application/json'});
    await page.waitForFunction(text=>{
      const status=document.querySelector('#lab-project-status').textContent;
      return status.includes(text)||/^(Invalid |Project import failed|Project needs different)/.test(status);
    },expected,{timeout});
    assert((await el('lab-project-status').textContent()).includes(expected),await el('lab-project-status').textContent());
    assert.equal(await page.evaluate(()=>runs),0);
  }
  async function exportMap() {
    await el('national-map-view').click();
    await page.waitForFunction(()=>document.querySelector('#map-loading').hidden &&
      !document.querySelector('#download-map').disabled && document.querySelector('#map-caption').textContent.includes('Albers projection'),null,{timeout});
    assert.doesNotMatch(await el('map-caption').textContent(),/unavailable:/);
    const waiting=page.waitForEvent('download',{timeout});await el('download-map').click({timeout});
    const hash=createHash('sha256');let bytes=0;
    for await(const chunk of createReadStream(await(await waiting).path())){hash.update(chunk);bytes+=chunk.length;}
    const map={bytes,sha256:hash.digest('hex')};assert.deepEqual(map,reference.geojson);return map;
  }
  await open(projectBytes,'original.bisect','Opened project.');
  const name=await el('name').inputValue();
  const first=await exportMap();
  const replacement=structuredClone(project);replacement.experiment.config.name='Verified same-tab replacement';
  const replacementBytes=Buffer.from(JSON.stringify(replacement));
  await page.evaluate(()=>{failParser=true;});
  await open(replacementBytes,'replacement.bisect','Project import failed. Current project was kept.');
  assert.equal(await el('name').inputValue(),name);
  assert.equal(await el('run-status').textContent(),'completed');
  assert.equal(await el('national-grid').isVisible(),true);
  assert.equal(await el('state-tiles').locator('.state-tile.completed').count(),50);
  assert.equal(await page.evaluate(()=>parserErrors),1);
  console.log('Parser startup refusal retained all 50 state results; reloading the map before retry.');
  // Rebuild and export the retained map again before retrying, so the valid
  // Open also starts with the large map active in this same tab.
  const second=await exportMap();
  assert.equal(await el('map-container').isVisible(),true);
  await open(replacementBytes,'replacement.bisect','Opened project.');
  assert.equal(await el('name').inputValue(),replacement.experiment.config.name);
  console.log('Same-tab Open succeeded after the full-map export.');
  const bad=structuredClone(project);bad.inputs.UT.prepared_graph_sha256='0'.repeat(64);
  // Keep the internal claims consistent to exercise the independent catalog
  // check, beyond the reader's rejection of inconsistent project fields.
  bad.experiment.states.find(state=>state.code==='UT').metrics.prepared_graph_sha256=bad.inputs.UT.prepared_graph_sha256;
  await open(Buffer.from(JSON.stringify(bad)),'changed-source.bisect','Project needs different prepared inputs. Current work was kept.');
  assert.equal(await el('name').inputValue(),replacement.experiment.config.name);
  assert.equal(await el('state-tiles').locator('.state-tile.completed').count(),50);
  const parserStarts=await page.evaluate(()=>parserStarts);
  assert.deepEqual(parserStarts.slice(1),[true,false,false].map(injected=>({gridVisible:true,mapHidden:true,injected})));
  assert.deepEqual(pageErrors,[]);
  await fs.writeFile(receipt,JSON.stringify({status:'passed-national-same-tab-open-recovery',first,second,
    parserStarts,parser_start_failure_preserved_plan:true,tampered_graph_refused:true,offline:true,
    valid_retry_started_with_full_map_active:true,
    engine_runs:0,browser:browser.version(),deployed:false},null,2)+'\n',{flag:'wx'});
}finally{await browser.close();}

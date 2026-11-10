// Deterministic regression for a test oracle, not product import validation.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright'),root=await fs.mkdtemp(path.resolve('target/report-cffi/observer-')),file=path.join(root,'intended.bisect');await fs.writeFile(file,'{}');
const browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
try{
 const context=await browser.newContext();await context.route('**/*',route=>route.abort());const page=await context.newPage();page.setDefaultTimeout(1200);
 async function fixture(mode){await page.setContent('<input id="open-project" type="file"><p id="project-status"></p>');await page.evaluate(mode=>{document.querySelector('#open-project').onchange=event=>{const status=document.querySelector('#project-status');if(mode==='coalesced')status.textContent='Opened '+event.target.files[0].name;if(mode==='wrong-name')status.textContent='Opened another.bisect';status.textContent='Autosaved on this browser.';};},mode);}
 await fixture('coalesced');const observed=await observedProjectOpen(page,file);assert.equal(observed.opened,'Opened intended.bisect');assert.equal(observed.final_status,'Autosaved on this browser.');assert(observed.messages.some(m=>m.text==='Opened intended.bisect'));
 for(const mode of ['autosave-only','wrong-name']){await fixture(mode);await assert.rejects(()=>observedProjectOpen(page,file),error=>error.import_observation?.opened===null&&error.message.includes('Timeout'));}
 let oldMissed=null;
 if(process.argv[2]){const previous=await import(pathToFileURL(path.resolve(process.argv[2])));await fixture('coalesced');await assert.rejects(()=>previous.observedProjectOpen(page,file),/Timeout/);oldMissed=true;}
 const helper=await fs.readFile('scripts/web/fixtures/observed-project-open.mjs');const evidence={status:'passed-import-oracle-regression',helper_sha256:createHash('sha256').update(helper).digest('hex'),coalesced_exact_open_observed:true,autosave_alone_refused:true,wrong_filename_refused:true,prior_oracle_missed_coalesced_open:oldMissed,scope:'Controlled actual DOM mutation delivery tests only the import-observation oracle. Actual product validation/project/worker behavior is exercised by the separate full UI suites.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));console.log(JSON.stringify({root,status:evidence.status}));
}finally{await browser.close();}

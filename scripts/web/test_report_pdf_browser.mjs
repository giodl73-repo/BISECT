// Own browser/server only. PDF bytes come from Chrome's print renderer, never a mocked print call.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
import {generatedReports,generatedReportText} from '../../web/lab/generated-report-files.js';
import {reportPrintCss} from '../../web/lab/report-print.js';
import {reportPdfCase} from './fixtures/report-pdf-case.mjs';
import {acceptedStateWorkflowRequest} from '../../web/lab/state-workflow-files.js';

const sha=data=>createHash('sha256').update(data).digest('hex');
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/report-pdf-delivery/run-'+Date.now());await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve(process.argv.slice(2).find(arg=>!arg.startsWith('--'))||'web/lab'),sourceTree=staticRoot===path.resolve('web/lab');
const wasm=path.resolve(process.env.BISECT_TEST_WASM||'target/cffi-legacy-gates/default.wasm');
const wasmFile=sourceTree?wasm:path.join(staticRoot,'bisect_wasm.wasm'),wasmBytes=await fs.readFile(wasmFile),engine=await instantiateEngine(wasmBytes);
// Builders produce these references; local rechecks may explicitly reuse frozen binaries.
// Hash the actual references without making another 60 MiB copy for each case.
const nativeHost=path.resolve(process.env.BISECT_TEST_NATIVE_TOOL||'target/release/examples/execute_tool_request.exe');
const nativeCli=path.resolve(process.env.BISECT_TEST_NATIVE_CLI||'target/release/bisect.exe');
const declaration={schema_version:'bisect-report-pdf-cases-v1',fixture:'Tracked grid3x3 RPLAN/RCTX and source-owned shared vertices; no new partition/build',wasm_sha256:sha(wasmBytes),cases:[{id:'commission-letter',operation:'assemble-commission-report',settings:{paper:'letter',orientation:'portrait'},embedded_native_map:true},{id:'commission-letter-landscape',operation:'assemble-commission-report',settings:{paper:'letter',orientation:'landscape'},embedded_native_map:true},{id:'label-a4-portrait',operation:'run-label-report',settings:{paper:'a4',orientation:'portrait'},embedded_native_map:false},{id:'label-a4-landscape',operation:'run-label-report',settings:{paper:'a4',orientation:'landscape'},embedded_native_map:false},{id:'comparison-letter',operation:'render-report-comparison',settings:{paper:'letter',orientation:'portrait'},embedded_native_map:false,draft_expected:true,min_pages:1}],comparison:'Exact native-host replies; DOM ordered-list decimal markers materialized explicitly; Page n of N added/verified in bottom margin; remaining PDF text NFKC/whitespace only, native/restricted page counts, media boxes within 1 pt, embedded PNG decoded pixels exact. No cross-renderer/PDF byte equality.',native_cli:'Controlled PATH excludes PDF tools: html+pdf preserves HTML and succeeds, pdf-only exits 1 without a PDF.',print_api:'GUI method invocation/events instrumented; actual PDF bytes generated separately by owned Chrome Page.printToPDF from the identical restricted DOM. No OS-dialog/save completion claim.'};
await fs.writeFile(root+'/case-declaration.json',JSON.stringify(declaration,null,2));
const fixture=await reportPdfCase(engine);
const acceptedTime='2026-10-23T00:00:00.000Z';
const acceptedState=fixture.labelRequest.input.files;
const acceptedPlan=JSON.parse(acceptedState[Object.keys(acceptedState).find(n=>n.endsWith('/plan.rplan'))]);
acceptedPlan.provenance.producer={engine_options:{balance_tolerance:0.5,seed:'42'}};
const acceptedContext=acceptedState[Object.keys(acceptedState).find(n=>n.endsWith('/context.rctx'))];
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const workflowSettings={analysis:true,map:true,report:true,dpi:'72',
 delivery:{label:'accepted_print_fixture',formats:['html'],audit_with_report:false}};
const acceptedFiles={auditPlanText:source('plan.rplan',JSON.stringify(acceptedPlan)),auditContextText:source('context.rctx',acceptedContext),
 planGeometry:source('units.geojson',fixture.mapRequest.input.unit_geometry_json),tuiRunWorkflowSettings:workflowSettings};
const acceptedRequest=acceptedStateWorkflowRequest(acceptedFiles,workflowSettings,acceptedTime),acceptedResult=engine.execute(acceptedRequest);
assert.equal(acceptedResult.partition_execution_performed,false);
// No optional demographic or election inputs: native records partial analysis,
 // while the map and report stages still complete and remain printable.
assert.equal(acceptedResult.workflow.status,'partial');
assert.deepEqual(acceptedResult.workflow.stages.map(({name,status})=>[name,status]),
 [['analysis','partial'],['map:districts','completed'],['report','completed']]);
assert.equal(acceptedResult.workflow.stages[0].report_ready,true);
assert.equal(generatedReports(acceptedResult).length,1);
assert.equal(acceptedResult.workflow.files_base64['maps/districts.png'],fixture.map.png_base64);
declaration.cases.push({id:'accepted-workflow-letter',operation:acceptedRequest.operation,settings:{paper:'letter',orientation:'portrait'},embedded_native_map:true});
await fs.writeFile(root+'/case-declaration.json',JSON.stringify(declaration,null,2));
const badMapRequest=structuredClone(fixture.request);badMapRequest.files['maps/districts.png']=[...Buffer.from('Bounded native report source, not a valid PNG.')];const badMapResult=engine.execute(badMapRequest);
for(const [name,request,expected]of [['map',fixture.mapRequest,fixture.map],['commission',fixture.request,fixture.result],['label',fixture.labelRequest,fixture.labelResult],['invalid-map',badMapRequest,badMapResult],['comparison',fixture.comparisonRequest,fixture.comparisonResult],['accepted-workflow',acceptedRequest,acceptedResult]]){
 const file=path.join(root,name+'-native-request.json');await fs.writeFile(file,JSON.stringify(request));
 const actual=JSON.parse(execFileSync(nativeHost,[file],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(actual,expected);
 await fs.writeFile(root+'/'+name+'-native-reply.json',JSON.stringify(actual));
}
await fs.writeFile(root+'/map.png',Buffer.from(fixture.map.png_base64,'base64'));
const cliRoot=path.join(root,'native-cli'),planDir=path.join(cliRoot,'vPDF/2020/plans',fixture.label);
for(const [name,bytes]of Object.entries(fixture.request.files)){const file=path.resolve(planDir,name);assert(file.startsWith(planDir+path.sep));await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,Buffer.from(bytes));}
const env=Object.fromEntries(Object.entries(process.env).filter(([key])=>key.toLowerCase()!=='path'));env.Path=path.join(process.env.SystemRoot||'C:/Windows','System32');
const cliCases=[];
for(const formats of [['html','pdf'],['pdf']]){
 const out=path.join(cliRoot,formats.join('-'));const args=['report','--label',fixture.label,'--year','2020','--version','vPDF','--output-base',cliRoot,'--out',out,'--format',...formats];
 const response=spawnSync(nativeCli,args,{cwd:root,env,encoding:'utf8'});assert.equal(response.status,formats.length===2?0:1,response.stdout+'\n'+response.stderr);
 await assert.rejects(fs.access(path.join(out,fixture.label+'_report.pdf')));
 if(formats.length===2)assert.match(await fs.readFile(path.join(out,fixture.label+'_report.html'),'utf8'),/Congressional Redistricting Commission Report/);
 cliCases.push({args,status:response.status,stdout:response.stdout,stderr:response.stderr,pdf_present:false,html_present:formats.length===2});
}
await fs.writeFile(root+'/native-cli-cases.json',JSON.stringify(cliCases,null,2));
const served=new Map();
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'&&sourceTree?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?(sourceTree?'/workbench.html':'/index.html'):url.pathname));
 if(file!==wasm&&!file.startsWith(staticRoot+path.sep))throw new Error('Path rejected');
 const bytes=await fs.readFile(file);served.set(file,{path:file,sha256:sha(bytes),byte_length:bytes.length});
 res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(bytes);
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let browser,page;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions','--disk-cache-size=1048576','--disable-gpu-shader-disk-cache']});
 const context=await browser.newContext({acceptDownloads:true});const remote=[],errors=[];
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():(remote.push(route.request().url()),route.abort()));
 page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.reportRequests=[];const W=Worker;window.Worker=class extends W{postMessage(value,...rest){if(value.type==='run')reportRequests.push(value.request);return super.postMessage(value,...rest);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const pdfCases=[];
 const open=async project=>{const file=path.join(root,project.name+'.bisect');await fs.writeFile(file,JSON.stringify(project));const before=await page.evaluate(()=>reportRequests.length);await page.locator('#open-project').setInputFiles(file);await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&document.querySelector('#result-status').textContent.startsWith('Imported result is unverified')&&!document.querySelector('#execute').disabled,project.name,{timeout:120000});assert.equal(await page.evaluate(()=>reportRequests.length),before);assert(await page.locator('#preview-commission').isDisabled());};
 const preview=async()=>{await page.locator('#preview-commission').click();await page.waitForFunction(()=>!document.querySelector('#print-commission').disabled,null,{timeout:120000});};
 for(const row of declaration.cases){
  const accepted=row.operation===acceptedRequest.operation;
  if(accepted)await page.evaluate(when=>{const RealDate=Date;globalThis.Date=class extends RealDate{
   constructor(...args){super(...(args.length?args:[when]));}static now(){return new RealDate(when).getTime();}
  };},acceptedTime);
  const label=row.id.startsWith('label'),comparison=row.id.startsWith('comparison'),result=accepted?acceptedResult:comparison?fixture.comparisonResult:label?fixture.labelResult:fixture.result;
  const input=fixture.labelRequest.input,{generated_at,format,audit_with_report,...batch}=input;
  const files=accepted?acceptedFiles:comparison?fixture.comparisonProjectFiles:label?{labelReportInputs:{source:{name:'analysis.json',base64:Buffer.from(JSON.stringify(batch)).toString('base64')},settings:{generated_at,format,audit_with_report}}}:fixture.projectFiles;
  const project=createProject({name:row.id,operation:row.operation,files,constraints:[],result,lastOperation:row.operation});await open(project);
  const before=await page.evaluate(()=>reportRequests.length);await page.locator('#execute').click();await page.waitForFunction(before=>reportRequests.length>before&&!document.querySelector('#execute').disabled,before,{timeout:120000});
  if(accepted)assert.deepEqual(await page.evaluate(i=>reportRequests[i],before),acceptedRequest);
  await page.locator('#report-print-paper').selectOption(row.settings.paper);await page.locator('#report-print-orientation').selectOption(row.settings.orientation);await preview();
  const entries=generatedReports(result);assert.equal(entries.length,1);const original=await generatedReportText(result,entries[0].key),originalHtml=row.id+'-native.html';await fs.writeFile(path.join(root,originalHtml),original.text);
  const frame=page.locator('#commission-preview-host iframe');const dom=await frame.evaluate(f=>f.contentDocument.documentElement.outerHTML);const headings=await frame.evaluate(f=>[...f.contentDocument.querySelectorAll('h2')].map(node=>node.textContent));if(comparison){assert(headings.length>=3);assert((await frame.evaluate(f=>f.contentDocument.querySelectorAll('table').length))>0);}else assert.equal(headings.length,10);
  await fs.writeFile(root+'/'+row.id+'-restricted.html','<!doctype html>'+dom);
  await frame.evaluate(f=>{window.printCalls=0;f.contentWindow.print=()=>{window.printCalls++;f.contentWindow.dispatchEvent(new Event('beforeprint'));f.contentWindow.dispatchEvent(new Event('afterprint'));};});
  await page.locator('#print-commission').click();await page.waitForFunction(()=>window.printCalls===1);assert.match(await page.locator('#commission-preview-status').innerText(),/not confirmed|cannot confirm|Saving|save remains|Print event/);
  const nativePage=await context.newPage();await nativePage.setContent(original.text.replace('</head>','<style>'+reportPrintCss(row.settings)+'</style></head>'));await nativePage.emulateMedia({media:'print'});await nativePage.evaluate(async()=>{await document.fonts.ready;await Promise.all([...document.images].map(image=>image.decode()));});
  const expectedText=await nativePage.evaluate(()=>{const body=document.body.cloneNode(true);for(const node of body.querySelectorAll('style,title'))node.remove();for(const list of body.querySelectorAll('ol')){let ordinal=Number(list.getAttribute('start')||1);for(const item of list.children)if(item.localName==='li'){if(item.hasAttribute('value'))ordinal=Number(item.getAttribute('value'));item.prepend(document.createTextNode(String(ordinal)+'. '));ordinal++;}}return body.textContent;}),expectedFile=row.id+'-expected.txt';await fs.writeFile(root+'/'+expectedFile,expectedText);
  const nativePdf=row.id+'-native.pdf';await nativePage.pdf({path:root+'/'+nativePdf,preferCSSPageSize:true,printBackground:true,displayHeaderFooter:false});await nativePage.close();
  const restrictedPage=await context.newPage();await restrictedPage.setContent('<!doctype html>'+dom);await restrictedPage.emulateMedia({media:'print'});await restrictedPage.evaluate(async()=>{await document.fonts.ready;await Promise.all([...document.images].map(image=>image.decode()));});
  assert.equal(await restrictedPage.evaluate(()=>document.documentElement.outerHTML),dom,'Standalone print DOM must equal the live restricted preview DOM');
  const restrictedPdf=row.id+'-restricted.pdf';await restrictedPage.pdf({path:root+'/'+restrictedPdf,preferCSSPageSize:true,printBackground:true,displayHeaderFooter:false});await restrictedPage.close();
  await page.locator('#report-pdf-file').setInputFiles(root+'/'+restrictedPdf);await page.waitForFunction(()=>document.querySelector('#report-pdf-file-name').textContent.includes('SHA-256:'));
  const waiting=page.waitForEvent('download');await page.locator('#download-report-print-record').click();const recordFile=row.id+'-record.json';await(await waiting).saveAs(root+'/'+recordFile);const record=JSON.parse(await fs.readFile(root+'/'+recordFile,'utf8'));
  assert.equal(record.original_html_sha256,original.sha256);assert.equal(record.runtime_artifact.sha256,sha(wasmBytes));assert.equal(record.pdf_save_observed,false);assert(record.beforeprint_observed&&record.afterprint_observed);assert.equal(record.print_call,'returned');assert.equal(record.selected_pdf.content_match_verified,false);assert.equal(record.selected_pdf.sha256,sha(await fs.readFile(root+'/'+restrictedPdf)));assert.deepEqual(record.settings,row.settings);
  assert(await frame.evaluate((f,css)=>[...f.contentDocument.querySelectorAll('style')].some(style=>style.textContent===css),reportPrintCss(row.settings)));
  pdfCases.push({id:row.id,settings:row.settings,original_html:originalHtml,native_pdf:nativePdf,restricted_pdf:restrictedPdf,expected_text:expectedFile,record:recordFile,headings,...(comparison?{draft_expected:true,min_pages:1,table_count:await frame.evaluate(f=>f.contentDocument.querySelectorAll('table').length)}:{}),...(row.embedded_native_map?{map_png:'map.png'}:{})});
  // Save/Open retains the native sources/result, never this private print session or PDF bytes.
  const saveWaiting=page.waitForEvent('download');await page.locator('#save-project').click();const savedFile=root+'/'+row.id+'-saved.bisect';await(await saveWaiting).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));assert(!JSON.stringify(saved).includes(record.selected_pdf.sha256));if(accepted){assert.deepEqual(saved.files,acceptedFiles);assert.deepEqual(saved.result,acceptedResult);}saved.name=row.id+'-reopened';await open(saved);assert(await page.locator('#download-report-print-record').isDisabled());assert.equal(await page.locator('#commission-preview-host iframe').count(),0);assert.equal(await page.locator('#report-pdf-file-name').innerText(),'No saved PDF selected.');
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#preview-commission').disabled);await preview();
  // Paper edits retire the old frame and selected-PDF observation without rerunning the engine.
  const noRun=await page.evaluate(()=>reportRequests.length);await page.locator('#report-print-orientation').selectOption(row.settings.orientation==='portrait'?'landscape':'portrait');assert.equal(await frame.count(),0);assert(await page.locator('#download-report-print-record').isDisabled());assert.equal(await page.evaluate(()=>reportRequests.length),noRun);
  await page.locator('#report-print-orientation').selectOption(row.settings.orientation);await preview();
  await frame.evaluate(f=>{f.contentDocument.body.append('changed after review');window.refusedPrintCalls=0;f.contentWindow.print=()=>window.refusedPrintCalls++;});await page.locator('#print-commission').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('Report preview changed'));assert.equal(await page.evaluate(()=>refusedPrintCalls),0);await preview();
  // A forged afterprint cannot mark a file saved; malformed selected bytes never execute.
  await page.locator('#report-pdf-file').setInputFiles({name:'bad.pdf',mimeType:'application/pdf',buffer:Buffer.from('<script>bad</script>')});await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('PDF header/end marker'));
  assert.equal(await page.locator('#report-pdf-file-name').innerText(),'No saved PDF selected.');
  // Delay genuine generated PDF bytes, independently of OS file-token lifetime.
  // The earlier selection above still reads the saved PDF from its local path.
  // A retired read must not clear a newer pending selection or replace its result.
  for(const oldFinishesFirst of [true,false]){
   await page.evaluate(()=>{window.pdfReleases={};window.pdfReadsFinished=[];const original=Blob.prototype.arrayBuffer;window.restorePdfRead=()=>Blob.prototype.arrayBuffer=original;Blob.prototype.arrayBuffer=async function(){const data=await original.call(this);if(this instanceof File&&this.name.endsWith('.pdf')){await new Promise(resolve=>window.pdfReleases[this.name]=resolve);window.pdfReadsFinished.push(this.name);}return data;};});
   await page.locator('#report-pdf-file').setInputFiles({name:restrictedPdf,mimeType:'application/pdf',buffer:await fs.readFile(root+'/'+restrictedPdf)});await page.waitForFunction(name=>typeof window.pdfReleases[name]==='function',restrictedPdf);
   await page.locator('#report-print-orientation').selectOption(row.settings.orientation==='portrait'?'landscape':'portrait');await page.locator('#report-print-orientation').selectOption(row.settings.orientation);await preview();
   await page.locator('#report-pdf-file').setInputFiles({name:nativePdf,mimeType:'application/pdf',buffer:await fs.readFile(root+'/'+nativePdf)});await page.waitForFunction(name=>typeof window.pdfReleases[name]==='function',nativePdf);
   const finish=async name=>{await page.evaluate(name=>window.pdfReleases[name](),name);await page.waitForFunction(name=>window.pdfReadsFinished.includes(name),name);await page.evaluate(()=>new Promise(resolve=>setTimeout(resolve,0)));};
   if(oldFinishesFirst){await finish(restrictedPdf);assert.equal(await page.locator('#report-pdf-file').evaluate(input=>input.files[0]?.name),nativePdf,'Retired read cleanup must preserve the newer pending file selection');}
   await finish(nativePdf);await page.waitForFunction(name=>document.querySelector('#report-pdf-file-name').textContent.startsWith(name),nativePdf);
   const newerStatus=await page.locator('#commission-preview-status').innerText();if(!oldFinishesFirst)await finish(restrictedPdf);
   assert((await page.locator('#report-pdf-file-name').innerText()).startsWith(nativePdf));assert.equal(await page.locator('#commission-preview-status').innerText(),newerStatus);await page.evaluate(()=>restorePdfRead());
  }
  await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));assert(await frame.evaluate(f=>f.contentDocument.documentElement.scrollWidth<=f.contentWindow.innerWidth+1));await page.locator('#commission-preview-panel').screenshot({path:root+'/'+row.id+'-390.png'});await page.setViewportSize({width:1280,height:900});
 }
 const invalidFiles=structuredClone(fixture.projectFiles);invalidFiles.commissionInputs.files['maps/districts.png'].base64=Buffer.from(badMapRequest.files['maps/districts.png']).toString('base64');await open(createProject({name:'invalid-map-preview',operation:'assemble-commission-report',files:invalidFiles,constraints:[],result:badMapResult,lastOperation:'assemble-commission-report'}));await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#preview-commission').disabled);await page.locator('#preview-commission').click();await page.waitForFunction(()=>document.querySelector('#commission-preview-status').textContent.includes('printing is disabled'));assert(await page.locator('#print-commission').isDisabled());assert(!(await page.locator('#download-commission-html').isDisabled()));
 await fs.writeFile(root+'/pdf-cases.json',JSON.stringify(pdfCases,null,2));
 const inspector=spawnSync('python',['scripts/web/validate_report_print_pdf.py',root],{encoding:'utf8',maxBuffer:4*1024*1024});await fs.writeFile(root+'/pdf-inspection.log',inspector.stdout+'\n'+inspector.stderr);assert.equal(inspector.status,0,inspector.stdout+'\n'+inspector.stderr);
 assert.deepEqual(remote,[]);assert.deepEqual(errors,[]);
 await fs.writeFile(root+'/evidence.json',JSON.stringify({schema_version:'bisect-report-pdf-browser-v1',package:staticRoot,source_tree:sourceTree,wasm:{sha256:sha(wasmBytes),byte_length:wasmBytes.length},native_cli:{path:nativeCli,sha256:sha(await fs.readFile(nativeCli))},native_host:{path:nativeHost,sha256:sha(await fs.readFile(nativeHost))},browser:browser.version(),served_assets:[...served.values()],native_result_matches:6,accepted_workflow_exact_native_and_browser:true,native_cli_pdf_dependency_cases:2,actual_pdf_renderer:'Chrome Page.printToPDF on DOM identical to restricted preview; UI print call instrumented separately',actual_pdf_cases:pdfCases.length,inert_open:true,explicit_print_record:true,save_not_inferred:true,user_selected_pdf_hash:true,settings_retire_preview:true,dom_mutation_refused:true,malformed_pdf_refused:true,late_pdf_read_cannot_replace_newer_observation:true,invalid_map_blocks_print_preserves_html:true,original_html_preserved:true,independent_pdf_inspection:true,viewport_width:390,os_print_dialog_tested:false,full_parity:false},null,2));
 if(process.argv.includes('--json'))console.log(JSON.stringify({root,wasm_sha256:sha(wasmBytes)}));else console.log('PASS genuine report PDFs, exact native replies and CLI dependency semantics, source/DOM/byte records, explicit print, unverified saved-file association, retirement/refusals and independent PDF inspection. Evidence: '+root);
}catch(error){if(page){console.error(await page.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,preview:document.querySelector('#commission-preview-status')?.textContent})));await page.screenshot({path:root+'/failure.png',fullPage:true});}throw error;}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}

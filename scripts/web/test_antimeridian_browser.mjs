import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/antimeridian-browser'),staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));
  if(file!==wasm&&!file.startsWith(staticRoot+path.sep))throw new Error('Outside root');
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.executions=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message?.type==='run')executions.push(message.request);return super.postMessage(message,...rest);}};});
 await page.goto(origin);
 const fixtures=['state-72','state-150','national-72','national-150',...(process.argv.includes('--original-ak')?['original-ak-national']:[])];
 let matches=0;
 for(const name of fixtures){
  const filename=path.resolve('target/antimeridian-wasm/'+name+'.bisect'),expected=JSON.parse(await fs.readFile(filename,'utf8')),before=await page.evaluate(()=>executions.length);
  await page.locator('#open-project').setInputFiles(filename);
  await page.waitForFunction(()=>document.querySelector('#open-project').value===''&&document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
  assert.equal(await page.evaluate(()=>executions.length),before);
  assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);assert.equal(await page.locator('#native-map-preview img').count(),0);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);
  const image=page.locator('#native-map-preview img');await image.evaluate(img=>img.decode());
  assert.deepEqual(await image.evaluate(img=>[img.naturalWidth,img.naturalHeight]),[expected.result.width,expected.result.height]);
  if(name.startsWith('state-'))assert(await image.evaluate(img=>img.naturalHeight/img.naturalWidth)>.25,'Antimeridian state map must retain a useful aspect ratio');
  for(const kind of ['png','svg']){
   const waiting=page.waitForEvent('download');waiting.catch(()=>{});await page.locator('#download-native-map-'+kind).click();const file=path.join(root,name+'.'+kind);await(await waiting).saveAs(file);
   assert.deepEqual(await fs.readFile(file),kind==='png'?Buffer.from(expected.result.png_base64,'base64'):Buffer.from(expected.result.svg_text));
  }
  const waiting=page.waitForEvent('download');waiting.catch(()=>{});await page.locator('#save-project').click();const savedFile=path.join(root,name+'.bisect');await(await waiting).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));assert.deepEqual(saved.result,expected.result);assert.deepEqual(saved.files,expected.files);matches++;
  await page.screenshot({path:path.join(root,name+'.png-preview.png'),fullPage:true});
 }
 await page.locator('#open-project').setInputFiles(path.join(root,fixtures.at(-1)+'.bisect'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);assert.equal(await page.locator('#native-map-preview img').count(),0);assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({matches,inert_open:true,whole_native_results:true,exact_sources_exports:true,decoded_map_dimensions:true,original_alaska:process.argv.includes('--original-ak')},null,2));
 console.log('Antimeridian browser: '+matches+' exact native worker results, decoded state/inset PNGs, source/project and SVG/PNG retention, safe inert reopening passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}

// Explicit live diagnostic: one pinned RI TIGER GET through a local instance
// of the Worker Fetch handler. Own profile/server; no user browser or deployment.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {Readable} from 'node:stream';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {relaySource} from '../../cloudflare/source-worker.mjs';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tigerRequest} from '../../web/lab/tiger-files.js';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/source-relay-live-browser'),source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI'],year:'2020',data_types:['tiger'],selected_paths:[]}});assert.equal(plan.items.length,1);const target=plan.items[0].url;let context;const upstreamRequests=[],incoming=[];
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://'+req.headers.host);incoming.push(url.pathname);if(url.pathname==='/api/source'){
 const controller=new AbortController();res.on('close',()=>controller.abort());const response=await relaySource(new Request(url,{method:req.method,headers:req.headers,signal:controller.signal}),{fetcher:async(u,options)=>{assert.equal(u,target);upstreamRequests.push(u);return fetch(u,options);}});res.writeHead(response.status,Object.fromEntries(response.headers));if(response.body){const body=Readable.fromWeb(response.body);body.on('error',()=>res.destroy());body.pipe(res);}else res.end();return;
 }const file=url.pathname==='/'?null:path.resolve(source,'.'+decodeURIComponent(url.pathname));if(file&&!file.startsWith(source+path.sep))throw Error('Path refused');res.setHeader('Content-Type',file?.endsWith('.js')?'application/javascript':'text/html');res.end(file?await fs.readFile(file):'<!doctype html><title>Isolated live public source relay diagnostic</title>');}catch{res.writeHead(500);res.end('Source test failed.');}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.stack));await page.goto(origin);
 const acquired=await page.evaluate(async plan=>{const {acquireFetchSource}=await import('/fetch-acquisition.js');return acquireFetchSource(plan,0,{sourceTransport:'site-relay',timeoutMs:60000});},plan);
 assert.equal(acquired.receipt.transport,'site-relay');assert.equal(acquired.receipt.availability,'acquired');assert.equal(acquired.receipt.url,target);assert.equal(upstreamRequests.length,1);assert.equal(acquired.inputs.archive.archive_sha256,acquired.receipt.source_sha256);assert.deepEqual(errors,[]);
 const archive=acquired.inputs.archive,request=tigerRequest({kind:'tracts',year:2020,shp:archive.sources.shp,dbf:archive.sources.dbf,shx:archive.sources.shx||null});const result=engine.execute(request),file=path.join(root,'native-tiger-request.json');await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',timeout:120000,maxBuffer:96*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(result,JSON.parse(native.stdout));
 await fs.writeFile(path.join(root,'acquisition.json'),JSON.stringify(acquired));await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),timestamp:new Date().toISOString(),live_provider_url:target,live_upstream_get_requests:upstreamRequests.length,browser_contacts_only_same_origin:true,raw_zip_bytes:acquired.receipt.byte_length,raw_zip_sha256:acquired.receipt.source_sha256,crc_checked_archive_members:archive.members.map(m=>({path:m.path,decoded_bytes:m.decoded_bytes})),native_tiger_preparation_exact:true,result_schema:result.schema_version,requests:incoming,page_errors:errors,scope:'Actual pinned RI2020 Census ZIP downloaded once through local Node execution of the Worker Fetch handler, then extracted in a real browser Worker and prepared through actual WASM/native tool comparison. Does not prove a deployed Cloudflare Worker or all-provider/vintage capacity.'},null,2)+'\n');console.log(`Live source relay: ${acquired.receipt.byte_length} exact RI ZIP bytes, browser worker CRC extraction and whole native/WASM TIGER preparation matched; one upstream GET, no direct browser provider request.`);
}finally{await context?.close();await new Promise(r=>server.close(r));}

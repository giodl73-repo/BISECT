// Actual Workbench replay of the native unused-flag counterfactual matrix.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const [toolkit, matrix, out] = process.argv.slice(2,5).map(v=>path.resolve(v));
const sourceFreePrint=process.argv.includes('--source-free-print');
const read=async p=>JSON.parse(await fs.readFile(p));
const sha=b=>createHash('sha256').update(b).digest('hex');
const status=await read(path.join(matrix,'status.json'));
const gate=await read(path.join(matrix,'gate-status.json'));
const wasm=await fs.readFile(path.join(toolkit,'bisect_wasm.wasm'));
assert.equal(gate.status,'passed');assert.equal(status.cases.length,25);
assert.equal(sha(wasm),status.artifacts['engine.wasm'].sha256);
await fs.mkdir(out);
const mime={'.js':'application/javascript','.mjs':'application/javascript','.wasm':'application/wasm',
  '.html':'text/html','.css':'text/css','.json':'application/json'};
const server=createServer(async(req,res)=>{
  try{
    const {pathname}=new URL(req.url,'http://localhost');
    const file=path.resolve(toolkit,'.'+decodeURIComponent(pathname==='/'?'/index.html':pathname));
    assert(file.startsWith(toolkit+path.sep));
    res.setHeader('Content-Type',mime[path.extname(file)]??'application/octet-stream');res.end(await fs.readFile(file));
  }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
let browser,page;
const cases=[],errors=[];
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe',
    args:['--disable-background-networking','--disable-extensions']});
  const context=await browser.newContext({acceptDownloads:true});
  await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
  await context.addInitScript(()=>{
    window.calls=[];window.replies=[];
    const W=Worker;
    window.Worker=class extends W{
      constructor(url,...args){super(url,...args);if(new URL(url,location.href).pathname==='/wasm-worker.js'){
        this.engine=true;this.addEventListener('message',({data})=>replies.push(data));}}
      postMessage(m,...args){if(this.engine&&m?.type==='run')calls.push({id:m.id,operation:m.request.operation,cMetis:m.cMetis??false});
        return super.postMessage(m,...args);}
    };
  });
  page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
  await page.goto(origin);
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const count=()=>page.evaluate(()=>({calls:calls.length,replies:replies.length}));
  const download=async(id,file)=>{const d=page.waitForEvent('download');await page.locator('#'+id).click();await(await d).saveAs(file);};
  const exportButton=page.locator('#download-native-states-run');
  const jobDir=path.join(matrix,'wasm-cli-execution');
  for(const row of status.cases){
    const name=row.case,family=row.family,print=row.changed_field==='print_only';
    const dir=path.join(out,name);await fs.mkdir(dir);
    const original=await read(path.join(jobDir,print?family+'-baseline':name,'result.bisect'));
    const project=structuredClone(original);
    if(print){
      const record=await read(path.join(jobDir,name,'raw.json'));
      project.name=name;project.result=null;project.lastOperation=null;
      if(family==='state'){
        project.files.stateWorkflowBinding=record.binding;
        project.files.stateCommandInputs=record.binding.engine_command;
      }else{
        project.files[family==='run'?'runCommandInputs':'statesCommandInputs']=record.original_command;
        if(sourceFreePrint)project.files.stateGraphInputs={...project.files.stateGraphInputs,graph:null,auxiliary:null};
      }
    }
    const input=path.join(dir,'source.bisect');await fs.writeFile(input,JSON.stringify(project),{flag:'wx'});
    const previous=await count();await openWorkbenchProject(page,input);
    assert.equal((await count()).calls,previous.calls);assert(await exportButton.isDisabled());
    await page.locator('#execute').click();
    await page.waitForFunction(n=>calls.length>n&&!document.querySelector('#execute').disabled,previous.calls,{timeout:30000});
    const trace=await page.evaluate(n=>({calls:calls.slice(n.calls),replies:replies.slice(n.replies)}),previous);
    const saved=path.join(dir,'saved.bisect');await download('save-project',saved);
    const packet=await read(saved);
    const result=packet.result?.result??packet.result;
    assert(trace.calls.length&&trace.calls.every(c=>!c.cMetis));
    if(print){
      assert(await exportButton.isDisabled());
      assert(!trace.replies.some(r=>r.type==='progress'&&/partition|source-preparation/.test(r.progress?.stage??'')));
      if(family==='state'){
        assert.match(await page.locator('#result-status').textContent(),/No source prepared or partition executed/);
        assert(!result?.prepared?.result?.assignments);
        assert.deepEqual(JSON.parse(packet.files.stateWorkflowBinding.source_command.arguments_json),row.arguments.slice(1));
      }else{
        assert.equal(result.execution_performed,false);
        assert.equal(result[family==='run'?'native_run_print_only':'native_states_print_only'],true);
        assert.deepEqual(result.native_command.arguments,row.arguments.slice(1));
        if(sourceFreePrint){
          assert.equal(packet.files.stateGraphInputs.graph,null);
          assert(!result.prepared_request_json&&!result.prepared_review&&!result.state_graph_source);
          assert.match(await page.locator('#state-command-output').textContent(),/print-only/);
          assert(await page.locator('#use-state-command-request').isHidden());
        }
      }
    }else{
      assert(!(await exportButton.isDisabled()));
      assert.deepEqual(result.prepared.result.assignments,original.result.prepared.result.assignments);
      assert.deepEqual(result.prepared.result.build_report.attempts.map(a=>a.seed),original.result.prepared.result.build_report.attempts.map(a=>a.seed));
      assert.deepEqual(result['native_'+family].recipe,original.result['native_'+family].recipe);
      const zip=path.join(dir,'result.zip');await download('download-native-states-run',zip);
    }
    await fs.writeFile(path.join(dir,'worker-trace.json'),JSON.stringify(trace),{flag:'wx'});
    cases.push({case:name,family,print_only:print,inert_open:true,
      native_produced_assignments:!!row.assignments,generated_assignment_sha256:result?.prepared?.result?.assignments?
        sha(JSON.stringify(result.prepared.result.assignments)):null});
  }
  assert.deepEqual(errors,[]);
  assert.equal(cases.filter(r=>r.generated_assignment_sha256).length,22);
  assert.equal(cases.filter(r=>r.print_only&&!r.generated_assignment_sha256).length,3);
  await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-workbench-native-cli-unused-flags',
    browser:browser.version(),wasm_sha256:sha(wasm),native_reference:status.artifacts['native-bisect.exe'].sha256,
    cases,source_free_bulk_print_only:sourceFreePrint,full_parity:false},null,2)+'\n',{flag:'wx'});
}catch(error){
  const status=await page?.locator('#result-status').textContent().catch(()=>null);
  await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({error:String(error.stack),status,cases,errors},null,2)+'\n');throw error;
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}

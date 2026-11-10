import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/real-census-preparation',engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
for(const kind of ['county','block-group','block']){
 const run=JSON.parse(await fs.readFile(root+'/RI/'+kind+'/run.json','utf8')),plan=run.practitioner_export.document_json,context=run.practitioner_export.context_json;
 assert(Buffer.byteLength(context)<=8*1024*1024);assert.deepEqual(JSON.parse(context),run.practitioner_export.context);
 const request={operation:'analyze-plan-contiguity',document_json:plan,context_json:context},file=root+'/RI/'+kind+'/context-reader-request.json';await fs.writeFile(file,JSON.stringify(request));
 const result=engine.execute(request),native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(result,native);
 await fs.writeFile(root+'/RI/'+kind+'/context-reader-proof.json',JSON.stringify(result));
 if(kind==='block'){assert(!context.includes('\n'));assert(Buffer.byteLength(JSON.stringify(run.practitioner_export.context,null,2))>8*1024*1024);}
 else assert(context.includes('\n'));
 console.log('Original RI '+kind+': '+Buffer.byteLength(context)+'-byte complete RCTX, exact typed fields/hash and independent native reader/BFS parity; '+(kind==='block'?'lossless compact encoding':'legacy pretty encoding')+'.');
}

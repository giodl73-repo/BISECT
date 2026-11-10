import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelRenameZip,readLabelRenameFolder} from '../../web/lab/label-rename-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root=path.resolve('target/label-rename-wasm');await fs.mkdir(root,{recursive:true});
const empty={built:[],analyzed:[],reported:[]};
const req=(files={},extra={})=>({operation:'rename-label-package',registry_json:JSON.stringify({a:{...empty,built:['2020','2020']}}),files_base64:Object.fromEntries(Object.entries(files).map(([p,b])=>[p,Buffer.from(b).toString('base64')])),directories:[],source_label:'a',destination_label:'b',force:false,...extra});
let comparisons=0,rejects=0;
async function inventory(base){const files={};const dirs=[];async function visit(rel=''){for(const e of await fs.readdir(path.join(base,rel),{withFileTypes:true})){const p=rel?rel+'/'+e.name:e.name;if(e.isDirectory()){dirs.push(p+'/');await visit(p);}else if(e.name!=='.bisect.lock')files[p]=await fs.readFile(path.join(base,p));}}await visit();return {files,dirs};}
async function exact(r){
 const out=engine.execute(r);const base=await fs.mkdtemp(path.join(root,'case-'+comparisons+'-'));
 await fs.writeFile(path.join(base,'.bisect'),r.registry_json);
 for(const [p,b]of Object.entries(r.files_base64)){await fs.mkdir(path.dirname(path.join(base,p)),{recursive:true});await fs.writeFile(path.join(base,p),Buffer.from(b,'base64'));}
 for(const d of r.directories)await fs.mkdir(path.join(base,d),{recursive:true});
 const n=spawnSync(path.resolve('target/debug/examples/label_rename_reference.exe'),[r.source_label,r.destination_label,...(r.force?['force']:[])],{cwd:base,encoding:'utf8'});assert.equal(n.status,0,n.stderr);
 const native=await inventory(base),wasm=Object.fromEntries(out.files.map(f=>[f.path,Buffer.from(f.base64??r.files_base64[f.source_path],'base64')]));
 assert.deepEqual(Object.keys(native.files).sort(),Object.keys(wasm).sort());
 for(const p of Object.keys(wasm)){if(p==='.bisect')assert.deepEqual(JSON.parse(native.files[p]),JSON.parse(wasm[p]));else assert.deepEqual(native.files[p],wasm[p],p);}
 for(const d of out.directories)assert.ok(native.dirs.includes(d),d);
 const inputs={registry:{name:'.bisect',base64:Buffer.from(r.registry_json).toString('base64')},files:r.files_base64,directories:r.directories,settings:{source_label:r.source_label,destination_label:r.destination_label,force:r.force}};
 const project=createProject({name:'Native label rename',files:{labelRenameInputs:inputs},operation:'rename-label-package',constraints:[],result:out,lastOperation:'rename-label-package'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await fs.writeFile(path.join(base,'result.zip'),labelRenameZip(out,inputs));
 await fs.writeFile(path.join(base,'expected.json'),JSON.stringify({files:Object.fromEntries(out.files.map(f=>[f.path,f.base64??r.files_base64[f.source_path]])),directories:out.directories}));
 if(comparisons===1)await fs.writeFile(path.join(root,'browser-project.bisect'),JSON.stringify(project));
 comparisons++;return out;
}
await exact(req());
await exact(req({'runs/a/index.json':'{"label":"a","x":1}','runs/a/2020/index.json':'{"label":"a"}','analysis/a/data.bin':Buffer.from([0,255]),'configs/a.yml':'unchanged'}));
await exact(req({'runs/a/index.json':'{"label":"different"}','analysis/a/index.json':'[1,2]','reports/a/index.json':'null'}));
await exact(req({}, {directories:['runs/a/empty/','reports/a/']}));
await exact(req({'runs/a/new':'new','runs/b/old':'old','analysis/b/keep':'keep'}, {registry_json:JSON.stringify({a:empty,b:empty}),force:true}));
await exact(req({'runs/a/new':'new','runs/b/old':'old'}, {force:true}));
await exact(req({'runs/a/file':'bytes','unrelated/file':'stay'},{source_label:'a',destination_label:'final_2026'}));
await exact(req({'runs/a/large.bin':Buffer.alloc(16*1024*1024,255)}));
await exact(req(Object.fromEntries(Array.from({length:512},(_,i)=>['runs/a/f'+i,'']))));
const paddedRegistry=JSON.stringify({a:empty});await exact(req({}, {registry_json:paddedRegistry+' '.repeat(1024*1024-Buffer.byteLength(paddedRegistry))}));
for(const r of [req({}, {source_label:'a',destination_label:'a',force:true}),req({}, {destination_label:'../b'}),req({}, {source_label:'missing'}),req({}, {registry_json:JSON.stringify({a:empty,b:empty})}),req({'runs/a/x':'x','runs/b/y':'y'}),req({'runs/a/index.json':'bad'}),req({'../escape':'x'}),req({'runs/a/X':'x','runs/a/x':'y'}),req({}, {destination_label:'A'}),req({}, {extra:true}),req({}, {directories:['runs/a']}),req({'runs/a':'file','runs/a/x':'child'})]){assert.throws(()=>engine.execute(r));rejects++;}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({comparisons,rejects},null,2));
let read=false;await assert.rejects(()=>readLabelRenameFolder([{name:'large',webkitRelativePath:'repo/runs/a/large',size:16*1024*1024+1,arrayBuffer:async()=>{read=true;}}]));assert.equal(read,false);
console.log(`Label rename: ${comparisons} actual native package comparisons and ${rejects} rejects passed.`);

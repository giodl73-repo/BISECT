import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {inspectNativeWorkspaceFiles} from '../../web/lab/native-aggregate-workspace-inspection.js';
await fs.mkdir('target/native-aggregate-effects',{recursive:true});
const out=await fs.mkdtemp(path.resolve('target/native-aggregate-effects/workspace-inspection-'));
function ioFrom(rows){return {async readdir(p){return Object.entries(rows).filter(([n])=>n.slice(0,n.lastIndexOf('/')<0?0:n.lastIndexOf('/'))===p).map(([n,v])=>({name:n.slice(n.lastIndexOf('/')+1),kind:v===null?'directory':'file'}));},async stat(p){return p in rows?{kind:rows[p]===null?'directory':'file'}:null;},async fileMetadata(p){return {byte_length:rows[p]};}};}
try{
 const deep={};for(let i=1;i<=24;i++)deep[Array(i).fill('d').join('/')]=null;deep[Array(24).fill('d').join('/')+'/file']=0;
 const deepReport=await inspectNativeWorkspaceFiles(ioFrom(deep));assert.equal(deepReport.portable_snapshot_eligible,false,'A 25-component file exceeds the backup path profile');assert(deepReport.snapshot_limits.some(s=>/depth|24/.test(s)));
 const manifestHeavy={};for(let i=0;i<18000;i++)manifestHeavy[String(i).padStart(5,'0')+'x'.repeat(190)]=0;
 const heavyReport=await inspectNativeWorkspaceFiles(ioFrom(manifestHeavy));assert.equal(heavyReport.portable_snapshot_eligible,false,'Manifest bytes must be considered without reading bodies');assert(heavyReport.snapshot_limits.some(s=>/manifest|4 MiB/.test(s)));
 const empty=await inspectNativeWorkspaceFiles(ioFrom({}));assert.equal(empty.portable_snapshot_eligible,true);assert(Object.isFrozen(empty.entries));
 const blocked=await inspectNativeWorkspaceFiles(ioFrom({'CON':0,'safe.bin':0}));assert.equal(blocked.blocked_entries.length,1);assert.equal(blocked.portable_snapshot_eligible,false);assert.deepEqual(blocked.entries,[{path:'safe.bin',kind:'file',byte_length:0}]);
 const refused=[];async function refusal(label,fn){await assert.rejects(fn);refused.push(label);}
 const one=ioFrom({'one':1});await refusal('case collision',()=>inspectNativeWorkspaceFiles({...one,readdir:async()=>[{name:'a',kind:'file'},{name:'A',kind:'file'}]}));
 let reads=0;await refusal('roster drift',()=>inspectNativeWorkspaceFiles({...one,readdir:async()=>++reads===1?[{name:'one',kind:'file'}]:[]}));
 await refusal('kind drift',()=>inspectNativeWorkspaceFiles({...one,stat:async()=>({kind:'directory'})}));
 await refusal('invalid length',()=>inspectNativeWorkspaceFiles({...one,fileMetadata:async()=>({byte_length:-1})}));
 await refusal('extra metadata authority',()=>inspectNativeWorkspaceFiles({...one,fileMetadata:async()=>({byte_length:1,confirmed:true})}));
 await refusal('unknown option',()=>inspectNativeWorkspaceFiles(one,{confirmed:true}));
 const abort=new AbortController();abort.abort();let called=false;await refusal('cancel before any IO',()=>inspectNativeWorkspaceFiles({...one,readdir:async()=>{called=true;return []; }},{signal:abort.signal}));assert.equal(called,false);
 await refusal('stale before any IO',()=>inspectNativeWorkspaceFiles({...one,readdir:async()=>{called=true;return []; }},{assertCurrent(){throw Error('Stale');}}));assert.equal(called,false);
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed',deep_path:deepReport,heavy_manifest:{paths:heavyReport.entries.length,snapshot_limits:heavyReport.snapshot_limits},blocked,refused,scope:'Metadata profile controls. Real browser zero-body inspection is covered by a separate Chrome fixture.'},null,2));console.log('PASS native workspace inspection profiles: '+out);
}catch(error){await fs.writeFile(path.join(out,'failure.json'),JSON.stringify({message:error.message,stack:error.stack},null,2));throw error;}
console.log(JSON.stringify({native_aggregate_evidence_root:out}));

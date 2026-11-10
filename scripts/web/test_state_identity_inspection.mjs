// Inspection foundation only: no State flag is enabled for execution here.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {statePreparedFlagRefusals} from '../../web/lab/state-command-files.js';
const file=process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm';
const bytes=await fs.readFile(file),engine=await instantiateEngine(bytes);
const cases=[
 ['registered-default',['--state','RI'],'rhode_island',false,false],
 ['ignored-override',['--state','RI','--state-name','Ignored'],'rhode_island',false,true],
 ['registered-adjacency',['--state','RI','--adjacency','not-opened.adj.bin'],'rhode_island',false,false],
 ['unknown-default',['--state','ZZ','--adjacency','not-opened.adj.bin','--districts','2'],'zz',false,false],
 ['raw-override',['--state','ZZ','--adjacency','not-opened.adj.bin','--districts','2','--state-name','MiXeD Research'],'MiXeD Research',true,false],
 ['unsafe-observation',['--state','ZZ','--adjacency','not-opened.adj.bin','--state-name','../outside'],'../outside',true,false],
 ['unicode-observation',['--state','ZZ','--adjacency','not-opened.adj.bin','--state-name','東京'],'東京',true,false],
 ['empty-observation',['--state','ZZ','--adjacency','not-opened.adj.bin','--state-name='],'',true,false],
 ['fallback-space',['--state','North State','--adjacency','not-opened.adj.bin'],'north state',false,false],
];
const outcomes=[];
for(const [name,args,expected,effective,ignored] of cases){
 const result=engine.execute({operation:'inspect-native-state-identity',arguments:args});
 assert.equal(result.schema_version,'bisect-native-state-identity-v1');
 assert.equal(result.effective_state_name,expected,name);
 assert.equal(result.override_effective,effective);assert.equal(result.override_ignored,ignored);
 assert.equal(result.execution_performed,false);assert.equal(result.filesystem_reads_performed,false);assert.equal(result.path_authority_granted,false);
 assert.match(result.policy_sha256,/^sha256:[0-9a-f]{64}$/);
 if(args.some(v=>v==='--state-name'||v.startsWith('--state-name=')))assert(statePreparedFlagRefusals(engine.execute({operation:'inspect-native-state-command',arguments:args})).includes('--state-name'));
 outcomes.push({name,arguments:args,result});
}
assert.throws(()=>engine.execute({operation:'inspect-native-state-identity',arguments:['--not-a-native-flag']}));
const root=path.resolve('target/state-identity/wasm-'+Date.now());await fs.mkdir(root,{recursive:true});
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed',wasm:{path:file,sha256:createHash('sha256').update(bytes).digest('hex'),byte_length:bytes.length},outcomes,parser_error_refused:true,direct_state_name_guard_retained:true,scope:'Inspection foundation, not exposed workflow, source/target grant, partition execution or complete flag acceptance.'},null,2));
console.log(JSON.stringify({root,status:'passed',cases:outcomes.length}));

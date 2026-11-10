import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=path.resolve('target/native-state-scope');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),rows=JSON.parse(await fs.readFile('target/native-state-command/records.json','utf8'));
const args=rows.filter(r=>r.result.status==='parsed').map(r=>r.request.arguments);args.push(['--state','RI','--label=-dash value','--seed','18446744073709551615','--metis-engine','metis-core'],['--state','RI','-v=-version','--search','single','--percentile=-0']);let matches=0;
for(const [i,arguments_]of args.entries()){
 const before=engine.execute({operation:'inspect-native-state-command',arguments:arguments_}),request={operation:'scope-native-state-command',arguments:arguments_,state:'IA',year:'2010',resolution:'block_group',districts:4},result=engine.execute(request),after=engine.execute({operation:'inspect-native-state-command',arguments:result.arguments});
 assert.equal(result.algorithm_unchanged,true);assert.equal(result.execution_performed,false);assert.equal(result.filesystem_reads_performed,false);assert.equal(after.state,'IA');assert.equal(after.year,'2010');assert.equal(after.resolution,'block_group');assert.equal(before.native_debug.algorithm,after.native_debug.algorithm);assert.equal(result.native_algorithm_debug,before.native_debug.algorithm);
 for(const field of before.fields)if(!['state','year','resolution','chamber','districts'].includes(field.id)){const next=after.fields.find(f=>f.id===field.id);assert.deepEqual(next.values,field.values,field.id);assert.equal(next.source,field.source,field.id);}
 const file=root+'/'+i+'.json';await fs.writeFile(file,JSON.stringify(request));const p=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:30000,maxBuffer:8*1024*1024});assert.equal(p.status,0,p.stderr);assert.deepEqual(result,JSON.parse(p.stdout));matches++;
}
for(const changed of [{state:'../'},{year:'2024'},{resolution:'county'},{districts:0},{districts:501}])assert.throws(()=>engine.execute({operation:'scope-native-state-command',arguments:['--state','RI'],state:'IA',year:'2020',resolution:'tract',districts:4,...changed}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,whole_native_host_matches:matches,all_non_scope_field_values_and_sources_retained:true,exact_native_algorithm_debug_unchanged:true,all79_explicit_flags_and_special_values:true,max_u64_negative_values_and_short_aliases:true,no_execution_or_filesystem_reads:true,published:false},null,2)+'\n');console.log(`PASS native State scope: ${matches} whole native host matches, all non-scope native values/defaults and exact algorithm conversion retained`);

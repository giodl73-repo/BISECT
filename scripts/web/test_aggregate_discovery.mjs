import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readNativeAggregateFolder,nativeAggregateRequest} from '../../web/lab/native-aggregate-files.js';

const version='wasm_aggregate_discovery_'+Date.now(),root=path.resolve('outputs',version,'2020');
assert(root.startsWith(path.resolve('outputs')+path.sep));
const sources={
 'alabama/analysis/summary.json':'{"districts":[{"district":1,"population":10}]}',
 'vermont/data/final_assignments.json':'{"50001000100":1}',
 'wyoming/backup/analysis/summary.json':'{"districts":[{"district":99,"population":99}]}',
 'README.md':'collection'
};
const files=[];let reads=0;
for(const [relative,text]of Object.entries(sources)){
 const file=path.join(root,'states',relative);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,text);
 const bytes=Buffer.from(text);files.push({name:path.basename(file),webkitRelativePath:'states/'+relative,size:bytes.length,arrayBuffer:async()=>{reads++;return bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);}});
}
const input=await readNativeAggregateFolder(files,{year:'2020',types:['summary'],format:'json',csv:true});
assert.deepEqual(Object.keys(input.reports),['alabama','vermont','wyoming']);assert.equal(reads,1);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const actual=engine.execute(nativeAggregateRequest(input));
const native=spawnSync(path.resolve('target/debug/bisect.exe'),['aggregate','--version',version,'--year','2020','--types','summary','--format','json','--csv'],{encoding:'utf8'});
assert.equal(native.status,0,native.stderr);assert.match(native.stderr,/\[aggregate\] 3 states/);
const names=(await fs.readdir(path.join(root,'national'))).sort();assert.deepEqual(names,Object.keys(actual.files).sort());
for(const name of names)assert.equal(actual.files[name],await fs.readFile(path.join(root,'national',name),'utf8'));
await fs.mkdir('target/aggregate-discovery',{recursive:true});
await fs.writeFile('target/aggregate-discovery/evidence.json',JSON.stringify({native_discovered_states:3,browser_discovered_states:Object.keys(input.reports),source_reads:reads,nested_analysis_ignored:true,actual_cli_output_byte_matches:names.length},null,2));
console.log('Aggregate discovery: native and browser discover three immediate states, read one source, ignore nested backup analyses and match all three native output files.');

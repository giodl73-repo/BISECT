import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';
import {nativeOutputDirectoryHarness} from './native-output-directory.mjs';
export async function createNativeAggregateOracle(nativeExecutable){
 await fs.mkdir(path.resolve('target/native-aggregate-effects'),{recursive:true});
 const out=await fs.mkdtemp(path.resolve('target/native-aggregate-effects/native-oracle-')),h=nativeOutputDirectoryHarness(out),source=path.resolve(nativeExecutable),cli=path.join(out,'native.exe'),cliBytes=await fs.readFile(source),sha=b=>createHash('sha256').update(b).digest('hex'),prefix='v1/2020/',national=prefix+'national/',summary='{"analyzer":"summary","districts":[{"district":1,"population":17}]}',prop='{"analyzer":"proportionality","votes":100}',baseSource={[prefix+'states/alpha/analysis/summary.json']:summary},rows=[];
 await fs.writeFile(cli,cliBytes);if(process.platform!=='win32')await fs.chmod(cli,0o700);
 const nativeSource=await fs.readFile('crates/bisect-cli/src/aggregate.rs');await fs.writeFile(path.join(out,'aggregate.rs'),nativeSource);
 await fs.writeFile(path.join(out,'identity.json'),JSON.stringify({native_cli_sha256:sha(cliBytes),native_source_sha256:sha(nativeSource)},null,2));
async function seed(root,files){for(const [relative,value]of Object.entries(files)){const p=path.join(root.dir,relative);if(value===null)await fs.mkdir(p,{recursive:true});else{await fs.mkdir(path.dirname(p),{recursive:true});await fs.writeFile(p,value);}}}

async function snapshot(root){const tree={};async function walk(dir,base=''){for(const e of await fs.readdir(dir,{withFileTypes:true})){const relative=base?base+'/'+e.name:e.name,p=path.join(dir,e.name);if(e.isDirectory()){tree[relative]=null;await walk(p,relative);}else{const raw=await fs.readFile(p);tree[relative]={sha256:sha(raw),byte_length:raw.length,base64:raw.toString('base64')};}}}await walk(root.dir);return Object.fromEntries(Object.entries(tree).sort());}

const definitions=[

 {name:'reference',outputs:['us_summary.json','us_summary.csv','us_summary_districts.json']},

 {name:'missing-states',files:{[prefix+'retained.txt']:'KEEP'},failure:true,outputs:[],candidate:true},

 {name:'states-is-file',files:{[prefix+'states']:'KEEP'},failure:true,outputs:[]},

 {name:'empty-states',files:{[prefix+'states']:null},outputs:[]},

 {name:'national-is-file',extra:{[prefix+'national']:'KEEP'},failure:true,outputs:[]},

 {name:'merged-temp-is-directory',extra:{[national+'us_summary.tmp.json/keep.txt']:'KEEP'},failure:true,outputs:[],candidate:true},

 {name:'merged-final-is-directory-force',extra:{[national+'us_summary.json/keep.txt']:'KEEP'},force:true,failure:true,outputs:['us_summary.tmp.json']},

 {name:'merged-final-is-directory-skip',extra:{[national+'us_summary.json/keep.txt']:'KEEP'},outputs:[]},

 {name:'csv-is-directory',extra:{[national+'us_summary.csv/keep.txt']:'KEEP',[national+'us_summary_districts.json']:'OLD RECORDS'},failure:true,outputs:['us_summary.json']},

 {name:'records-temp-is-directory',extra:{[national+'us_summary_districts.tmp.json/keep.txt']:'KEEP'},failure:true,outputs:['us_summary.json','us_summary.csv']},

 {name:'records-final-is-directory',extra:{[national+'us_summary_districts.json/keep.txt']:'KEEP'},failure:true,outputs:['us_summary.json','us_summary.csv','us_summary_districts.tmp.json']},

 {name:'existing-scratch-files',extra:{[national+'us_summary.tmp.json']:'OLD MERGED TEMP',[national+'us_summary_districts.tmp.json']:'OLD ROW TEMP'},outputs:['us_summary.json','us_summary.csv','us_summary_districts.json'],removed:['us_summary.tmp.json','us_summary_districts.tmp.json'],candidate:true},

 {name:'later-analyzer-temp-collision',extra:{[prefix+'states/alpha/analysis/proportionality.json']:prop,[national+'us_proportionality.tmp.json/keep.txt']:'KEEP'},types:['summary','proportionality','summary'],failure:true,outputs:['us_summary.json','us_summary.csv','us_summary_districts.json']},

 {name:'repeated-analyzer-skip',types:['summary','summary'],outputs:['us_summary.json','us_summary.csv','us_summary_districts.json']},

 {name:'repeated-analyzer-force',types:['summary','summary'],force:true,outputs:['us_summary.json','us_summary.csv','us_summary_districts.json']},

 {name:'source-is-directory',files:{[prefix+'states/alpha/analysis/summary.json/keep.txt']:'KEEP'},outputs:[]},
 {name:'source-invalid-json',files:{[prefix+'states/alpha/analysis/summary.json']:'not JSON'},outputs:[]},
 {name:'source-empty-json',files:{[prefix+'states/alpha/analysis/summary.json']:''},outputs:[]},
 {name:'source-BOM-json',files:{[prefix+'states/alpha/analysis/summary.json']:'\ufeff'+summary},outputs:[]},
 {name:'source-invalid-utf8',files:{[prefix+'states/alpha/analysis/summary.json']:Buffer.from([255])},outputs:[]},
 {name:'state-roster-nondirectory',extra:{[prefix+'states/ignored.txt']:'KEEP'},outputs:['us_summary.json','us_summary.csv','us_summary_districts.json']},
 {name:'parquet-guidance',format:'parquet',csv:false,outputs:['us_summary.json']},

 {name:'json-none-format',format:'none',csv:false,outputs:['us_summary.json']}

];
let reference;
for(const definition of definitions){const nativeBase=await h.directory(definition.name+'-native'),root=await nativeBase.getDirectoryHandle('outputs',{create:true}),files={...(definition.files||baseSource),...definition.extra};await seed(root,files);const before=await snapshot(root),args=['aggregate','--version','v1','--year','2020','--types',...(definition.types||['summary']),'--format',definition.format||'json',...(definition.csv===false?[]:['--csv']),...(definition.force?['--force']:[])],result=spawnSync(cli,args,{cwd:nativeBase.dir,maxBuffer:16*1024*1024});await fs.writeFile(path.join(nativeBase.dir,'arguments.json'),JSON.stringify(args));await fs.writeFile(path.join(nativeBase.dir,'stdout.bin'),result.stdout);await fs.writeFile(path.join(nativeBase.dir,'stderr.bin'),result.stderr);await fs.writeFile(path.join(nativeBase.dir,'before.json'),JSON.stringify(before,null,2));const after=await snapshot(root);await fs.writeFile(path.join(nativeBase.dir,'after.json'),JSON.stringify(after,null,2));assert.equal(result.status,definition.failure?1:0,result.stderr.toString());

 if(!reference)reference=after;

 for(const name of definition.outputs){const ref=name.replace('.tmp.json','.json');assert.deepEqual(after[national+name],reference[national+ref],definition.name+' '+name);}

 for(const name of definition.removed||[])assert.equal(after[national+name],undefined);

 if(definition.name==='missing-states'||definition.name==='states-is-file'||definition.name==='empty-states')assert.equal(after[prefix+'national'],null);

 for(const [name,value]of Object.entries(before))if(!definition.removed?.includes(name.slice(national.length))&&!definition.outputs.includes(name.slice(national.length)))assert.deepEqual(after[name],value,definition.name+' retained '+name);

 rows.push({name:definition.name,native_exit:result.status,native_root:root.dir,args,expected_outputs:definition.outputs,removed:definition.removed||[]});

}

assert.equal(sha(await fs.readFile(source)),sha(cliBytes));const evidence={status:'passed-native-aggregate-effect-prefix-audit',cases:rows,source_sha256:sha(nativeSource),native_cli_sha256:sha(cliBytes),scope:'Twenty-three fresh actual native Aggregate lifecycle/source/format cases. Native byte trees and failure prefixes; no natural failure, cross-platform rename or full host acceptance claim.'};await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify(evidence,null,2));return {directory:out,...evidence};
}

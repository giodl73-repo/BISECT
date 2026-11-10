import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {pl94Request,readPl94Source} from '../../web/lab/pl94-files.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/pl94-large-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const hash=data=>createHash('sha256').update(data).digest('hex');
const options={encoding:'utf8',maxBuffer:128*1024*1024};
// Independently written file source crosses the former decoded limit. Blank
// records exercise streaming without allocating an enormous individual line.
execFileSync('python',['-c',String.raw`
import pathlib,gzip,shutil
r=pathlib.Path('target/pl94-large-wasm')
with (r/'geography.pl').open('wb') as f:
 for _ in range(129):f.write((b' '*1023+b'\n')*1024)
 f.write(b'PLST|AA|750|||||0000001|7500000US440010001001001|unused\n')
(r/'population.pl').write_bytes(b'PLST|AA|000|01|0000001|11|unused\n')
with (r/'geography.pl').open('rb') as f,gzip.open(r/'geography.gz','wb') as g:shutil.copyfileobj(f,g)
`]);
async function check(label,geoPath,popPath,expectedTotal){
 const input={year:2020,geography:null,population:null};
 for(const [part,file]of [['geography',geoPath],['population',popPath]]){
  const raw=await fs.readFile(file),bytes=label==='synthetic'&&part==='geography'?await fs.readFile(root+'/geography.gz'):gzipSync(raw);
  input[part]=await readPl94Source({name:file.split('/').at(-1)+'.gz',size:bytes.length,arrayBuffer:async()=>bytes});
 }
 const request=pl94Request(input),result=engine.execute(request);
 await fs.writeFile(root+'/'+label+'-request.json',JSON.stringify(request));
 const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/'+label+'-request.json'],options));assert.deepEqual(host,result);
 const native=execFileSync('target/debug/examples/pl94_reference.exe',['2020',geoPath,popPath],options);assert.equal(result.csv_text,native);assert.equal(result.csv_sha256,hash(native));assert.equal(result.total_population,expectedTotal);
 for(const [part,file]of [['geography',geoPath],['population',popPath]]){const raw=await fs.readFile(file);assert.equal(result[part+'_decoded'].decoded_sha256,hash(raw));assert.equal(result[part+'_decoded'].decoded_byte_length,raw.length);assert.equal(result[part].sha256,hash(Buffer.from(input[part].base64,'base64')));}
 const text=createWorkbenchArchive({name:label+' PL94 original sources',files:{pl94Inputs:input},operation:'prepare-pl94-source',result,lastOperation:'prepare-pl94-source'});
 assert.deepEqual(parseWorkbenchArchive(text).files.pl94Inputs,input);await fs.writeFile(root+'/'+label+'.bisect-workbench',text);
 await fs.writeFile(root+'/'+label+'-receipt.json',JSON.stringify({block_count:result.block_count,total_population:result.total_population,geography_decoded:result.geography_decoded,population_decoded:result.population_decoded,csv_sha256:result.csv_sha256,scope:'PL94 reader and source archive parity; graph preparation and engine execution are separate.'}));
 console.log(label+': '+result.block_count+' blocks, total '+result.total_population+', native-host/WASM/filesystem CSV and source archive parity passed.');
}
await check('synthetic',root+'/geography.pl',root+'/population.pl','11');
if(process.argv.includes('--local-ca'))await check('california','data/2020/redistricting/california/cageo2020.pl','data/2020/redistricting/california/ca000012020.pl','39538223');

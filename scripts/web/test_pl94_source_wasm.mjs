import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {pl94Request,pl94Download,readPl94Source} from '../../web/lab/pl94-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/pl94-source-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const geo2020=(record,id,sum='750')=>`PLST|AA|${sum}|||||${record}|7500000US${id}|unused\n`;
const pop2020=(record,n)=>`PLST|AA|000|01|${record}|${n}|unused\n`;
function fixed(year,record,id,pop){const bytes=Buffer.alloc(year===2000?301:65,32);bytes.write('PLST',0);bytes.write('750',8);bytes.write(record,18);for(const [start,text]of year===2000?[[29,id.slice(0,2)],[31,id.slice(2,5)],[55,id.slice(5,11)],[62,id.slice(11)],[292,String(pop).padStart(9)]]:[[27,id.slice(0,2)],[29,id.slice(2,5)],[54,id.slice(5,11)],[61,id.slice(11)]])bytes.write(text,start);return Buffer.concat([bytes,Buffer.from('\n')]);}
const a='440010001001001',b='440010002001001';
const cases=[
 ['2020',2020,geo2020('0000002',b)+geo2020('0000001',a)+geo2020('0000003',a,'040'),pop2020('0000003',99)+pop2020('0000002',7)+pop2020('0000001',11),`geoid,population\n${a},11\n${b},7\n`,'18'],
 ['2010',2010,Buffer.concat([fixed(2010,'0000002',b),fixed(2010,'0000001',a)]),'PLST,AA,000,01,0000002,7\nPLST,AA,000,01,0000001,11\n',`geoid,population\n${a},11\n${b},7\n`,'18'],
 ['2000',2000,Buffer.concat([fixed(2000,'0000002',b,7),fixed(2000,'0000001',a,11)]),null,`geoid,population\n${a},11\n${b},7\n`,'18'],
 ['exact-i64',2020,geo2020('0000001',a)+geo2020('0000002',b),pop2020('0000001','9223372036854775807')+pop2020('0000002',-1),`geoid,population\n${a},9223372036854775807\n${b},-1\n`,'9223372036854775806'],
 ['empty',2020,'\n','\n','geoid,population\n','0'],
 ['duplicate-geoid-native',2010,Buffer.concat([fixed(2010,'0000001',a),fixed(2010,'0000002',a)]),'PLST,AA,000,01,0000001,7\nPLST,AA,000,01,0000002,7\n',`geoid,population\n${a},7\n${a},7\n`,'14'],
 ['2000-unused-pop',2000,fixed(2000,'0000001',a,11),Buffer.from([255,0,1]),`geoid,population\n${a},11\n`,'11'],
];
const source=(name,data)=>({name,base64:Buffer.from(data).toString('base64')});
async function requestFor(year,geo,pop){await fs.writeFile(root+'/geo.txt',geo);if(pop!==null)await fs.writeFile(root+'/pop.txt',pop);return pl94Request({year,geography:source('geography.pl',geo),population:pop===null?null:source('population.pl',pop)});}
for(const [name,year,geo,pop,expected,total]of cases){
 const request=await requestFor(year,geo,pop),result=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8'})),result);
 const native=execFileSync('target/debug/examples/pl94_reference.exe',[String(year),root+'/geo.txt',pop===null?root+'/does-not-exist.txt':root+'/pop.txt'],{encoding:'utf8'});assert.equal(result.csv_text,native);assert.equal(native,expected);assert.equal(result.total_population,total);assert.equal(result.population_used,year!==2000);assert.equal(result.negative_population_rows,name==='exact-i64'?1:0);assert.equal(result.geography.sha256,createHash('sha256').update(Buffer.from(geo)).digest('hex'));assert.equal(result.csv_sha256,createHash('sha256').update(expected).digest('hex'));assert.equal(result.execution_performed,false);assert.equal(Buffer.from(pl94Download(result).bytes).toString(),expected);
 const files={pl94Inputs:request.input},project=createProject({name:'PL94 '+name,files,operation:'prepare-pl94-source',constraints:[],result,lastOperation:'prepare-pl94-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
 const zipped=engine.execute({...request,input:{...request.input,geography:source('geography.pl.gz',gzipSync(Buffer.from(geo))),population:pop===null?null:source('population.pl.gz',gzipSync(Buffer.from(pop)))}});assert.equal(zipped.csv_text,expected);assert.equal(zipped.total_population,total);assert.equal(zipped.geography_decoded.encoding,'gzip');assert.equal(zipped.geography_decoded.decoded_sha256,createHash('sha256').update(Buffer.from(geo)).digest('hex'));if(year===2000)assert.equal(zipped.population_decoded,null);
}
for(const [name,year,geo,pop]of [
 ['duplicate-record',2020,geo2020('0000001',a)+geo2020('0000001',b),pop2020('0000001',1)],
 ['duplicate-population',2020,geo2020('0000001',a),pop2020('0000001',1)+pop2020('0000001',2)],
 ['missing-population',2020,geo2020('0000001',a),'\n'],
 ['overflow',2020,geo2020('0000001',a),pop2020('0000001','9223372036854775808')],
 ['bad-geoid',2020,geo2020('0000001','not-a-block'),pop2020('0000001',1)],
 ['short-2010',2010,'bad\n','bad\n'],
 ['duplicate-2000',2000,Buffer.concat([fixed(2000,'0000001',a,1),fixed(2000,'0000002',a,2)]),null],
 ['short-2000',2000,'bad\n',null],
]){const request=await requestFor(year,geo,pop);assert.throws(()=>engine.execute(request));assert.throws(()=>execFileSync('target/debug/examples/pl94_reference.exe',[String(year),root+'/geo.txt',root+'/pop.txt'],{stdio:'pipe'}));}
const request=await requestFor(2020,geo2020('0000001',a),pop2020('0000001',1));assert.throws(()=>engine.execute({...request,input:{...request.input,population:null}}));assert.throws(()=>engine.execute({...request,input:{...request.input,year:1990}}));assert.throws(()=>engine.execute({...request,input:{...request.input,geography:{name:'geo',base64:'!!!!'}}}));
const validGzip=gzipSync(Buffer.from(request.input.geography.base64,'base64')),badCrc=Buffer.from(validGzip);badCrc[badCrc.length-8]^=1;for(const data of [badCrc,validGzip.subarray(0,validGzip.length-6)])assert.throws(()=>engine.execute({...request,input:{...request.input,geography:source('bad.gz',data)}}));
// Generate a large decoded stream without retaining a half-gigabyte input buffer.
execFileSync('python',['-c',"import gzip; f=gzip.open('target/pl94-source-wasm/expansion.gz','wb'); chunk=(b' '*1023+b'\\n')*1024; [f.write(chunk) for _ in range(512)]; f.write(b'\\n'); f.close()"]);
const expansion=await fs.readFile(root+'/expansion.gz');assert.throws(()=>engine.execute({...request,input:{...request.input,geography:source('expansion.gz',expansion)}}),/decoded source exceeds 512 MiB/);
let read=false;await assert.rejects(()=>readPl94Source({name:'large',size:64*1024*1024+1,arrayBuffer(){read=true;}}));assert.equal(read,false);const bytes=Buffer.from([255,0,1]);assert.deepEqual(await readPl94Source({name:'raw.pl',size:bytes.length,arrayBuffer:async()=>bytes}),source('raw.pl',bytes));
console.log('PL94: seven native-host/WASM/file-oracle exact cases and seven gzip equivalents across three vintages, eight native errors, exact i64 counts, unused 2000 segment, decoded identities, CRC/truncation/512 MiB expansion rejection, inert project preservation and pre-read bounds passed.');

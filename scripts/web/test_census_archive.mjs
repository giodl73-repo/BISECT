import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {gunzipSync} from 'node:zlib';
import {extractCensusZip,readCensusArchive,MAX_CENSUS_ZIP_BYTES} from '../../web/lab/census-archive.js';
const root='target/census-archive-tests';await fs.mkdir(root,{recursive:true});
// Python's native zipfile writes independent stored/deflated fixtures.
execFileSync('python',['-c',String.raw`
import pathlib,zipfile,json,base64
r=pathlib.Path('target/census-archive-tests')
t=json.loads(pathlib.Path('target/tiger-source-wasm/tracts-tracts.bisect').read_text())['files']['tigerInputs']
parts={ 'units.'+k:base64.b64decode(t[k]['base64']) for k in ['shp','dbf','shx'] if t[k] }
parts['units.prj']=b'GEOGCS["North_American_1983",UNIT["Degree",1]]'
for compression,label in [(0,'stored'),(8,'deflated')]:
 with zipfile.ZipFile(r/(label+'.zip'),'w',compression=compression) as z:
  for name,data in parts.items():z.writestr('folder/'+name,data)
with zipfile.ZipFile(r/'large-tiger.zip','w',compression=8) as z:
 z.writestr('large.shp',b'L'*(16*1024*1024+1))
 z.writestr('large.dbf',parts['units.dbf'])
with zipfile.ZipFile(r/'large-pl94.zip','w',compression=8) as z:
 with z.open('cageo2020.pl','w') as f:
  for _ in range(129):f.write(b'\n'*(1024*1024))
 z.writestr('ca000012020.pl',b'PLST|CA|000|01|0000001|11\n')
for year in [2000,2010,2020]:
 p=json.loads(pathlib.Path(f'target/pl94-source-wasm/{year}.bisect').read_text())['files']['pl94Inputs']
 with zipfile.ZipFile(r/f'pl{year}.zip','w',compression=8) as z:
  z.writestr(f'rigeo{year}.pl',base64.b64decode(p['geography']['base64']))
  if p['population']:z.writestr(f'ri00001{year}.pl',base64.b64decode(p['population']['base64']))
  z.writestr(f'ri00002{year}.pl',b'unselected segment')
for label,names in [('traversal',['../units.shp','units.dbf']),('ambiguous',['a.shp','a.dbf','b.shp']),('duplicate',['units.shp','UNITS.SHP','units.dbf'])]:
 with zipfile.ZipFile(r/(label+'.zip'),'w') as z:
  for name in names:z.writestr(name,b'bytes')
with zipfile.ZipFile(r/'symlink.zip','w') as z:
 i=zipfile.ZipInfo('units.shp');i.create_system=3;i.external_attr=(0o120777<<16);z.writestr(i,b'../target');z.writestr('units.dbf',b'bytes')
`]);
const bytes=async name=>new Uint8Array(await fs.readFile(root+'/'+name+'.zip'));
const original=JSON.parse(await fs.readFile('target/tiger-source-wasm/tracts-tracts.bisect','utf8')).files.tigerInputs;
// Exercise the large selected-member transport branch independently of the
// shapefile reader: ZIP extraction must preserve every decoded byte.
const largeTiger=await extractCensusZip(await bytes('large-tiger'),{kind:'tiger'});
assert.equal(largeTiger.sources.shp.name,'large.shp.gz');
assert.deepEqual(gunzipSync(Buffer.from(largeTiger.sources.shp.base64,'base64')),Buffer.alloc(16*1024*1024+1,76));
assert.equal(largeTiger.sources.dbf.base64,original.dbf.base64);
const largePl94=await extractCensusZip(await bytes('large-pl94'),{kind:'pl94',year:2020});
const largeDecoded=gunzipSync(Buffer.from(largePl94.sources.geography.base64,'base64'));
assert.equal(largeDecoded.length,129*1024*1024);
assert.ok(largeDecoded.every(byte=>byte===10));
for(const kind of ['stored','deflated']){const result=await extractCensusZip(await bytes(kind),{kind:'tiger'});for(const part of ['shp','dbf','shx'])if(original[part])assert.equal(result.sources[part].base64,original[part].base64);assert.equal(result.execution_performed,false);assert.match(result.archive_sha256,/^[a-f0-9]{64}$/);}
for(const year of [2000,2010,2020]){const result=await extractCensusZip(await bytes('pl'+year),{kind:'pl94',year});const original=JSON.parse(await fs.readFile('target/pl94-source-wasm/'+year+'.bisect','utf8')).files.pl94Inputs;for(const part of ['geography','population'])if(original[part])assert.deepEqual(gunzipSync(Buffer.from(result.sources[part].base64,'base64')),Buffer.from(original[part].base64,'base64'));assert.equal(Object.hasOwn(result.sources,'population'),year!==2000);}
for(const name of ['traversal','ambiguous','duplicate','symlink']){const data=await bytes(name);await assert.rejects(()=>extractCensusZip(data,{kind:'tiger'}));}
const valid=await bytes('stored');
const corrupted=valid.slice();const cv=new DataView(corrupted.buffer),start=30+cv.getUint16(26,true)+cv.getUint16(28,true);corrupted[start]^=1;await assert.rejects(()=>extractCensusZip(corrupted,{kind:'tiger'}),/CRC/);
await assert.rejects(()=>extractCensusZip(valid.subarray(0,valid.length-1),{kind:'tiger'}));
const bomb=valid.slice(),bv=new DataView(bomb.buffer);let central=0;while(bv.getUint32(central,true)!==0x02014b50)central++;bv.setUint32(central+24,128*1024*1024+1,true);await assert.rejects(()=>extractCensusZip(bomb,{kind:'tiger'}),/expansion/);
const plBomb=(await bytes('pl2020')).slice(),plView=new DataView(plBomb.buffer);let plCentral=0;while(plView.getUint32(plCentral,true)!==0x02014b50)plCentral++;plView.setUint32(plCentral+24,512*1024*1024+1,true);await assert.rejects(()=>extractCensusZip(plBomb,{kind:'pl94'}),/expansion/);
const encrypted=valid.slice(),ev=new DataView(encrypted.buffer);ev.setUint16(central+8,1,true);await assert.rejects(()=>extractCensusZip(encrypted,{kind:'tiger'}),/encrypted/);
let created=0,terminated=0;const previousWorker=globalThis.Worker;globalThis.Worker=class{constructor(){created++;}postMessage(){}terminate(){terminated++;}};
try{
 await assert.rejects(()=>readCensusArchive({size:MAX_CENSUS_ZIP_BYTES+1},{kind:'tiger'}));assert.equal(created,0);
 const cancelled=new AbortController();cancelled.abort();await assert.rejects(()=>readCensusArchive({size:22},{kind:'tiger'},{signal:cancelled.signal}),/cancelled/);assert.equal(created,0);
 const controller=new AbortController(),pending=readCensusArchive({size:22},{kind:'tiger'},{signal:controller.signal});controller.abort();await assert.rejects(()=>pending,/cancelled/);assert.equal(created,1);assert.equal(terminated,1);
}finally{globalThis.Worker=previousWorker;}
console.log('Census ZIP: native stored/deflate byte parity, all PL94 vintages lossless gzip, path/duplicate/link/ambiguity/CRC/truncation/expansion guards passed.');

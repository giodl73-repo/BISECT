import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gunzipSync} from 'node:zlib';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {validateTigerInputs} from '../../web/lab/tiger-files.js';
const root='target/real-census-preparation/county-national',bytes=new Uint8Array(await fs.readFile(root+'/tl_2020_us_county.zip'));
assert(bytes.length>64*1024*1024&&bytes.length<96*1024*1024);
await assert.rejects(()=>extractCensusZip(bytes,{kind:'tiger',year:2020}));
const result=await extractCensusZip(bytes,{kind:'tiger',year:2020,unit_kind:'counties'});
validateTigerInputs({kind:'counties',year:2020,shp:result.sources.shp,dbf:result.sources.dbf,shx:result.sources.shx});
for(const part of ['shp','dbf','shx','prj']){const s=result.sources[part],data=Buffer.from(s.base64,'base64'),decoded=s.name.endsWith('.gz')?gunzipSync(data):data;assert.deepEqual(decoded,await fs.readFile(root+'/tl_2020_us_county.'+part));}
assert.equal(result.execution_performed,false);
console.log('Original national county ZIP: county-only 96-MiB allowance, unchanged default rejection, CRC/part-selection/compression and byte-identical original SHP/DBF/SHX/PRJ preservation passed ('+bytes.length+' archive bytes).');

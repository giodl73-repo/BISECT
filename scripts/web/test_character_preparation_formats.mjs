import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gzipSync} from 'node:zlib';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {characterPreparationProject,preparedCharacterInput,verifyCharacterPreparation} from '../../web/lab/character-handoff.js';
import {lodesRequest,LODES_DEFAULTS} from '../../web/lab/lodes-files.js';
import {acsHousingRequest} from '../../web/lab/acs-housing-files.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {createProject} from '../../web/lab/project.js';
const out='target/character-preparation-formats';await fs.mkdir(out,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),hash=b=>createHash('sha256').update(b).digest('hex');
const wac='w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS08,CNS09,CNS10,CNS11\n440010001001001,10,1,2,3,4,5,6,7,8\n',housing=JSON.parse(await fs.readFile('target/character-handoff/acs-housing.bisect','utf8')).files.acsHousingInputs;
const cases=[['economic-bom','lodes',Buffer.concat([Buffer.from([239,187,191]),Buffer.from(wac)])],['housing-bom','housing',Buffer.concat([Buffer.from([239,187,191]),Buffer.from(housing.source.base64,'base64')])],['economic-gzip','lodes',gzipSync(wac)],['economic-expanded-20mib','lodes',gzipSync(wac+(' '.repeat(1023)+'\n').repeat(20*1024))]];const records=[];let refusals=0;
for(const [name,type,bytes]of cases){
 const source={name:name+(type==='housing'?'.json':'.csv'),base64:bytes.toString('base64')},inputs={source,settings:type==='housing'?housing.settings:LODES_DEFAULTS},request=(type==='housing'?acsHousingRequest:lodesRequest)(inputs),result=engine.execute(request);await fs.writeFile(out+'/request.json',JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[out+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024})));
 const project=characterPreparationProject(result,{[type==='housing'?'acsHousingInputs':'lodesInputs']:inputs});await verifyCharacterPreparation(project,preparedCharacterInput(project));const archive=createWorkbenchArchive(project),opened=parseWorkbenchArchive(archive);assert.equal(opened.files[type==='housing'?'acsHousingInputs':'lodesInputs'].source.base64,source.base64);const restored=createProject(opened);await verifyCharacterPreparation(restored,preparedCharacterInput(restored));
 if(name.endsWith('bom'))assert.notEqual(hash(bytes),result.source_sha256);else assert.equal(hash(bytes),result.source_sha256);
 if(name.includes('20mib'))assert(result.decoded_byte_length>20*1024*1024);
 for(const mutate of [p=>p.result.state='IA',p=>p.result.year='2010',p=>p.result.data_year='1999',p=>p.result.written_rows++,p=>p.result.csv_text='wrong header\n']){const bad=structuredClone(project);mutate(bad);assert.throws(()=>preparedCharacterInput(bad));refusals++;}
 if(type==='lodes')for(const mutate of [p=>p.result.kind='od',p=>p.result.source_encoding='wrong',p=>p.result.source_byte_length++]){const bad=structuredClone(project);mutate(bad);await assert.rejects(verifyCharacterPreparation(bad,project.result.character_input));refusals++;}
 await fs.writeFile(out+'/'+name+'.bisect',JSON.stringify(project));records.push({name,source_bytes:bytes.length,decoded_bytes:result.decoded_byte_length??null,source_sha256:hash(bytes),native_request_source_sha256:result.source_sha256,archive_bytes:archive.length});
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({cases:cases.length,receipt_refusals:refusals,native_host_preparation_exact:true,source_bytes_preserved:true,utf8_bom_native_request_policy:true,expanded_gzip_over_20mib:true,records,wasm_sha256:engine.artifactIdentity.sha256,scope:'Synthetic source-format and size fixtures. Native host preparation and original compressed/BOM source archive retention; no observed-data provenance or national acquisition/capacity claim.'},null,2));console.log(`Character preparation formats: ${cases.length} Native-host/WASM BOM/gzip/20MiB-expanded preparations, exact source archives and ${refusals} receipt/type/length refusals passed.`);

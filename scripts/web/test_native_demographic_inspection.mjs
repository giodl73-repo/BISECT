import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/native-demographic-inspection';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/native_demographic_reference.exe');
const fixtures=[
 ['total-population','GEOID,total_pop,white_non_hispanic\n1001020100,100,20\n01001020200,0,0\n'],
 ['total-population','GEOID,total_pop,white_non_hispanic\n01001020100,100,bad\n01001020200,bad,10\n01001020300,100,200\n01001020100,100,60\n01001020400,NaN,0\n'],
 ['total-population','GEOID,total_pop,white_non_hispanic\r\n01001020100,100,"20"\r\n'],
 ['total-population','GEOID,total_pop,white_non_hispanic\n'],
 ...['total_vap','vap','cvap'].map(column=>['citizen-voting-age-population',`GEOID,${column},minority_vap\n1001020100,100,25\n01001020200,bad,bad\n1001020100,200,50\n`]),
 ['voting-age-population','geoid,cvap,vap,total_vap,minority_vap\n1001020100,80,90,100,20\n'],
];
const results=[];
for(const [basis,text]of fixtures){
 const file=path.resolve(root,`case-${results.length}.csv`);await fs.writeFile(file,text);const native=spawnSync(exe,[file,basis],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);
 const source_base64=Buffer.from(text).toString('base64'),request={operation:'inspect-native-demographic-csv',source_base64,basis,source_label:'Synthetic native CSV compatibility fixture'},result=engine.execute(request);assert.deepEqual(result.records,JSON.parse(native.stdout));assert.equal(result.source_sha256,createHash('sha256').update(text).digest('hex'));
 const project=createProject({constraints:[],operation:request.operation,files:{demographicSource:{name:path.basename(file),base64:source_base64},demographicSettings:{basis,source_label:request.source_label}},result,lastOperation:request.operation});const opened=parseProject(JSON.stringify(project));assert.deepEqual(opened.result,result);assert.equal(opened.files.demographicSource.base64,source_base64);
 await fs.writeFile(path.join(root,`case-${results.length}.bisect`),JSON.stringify(project));results.push(result);
}
assert.deepEqual(results[1].diagnostics,{source_rows:5,numeric_zero_defaults:2,duplicate_replacements:1,nonpositive_totals:1,clamped_fractions:1});assert.equal(results[1].records.at(-1).non_finite_fraction,true);
let refusals=0;for(const [basis,text]of [['total-population','\ufeffGEOID,total_pop,white_non_hispanic\n'],['voting-age-population','GEOID,total_vap,minority_vap\n01001020100,100,101\n'],['voting-age-population','GEOID,total_vap,minority_vap\n01001020100,NaN,0\n']]){const file=path.resolve(root,'bad-'+refusals+'.csv');await fs.writeFile(file,text);assert.notEqual(spawnSync(exe,[file,basis],{encoding:'utf8'}).status,0);assert.throws(()=>engine.execute({operation:'inspect-native-demographic-csv',source_base64:Buffer.from(text).toString('base64'),basis,source_label:'bad'}));refusals++;}
for(const mutation of [{basis:'bad'},{source_label:''},{source_base64:'bad!'}, {source_base64:Buffer.from([255]).toString('base64')},{source_base64:Buffer.from('GEOID,total_pop,white_non_hispanic\n'+'01001020100,100,20\n'.repeat(100001)).toString('base64')}])assert.throws(()=>engine.execute({operation:'inspect-native-demographic-csv',source_base64:Buffer.from(fixtures[0][1]).toString('base64'),basis:'total-population',source_label:'bad',...mutation}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),native_reference_sha256:createHash('sha256').update(await fs.readFile(exe)).digest('hex'),exact_native_file_records:results.length,matching_refusals:refusals,bounded_invalid_refusals:5,source_bytes_retained:true,defaults_replacements_clamps_nonfinite_visible:true,scope:'Synthetic original-format CSV; inspection only, no graph alignment, automatic attachment or legal certification.'},null,2));
console.log(`Native demographic inspection: ${results.length} exact native file-reader results, ${refusals} matching refusals, five bounded/invalid refusals, exact source hashes and project byte retention.`);

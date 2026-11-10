import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preparedCharacterInput,verifyCharacterPreparation} from '../../web/lab/character-handoff.js';
import {characterPreparationReplayRequest,recheckCharacterPreparation,validateCharacterRecheck,verifyCharacterRecheckBinding} from '../../web/lab/character-recheck.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const out='target/character-recheck';await fs.mkdir(out,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),hash=b=>createHash('sha256').update(b).digest('hex');let matches=0,mismatches=0,refusals=0;
const projects=[];for(const name of ['economic-bom','housing-bom','economic-gzip','economic-expanded-20mib'])projects.push([name,JSON.parse(await fs.readFile('target/character-preparation-formats/'+name+'.bisect','utf8'))]);
for(const kind of ['lodes','acs-housing']){
 const original=JSON.parse(await fs.readFile('target/character-handoff/'+kind+'.bisect','utf8')),wrong=structuredClone(original),data=wrong.result.character_input.data,id=Object.keys(data.characters)[0];
 if(kind==='acs-housing')data.characters[id].pct_owner=(data.characters[id].pct_owner+.125)%1;
 else{const r=data.raw_counts[id];r.c000+=10;data.characters[id]={commercial_intensity:Math.min(1,(r.cns07+r.cns09+r.cns10+r.cns11)/r.c000),industrial_fraction:Math.min(1,(r.cns01+r.cns02+r.cns05+r.cns08)/r.c000),jobs_per_resident:Math.min(1,r.c000/10000)};}
 const csv=structuredClone(original);csv.result.csv_text+='\n';csv.result.csv_sha256=hash(csv.result.csv_text);projects.push([kind+'-changed-character',wrong],[kind+'-changed-csv',csv]);
}
for(const [name,p]of projects){
 await verifyCharacterPreparation(p,preparedCharacterInput(p));const receipt=await recheckCharacterPreparation(engine,p),request=characterPreparationReplayRequest(p);await fs.writeFile(out+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[out+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));
 const fields=[...new Set([...Object.keys(native),...Object.keys(p.result)])].sort().filter(k=>{try{assert.deepEqual(native[k],p.result[k]);return false;}catch{return true;}});assert.deepEqual(receipt.differing_fields,fields);assert.equal(receipt.recomputed_csv_sha256,native.csv_sha256);assert.equal(receipt.status,name.includes('changed')?'mismatch':'matched');if(receipt.status==='matched')matches++;else mismatches++;
 await verifyCharacterRecheckBinding(receipt,p);const saved=createProject({name,files:{characterPreparation:p,characterPreparationRecheck:receipt},operation:'validate-rplan',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(saved)),saved);await fs.writeFile(out+'/'+name+'.bisect',JSON.stringify(saved));
 const altered=structuredClone(p);altered.name+=' changed';await assert.rejects(verifyCharacterRecheckBinding(receipt,altered));refusals++;
 for(const mutate of [r=>r.scope='legal-certification',r=>r.status=receipt.status==='matched'?'mismatch':'matched',r=>r.wasm_sha256='bad',r=>r.differing_fields=['__proto__'],r=>r.expected_csv_sha256='0'.repeat(64)]){const bad=structuredClone(receipt);mutate(bad);assert.throws(()=>validateCharacterRecheck(bad,p));refusals++;}
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({matched_formats:matches,detected_transform_mismatches:mismatches,native_preparation_comparisons:projects.length,receipt_binding_refusals:refusals,source_hash_only_checks_passed_for_mismatches:true,project_receipt_roundtrips:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Preparation recheck: ${matches} matching BOM/gzip/large formats, ${mismatches} self-consistent but incorrect saved transforms detected, ${projects.length} Native preparation comparisons and ${refusals} receipt/binding refusals passed.`);

// Compare complete old/new retained national replies without masking differences.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const [prior,current,output]=process.argv.slice(2).map(v=>path.resolve(v));assert(prior&&current&&output);await fs.mkdir(output,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex'),pin=async name=>{const b=await fs.readFile(name);return {path:name,sha256:sha(b),byte_length:b.length};};
const old=JSON.parse(await fs.readFile(path.join(prior,'evidence.json'),'utf8')),fresh=JSON.parse(await fs.readFile(path.join(current,'evidence.json'),'utf8'));assert.equal(old.status,'passed-national-wasm-tier');assert.equal(fresh.status,old.status);assert.equal(fresh.campaign_id,old.campaign_id);assert.equal(old.outcomes.length,50);assert.equal(fresh.outcomes.length,50);
function difference(a,b,p='result'){
 if(Object.is(a,b))return null;if(typeof a!==typeof b||a===null||b===null||typeof a!=='object')return p;
 if(Array.isArray(a)!==Array.isArray(b))return p;const ak=Object.keys(a).sort(),bk=Object.keys(b).sort();if(ak.join('\0')!==bk.join('\0'))return p+'.<keys>';
 for(const k of ak){const found=difference(a[k],b[k],p+'.'+k);if(found)return found;}return null;
}
const outcomes=[];let failed=false;
for(const row of fresh.outcomes){assert(old.outcomes.some(r=>r.state===row.state&&r.units===row.units&&r.districts===row.districts&&r.population===row.population));const a=path.join(prior,row.state,'wasm-reply.json'),b=path.join(current,row.state,'wasm-reply.json'),original=JSON.parse(await fs.readFile(a,'utf8')),actual=JSON.parse(await fs.readFile(b,'utf8')),where=difference(original,actual);outcomes.push({state:row.state,status:where?'failed-whole-reply-difference':'passed-exact-whole-reply',first_difference:where,prior:await pin(a),current:await pin(b)});failed ||=where!==null;}
await fs.writeFile(path.join(output,'evidence.json'),JSON.stringify({status:failed?'failed':'passed',prior:await pin(path.join(prior,'evidence.json')),current:await pin(path.join(current,'evidence.json')),states:50,outcomes,scope:'Every field in all 50 complete replies, with no runtime/algorithm/seed/result masking. This does not extend source or algorithm coverage beyond the retained campaign.',acceptance_v1:false},null,2)+'\n');console.log(JSON.stringify({status:failed?'failed':'passed',states:50,first_difference:outcomes.find(r=>r.first_difference)?.first_difference??null,evidence:path.join(output,'evidence.json')}));if(failed)process.exitCode=1;

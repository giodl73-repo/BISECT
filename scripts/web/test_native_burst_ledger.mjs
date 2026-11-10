import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {verifyNativeBurstSeeds} from '../../web/lab/native-burst-ledger.js';
let cases=0,refusals=0;
for(const seed of [null,0,'18446744073709551615'])for(const n_bursts of [0,3])for(const burst_length of [0,2]){
 const config={search:'short-burst',seed,n_bursts,burst_length,percentile:1,steps:4},seeds=[];
 for(let i=0;i<n_bursts;i++){const bytes=Buffer.alloc(17);bytes.writeBigUInt64LE(BigInt(i));bytes[8]=95;bytes.writeBigUInt64LE(BigInt(seed??0),9);seeds.push(createHash('sha256').update('SHORT_BURST_CHAIN_').update(bytes).digest().readBigUInt64LE().toString());}
 const e={native_burst_ledger:{burst_seeds:seeds,selected_burst_index:Math.max(0,n_bursts-1)}};await verifyNativeBurstSeeds(e,config);cases++;
 const mutations=[v=>v.native_burst_ledger.selected_burst_index=n_bursts+1,v=>v.native_burst_ledger.burst_seeds.push('0'),v=>v.native_burst_ledger.unrecognized=true];
 if(n_bursts)mutations.push(v=>v.native_burst_ledger.burst_seeds[0]='0',v=>v.native_burst_ledger.burst_seeds[0]=Number(seeds[0]),v=>v.native_burst_ledger.burst_seeds[0]='18446744073709551616');
 for(const mutate of mutations){const v=structuredClone(e);mutate(v);await assert.rejects(()=>verifyNativeBurstSeeds(v,config));refusals++;}
 await verifyNativeBurstSeeds({},config);
}
await fs.mkdir('target/native-burst-ledger',{recursive:true});await fs.writeFile('target/native-burst-ledger/seed-verifier.json',JSON.stringify({independent_node_crypto_cases:cases,tamper_refusals:refusals,historical_omission_accepted_as_claim:true},null,2));console.log(`Burst ledger verifier: ${cases} independent Node crypto cases, ${refusals} altered/invalid ledger refusals; historical omission accepted as a claim.`);

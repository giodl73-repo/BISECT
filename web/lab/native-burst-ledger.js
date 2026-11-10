// Optional native ReCom metadata. Historical omission remains an imported claim.
const canonicalU64=v=>typeof v==='string'&&/^(0|[1-9][0-9]*)$/.test(v)&&v.length<=20&&BigInt(v)<=18446744073709551615n;
export function validateNativeBurstLedger(e,config){
 if(!Object.hasOwn(e??{},'native_burst_ledger'))return;
 const l=e.native_burst_ledger,n=config.n_bursts??Math.ceil(config.steps/20),length=config.burst_length??20;
 if(config.search!=='short-burst'||!l||Array.isArray(l)||Object.keys(l).length!==2||!Object.hasOwn(l,'burst_seeds')||!Object.hasOwn(l,'selected_burst_index')||!Array.isArray(l.burst_seeds)||l.burst_seeds.length!==n||!l.burst_seeds.every(canonicalU64)||!Number.isSafeInteger(l.selected_burst_index)||l.selected_burst_index<0||l.selected_burst_index>=Math.max(1,n))throw new Error('Invalid native burst ledger.');
 if((n===0&&l.selected_burst_index!==0)||(length===0&&n>0&&l.selected_burst_index!==Math.min(n-1,Math.floor(config.percentile*n))))throw new Error('Native zero-work burst selection disagrees with percentile.');
}
export async function verifyNativeBurstSeeds(e,config){
 validateNativeBurstLedger(e,config);if(!e?.native_burst_ledger)return;
 const prefix=new TextEncoder().encode('SHORT_BURST_CHAIN_'),bytes=new Uint8Array(prefix.length+17);bytes.set(prefix);bytes[prefix.length+8]=95;
 const view=new DataView(bytes.buffer);view.setBigUint64(prefix.length+9,BigInt(config.seed??0),true);
 for(let i=0;i<e.native_burst_ledger.burst_seeds.length;i++){
  view.setBigUint64(prefix.length,BigInt(i),true);const digest=await crypto.subtle.digest('SHA-256',bytes);
  if(new DataView(digest).getBigUint64(0,true).toString()!==e.native_burst_ledger.burst_seeds[i])throw new Error('Native burst seed disagrees with run seed and burst index.');
 }
}

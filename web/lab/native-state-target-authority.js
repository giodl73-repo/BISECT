const authorities=new WeakMap();
export const nativeTargetHash=async value=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(value)))),b=>b.toString(16).padStart(2,'0')).join('');
export function validateNativeTargetActions(value){if(!value||Object.keys(value).sort().join()!=='force,reset'||typeof value.reset!=='boolean'||typeof value.force!=='boolean'||!value.reset&&!value.force)throw new Error('Invalid native State target actions.');return value;}
// A stored receipt is not the callback which performs a current explicit action.
// Workers mint this private capability only after their live parent handshake.
export async function authorizeNativeStateTarget(intent,perform){
 if(typeof perform!=='function')throw new Error('Native target authorization requires a current explicit action callback, not an imported receipt.');
 const receipt=await perform(structuredClone(intent));
 if(receipt?.schema_version!=='bisect-native-state-target-actions-v1'||receipt.status!=='completed'||receipt.request_sha256!==intent.request_sha256||receipt.target_namespace!==intent.target_namespace||receipt.reset!==intent.reset||receipt.force!==intent.force)throw new Error('Native target action does not match this exact Run.');
 const authority=Object.freeze({});authorities.set(authority,{binding_sha256:intent.binding_sha256,source_sha256:intent.derived_source_sha256,receipt:structuredClone(receipt)});return authority;
}
export async function requireNativeStateTargetAuthority(authority,binding,{consume=false,sourceSha256}={}){
 const current=authorities.get(authority);if(!current||current.binding_sha256!==await nativeTargetHash(binding))throw new Error('Review and explicitly complete the native target action for this Run. Saved receipts grant no execution authority.');
 if(consume&&(!sourceSha256||sourceSha256!==current.source_sha256))throw new Error('Native target authority belongs to different State source bytes.');
 if(consume)authorities.delete(authority);return structuredClone(current.receipt);
}

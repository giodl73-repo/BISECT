// Match native min(requested, available threads). Rayon treats zero as its
// default pool size, so zero requests the browser's reported CPU capacity.
export function nativeStatesWorkerPolicy(review,selectedStates,hardwareConcurrency=globalThis.navigator?.hardwareConcurrency??1){
 const requested=Number(review?.workers),available=Number.isSafeInteger(hardwareConcurrency)&&hardwareConcurrency>0?hardwareConcurrency:1;
 if(!Number.isSafeInteger(requested)||requested<0||!Number.isSafeInteger(selectedStates)||selectedStates<0||selectedStates>51)throw new Error('Invalid native States worker policy.');
 return {requested_workers:review.workers,effective_workers:Math.min(requested||available,available,selectedStates),available_workers:available,zero_requests_automatic_pool:requested===0,policy:'native-request-capped-by-browser-cpu-and-selected-states'};
}
// Serialize shared directory installation and receipt writes. A rejected write
// is still observed by its caller; it does not strand later cleanup/checkpoints.
export function serializeStateWrites(){
 let tail=Promise.resolve();
 return action=>{const result=tail.then(action);tail=result.catch(()=>{});return result;};
}

export function validateFetchCachePolicy({freshness='immutable',maxAgeDays=null}={}){
 if(!['immutable','always-check','max-age-days'].includes(freshness)||(freshness==='max-age-days'?(!Number.isInteger(maxAgeDays)||maxAgeDays<0||maxAgeDays>4294967295):maxAgeDays!==null))throw new Error('Select a cache freshness policy and unsigned 32-bit maximum-age days.');
 return {freshness,maxAgeDays};
}

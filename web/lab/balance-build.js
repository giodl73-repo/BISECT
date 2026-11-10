import {parseSeed} from './static.js';
const record=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const exact=(v,keys)=>record(v)&&Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
// Imported reports prove internal consistency only; generation history remains unverified.
export function acceptedBuildConfig(config,metrics){
 const report=metrics?.balance_build;
 if(config.native_build!==true){if(report!==undefined)throw new Error('Unexpected population retry report.');return config;}
 if(!exact(report,['schema_version','accepted_attempt','max_balance_retries','attempts','scope'])||report.schema_version!=='bisect-balance-build-v1'||report.max_balance_retries!==50||!Number.isSafeInteger(report.accepted_attempt)||report.accepted_attempt<0||report.accepted_attempt>50||!Array.isArray(report.attempts)||report.attempts.length!==report.accepted_attempt+1||typeof report.scope!=='string'||report.scope.length>500)throw new Error('Invalid population retry report.');
 const base=BigInt(config.seed??0),mod=1n<<64n;
 for(const [i,trial] of report.attempts.entries()){
  const expectedSeed=config.seed===null&&i===0?null:((base+BigInt(i))%mod).toString();
  if(!exact(trial,['attempt','seed','balanced','max_deviation_percent'])||trial.attempt!==i||trial.seed!==expectedSeed||trial.balanced!==(i===report.accepted_attempt)||!Number.isFinite(trial.max_deviation_percent)||trial.max_deviation_percent<0||trial.max_deviation_percent>Number.MAX_SAFE_INTEGER||trial.balanced!==(trial.max_deviation_percent<=config.balance_tolerance))throw new Error('Population retry report disagrees with seed order or tolerance.');
 }
 if(metrics.balance_passed!==true||metrics.max_deviation_percent!==report.attempts.at(-1).max_deviation_percent)throw new Error('Population retry report disagrees with accepted metrics.');
 return {...config,seed:report.attempts.at(-1).seed===null?null:parseSeed(report.attempts.at(-1).seed)};
}

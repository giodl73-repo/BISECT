export function validateElectionAnalysisSettings(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'bootstrap_seed')||!Object.hasOwn(value,'bootstrap_replicates'))throw new Error('Invalid election analysis settings.');
 if(typeof value.bootstrap_seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(value.bootstrap_seed)||BigInt(value.bootstrap_seed)>18446744073709551615n)throw new Error('Supply a canonical unsigned 64-bit bootstrap seed.');
 if(!Number.isInteger(value.bootstrap_replicates)||value.bootstrap_replicates<1||value.bootstrap_replicates>5000)throw new Error('Bootstrap replicates must be an integer from 1 to 5,000.');return value;
}

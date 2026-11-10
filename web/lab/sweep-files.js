export const SWEEP_DEFAULTS={state:'RI',year:'2020',chamber:'congressional',seeds:10,seed_start:'1',keep_top:3,optimize_by:'compactness',version:'sweep',output_base:'outputs',run:false};
export function validateSweepSettings(v){
 if(!v||Object.keys(v).sort().join()!==Object.keys(SWEEP_DEFAULTS).sort().join())throw new Error('Invalid sweep settings.');
 for(const key of ['state','year','chamber','version','output_base'])if(typeof v[key]!=='string'||!v[key]||v[key].length>240||!/^[A-Za-z0-9_.:/\\+-]+$/.test(v[key]))throw new Error('Sweep command fields require bounded tokens without whitespace or shell metacharacters.');
 if(!Number.isSafeInteger(v.seeds)||v.seeds<1||v.seeds>1000||!Number.isSafeInteger(v.keep_top)||v.keep_top<0||v.keep_top>v.seeds)throw new Error('Use 1–1000 seeds and keep-top between zero and the seed count.');
 if(typeof v.seed_start!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(v.seed_start)||BigInt(v.seed_start)+BigInt(v.seeds-1)>18446744073709551615n)throw new Error('Sweep seed range must fit exact unsigned u64.');
 if(!['compactness','splits','deviation'].includes(v.optimize_by)||typeof v.run!=='boolean')throw new Error('Select a native sweep metric and boolean reserved-run flag.');return v;
}
export function sweepRequest(settings){validateSweepSettings(settings);return {operation:'plan-sweep',settings};}
export function sweepCommandBytes(result){
 if(result?.schema_version!=='bisect-sweep-plan-v1'||result.status!=='planned'||result.execution_performed!==false||result.ranking_performed!==false||!Array.isArray(result.plan?.rows)||result.plan.rows.length!==result.settings?.seeds||typeof result.plan.commands_stdout!=='string'||result.plan.commands_stdout.length>8*1024*1024)throw new Error('Generate a native sweep plan first.');
 validateSweepSettings(result.settings);const expected=result.plan.rows.map(r=>{if(typeof r.state_command!=='string'||typeof r.analyze_command!=='string')throw new Error('Invalid sweep command row.');return r.state_command+'\n'+r.analyze_command+'\n';}).join('');if(expected!==result.plan.commands_stdout)throw new Error('Sweep command text differs from its rows.');return new TextEncoder().encode(expected);
}

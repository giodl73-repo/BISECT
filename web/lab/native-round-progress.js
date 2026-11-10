export const isNativeRoundProgress=p=>['bisect-native-round-progress-v1','bisect-native-round-monitor-v1'].includes(p?.schema_version);
const u64=value=>{if(typeof value!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(value)||BigInt(value)>18446744073709551615n)throw new Error('Invalid native observation count.');return BigInt(value);};
export function validateNativeRoundMonitor(p){
 const keys=['schema_version','phase','name','index','notified','omitted','units','assigned_units','invalid_entries','invalid_name','complete_universe'];
 if(!p||typeof p!=='object'||Array.isArray(p)||Object.keys(p).sort().join()!==keys.sort().join()||p.schema_version!=='bisect-native-round-monitor-v1'||!['entered','snapshot','closed'].includes(p.phase)||!Number.isInteger(p.units)||p.units<1||p.units>1000000||!Number.isInteger(p.assigned_units)||p.assigned_units<0||p.assigned_units>p.units||typeof p.invalid_name!=='boolean'||typeof p.complete_universe!=='boolean'||p.name!==null&&(typeof p.name!=='string'||!/^[a-zA-Z0-9_]{1,80}$/.test(p.name)))throw new Error('Invalid native round monitor event.');
 const index=u64(p.index),notified=u64(p.notified),omitted=u64(p.omitted),invalid=u64(p.invalid_entries);let expected=index<16n?index:16n;for(let power=32n;power<=index;power*=2n)expected++;
 if(notified!==expected||omitted!==index-notified||p.complete_universe!==(index>0n&&p.assigned_units===p.units&&invalid===0n)||index===0n&&(p.name!==null||p.invalid_name||p.assigned_units!==0||invalid!==0n)||index>0n&&p.invalid_name!==(p.name===null)||p.phase==='entered'&&index!==0n||p.phase==='snapshot'&&(index===0n||index>16n&&(index&(index-1n))!==0n))throw new Error('Native observation counters or snapshot summary disagree.');
 return p;
}
export function validateNativeRoundProgress(p){
 if(p?.schema_version==='bisect-native-round-monitor-v1')return validateNativeRoundMonitor(p);
 const keys=['schema_version','name','index','assigned_units','units','complete_universe'];
 if(!p||typeof p!=='object'||Array.isArray(p)||Object.keys(p).sort().join()!==keys.sort().join()||p.schema_version!=='bisect-native-round-progress-v1'||typeof p.name!=='string'||!/^[a-zA-Z0-9_]{1,80}$/.test(p.name)||!Number.isInteger(p.index)||p.index<1||p.index>128||!Number.isInteger(p.units)||p.units<1||p.units>100000||!Number.isInteger(p.assigned_units)||p.assigned_units<0||p.assigned_units>p.units||typeof p.complete_universe!=='boolean'||p.complete_universe!==(p.assigned_units===p.units))throw new Error('Invalid native round progress event.');
 return p;
}
export function nativeRoundProgressText(progress){
 const p=validateNativeRoundProgress(progress);
 if(p.schema_version==='bisect-native-round-monitor-v1'){
  const count=value=>BigInt(value).toLocaleString(),note=' Counters restart for each invocation or retry; multiscale observations describe parent-tract initialization.';
  if(p.phase==='entered')return 'Native engine · checking inputs and running the selected method. Waiting for an intermediate observation.';
  if(p.phase==='closed')return `Native invocation returned · ${count(p.index)} intermediate observations; ${count(p.omitted)} not shown as live snapshots. Checks or retries may continue.`+note;
  return `Native observation ${count(p.index)}: ${p.name??'unavailable native name'} · ${p.assigned_units.toLocaleString()}/${p.units.toLocaleString()} units assigned. ${count(p.omitted)} earlier observations not shown. Intermediate candidate; checks or retries continue.`+(p.invalid_name||p.invalid_entries!=='0'?' Invalid native snapshot entries or name were observed; this is not validated plan data.':'')+note;
 }
 return `Native snapshot ${p.index}: ${p.name} · ${p.assigned_units.toLocaleString()}/${p.units.toLocaleString()} capture units assigned. Candidate or partial snapshot; execution continues. Snapshot numbers restart for each capture or retry. Multiscale snapshots cover parent-tract initialization.`;
}

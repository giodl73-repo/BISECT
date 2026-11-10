// Browser coordination for native State STATUS positions. No engine settings.
const keys=(v,names)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===names.split(',').sort().join();
export function validateStateDiagnosticSettings(v){
 if(!keys(v,'schema_version,position,debug_requested,debug_behavior'+(Object.hasOwn(v??{},'time_partition_requested')?',time_partition_requested':''))||v.schema_version!=='bisect-native-state-diagnostic-settings-v1'||!Number.isInteger(v.position)||v.position< -2147483648||v.position>2147483647||typeof v.debug_requested!=='boolean'||v.debug_behavior!=='native-unused'||Object.hasOwn(v??{},'time_partition_requested')&&v.time_partition_requested!==true)throw new Error('Invalid native State diagnostic settings.');return v;
}
export const isNativeStateStatus=p=>p?.schema_version==='bisect-native-state-status-v1';
export function validateNativeStateStatus(v){
 if(!keys(v,'schema_version,position,message,source_stage,status_line,coordinator_ignored')||!isNativeStateStatus(v)||!Number.isInteger(v.position)||v.position< -2147483648||v.position>2147483647||typeof v.message!=='string'||v.message.length>2000||/[^\x20-\x7e]/.test(v.message)||typeof v.source_stage!=='string'||!/^[a-z0-9-]{1,100}$/.test(v.source_stage)||v.status_line!=='STATUS:'+v.position+':'+v.message+'\n'||v.coordinator_ignored!==(v.position===999))throw new Error('Invalid native State status event.');return v;
}
export function nativeStateStatusText(value){const v=validateNativeStateStatus(value);return v.coordinator_ignored?null:'State progress '+v.position+' · '+v.message;}
export function stateDiagnosticObserver(settings,onProgress){
 if(!settings)return {onProgress,partitionTiming:false,finish(){}};const config=structuredClone(validateStateDiagnosticSettings(settings)),events=[];let omitted=0;
 const observe=p=>{
  // Preserve strict native round snapshots verbatim, including their schemas.
  if(typeof p?.stage!=='string'||!/^[a-z0-9-]{1,100}$/.test(p.stage)){onProgress?.(p);return;}
  const message=Array.from(typeof p.message==='string'?p.message:p.stage,c=>c.codePointAt(0)>=32&&c.codePointAt(0)<=126?c:'?').join('').slice(0,2000);
  const event=validateNativeStateStatus({schema_version:'bisect-native-state-status-v1',position:config.position,message,source_stage:p.stage,status_line:'STATUS:'+config.position+':'+message+'\n',coordinator_ignored:config.position===999});
  if(events.length<64)events.push(event);else omitted++;onProgress?.(event);
 };
 return {onProgress:observe,partitionTiming:config.time_partition_requested===true,finish(result){observe({stage:'native-state-returned',message:'State workflow returned; inspect retained stage results'});result.native_state.diagnostics={schema_version:'bisect-native-state-diagnostics-v1',settings:config,status_events:structuredClone(events),omitted_events:omitted,scope:'Browser orchestration STATUS lanes, not a byte-identical native message transcript. Native round events remain unchanged. Debug is retained without an algorithm or logging effect; requested partition dispatch timing is retained separately in the prepared receipt.'};}};
}

import{validateSourceFile}from'./package-files.js';
import{nativeReplaySourceRequest}from'./native-replay-sources.js';
export const RECORDED_REPLAY_DEFAULTS=Object.freeze({label:'RecordedReplay',created_at:''});
export function validateRecordedReplaySettings(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!=='created_at,label'||typeof value.label!=='string'||!/^[A-Za-z0-9][A-Za-z0-9 _.-]{0,79}$/.test(value.label)||typeof value.created_at!=='string'||value.created_at.length>200||(value.created_at!==''&&!Number.isFinite(Date.parse(value.created_at))))throw Error('Record a replay label of 1–80 letters, numbers, spaces, dots, underscores or hyphens and an optional valid timestamp.');
 return value;
}
export function recordedReplayRequest(packageInputs,sources,settings,execute){
 const checked=validateRecordedReplaySettings(settings);
 if(typeof execute!=='boolean')throw Error('Choose explicit replay preparation or execution.');
 return {operation:execute?'run-native-recorded-replay':'prepare-native-recorded-replay',input:{sources:nativeReplaySourceRequest(packageInputs,sources).input,label:checked.label,created_at:checked.created_at||new Date().toISOString()}};
}

export function validateRecordedReplayReceiptSource(source){return validateSourceFile(source,{maxBytes:8*1024*1024});}

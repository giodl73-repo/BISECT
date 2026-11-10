import {validateSourceFile,exactContextText,readElectionSource} from './package-files.js';
import {validatePolicyInputs,policyRequest} from './policy-files.js';
import {validateNativeDoctorInputs,nativeDoctorWorkspaceRequest} from './doctor-workspace-files.js';
export const DOCTOR_DEFAULTS={state:'RI',year:'2020',chamber:'congressional',resolution:'tract'};
export function validateDoctorInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(k=>!['settings','policy','context','native'].includes(k))||!['settings','policy','context'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid preflight inputs.');
 if(Object.hasOwn(value,'native'))validateNativeDoctorInputs(value.native);
 const settings=value.settings;if(!settings||typeof settings!=='object'||Array.isArray(settings)||Object.keys(settings).length!==4||!Object.keys(DOCTOR_DEFAULTS).every(k=>Object.hasOwn(settings,k)))throw new Error('Invalid preflight settings.');
 for(const v of Object.values(settings))if(typeof v!=='string'||new TextEncoder().encode(v).length>200)throw new Error('Preflight fields are limited to 200 UTF-8 bytes each.');
 validatePolicyInputs({query:settings.state,source:value.policy});
 if(value.context!==null){validateSourceFile(value.context,{maxBytes:8*1024*1024});exactContextText(value.context);}
 return value;
}
export function doctorRequest(value){
 validateDoctorInputs(value);if(Object.values(value.settings).some(v=>!v.trim()))throw new Error('Supply a location code, year, chamber and resolution.');
 if(value.native?.enabled)return nativeDoctorWorkspaceRequest(value.settings,value.native);
 const policy=policyRequest({query:value.settings.state,source:value.policy});
 return {operation:'diagnose-location-inputs',settings:value.settings,source_name:policy.source_name,policy_json:policy.policy_json,context_source_name:value.context?.name??null,context_json:value.context?exactContextText(value.context):null};
}
export async function readDoctorContext(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select nonempty RCTX of at most 8 MiB.');
 const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('Context source size changed while reading.');exactContextText(source);return source;
}

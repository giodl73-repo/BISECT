import {validatePolicyInputs,policyRequest,readPolicySource} from './policy-files.js';
import {validateSourceFile} from './package-files.js';
import {validatePreparedGraphInputs} from './prepared-graph-files.js';
import {inspectJsonText} from './project.js';
export {readPolicySource as readChamberPolicySource};
export const CHAMBER_COUNT_DEFAULTS={state:'RI',year:'2020',chamber:'house',policy:null,manifest:null};
export function validateChamberCountInputs(value){
 if(!value||Object.keys(value).sort().join()!=='chamber,manifest,policy,state,year'||typeof value.state!=='string'||! /^[A-Z]{2}$/.test(value.state)||!['2000','2010','2020'].includes(value.year)||!['congressional','house','senate'].includes(value.chamber))throw new Error('Select state, Census year and chamber.');
 validatePolicyInputs({query:value.state,source:value.policy});if(value.manifest!==null)validateSourceFile(value.manifest,{maxBytes:2*1024*1024});return value;
}
export function chamberCountRequest(value){
 validateChamberCountInputs(value);const policy=policyRequest({query:value.state,source:value.policy});
 return {operation:'resolve-native-chamber-count',input:{state:value.state,year:value.year,chamber:value.chamber,source_name:policy.source_name,policy_json:value.policy?new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(value.policy.base64),c=>c.charCodeAt(0))):null,manifest_json:value.manifest?new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(value.manifest.base64),c=>c.charCodeAt(0))):null}};
}
export function graphWithChamberCount(graphInputs,countInputs,result){
 const next=structuredClone(validatePreparedGraphInputs(graphInputs));validateChamberCountInputs(countInputs);
 if(result?.schema_version!=='bisect-native-chamber-count-v1'||result.execution_performed!==false||result.state!==countInputs.state||result.year!==countInputs.year||result.chamber!==countInputs.chamber||!Number.isSafeInteger(result.districts)||result.districts<1||result.districts>500||!next.graph)throw new Error('Resolve a fresh matching chamber count first.');
 inspectJsonText(next.graph.text,4000000);const scope=JSON.parse(next.graph.text);
 if(scope.state!==result.state||scope.year!==result.year)throw new Error('Prepared graph state/year differs from the chamber count.');
 next.districts=result.districts;next.settings.chamber=result.chamber;return validatePreparedGraphInputs(next);
}

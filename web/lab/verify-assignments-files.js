import {packageVerificationRequest} from './package-verification-files.js';
import {validateAssignmentDiscoveryInputs} from './assignment-discovery-files.js';
import {validateAssignmentVerificationInputs,assignmentSourceText} from './assignment-verification-files.js?v=assignments-only-v1';
import {parseSafeJson} from './project.js';
export function validateVerifyAssignmentsSettings(v){
 if(!v||Object.keys(v).sort().join()!=='minSimilarity,reference')throw new Error('Select a reference and native agreement threshold.');
 validateAssignmentVerificationInputs({original:null,reference:v.reference,minSimilarity:v.minSimilarity});return v;
}
export function verifyAssignmentsRequest(packageInputs,discoveryInputs,settings){
 validateVerifyAssignmentsSettings(settings);
 // The prechecks manifest is also the discovery manifest; there is one authority.
 validateAssignmentDiscoveryInputs({...discoveryInputs,manifest:packageInputs.manifest});
 const prechecks=packageVerificationRequest(packageInputs,'verify-manifest-prechecks').input;
 const reference_json=settings.reference?assignmentSourceText(settings.reference):null;
 const output_files_base64=Object.fromEntries(Object.entries(discoveryInputs.files).map(([p,s])=>[p,s.base64]));
 const size=new TextEncoder().encode(prechecks.manifest_json).length+(reference_json?new TextEncoder().encode(reference_json).length:0)+[...Object.values(prechecks.files_base64),...Object.values(output_files_base64)].reduce((n,b)=>n+atob(b).length,0);
 if(size>16*1024*1024)throw new Error('Combined assignments-only sources exceed 16 MiB.');
 return {operation:'verify-assignments-only',input:{prechecks,output_files_base64,explicit_label:discoveryInputs.explicitLabel||null,reference_json,min_similarity:settings.minSimilarity}};
}
export function verifyAssignmentsReceiptBytes(result){
 if(result?.schema_version!=='bisect-assignments-only-verification-v1'||!['match','different','unavailable','precheck-failed','discovery-failed','reference-required','reference-failed'].includes(result.status)||result.native_exit_code!==(result.status==='match'?0:1)||result.execution_performed!==false)throw new Error('Run assignments-only verification first.');
 const text=JSON.stringify(result,null,2)+'\n';parseSafeJson(text);return new TextEncoder().encode(text);
}

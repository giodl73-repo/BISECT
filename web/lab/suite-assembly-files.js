import {validateSuiteInputs} from './suite-files.js';
import {validatePolicyInputs} from './policy-files.js';
import {validateSourceFile,exactSourceText} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const SUITE_ASSEMBLY_DEFAULTS={nest_mode:'none',congressional:null,policy:null};
export function validateSuiteAssemblyInputs(v){
 if(!v||Object.keys(v).sort().join()!=='congressional,nest_mode,policy'||!['none','senate-in-house'].includes(v.nest_mode))throw new Error('Invalid suite assembly inputs.');
 if(v.congressional!==null){validateSourceFile(v.congressional,{maxBytes:3*1024*1024});exactSourceText(v.congressional);}
 validatePolicyInputs({query:'RI',source:v.policy});return v;
}
export function suiteAssemblyRequest(suite,inputs){
 validateSuiteInputs(suite);validateSuiteAssemblyInputs(inputs);if(!suite.house||!suite.senate)throw new Error('Select House and Senate RPLAN files.');if(!/^[A-Za-z0-9_-]{1,100}$/.test(suite.suite_name)||['__proto__','constructor','prototype'].includes(suite.suite_name))throw new Error('Suite package name needs 1–100 letters, digits, underscores or hyphens and cannot use reserved object names.');
 const house_json=exactSourceText(suite.house),senate_json=exactSourceText(suite.senate),congressional_json=inputs.congressional?exactSourceText(inputs.congressional):null;
 if(new TextEncoder().encode(house_json+senate_json+(congressional_json||'')).length>3*1024*1024)throw new Error('Chamber plans must total at most 3 MiB.');
 return {operation:'assemble-plan-suite',house_json,senate_json,congressional_json,suite_name:suite.suite_name,nest_mode:inputs.nest_mode,required_ratio:suite.required_ratio,policy_source_name:inputs.policy?.name??null,policy_json:inputs.policy?exactSourceText(inputs.policy):null};
}
export function suiteAssemblyZip(result){
 if(result?.schema_version!=='bisect-suite-assembly-v1'||!result.files||!result.suite||!/^[A-Za-z0-9_-]{1,100}$/.test(result.suite.suite_name))throw new Error('Run a fresh suite assembly first.');
 const name=result.suite.suite_name,entries=Object.entries(result.files),required=[name+'/suite.json',name+'_house.rplan',name+'_senate.rplan'];
 if(entries.length<3||entries.length>4||required.some(p=>!Object.hasOwn(result.files,p))||entries.some(([p,t])=>typeof t!=='string'||![...required,name+'_congressional.rplan'].includes(p)))throw new Error('Invalid suite package paths.');
 return storedZipBytes(entries.map(([p,t])=>[p,new TextEncoder().encode(t)]));
}

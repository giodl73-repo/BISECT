import {validateSourceFile,exactSourceText} from './package-files.js';
export function validateStatisticsInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==4||!['demographic','urban','election','tolerance_percent'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid plan statistics inputs.');
 if(!Number.isFinite(value.tolerance_percent)||value.tolerance_percent<0||value.tolerance_percent>100)throw new Error('Population tolerance percent must be between 0 and 100.');
 let bytes=0;for(const key of ['demographic','urban','election'])if(value[key]!==null){validateSourceFile(value[key]);if(value[key].base64.length>4*1024*1024)throw new Error('Statistics sources exceed 3 MiB.');exactSourceText(value[key]);bytes+=atob(value[key].base64).length;}
 if(bytes>3*1024*1024)throw new Error('Statistics sources must total at most 3 MiB.');return value;
}
export function statisticsRequest(files,inputs){
 validateStatisticsInputs(inputs);if(!files.auditPlanText)throw new Error('Select an exact RPLAN.');if(['demographic','urban','election'].every(k=>inputs[k]===null))throw new Error('Select at least one analyzer CSV source.');
 const document_json=exactSourceText(files.auditPlanText);let bytes=atob(files.auditPlanText.base64).length;for(const key of ['demographic','urban','election'])if(inputs[key])bytes+=atob(inputs[key].base64).length;if(bytes>3*1024*1024)throw new Error('Plan and analyzer sources must total at most 3 MiB.');
 return {operation:'analyze-plan-statistics',document_json,demographic_csv:inputs.demographic?exactSourceText(inputs.demographic):null,urban_csv:inputs.urban?exactSourceText(inputs.urban):null,election_csv:inputs.election?exactSourceText(inputs.election):null,tolerance_percent:inputs.tolerance_percent};
}

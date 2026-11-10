// Select retained State outputs without running or authenticating their sources.
import {validateStateRunBatchInputs,stateRunJobs} from './state-run-batch.js';
import {stateJobScope} from './state-source-job.js';
export const stateOutputSha=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function stateRunOutputJobs(result,selectedInputs){
 if(result?.schema_version!=='bisect-state-run-batch-v1'||!['completed','cancelled','interrupted'].includes(result.status)||!Array.isArray(result.states)||result.states.length>51||new Set(result.states.map(r=>r.state_code)).size!==result.states.length)throw new Error('Finish or cancel a valid state batch first.');
 const inputs=structuredClone(validateStateRunBatchInputs(selectedInputs));if(!inputs.jobs)throw new Error('Retain the original selected jobs for State output handoff.');
 const jobs=stateRunJobs(inputs.jobs,inputs.all_years?153:51),year=result.selection?.year;
 if(!['2000','2010','2020'].includes(year))throw new Error('Select a completed native Census year.');
 const text=inputs.all_years?JSON.stringify(jobs.filter(j=>stateJobScope(j).year===year)):new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(inputs.jobs.base64),c=>c.charCodeAt(0)));
 const name=inputs.all_years?'prepared-'+year+'-jobs.json':inputs.jobs.name;
 if(await stateOutputSha(text)!==result.jobs_source_sha256||name!==result.jobs_source_name)throw new Error('Selected job bytes differ from the completed year. Run the batch again.');
 return {jobs,year};
}

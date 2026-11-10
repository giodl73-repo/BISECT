import {inspectJsonText} from './project.js';
export function validateFletchHistory(v){
 if(!v||Object.keys(v).sort().join()!=='attempts,last_retryable_error,retries'||![v.attempts,v.retries].every(n=>Number.isInteger(n)&&n>=0&&n<=4294967295)||v.last_retryable_error!==null&&(typeof v.last_retryable_error!=='string'||new TextEncoder().encode(v.last_retryable_error).length>16384))throw new Error('Invalid bounded native fetch history.');
 return v;
}
export function fletchHistoryFromEntry(text){
 if(typeof text!=='string'||text.length>128*1024)throw new Error('Invalid stored native entry.');
 inspectJsonText(text);const entries=JSON.parse(text);
 if(!Array.isArray(entries)||entries.length!==1)throw new Error('Select one native cache entry.');
 const e=entries[0];return validateFletchHistory({attempts:e.fetch_attempts,retries:e.retry_count,last_retryable_error:e.last_retryable_error});
}

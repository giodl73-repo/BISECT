import {fetchPlanRequest} from './fetch-plan-files.js';
import {validateFletchCacheSettings} from './fletch-cache-files.js';
export function validateFletchSourceSettings(value){return validateFletchCacheSettings(value);}
export function fletchSourcesRequest(value){
 if(!value||Object.keys(value).sort().join()!=='selection,settings')throw new Error('Invalid FLETCH source selection.');
 validateFletchSourceSettings(value.settings);
 const input=fetchPlanRequest(value.selection).input;
 if(value.settings.year!=='selection')input.year=value.settings.year;
 return {operation:'inspect-fletch-sources',input};
}
export function fletchSourcesReportBytes(result){
 if(result?.schema_version!=='bisect-fletch-source-handoff-v1'||typeof result.report_csv!=='string'||result.report_csv.length>16*1024*1024)throw new Error('Run source handoff inspection before downloading its CSV.');
 return new TextEncoder().encode(result.report_csv);
}

import {createPreparedCensusJob} from './prepared-census-job.js';
import {censusPreparationRequest,preparedTractRunSources} from './tract-input-files.js';
import {configValidationRequest} from './config-files.js';
import {validateConfigRunInputs,validateConfigRunSettings} from './config-run.js';
import {validateTuiRunSettings,tuiRunRequest,tuiRunApplyPreferences} from './tui-run-files.js';
import {validateTuiRunWorkflowSettings,stateWorkflowOptions} from './state-workflow-files.js';
import {runTargetRequest} from './run-target-files.js';
import {validateStateSourceAcquisition} from './state-source-acquisition.js';
import {validateElectionSourceFormat} from './election-source-format.js';

const resolution=kind=>({'counties':'county','tracts':'tract','block-groups':'block_group','blocks':'block'})[kind];
const workflowKeys=['statisticsInputs','splitsGeography','electionAnalysisSource','electionAnalysisSettings','electionCsvFormat'];
export function stateJobScope(job){return job.native_state?job.scope:job.prepared_engine_template?job.scope:job.prepared_engine?job.scope:job.source_preparation?{state:job.source_preparation.state,year:String(job.source_preparation.pl94.year),districts:job.configuration.settings.districts,chamber:job.configuration.settings.chamber}:{state:job.run.graph.state,year:job.run.graph.year,districts:job.run.districts,chamber:job.run.chamber};}
function sourceFiles(input){
 if(!input||Object.keys(input).some(k=>!['state','projection','tiger','pl94','geometry_profile'].includes(k)))throw new Error('Invalid Census source preparation input.');
 return {tractPreparationSettings:{state:input.state,projection:input.projection,...(Object.hasOwn(input,'geometry_profile')?{geometry_profile:input.geometry_profile}:{})},tigerInputs:input.tiger,pl94Inputs:input.pl94};
}
export function validateStateSourceJob(job){
 if(!job||Object.keys(job).some(k=>!['form','created_at','source_preparation','configuration','target','followup','acquisition'].includes(k)))throw new Error('Invalid source-based State job.');
 if(Object.hasOwn(job,'acquisition')){validateStateSourceAcquisition(job.acquisition);if(job.acquisition.receipts[0].state!==job.source_preparation?.state||job.acquisition.receipts[0].year!==String(job.source_preparation?.pl94?.year)||job.source_preparation?.tiger?.kind!=='tracts')throw new Error('Acquisition scope differs from the source job.');}
 validateTuiRunSettings({form:job.form,created_at:job.created_at});
 const preparation=censusPreparationRequest(sourceFiles(job.source_preparation));
 if(job.form.location.toUpperCase()!==preparation.input.state||job.form.year!==String(preparation.input.pl94.year)||job.form.resolution!==resolution(preparation.input.tiger.kind))throw new Error('Source preparation scope differs from the queued Run form.');
 const c=job.configuration;if(!c||Object.keys(c).sort().join()!=='inputs,profile,settings,source'||Object.keys(c.inputs||{}).some(k=>!['demographics','coi'].includes(k)))throw new Error('Source jobs require retained YAML, profile and optional demographic/COI inputs.');
 configValidationRequest(c.source,c.profile);validateConfigRunInputs(c.inputs);
 if(c.settings?.districts!==null&&!Number.isSafeInteger(c.settings?.districts))throw new Error('Source districts must be explicit or selected from the native allocation.');
 validateConfigRunSettings({...c.settings,districts:c.settings.districts??1});
 if(c.settings.chamber!=='congressional'||job.form.chamber!=='congressional')throw new Error('State batches use native congressional allocation.');
 if(Object.hasOwn(job,'followup')&&job.followup!==null){if(Object.keys(job.followup).sort().join()!=='settings,sources'||Object.keys(job.followup.sources||{}).some(k=>!workflowKeys.includes(k)))throw new Error('Invalid source job follow-up recipe.');validateTuiRunWorkflowSettings(job.followup.settings);if(job.followup.sources.electionCsvFormat!==undefined)validateElectionSourceFormat(job.followup.sources.electionCsvFormat);}
 return job;
}
export function createStateSourceJob(files,runSettings,formSettings,workflowSettings){
 if(files.preparedCensusTemplateInputs?.enabled)return createPreparedCensusJob(files);
 const preparation=censusPreparationRequest(files).input;
 const preferences={location:preparation.state,year:String(preparation.pl94.year),resolution:resolution(preparation.tiger.kind),chamber:'congressional',version:formSettings.form.version};
 const settings=tuiRunApplyPreferences(formSettings,preferences);
 const inputs=Object.fromEntries(Object.entries(files.configRunInputs||{}).filter(([k])=>['demographics','coi'].includes(k)));
 const job={form:settings.form,created_at:settings.created_at,source_preparation:preparation,configuration:{source:files.configSource,profile:files.configSettings||{profile:'open'},inputs,settings:{...runSettings,chamber:'congressional',native_build:true,districts:null}},...(settings.target?{target:runTargetRequest(settings.target)}:{})};
 if(workflowSettings.analysis||workflowSettings.map||workflowSettings.report)job.followup={settings:workflowSettings,sources:Object.fromEntries(workflowKeys.filter(k=>files[k]).map(k=>[k,files[k]]))};
 return structuredClone(validateStateSourceJob(job));
}
export function executeStateSourceJob(engine,job,{onProgress}={}){
 validateStateSourceJob(job);if(!Number.isSafeInteger(job.configuration.settings.districts)||job.configuration.settings.districts<1)throw new Error('Resolve the native district allocation before source execution.');
 onProgress?.({schema_version:'bisect-source-state-progress-v1',stage:'source-preparation',message:'Preparing selected Census sources'});
 const prepared=engine.execute(censusPreparationRequest(sourceFiles(job.source_preparation)),{onProgress}),sources=preparedTractRunSources(prepared),c=job.configuration;
 if(prepared.state!==job.form.location.toUpperCase()||prepared.year!==job.form.year)throw new Error('Prepared Census scope differs from the queued state.');
 try{
 const request=tuiRunRequest({form:job.form,created_at:job.created_at},c.source,c.profile,{...c.inputs,graph:sources.graph},c.settings);
 if(Object.hasOwn(job,'target'))request.input.target=job.target;
 if(job.followup)request.input.workflow=stateWorkflowOptions({...job.followup.sources,planGeometry:sources.geometry},job.followup.settings,job.created_at);
 const result=engine.execute(request,{onProgress});
 return {schema_version:'bisect-source-state-run-v1',source_preparation:prepared,result};
 }catch(error){error.source_preparation=prepared;throw error;}
}

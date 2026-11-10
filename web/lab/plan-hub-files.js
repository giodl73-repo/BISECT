import {validateLabelInventoryInputs,nativeRegistryText} from './label-inventory-files.js';
import {validateConfigSource} from './config-files.js';
import {tuiLibraryRequest} from './tui-library-files.js';
const encoder=new TextEncoder();
export const PLAN_HUB_DEFAULTS={label:'',configure:false,config_path:''};
export function validatePlanHubInputs(v){
 if(!v||Object.keys(v).sort().join()!=='registry,settings,yaml'||!v.settings||Object.keys(v.settings).sort().join()!=='config_path,configure,label'||typeof v.settings.configure!=='boolean')throw new Error('Invalid plan hub selections.');
 validateLabelInventoryInputs({registry:v.registry,label:v.settings.label});
 if(typeof v.settings.config_path!=='string'||encoder.encode(v.settings.config_path).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(v.settings.config_path))throw new Error('Recorded config path must be text up to 200 bytes without controls.');
 if(v.yaml!==null)validateConfigSource(v.yaml);return v;
}
export function planHubRequest(v,library){
 validatePlanHubInputs(v);const registry={registry_json:v.registry?nativeRegistryText(v.registry):null,label:v.settings.label||null};
 return {operation:'inspect-plan-hub',input:{registry_json:registry.registry_json,label:registry.label,configure:v.settings.configure,config_path:v.settings.config_path||(v.yaml?.name??null),library:!v.settings.configure&&library?tuiLibraryRequest({...library,settings:{...library.settings,filter:'',sort_column:'label',sort_direction:'asc'}}).input:null}};
}
export function planHubTarget(result,v,library){
 const request=planHubRequest(v,library);
 if(result?.schema_version!=='bisect-plan-hub-v1'||result.execution_performed!==false||result.selected_label!==request.input.label||result.config_path!==request.input.config_path||result.mode!==(v.settings.configure?'configure':v.settings.label?'label':'picker'))throw new Error('Run the plan hub with these selections first.');
 if(result.mode==='configure')return v.yaml?{operation:'validate-algorithm-config',configSource:v.yaml}:{operation:'create-algorithm-config',name:/^[A-Za-z0-9_-]{1,100}$/.test(v.settings.label)?v.settings.label:null};
 const selected=result.selected_directory??'';if(selected&&!library?.plans?.some(plan=>plan.directory===selected))throw new Error('Selected plan is absent from retained library sources.');
 return {operation:'inspect-tui-plan-library',settings:{filter:'',sort_column:'label',sort_direction:'asc',selected_directory:selected}};
}

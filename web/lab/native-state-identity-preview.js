import {nativeStateNameComponent} from './native-state-name-effects.js';
// Read-only UI composition. Empty folder evidence never establishes live presence.
export async function inspectStateCommandWithIdentity(engine,arguments_){
 const command=await engine.execute({operation:'inspect-native-state-command',arguments:arguments_});
 if(command?.status!=='parsed'||command.profile_filter?.allowed!==true)return command;
 const value=id=>command.fields?.find(f=>f.id===id)?.values?.[0]??null;
 const version=value('version');
 let identity=null,target=null,error=null;
 try{
  identity=await engine.execute({operation:'inspect-native-state-identity',arguments:arguments_});
  if(identity?.schema_version!=='bisect-native-state-identity-v1'||typeof identity.native_year_valid!=='boolean'||identity.execution_performed!==false||identity.filesystem_reads_performed!==false||identity.path_authority_granted!==false)throw Error('This engine does not provide the complete native naming review.');
  if(!identity.native_year_valid)throw Error(identity.native_year_error||'Native location/year preflight failed.');
  target=await engine.execute({operation:'inspect-native-named-state-run-target',arguments:arguments_,version,input:{source_name:'unobserved-name-preview.json',folder_json:JSON.stringify({version,paths:[],manifests:{}}),force:false}});
  if(target?.schema_version!=='bisect-native-run-target-v1'||target.layout!==identity.layout||target.identity?.effective_state_name!==identity.effective_state_name||target.identity?.effective_label!==identity.effective_label||typeof target.target_namespace!=='string'||!target.target_namespace.startsWith('outputs/'+version+'/'))throw Error('Native naming target differs from its inspected identity.');
 }catch(failure){error=String(failure.message).slice(0,2000);target=null;}
 return {...command,browser_identity_preview:{schema_version:'bisect-native-state-identity-preview-v1',identity,target,error,output_dir_intent:value('output_dir')??'outputs/'+version,relative_target:target?.target_namespace.slice(('outputs/'+version+'/').length)??null,folder_presence_observed:false,execution_authorized:false}};
}

export function renderStateIdentityPreview(target,preview){
 if(preview?.schema_version!=='bisect-native-state-identity-preview-v1'||preview.folder_presence_observed!==false||preview.execution_authorized!==false)return;
 const section=document.createElement('section');section.id='native-state-identity-preview';
 const title=document.createElement('h3');title.textContent='Native State name and output preview';section.append(title);
 const row=(label,value)=>{const p=document.createElement('p'),strong=document.createElement('strong'),text=document.createElement('span');p.style.overflowWrap='anywhere';strong.textContent=label+': ';text.style.whiteSpace='pre-wrap';for(const part of String(value).slice(0,2000).split(/(?<=[_/])/)){text.append(document.createTextNode(part));if(/[_/]$/.test(part))text.append(document.createElement('wbr'));}p.append(strong,text);section.append(p);};
 const identity=preview.identity;
 if(identity?.schema_version==='bisect-native-state-identity-v1'){
  row('Effective state name',JSON.stringify(identity.effective_state_name));
  row('Name selection',identity.override_ignored?'The native State path ignores --state-name without --adjacency.':identity.override_effective?'Explicit adjacency name, preserved as entered.':'Native registered name or lowercase code fallback.');
  if(typeof identity.effective_label==='string')row('Plan label',identity.effective_label);
  if(!nativeStateNameComponent(identity.effective_state_name))row('Source/Run handoff','Choose a portable effective name before applying this workflow. The original name remains an observation; it does not authorize named source paths.');
 }
 if(typeof preview.relative_target==='string'){
  row('Folder within output root',preview.relative_target);
  row('Recorded output root',preview.output_dir_intent);
 }
 if(typeof preview.error==='string')row('Naming review refused',preview.error);
 row('Review status','No output folder was observed or opened. This preview grants no Run, overwrite or deletion authority. Source selection and target review remain separate.');
 target.append(section);
}

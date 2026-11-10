import {validateNativeAggregateInputs,resolvedNativeAggregateTypes,NATIVE_AGGREGATE_TYPES} from './native-aggregate-files.js';
import {encodeBytes} from './package-files.js';
import {workspaceCheck,workspacePathParts,workspacePermission,workspaceEntries,workspaceLookup,workspaceReadFile} from './selected-workspace-io.js';
export function aggregateWorkspaceVersion(version){
 try{if(typeof version!=='string'||version.length>100||version.includes('/'))throw Error();workspacePathParts(version);}catch{throw Error('Choose a portable aggregate version component.');}return version;
}
export function validateNativeAggregateWorkspaceSettings(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).join()!=='version')throw Error('Invalid aggregate workspace settings.');aggregateWorkspaceVersion(value.version);return value;
}
// Read-only source selection. This record never retains a root or write grant.
export async function readNativeAggregateWorkspace(root,settings,{version='v1',force=false,signal,assertCurrent,...unknown}={}){
 if(Object.keys(unknown).length)throw Error('Unsupported workspace read option.');const o={signal,assertCurrent};
 aggregateWorkspaceVersion(version);if(typeof force!=='boolean')throw Error('Choose the aggregate Force setting.');validateNativeAggregateInputs({settings,reports:{}});const retainedSettings=structuredClone(settings);settings=retainedSettings;const prefix=version+'/'+settings.year;
 await workspacePermission(root,o);const states=await workspaceLookup(root,prefix+'/states',{...o,kind:'directory',create:false});if(!states)throw Error('Selected outputs folder has no '+prefix+'/states directory.');
 const national=await workspaceLookup(root,prefix+'/national',{...o,kind:'directory',create:false}),existing=[];
 if(national)for(const [name]of await workspaceEntries(national,o)){const expected=NATIVE_AGGREGATE_TYPES.find(type=>('us_'+type+'.json').toLowerCase()===name.toLowerCase());if(expected){if(name!=='us_'+expected+'.json')throw Error('Case-colliding native output name.');existing.push(name);}}
 existing.sort();const selected=[...new Set(resolvedNativeAggregateTypes(settings.types))],active=selected.filter(type=>force||!existing.includes('us_'+type+'.json')),deferred=selected.filter(type=>!active.includes(type));
 const stateEntries=(await workspaceEntries(states,o)).filter(([,handle])=>handle.kind==='directory').sort(([a],[b])=>a<b?-1:a>b?1:0);
 if(!stateEntries.length||stateEntries.length>60)throw Error('Choose a workspace containing 1-60 state folders.');
 const reports=Object.create(null),sources=[];let total=0;
 for(const [state]of stateEntries){
  if(!/^[A-Za-z0-9_ -]{1,60}$/.test(state))throw Error('State folder label is outside the portable aggregate scope.');workspacePathParts(prefix+'/states/'+state);reports[state]=Object.create(null);
  // Do not read unused analyzers, backups or large unrelated data files.
  for(const type of active){workspaceCheck(o);const sourcePath=prefix+'/states/'+state+'/analysis/'+type+'.json',handle=await workspaceLookup(root,sourcePath,{...o,kind:'file',create:false});if(!handle)continue;const remaining=8*1024*1024-total-state.length-type.length;if(remaining<0)throw Error('Aggregate source collection exceeds 8 MiB.');const file=await workspaceReadFile(handle,{...o,maxBytes:Math.min(3*1024*1024,remaining)});new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(file.data);
   if((total+=state.length+type.length+file.byte_length)>8*1024*1024)throw Error('Aggregate source collection exceeds 8 MiB.');reports[state][type]={name:handle.name,base64:encodeBytes(file.data)};sources.push({path:sourcePath,sha256:file.sha256,byte_length:file.byte_length});
  }
 }
 const roster=(await workspaceEntries(states,o)).filter(([,handle])=>handle.kind==='directory').map(([name])=>name).sort();if(JSON.stringify(roster)!==JSON.stringify(stateEntries.map(([name])=>name)))throw Error('State folder selection changed while reading.');await workspacePermission(root,o);workspaceCheck(o);
 const input=validateNativeAggregateInputs({settings:retainedSettings,reports,completion:{year:settings.year,existing_outputs:existing,force},...(deferred.length?{deferred_types:deferred}: {})});
 return {input,observation:{schema_version:'bisect-native-aggregate-source-observation-v1',root_name:root.name,version,year:settings.year,states:roster,sources,existing_outputs:existing,deferred_types:deferred,scope:'Read-only bounded UTF-8 native source snapshot, including empty state directories. Output existence is metadata only; unrelated/unselected/skipped source bodies are not read. No root or write authority is retained. Missing source/kind/encoding/size and nonportable paths refuse explicitly.'}};
}

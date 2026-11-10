import {aggregateWorkspaceVersion} from './native-aggregate-workspace.js';
import {validateNativeAggregateInputs,resolvedNativeAggregateTypes} from './native-aggregate-files.js';
const encode=s=>new TextEncoder().encode(s),digest=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
// Private prototype of native orchestration, not an imported-project entry point.
// The caller supplies a live, authorized/coordinated IO capability. This module
// creates no grants, picker, recovery authority or implicit copy/delete rename.
export async function executeNativeAggregateEffects(execute,io,{settings,version='v1',force=false,signal,assertCurrent,...unknown}={}){
 if(Object.keys(unknown).length)throw Error('Unsupported native effect option.');
 if(typeof execute!=='function'||!io||['mkdir','stat','readdir','read','write','rename'].some(n=>typeof io[n]!=='function'))throw Error('Live native-effect IO capability is required.');
 aggregateWorkspaceVersion(version);if(typeof force!=='boolean')throw Error('Invalid native aggregate Force option.');
 if(!settings||!['csv','json','none','parquet'].includes(settings.format))throw Error('Invalid native aggregate format.');
 const normalized={...settings,format:settings.format==='parquet'?'none':settings.format};validateNativeAggregateInputs({settings:normalized,reports:{}});settings=structuredClone(settings);
 const prefix=version+'/'+settings.year,national=prefix+'/national',statesPath=prefix+'/states',result={schema_version:'bisect-native-aggregate-effects-prototype-v1',status:'incomplete',settings,version,force,events:[],frames:[],warnings:[],effects_may_have_started:false,failed_operation:null,scope:'Low-level private prototype over supplied live IO. Native create/write-temp/rename/CSV order on bounded portable state inputs. No UI review, durable journal/recovery, permission grant, authenticity, copy/delete rename emulation or complete host/failure equivalence.'};
 const check=()=>{signal?.throwIfAborted();assertCurrent?.();};let active=null;
 const event=value=>{if(result.events.length>=20000)throw Error('Native effect trace exceeds its event bound.');result.events.push({sequence:result.events.length,...value});};
 async function action(operation,path,task,{mutates=false,...detail}={}){check();active={operation,path,...detail};event({phase:'intent',...active,mutates});if(mutates)result.effects_may_have_started=true;const value=await task();check();event({phase:'done',...active,mutates});active=null;return value;}
 try{
  await action('mkdir',national,()=>io.mkdir(national),{mutates:true});
  if(!await action('stat',statesPath,()=>io.stat(statesPath))){active={operation:'discover-states',path:statesPath};throw Error('No states directory at '+statesPath+'.');}
  const entries=await action('readdir',statesPath,()=>io.readdir(statesPath));if(!Array.isArray(entries)||entries.length>20000)throw Error('Native state enumeration exceeds its bounds.');
  const states=entries.filter(e=>e.kind==='directory').map(e=>e.name).sort(),folded=new Set();if(states.length>60)throw Error('Native effect prototype supports at most 60 state folders.');
  for(const state of states){if(typeof state!=='string'||!/^[A-Za-z0-9_ -]{1,60}$/.test(state)||folded.has(state.toUpperCase()))throw Error('Native effect state label is outside the bounded portable scope.');folded.add(state.toUpperCase());}
  for(const [frame,type]of resolvedNativeAggregateTypes(settings.types).entries()){
   check();const output=national+'/us_'+type+'.json';
   if(await action('stat',output,()=>io.stat(output))&&!force){result.frames.push({frame,analyzer:type,status:'skipped-existing'});continue;}
   if(!states.length){result.frames.push({frame,analyzer:type,status:'no-data'});continue;}
   const reports=Object.create(null);let total=0;
   for(const state of states){reports[state]=Object.create(null);const source=statesPath+'/'+state+'/analysis/'+type+'.json';if(!await action('stat',source,()=>io.stat(source)))continue;
    try{const remaining=8*1024*1024-total-state.length-type.length;if(remaining<0)throw Object.assign(Error('Native effect source collection exceeds 8 MiB.'),{code:'ResourceLimit'});const bytes=await action('read',source,()=>io.read(source,Math.min(3*1024*1024,remaining)));if(!(bytes instanceof Uint8Array)||bytes.length>Math.min(3*1024*1024,remaining))throw Object.assign(Error('Native effect source exceeds its byte allowance.'),{code:'ResourceLimit'});total+=state.length+type.length+bytes.length;reports[state][type]=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);}
    catch(e){check();if(e.code==='ResourceLimit')throw e;result.warnings.push({frame,source,error:e.message});event({phase:'warning',operation:'read',path:source,frame,mutates:false});active=null;}
   }
   const reply=await action('compute',type,()=>execute({operation:'aggregate-native-analysis',input:{year:settings.year,types:[type],format:normalized.format,csv:settings.csv,reports}}),{frame});
   if(reply?.schema_version!=='bisect-native-aggregate-v1'||reply.year!==settings.year||reply.summaries?.length!==1||reply.summaries[0].analyzer!==type)throw Error('Invalid native effect kernel reply.');
   const status=reply.summaries[0].status;if(!['aggregated','no-data'].includes(status))throw Error('Unexpected native effect kernel completion.');
   result.warnings.push(...(reply.skipped||[]).map(w=>({frame,...w})));const frameResult={frame,analyzer:type,kernel_status:status,effects_status:status==='aggregated'?'incomplete':'not-needed'};result.frames.push(frameResult);
   const names=status==='aggregated'?['us_'+type+'.json',...(settings.csv||settings.format==='csv'?['us_'+type+'.csv']:[]),...(settings.format==='json'?['us_'+type+'_'+(type==='proportionality'?'states':'districts')+'.json']:[])]:[];
   if(!reply.files||!reply.file_sha256||Object.keys(reply.files).sort().join()!==[...names].sort().join()||Object.keys(reply.file_sha256).sort().join()!==[...names].sort().join())throw Error('Native effect kernel file roster differs.');
   const buffers=[];for(const name of names){if(typeof reply.files[name]!=='string')throw Error('Invalid native output text.');const bytes=encode(reply.files[name]);if(bytes.length>25*1024*1024)throw Error('Native effect output exceeds 25 MiB.');const sha256=await digest(bytes);check();if(sha256!==reply.file_sha256[name])throw Error('Native effect output hash mismatch.');buffers.push({name,bytes,sha256});}
   for(const {name,bytes,sha256}of buffers){const target=national+'/'+name,detail={frame,sha256,byte_length:bytes.length,mutates:true};if(name.endsWith('.csv'))await action('write',target,()=>io.write(target,bytes),detail);else{const scratch=target.slice(0,-5)+'.tmp.json';await action('write',scratch,()=>io.write(scratch,bytes),detail);await action('rename',scratch,()=>io.rename(scratch,target),{frame,to:target,mutates:true});}}
   if(status==='aggregated')frameResult.effects_status='completed';
   if(status==='aggregated'&&settings.format==='parquet')result.warnings.push({frame,kind:'parquet-guidance',message:'Parquet format not yet supported. Use --format csv and load with pandas.read_csv() or polars.'});
  }
  check();result.status='completed';return result;
 }catch(e){result.status=signal?.aborted?'cancelled':'failed';result.failed_operation=active;result.error={message:e.message,code:e.code||null};return result;}
}

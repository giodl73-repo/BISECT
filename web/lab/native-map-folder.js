// Explicit, read-only selected roots; directory handles never enter projects.
import {encodeBytes} from './package-files.js';
import {characterStateFips} from './character-input.js';
const check=o=>{if(o.signal?.aborted)throw o.signal.reason||new Error('Folder read cancelled.');if(o.isCurrent&&!o.isCurrent())throw new Error('Project changed during folder selection. Select again.');};
async function child(root,name,kind,optional=false){
 if(root?.kind!=='directory')throw new Error('Select a local directory.');
 try {const h=await root[kind==='file'?'getFileHandle':'getDirectoryHandle'](name);if(h.kind!==kind)throw new Error(name+' has the wrong entry type.');return h;}
 catch(error){if(optional&&error.name==='NotFoundError')return null;throw error;}
}
async function source(handle,limit,o){
 check(o);const file=await handle.getFile();check(o);
 if(!Number.isSafeInteger(file.size)||file.size<0||file.size>limit)throw new Error('Selected '+file.name+' exceeds its source size limit.');
 const bytes=new Uint8Array(await file.arrayBuffer());check(o);const current=await handle.getFile();check(o);
 if(current.size!==file.size||current.lastModified!==file.lastModified||bytes.length!==file.size)throw new Error('Selected '+file.name+' changed while reading. Select again.');
 return {name:file.name,base64:encodeBytes(bytes)};
}
export async function readNativeMapStateDirectory(root,{map_type='districts',...o}={}){
 if(!['districts','political','demographic','compactness'].includes(map_type))throw new Error('Select a final state map type. Round folders use the separate round workflow.');
 check(o);const data=await child(root,'data','directory'),assignments=await source(await child(data,'final_assignments.json','file'),3*1024*1024,o);
 let analysis=null;
 if(map_type!=='districts'){
  const dir=await child(root,'analysis','directory',true),file=dir&&await child(dir,map_type+'.json','file',true);
  if(file)analysis=await source(file,3*1024*1024,o);
 }
 check(o);return {assignments,analysis};
}
export async function readNativeMapTigerDirectory(root,{state,year,kind='tracts',...o}){
 const fips=characterStateFips(state);if(!fips||![2000,2010,2020].includes(year))throw new Error('Select a state and Census vintage first.');
 if(!['tracts','block-groups'].includes(kind))throw new Error('Select tract or block-group native map geometry.');
 const stem='tl_'+year+'_'+fips+(kind==='block-groups'?'_bg':'_tract'),result={kind,year,shp:null,dbf:null,shx:null};let total=0;
 for(const kind of ['shp','dbf','shx']){
  check(o);const handle=await child(root,stem+'.'+kind,'file',kind==='shx');
  if(handle){result[kind]=await source(handle,Math.min(64*1024*1024,90*1024*1024-total),o);const encoded=result[kind].base64;total+=encoded.length/4*3-(encoded.endsWith('==')?2:encoded.endsWith('=')?1:0);}
 }
 check(o);return result;
}
export function nativeMapTigerRequest(tiger,settings){
 if(!tiger?.shp||!tiger?.dbf||!['tracts','block-groups'].includes(tiger.kind))throw new Error('Select matching native tract or block-group SHP and DBF parts.');
 const geometry_profile=settings?.geometry_profile;
 if(!['source-exact','source-exact-large'].includes(geometry_profile))throw new Error('Select an exact source geometry profile for native maps.');
 return {operation:'prepare-native-map-tiger',input:{state:settings.state,tiger,geometry_profile}};
}
export function nativeMapTigerHandoff(result,previous={},selectedTiger=null){
 if(result?.schema_version!=='bisect-native-map-tiger-v1'||result.status!=='prepared'||result.execution_performed!==false||typeof result.geometry_json!=='string')throw new Error('Prepare fresh native map geometry first.');
 if(result.unit_kind==='block-group'){if(selectedTiger?.kind!=='block-groups'||selectedTiger.year!==result.year||!selectedTiger.shp||!selectedTiger.dbf)throw new Error('Retain the selected block-group TIGER sources before handoff.');return {plan:null,dpi:150,map_type:'districts',dissolve_method:'sequential',assignments:null,assignment_index:null,assignment_key_basis:'unit-id',...previous,source_mode:'native-assignments',raw_state:result.state,raw_year:result.year,geometry:null,geometry_source:'tiger',tiger:structuredClone(selectedTiger)};}
 return {plan:null,dpi:150,map_type:'districts',dissolve_method:'sequential',assignments:null,assignment_index:null,assignment_key_basis:'unit-id',...previous,source_mode:'native-assignments',raw_state:result.state,raw_year:result.year,geometry:{name:result.state+'-'+result.year+'-native-map-geometry.json',base64:encodeBytes(new TextEncoder().encode(result.geometry_json))}};
}

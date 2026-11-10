// Explicit local release-file selection; no pickle decoding or automatic execution.
import {validateNativeAdjacencyInputs,validateNativeAdjacencySettings,NATIVE_ADJACENCY_DEFAULTS} from './native-adjacency-files.js';
import {characterStateFips} from './character-input.js';
import {encodeBytes} from './package-files.js';
export async function readNativeAdjacencyBundle(selection,settings,{signal,assertCurrent=()=>{}}={}){
 const check=()=>{signal?.throwIfAborted();assertCurrent();};check();
 const scope=structuredClone(validateNativeAdjacencySettings(settings));
 if(!characterStateFips(scope.state)||!['2000','2010','2020'].includes(scope.year)||scope.unit_kind!==undefined&&scope.unit_kind!=='tract')throw new Error('Release-file import supports Census tract inputs for the selected state/year.');
 const stem=scope.state.toLowerCase()+'_adjacency_'+scope.year;
 const names=new Map([[stem+'.adj.bin','adjacency'],[stem+'_geoids.json','geoids'],[stem+'_centroids.json','centroids']]);
 const ignoredNames=new Set([stem+'.pkl',stem+'.request.json',stem+'_display.geojson',stem+'_county_ids.json']),files=Array.from(selection||[]);
 if(!files.length||files.length>7)throw new Error('Select up to seven matching release files, including one .adj.bin.');
 const seen=new Set(),selected=[],ignored=[];let total=0;
 // Validate every name/size before reading any body. Never mix a previous
 // companion with a new binary, or read pickle/request files opportunistically.
 for(const file of files){
  check();const name=file?.name;
  if(typeof name!=='string'||name.length>200||!/^[-A-Za-z0-9_.]+$/.test(name)||seen.has(name.toLowerCase()))throw new Error('Unsafe or duplicate release filename.');seen.add(name.toLowerCase());
  if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid release-file size.');
  const kind=names.get(name);
  if(!kind&&!ignoredNames.has(name))throw new Error('Release files must match '+stem+' and its supported companion names.');
  if(ignoredNames.has(name)){if(file.size>64*1024*1024)throw new Error('Ignored release-file metadata exceeds the selection bound.');ignored.push({name,byte_length:file.size,reason:'Not read, applied or executed.'});continue;}
  if(file.size<1||file.size>(kind==='adjacency'?64:16)*1024*1024||(total+=file.size)>64*1024*1024)throw new Error('Release inputs exceed the binary/companion or 64 MiB combined limit.');
  selected.push({file,kind,name,size:file.size});
 }
 if(!selected.some(r=>r.kind==='adjacency'))throw new Error('Select the matching .adj.bin. Python pickle cannot be executed in this import.');
 const sources={...NATIVE_ADJACENCY_DEFAULTS},receipts=[];
 for(const row of selected){
  check();const bytes=new Uint8Array(await row.file.arrayBuffer());check();if(bytes.byteLength!==row.size)throw new Error('Release source changed while reading.');
  sources[row.kind]={name:row.name,base64:encodeBytes(bytes)};
  const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');check();receipts.push({kind:row.kind,name:row.name,byte_length:bytes.byteLength,sha256});
 }
 validateNativeAdjacencyInputs(sources,{requireComplete:true});check();
 return {sources,settings:scope,receipt:{schema_version:'bisect-native-adjacency-bundle-import-v1',state:scope.state,year:scope.year,files:receipts,ignored,execution_performed:false,source_authenticity_verified:false,scope:'Explicit local selection matching native release filenames. Pickle and request JSON are not read or executed. Native inspection/preparation and any plan Run remain separate explicit actions.'}};
}

import {parseSafeJson} from './project.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
export const portableRelativePath=p=>typeof p==='string'&&p.length>0&&p.length<=300&&/^[-A-Za-z0-9_/\.]+$/.test(p)&&p.split('/').every(s=>s&&s!=='.'&&s!=='..');
export function selectedFolderEntries(files){
 const selected=[...files];if(!selected.length||selected.length>10000)throw new Error('Select a nonempty output folder with at most 10,000 files.');
 let root;const seen=new Set();const entries=selected.map(file=>{const relative=file.webkitRelativePath;if(typeof relative!=='string'||!relative.includes('/'))throw new Error('Select an output folder, rather than individual files.');const slash=relative.indexOf('/'),name=relative.slice(0,slash),path=relative.slice(slash+1);if(!name||root&&name!==root||!portableRelativePath(path)||seen.has(path))throw new Error('Folder selection requires one root and distinct bounded portable relative paths.');root=name;seen.add(path);return {file,path};});return {root,entries};
}
export function observedFolderPaths(entries){const paths=new Set();for(const {path}of entries){const parts=path.split('/');for(let i=1;i<=parts.length;i++){paths.add(parts.slice(0,i).join('/'));if(paths.size>10000)throw new Error('Folder metadata exceeds 10,000 observed paths.');}}return [...paths].sort();}
export function runTargetText(source){
 validateSourceFile(source,{maxBytes:2*1024*1024});const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0))),v=parseSafeJson(text);
 if(!v||Object.keys(v).sort().join()!=='manifests,paths,version'||typeof v.version!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(v.version)||!Array.isArray(v.paths)||v.paths.length>10000||v.paths.some(p=>!portableRelativePath(p))||!v.manifests||Array.isArray(v.manifests)||typeof v.manifests!=='object'||Object.keys(v.manifests).length>1000||Object.entries(v.manifests).some(([p,t])=>!v.paths.includes(p)||!p.endsWith('/manifest.json')||typeof t!=='string'||new TextEncoder().encode(t).length>1024*1024))throw new Error('Invalid bounded native output folder snapshot.');return text;
}
function snapshotSource(name,value){const source={name,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(value,null,2)+'\n'))};runTargetText(source);return source;}
export async function readRunTargetFolder(files){
 const {root,entries}=selectedFolderEntries(files),manifests={};if(!/^[A-Za-z0-9_-]{1,100}$/.test(root))throw new Error('Select the native version output folder, such as v1.');
 const paths=observedFolderPaths(entries),readable=entries.filter(({file,path})=>path.endsWith('/manifest.json')&&file.size<=1024*1024);
 if(readable.length>1000||readable.reduce((total,{file})=>total+file.size,0)>2*1024*1024)throw new Error('Selected target manifests exceed the 1,000-manifest or 2 MiB read budget.');
 for(const {file,path}of readable){let bytes;try{bytes=await file.arrayBuffer();if(bytes.byteLength!==file.size)throw new Error('Target manifest changed during selection.');}catch(error){throw new Error('Cannot retain selected target manifest: '+error.message);}try{manifests[path]=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);}catch{/* Native unreadable manifest still establishes collision; timestamp is unknown. */}}
 return snapshotSource(root+'-run-target.json',{version:root,paths,manifests});
}
export async function readRunTargetSource(file){if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>2*1024*1024)throw new Error('Select a run-target snapshot up to 2 MiB.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Target snapshot changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};runTargetText(source);return source;}
export function validateRunTargetSettings(v){if(!v||Object.keys(v).sort().join()!=='force,source'||typeof v.force!=='boolean')throw new Error('Invalid run-target settings.');runTargetText(v.source);return v;}
export function runTargetRequest(v){validateRunTargetSettings(v);return {source_name:v.source.name,folder_json:runTargetText(v.source),force:v.force};}

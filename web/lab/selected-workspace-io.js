import {validatePackagePath} from './package-files.js';
export const workspaceCheck=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
export function workspacePathParts(path){
 if(typeof path!=='string')throw Error('Invalid workspace path.');validatePackagePath(path);
 const names=path.split('/');if(names.some(name=>name.length>200||name.endsWith('.')||name.endsWith(' ')||name.startsWith(' ')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(name)))throw Error('Workspace path is not portable.');return names;
}
export async function workspacePermission(root,o={},mode='read'){
 if(!['read','readwrite'].includes(mode))throw Error('Invalid workspace permission mode.');
 workspaceCheck(o);if(root?.kind!=='directory'||typeof root.entries!=='function'||typeof root.getDirectoryHandle!=='function')throw Error('Select a live workspace folder.');
 if(root.queryPermission&&await root.queryPermission({mode})!=='granted')throw Error('Workspace permission is unavailable. Select the folder again.');workspaceCheck(o);
}
export async function workspaceEntries(directory,o={}){
 workspaceCheck(o);if(directory?.kind!=='directory')throw Error('Expected a workspace directory.');const entries=[],folded=new Set();
 for await(const [name,handle]of directory.entries()){workspaceCheck(o);if(entries.length>=20000)throw Error('Workspace directory exceeds 20,000 entries.');if(typeof name!=='string'||!handle||!['file','directory'].includes(handle.kind))throw Error('Unsupported workspace entry.');const key=name.toLowerCase();if(folded.has(key))throw Error('Case-colliding workspace entries.');folded.add(key);entries.push([name,handle]);}
 workspaceCheck(o);return entries;
}
export async function workspaceLookup(root,path,{kind=null,create=false,...o}={}){
 if(![null,'file','directory'].includes(kind)||typeof create!=='boolean')throw Error('Invalid workspace lookup policy.');
 const names=workspacePathParts(path);await workspacePermission(root,o,create?'readwrite':'read');let current=root;
 for(const [index,name]of names.entries()){
  const required=index===names.length-1?kind:'directory';let found=null;
  for(const [actual,handle]of await workspaceEntries(current,o))if(actual.toLowerCase()===name.toLowerCase()){if(actual!==name)throw Error('Case-colliding workspace path.');found=handle;}
  if(!found){if(!create)return null;workspaceCheck(o);found=required==='file'?await current.getFileHandle(name,{create:true}):await current.getDirectoryHandle(name,{create:true});workspaceCheck(o);}
  if(required&&found.kind!==required)throw Error('Workspace path has the wrong file/directory kind: '+names.slice(0,index+1).join('/'));current=found;
 }
 workspaceCheck(o);return current;
}
export async function workspaceReadFile(handle,{maxBytes=25*1024*1024,...o}={}){
 workspaceCheck(o);if(handle?.kind!=='file'||!Number.isSafeInteger(maxBytes)||maxBytes<0||maxBytes>128*1024*1024)throw Error('Invalid workspace file read.');const file=await handle.getFile();workspaceCheck(o);
 if(!Number.isSafeInteger(file.size)||file.size<0||file.size>maxBytes)throw Error('Workspace file exceeds its byte limit.');const data=new Uint8Array(await file.arrayBuffer());workspaceCheck(o);if(data.length!==file.size)throw Error('Workspace file changed while reading.');
 const digest=await crypto.subtle.digest('SHA-256',data);workspaceCheck(o);return {data,sha256:Array.from(new Uint8Array(digest),b=>b.toString(16).padStart(2,'0')).join(''),byte_length:data.length};
}

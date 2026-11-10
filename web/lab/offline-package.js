// Published application bytes only. Projects and provider responses never enter this cache.
export const OFFLINE_SCHEMA='bisect-offline-package-v1';
export function offlinePath(path){
 if(typeof path!=='string'||!path.length||path.length>250||path.startsWith('/')||/[:\\?#%\u0000-\u0020]/.test(path)||path.split('/').some(p=>!p||p==='.'||p==='..'))throw new Error('Unsafe offline asset path.');
 return path;
}
export function validateOfflineManifest(m){
 const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===keys.slice().sort().join();
 if(!exact(m,['schema_version','version','shell','states','assets'])||m.schema_version!==OFFLINE_SCHEMA||!/^[a-f0-9]{64}$/.test(m.version)||!Array.isArray(m.shell)||!m.shell.length||m.shell.length>2000||!exact(m.assets,Object.keys(m.assets??{}))||!exact(m.states,Object.keys(m.states??{}))||Object.keys(m.assets).length>5000)throw new Error('Invalid offline package manifest.');
 for(const [path,a]of Object.entries(m.assets)){offlinePath(path);if(!exact(a,['sha256','bytes'])||!/^[a-f0-9]{64}$/.test(a.sha256)||!Number.isSafeInteger(a.bytes)||a.bytes<1||a.bytes>128*1024*1024)throw new Error('Invalid offline asset identity.');}
 const refs=paths=>{if(!Array.isArray(paths)||new Set(paths).size!==paths.length||paths.some(p=>!Object.hasOwn(m.assets,p)))throw new Error('Missing or repeated offline asset.');};
 refs(m.shell);for(const path of ['index.html','catalog.json','bisect_wasm.wasm','toolkit/index.html','toolkit/bisect_wasm.wasm'])if(!m.shell.includes(path))throw new Error('Incomplete offline application shell.');
 for(const [scope,paths]of Object.entries(m.states)){if(!/^[A-Z]{2}:\d{4}$/.test(scope))throw new Error('Invalid offline state/year.');refs(paths);if(paths.length!==2)throw new Error('Offline state needs graph and map.');}
 return m;
}
export function offlineSelection(manifest,states){
 const m=validateOfflineManifest(manifest);if(!Array.isArray(states)||states.length>150||new Set(states).size!==states.length||states.some(s=>!Object.hasOwn(m.states,s)))throw new Error('Choose available offline state/year data.');
 const paths=[...new Set([...m.shell,...states.flatMap(s=>m.states[s])])];let bytes=0;for(const p of paths)bytes+=m.assets[p].bytes;
 if(bytes>512*1024*1024)throw new Error('Selected offline package exceeds 512 MiB. Select fewer states.');
 return {paths,bytes,states:[...states].sort()};
}
export async function offlineDigest(bytes){return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');}
export async function offlineManifestText(response){
 const limit=2*1024*1024,reader=response.body.getReader(),chunks=[];let size=0;
 try{while(true){const {value,done}=await reader.read();if(done)break;size+=value.length;if(size>limit)throw new Error('Offline manifest exceeds 2 MiB.');chunks.push(value);}}catch(error){await reader.cancel();throw error;}
 const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}
 return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}

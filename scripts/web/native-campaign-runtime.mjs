// Freeze the exact four executable artifacts used by one serial native/WASM campaign.
import fs from 'node:fs/promises';import path from 'node:path';import {createHash} from 'node:crypto';
const names={wasm:'target/wasm32-unknown-unknown/release/bisect_wasm.wasm',host:'target/release/examples/execute_tool_request.exe',prepare:'target/debug/examples/census_prepare_reference.exe',seed:'target/debug/examples/config_seed_reference.exe'};
const sha=b=>createHash('sha256').update(b).digest('hex');
export async function freezeNativeCampaignRuntime(root){
 const folder=path.join(root,'runtime');await fs.mkdir(folder,{recursive:true});const files={};
 for(const [role,filename]of Object.entries(names)){const bytes=await fs.readFile(filename),name=role+'-'+sha(bytes)+path.extname(filename),target=path.join(folder,name);try{await fs.writeFile(target,bytes,{flag:'wx'});}catch(error){if(error.code!=='EEXIST')throw error;const retained=await fs.readFile(target);if(!retained.equals(bytes))throw new Error('Retained campaign runtime differs from its content hash.');}files[role]=name;}
 await fs.writeFile(path.join(root,'runtime.json'),JSON.stringify(files,null,2));return readNativeCampaignRuntime(root);
}
export async function readNativeCampaignRuntime(root){
 const descriptor=JSON.parse(await fs.readFile(path.join(root,'runtime.json'),'utf8'));if(Object.keys(descriptor).sort().join()!=='host,prepare,seed,wasm')throw new Error('Invalid campaign runtime descriptor.');const files={};
 for(const role of Object.keys(names)){const name=descriptor[role];if(typeof name!=='string'||!new RegExp('^'+role+'-[a-f0-9]{64}\\'+path.extname(names[role])+'$').test(name))throw new Error('Invalid pinned runtime filename.');const filename=path.join(root,'runtime',name),bytes=await fs.readFile(filename);if(sha(bytes)!==name.slice(role.length+1,-path.extname(filename).length))throw new Error('Campaign executable integrity mismatch.');files[role]=filename;}
 return files;
}

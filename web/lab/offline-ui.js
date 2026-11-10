import {offlineSelection,validateOfflineManifest,offlineManifestText} from './offline-package.js';
const base=new URL(document.querySelector('meta[name="bisect-offline-root"]')?.content??'./',import.meta.url);
const section=document.createElement('details');section.id='offline-package';
const summary=document.createElement('summary');summary.textContent='Use BISECT offline';section.append(summary);
const text=document.createElement('p');text.textContent='Install this application, its WASM engine and selected published state maps on this browser. Local projects are saved separately. Browser storage may be cleared or evicted; keep project backups on disk.';section.append(text);
const selection=document.createElement('select');selection.id='offline-states';selection.multiple=true;selection.size=5;selection.setAttribute('aria-label','State and year data to install offline');section.append(selection);
const estimate=document.createElement('p');estimate.id='offline-estimate';section.append(estimate);
const install=document.createElement('button');install.type='button';install.id='offline-install';install.textContent='Install / replace offline package';install.disabled=true;section.append(install);
const remove=document.createElement('button');remove.type='button';remove.id='offline-remove';remove.textContent='Remove offline application data';remove.disabled=true;section.append(remove);
const status=document.createElement('p');status.id='offline-status';status.setAttribute('role','status');section.append(status);
const host=document.querySelector('.configuration')??document.querySelector('main');host?.append(section);
let manifest=null,registration=null,busy=false,installed=null;
const states=()=>[...selection.selectedOptions].map(o=>o.value),mib=n=>(n/1024/1024).toFixed(1)+' MiB';
function render(){
 selection.disabled=busy;install.disabled=busy||!manifest;remove.disabled=busy||!installed;
 try{const s=offlineSelection(manifest,states());estimate.textContent=mib(s.bytes)+' · application and '+s.states.length+' state/year packs. Published updates require explicit replacement.';}catch{estimate.textContent='';}
}
function request(worker,message){return new Promise((resolve,reject)=>{
 const channel=new MessageChannel();const timeout=setTimeout(()=>finish(new Error('Offline operation timed out. Check installed status before retrying.')),15*60*1000);
 function finish(error,value){clearTimeout(timeout);channel.port1.close();error?reject(error):resolve(value);}
 channel.port1.onmessage=({data})=>{if(data.type==='progress')status.textContent=`Installing ${data.completed}/${data.total} files · ${mib(data.bytes)} verified`;else if(data.type==='error')finish(new Error(data.error));else if(data.type==='complete')finish(null,data.installed);};
 worker.postMessage(message,[channel.port2]);
});}
async function existing(){const registrations=await navigator.serviceWorker.getRegistrations();return registrations.find(r=>r.scope===base.href&&(r.active??r.waiting??r.installing)?.scriptURL===new URL('offline-worker.js',base).href);}
function describe(){status.textContent=installed?`Installed ${mib(installed.bytes)} · ${installed.states.join(', ')||'application only'}. Version ${installed.version.slice(0,12)}. Reload before using the installed version offline.`:'No offline package installed.';}
selection.onchange=render;
install.onclick=async()=>{
 if(busy||!manifest)return;busy=true;render();try{
  // Installation is always explicit, including worker registration and downloads.
  registration=await navigator.serviceWorker.register(new URL('offline-worker.js',base),{scope:base.href,type:'module'});
  if(!registration.active)await new Promise((resolve,reject)=>{const worker=registration.installing??registration.waiting;if(!worker){reject(new Error('Offline worker unavailable.'));return;}const check=()=>{if(worker.state==='activated')resolve();else if(worker.state==='redundant')reject(new Error('Offline worker failed to activate.'));};worker.addEventListener('statechange',check);check();});
  installed=await request(registration.active,{type:'install',states:states()});describe();
 }catch(error){status.textContent=error.message+(installed?' Previous successfully installed package was kept.':' No offline package was activated.');}finally{busy=false;render();}
};
remove.onclick=async()=>{
 if(busy||!registration)return;busy=true;render();try{installed=await request(registration.active,{type:'remove'});await registration.unregister();registration=null;describe();status.textContent='Offline application data removed. Saved projects and provider caches were kept.';}catch(error){status.textContent=error.message;}finally{busy=false;render();}
};
try{
 if(!isSecureContext||!('serviceWorker'in navigator))throw new Error('Offline installation requires HTTPS or localhost and service-worker support.');
 const response=await fetch(new URL('offline-manifest.json',base),{cache:'no-store'});if(!response.ok)throw new Error('Offline package manifest is unavailable in this build.');
 const text=await offlineManifestText(response);manifest=validateOfflineManifest(JSON.parse(text));
 for(const scope of Object.keys(manifest.states).sort()){const option=document.createElement('option');option.value=scope;option.textContent=scope.replace(':',' · ');selection.append(option);}
 registration=await existing();if(registration?.active){installed=await request(registration.active,{type:'status'});for(const option of selection.options)option.selected=installed?.states?.includes(option.value);}
 describe();render();
}catch(error){status.textContent=error.message;}

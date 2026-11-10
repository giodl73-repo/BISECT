import path from 'node:path';
let sequence=0;
// Latch the exact Open completion before the normal autosave status replaces it.
// Watching starts before file selection; an already-visible old status cannot pass.
export async function openWorkbenchProject(page,filename,{timeout=30000,afterSelect,selectFile,ready,readyArg,polling}={}){
 const token='workbench-open-'+(++sequence),name=path.basename(filename);
 await page.evaluate(({token,name})=>{
  const element=document.querySelector('#project-status');if(!element)throw Error('Workbench project status is missing.');
  const state={opened:false,openedStatus:null,history:[],name,observer:null};
  state.observer=new MutationObserver(()=>{const text=element.textContent;state.history.push(text);if(text==='Opened '+name){state.opened=true;state.openedStatus=text;state.observer.disconnect();}});
  window.__workbenchProjectOpenWatches??={};window.__workbenchProjectOpenWatches[token]=state;
  state.observer.observe(element,{childList:true,characterData:true,subtree:true});
 },{token,name});
 try{
  if(selectFile)await selectFile(path.resolve(filename));else await page.locator('#open-project').setInputFiles(path.resolve(filename));
  await afterSelect?.();
  const deadline=timeout===0?Infinity:performance.now()+timeout;
  const remaining=()=>{if(timeout===0)return 0;const ms=Math.ceil(deadline-performance.now());if(ms>0)return ms;const error=new Error('Project Open observation deadline exceeded.');error.name='TimeoutError';throw error;};
  await page.waitForFunction(token=>window.__workbenchProjectOpenWatches?.[token]?.opened===true,token,{timeout:remaining(),...(polling===undefined?{}:{polling})});
  if(ready){const openedStatus=await page.evaluate(token=>window.__workbenchProjectOpenWatches[token].openedStatus,token);await page.waitForFunction(ready,{argument:readyArg,openedStatus},{timeout:remaining(),...(polling===undefined?{}:{polling})});}
  return await page.evaluate(token=>({opened:window.__workbenchProjectOpenWatches[token].opened,history:window.__workbenchProjectOpenWatches[token].history}),token);
 }catch(error){
  const diagnostic=await page.evaluate(token=>({status:document.querySelector('#project-status')?.textContent,history:window.__workbenchProjectOpenWatches?.[token]?.history}),token).catch(()=>null);
  error.message+='; project Open observation: '+JSON.stringify(diagnostic);throw error;
 }finally{
  await page.evaluate(token=>{const state=window.__workbenchProjectOpenWatches?.[token];state?.observer.disconnect();if(window.__workbenchProjectOpenWatches)delete window.__workbenchProjectOpenWatches[token];},token).catch(()=>{});
 }
}

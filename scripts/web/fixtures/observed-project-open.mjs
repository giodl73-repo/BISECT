// The exact Open transition is brief: autosave replaces it after 300 ms.
// Observe before the action; do not broaden success to an arbitrary autosave.
import assert from 'node:assert/strict';
import path from 'node:path';
export async function observedProjectOpen(page,file){
 const expected='Opened '+path.basename(file);
 await page.evaluate(expected=>{
  globalThis.reportImportObserver?.disconnect();globalThis.reportImportObservation={expected,opened:null,messages:[]};
  const node=document.querySelector('#project-status'),started=performance.now();
  globalThis.reportImportObserver=new MutationObserver(mutations=>{
   const record=globalThis.reportImportObservation;
   // Reading only current text loses intermediate added text nodes when the
   // browser batches multiple mutations before delivering this callback.
   const transitions=mutations.flatMap(m=>m.type==='childList'?Array.from(m.addedNodes,n=>n.textContent):[]);
   transitions.push(node.textContent);
   for(const text of transitions){if(record.messages.length<32)record.messages.push({text,elapsed_ms:performance.now()-started});if(text===expected)record.opened=text;}
  });
  reportImportObserver.observe(node,{childList:true,subtree:true,characterData:true});
 },expected);
 try{
  await page.locator('#open-project').setInputFiles(file);
  await page.waitForFunction(()=>reportImportObservation.opened||/changed during|Finish or cancel|Invalid|exceeds|cancelled/.test(document.querySelector('#project-status').textContent));
  const observation=await page.evaluate(()=>({...reportImportObservation,final_status:document.querySelector('#project-status').textContent}));
  assert.equal(observation.opened,expected,JSON.stringify(observation));return observation;
 }catch(error){
  const observation=await page.evaluate(()=>({...reportImportObservation,final_status:document.querySelector('#project-status').textContent})).catch(()=>null);
  error.import_observation=observation;const detail='\nImport observation: '+JSON.stringify(observation);error.message+=detail;error.stack=String(error.stack)+detail;throw error;
 }finally{await page.evaluate(()=>reportImportObserver?.disconnect()).catch(()=>{});}
}

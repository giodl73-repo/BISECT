import {createStoredWorkbenchArchive,parseWorkbenchArchiveBytes} from './workbench-archive.js';
self.onmessage=async({data})=>{try{
 if(data?.type==='save-workbench-archive'){
  const stored=await createStoredWorkbenchArchive(data.data);
  if(typeof stored==='string')self.postMessage({ok:true,text:stored});else self.postMessage({ok:true,bytes:stored},[stored.buffer]);
 }else{
  if(!data||!Number.isSafeInteger(data.size)||data.size<1||data.size>128*1024*1024)throw new Error('Select a workbench archive within 128 MiB.');
  const buffer=await data.arrayBuffer();if(buffer.byteLength!==data.size)throw new Error('Archive changed while reading.');
  self.postMessage({ok:true,project:await parseWorkbenchArchiveBytes(new Uint8Array(buffer))});
 }
}catch(error){self.postMessage({ok:false,error:error.message||'Invalid workbench archive.'});}};

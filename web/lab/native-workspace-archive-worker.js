import {decodeNativeWorkspaceArchive} from './native-workspace-source.js';
self.onmessage=async({data})=>{try{
 const file=data.file;if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>64*1024*1024)throw new Error('Select a native workspace ZIP up to 64 MiB.');
 const original=await file.arrayBuffer();if(original.byteLength!==file.size)throw new Error('Native workspace ZIP changed while reading.');
 const {original_bytes,...result}=await decodeNativeWorkspaceArchive(new Uint8Array(original)),snapshot=original_bytes.buffer;
 self.postMessage({ok:true,result,original:snapshot},[snapshot,...result.files.map(f=>f.bytes.buffer)]);
}catch(error){self.postMessage({ok:false,error:error.message});}};

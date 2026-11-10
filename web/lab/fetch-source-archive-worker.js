import {decodeFetchSourceArchive} from './fetch-source-archive.js';
self.onmessage=async({data})=>{try{
 const file=data.file;if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>64*1024*1024)throw new Error('Select a source ZIP up to 64 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Source ZIP changed while reading.');
 const decoded=await decodeFetchSourceArchive(bytes);self.postMessage({ok:true,result:{archive_sha256:decoded.archive_sha256,files:decoded.files.map(file=>({path:file.path,byte_length:file.bytes.length})),expanded_bytes:decoded.files.reduce((total,file)=>total+file.bytes.length,0)}});
}catch(error){self.postMessage({ok:false,error:error.message});}};

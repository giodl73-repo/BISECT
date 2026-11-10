import {decodePackageFiles,validatePackagePath} from './package-files.js';
const PATH='transcripts/verify-transcript.json',MAX=8*1024*1024,reviews=new WeakMap();
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const bytes=text=>new TextEncoder().encode(text);
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
function fields(value,required,optional=[]){
 if(!value||typeof value!=='object'||Array.isArray(value)||required.some(key=>!Object.hasOwn(value,key))||Object.keys(value).some(key=>!required.includes(key)&&!optional.includes(key)))throw new Error('Invalid native verification transcript.');
 const out={};for(const key of [...required,...optional])if(Object.hasOwn(value,key))out[key]=value[key];return out;
}
// Native struct declaration order, not serde_json::Value's sorted map order.
export function nativeCountTranscriptBytes(value){
 const out=fields(value,['transcript_version','verifier','status','package_content_hash','manifest_content_hash','checks']);
 if(out.transcript_version!=='rcount-audit-transcript-v1'||out.verifier!=='rcount-audit'||!['pass','fail'].includes(out.status)||!Array.isArray(out.checks)||out.checks.length>100000)throw new Error('Unsupported native verification transcript.');
 for(const key of ['package_content_hash','manifest_content_hash'])if(typeof out[key]!=='string')throw new Error('Invalid transcript identity.');
 out.checks=out.checks.map(value=>{const row=fields(value,['equation_id','status'],['contest_id','reporting_unit_id','error']);for(const item of Object.values(row))if(typeof item!=='string')throw new Error('Invalid transcript check.');if(!['pass','fail'].includes(row.status))throw new Error('Invalid transcript check status.');return row;});
 const encoded=bytes(JSON.stringify(out,null,2));if(encoded.length>MAX)throw new Error('Transcript exceeds 8 MiB.');return encoded;
}
function safePath(path){validatePackagePath(path);if(path.split('/').some(part=>part.endsWith('.')||part.endsWith(' ')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)))throw new Error('Unsafe selected package path.');}
async function permission(root,o){check(o);if(root?.kind!=='directory')throw new Error('Select the count package directory.');if(root.queryPermission&&await root.queryPermission({mode:'readwrite'})!=='granted')throw new Error('Package write permission is no longer granted.');check(o);}
async function observe(root,o){
 await permission(root,o);const files=Object.create(null),folded=new Set();let count=0,total=0;
 async function walk(dir,prefix='',depth=0){if(depth>16)throw new Error('Package directory exceeds depth limit.');for await(const [name,handle]of dir.entries()){
  check(o);const path=prefix+name;safePath(path);const key=path.toLowerCase();if(folded.has(key))throw new Error('Case-colliding package paths.');folded.add(key);if(++count>2016)throw new Error('Too many package entries.');
  if(path.toLowerCase()==='transcripts'&&path!=='transcripts')throw new Error('Case-colliding transcript directory.');
  if(handle.kind==='directory'){if(path===PATH)throw new Error('Transcript destination is a directory.');await walk(handle,path+'/',depth+1);}else if(handle.kind==='file'){
   const file=await handle.getFile();if(!Number.isSafeInteger(file.size)||file.size>MAX||(total+=file.size)>2*MAX)throw new Error('Selected package exceeds byte limits.');const data=new Uint8Array(await file.arrayBuffer());check(o);if(data.length!==file.size)throw new Error('Package file changed while reading.');files[path]={sha256:await hash(data),byte_length:data.length};
  }else throw new Error('Unsupported package entry.');
 }}await walk(root);check(o);return files;
}
function equal(a,b){return JSON.stringify(a)===JSON.stringify(b);}
function matches(actual,expected){const names=Object.keys(actual).filter(p=>p!==PATH).sort(),wanted=Object.keys(expected).filter(p=>p!==PATH).sort();if(!equal(names,wanted)||wanted.some(p=>!equal(actual[p],expected[p])))throw new Error('Selected package bytes or file roster differ from the verified inputs.');}
export async function reviewCountTranscriptInstallation(engine,result,packageFiles,root,o={}){
 const files=decodePackageFiles(packageFiles),fresh=await engine.execute({operation:'verify-count-files',files});check(o);
 const output=nativeCountTranscriptBytes(fresh);if(!equal(Array.from(output),Array.from(nativeCountTranscriptBytes(result))))throw new Error('Verification differs from a fresh native recomputation.');
 const expected=Object.create(null);for(const [path,data]of Object.entries(files)){safePath(path);expected[path]={sha256:await hash(new Uint8Array(data)),byte_length:data.length};}check(o);
 const observed=await observe(root,o);matches(observed,expected);
 const sourceIdentity=await hash(bytes(JSON.stringify(Object.keys(expected).filter(path=>path!==PATH).sort().map(path=>({path,...expected[path]})))));check(o);
 const review=Object.freeze({schema_version:'rcount-transcript-review-v1',root_name:root.name,path:PATH,action:observed[PATH]?'replace':'add',previous_sha256:observed[PATH]?.sha256??null,transcript_sha256:await hash(output),source_inventory_sha256:sourceIdentity,byte_length:output.length,status:fresh.status,package_content_hash:fresh.package_content_hash,manifest_content_hash:fresh.manifest_content_hash,authority:'Private current-session review only; verification is not certification.'});
 reviews.set(review,{root,expected,observed,output,used:false});return review;
}
export function retireCountTranscriptReview(review){if(review)reviews.delete(review);}
export async function applyCountTranscriptInstallation(review,{confirmed=false,...o}={}){
 const grant=reviews.get(review);if(!grant||grant.used)throw new Error('Review the selected count package again.');if(confirmed!==true)throw new Error('Confirm the reviewed transcript destination.');grant.used=true;
 const receipt={schema_version:'rcount-transcript-installation-v1',status:'incomplete',path:PATH,root_name:review.root_name,action:review.action,verification_status:review.status,package_content_hash:review.package_content_hash,manifest_content_hash:review.manifest_content_hash,source_inventory_sha256:review.source_inventory_sha256,transcript_sha256:review.transcript_sha256,previous_sha256:review.previous_sha256,write_started:false,readback_verified:false,authority:'Historical byte record only; no filesystem grant or election certification.'};
 try{
  const current=await observe(grant.root,o);matches(current,grant.expected);if(!equal(current[PATH]??null,grant.observed[PATH]??null))throw new Error('Transcript destination changed after review.');check(o);
  receipt.write_started=true;o.onProgress?.(structuredClone(receipt));check(o);
  const dir=await grant.root.getDirectoryHandle('transcripts',{create:true});check(o);const handle=await dir.getFileHandle('verify-transcript.json',{create:true});check(o);const writer=await handle.createWritable();
  try{check(o);await writer.write(grant.output);check(o);await writer.close();}catch(error){await writer.abort().catch(()=>{});throw error;}
  const after=await observe(grant.root,o);matches(after,grant.expected);if(after[PATH]?.sha256!==review.transcript_sha256||after[PATH]?.byte_length!==review.byte_length)throw new Error('Transcript read-back differs.');
  receipt.readback_verified=true;receipt.status='completed';return receipt;
 }catch(error){receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=error.message;error.installationReceipt=receipt;throw error;}finally{reviews.delete(review);}
}

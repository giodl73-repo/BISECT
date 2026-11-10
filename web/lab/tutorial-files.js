import {validateSourceFile,exactNativeJsonText,readNativeJsonSource} from './package-files.js';
export function tutorialPath(path){if(typeof path!=='string'||!path||new TextEncoder().encode(path).length>1024||/[\\:\0]/.test(path)||path.startsWith('/')||path.split('/').some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p)))throw new Error('Select safe relative tutorial file paths.');return path;}
export function validateTutorialInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==2||!Object.hasOwn(value,'tutorial')||!Object.hasOwn(value,'checksums')||typeof value.tutorial!=='string'||new TextEncoder().encode(value.tutorial).length>200)throw new Error('Invalid tutorial settings.');
 if(value.checksums!==null){validateSourceFile(value.checksums,{maxBytes:512*1024,allowEmpty:true});exactNativeJsonText(value.checksums,{allowEmpty:true});}return value;
}
export const readTutorialChecksums=file=>readNativeJsonSource(file,{maxBytes:512*1024});
export function selectTutorialFiles(selected,{folder=false,checksums=null}={}){
 const entries=Array.from(selected);const result=[],seen=new Set();
 // Folder selection keeps only paths referenced by the explicit checksum text.
 // JSON numbers here are never used for hashing, identity or saved metadata.
 let wanted=null;if(folder){if(!checksums)throw new Error('Select checksum JSON before choosing a folder.');const c=JSON.parse(exactNativeJsonText(checksums,{allowEmpty:true}));wanted=new Set([...(c.pinned_inputs||[]).map(i=>i.local_path),...(c.expected_outputs||[]).map(o=>o.path)].map(tutorialPath));}
 for(const file of entries){const raw=folder?file.webkitRelativePath:file.name;const path=folder?raw.slice(raw.indexOf('/')+1):raw;if(wanted&&!wanted.has(path))continue;tutorialPath(path);if(seen.has(path))throw new Error('Duplicate tutorial file path.');seen.add(path);if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid tutorial file size.');result.push({path,file});if(result.length>2000)throw new Error('Select at most 2,000 referenced tutorial files.');}return result;
}
export function tutorialRequest(inputs,files=[]){validateTutorialInputs(inputs);if(!inputs.tutorial.trim()||!inputs.checksums)throw new Error('Supply tutorial name and checksum JSON.');return {operation:'verify-tutorial-files',tutorial:inputs.tutorial,source_name:inputs.checksums.name,checksums_json:exactNativeJsonText(inputs.checksums,{allowEmpty:true}),files};}
export async function verifyTutorialFiles(engine,request){
 if(!request||Object.keys(request).some(k=>!['operation','tutorial','source_name','checksums_json','files'].includes(k))||request.operation!=='verify-tutorial-files'||!Array.isArray(request.files)||request.files.length>2000)throw new Error('Invalid tutorial worker request.');
 const selected=new Map();for(const entry of request.files){if(!entry||Object.keys(entry).length!==2||!Object.hasOwn(entry,'path')||!Object.hasOwn(entry,'file'))throw new Error('Invalid tutorial file entry.');tutorialPath(entry.path);if(selected.has(entry.path))throw new Error('Duplicate tutorial file path.');selected.set(entry.path,entry);}
 const inspect=files=>engine.execute({operation:'verify-tutorial-data',tutorial:request.tutorial,source_name:request.source_name,checksums_json:request.checksums_json,files});
 const preliminary=inspect({});
 if(!preliminary.rows){preliminary.digest_source='local-files-streamed-in-worker';return preliminary;}
 const files=Object.create(null);
 for(const path of new Set(preliminary.rows.map(row=>row.path))){const entry=selected.get(path);if(!entry)continue;
  try{files[entry.path]={sha256:await engine.hashFile(entry.file),read_error:null};}catch(error){files[entry.path]={sha256:null,read_error:String(error.message||error).slice(0,500)};}
 }
 const result=inspect(files);
 result.digest_source='local-files-streamed-in-worker';return result;
}
export function tutorialConsoleText(result){
 if(result?.schema_version!=='bisect-tutorial-checks-v1')throw Error('Run the tutorial check first.');
 const tutorial=result.tutorial;
 let text='=== bisect doctor --check-tutorial-data (WASM) ===\n\nTutorial:  '+tutorial+'\nChecksums: '+result.source_name+'\n\n';
 if(result.status==='invalid-checksums')return text+'[FAIL] checksums.json is not valid: '+result.error+'\n';
 if(result.status==='invalid-schema')return text+'[FAIL] '+result.error+" (expected 'tutorial-checksums v1')\n";
 if(result.tutorial_warning){const prefix="Checksum document declares tutorial='",suffix="' but requested tutorial='"+tutorial+"'";if(!result.tutorial_warning.startsWith(prefix)||!result.tutorial_warning.endsWith(suffix))throw Error('Unexpected tutorial warning format.');const declared=result.tutorial_warning.slice(prefix.length,-suffix.length);text+="[WARN] checksums.json declares tutorial='"+declared+"' but --tutorial='"+tutorial+"'\n";}
 if(result.pinned_build_commit)text+='Pinned at build commit: '+result.pinned_build_commit+'\n';
 if(result.pinned_at)text+='Pinned at:              '+result.pinned_at+'\n';
 text+='\n';
 for(const kind of ['input','output']){
  text+=kind==='input'?'-- Pinned inputs ---------------------------------------------\n':'\n-- Expected outputs ------------------------------------------\n';
  for(const row of result.rows.filter(r=>r.kind===kind)){
   let label=kind==='input'?row.name+' ('+row.path+')':row.path;
   if(kind==='output'&&row.expected_tract_count!==null)label+='  (expects '+row.expected_tract_count+' tracts)';
   if(row.status==='missing'){
    text+='[MISSING] '+label+'\n';
    if(kind==='input'&&row.source_url)text+='          fetch from: '+row.source_url+'\n';
    if(kind==='output')text+='          run the tutorial first: bash examples/'+tutorial+'-walkthrough/run.sh\n';
   }else if(row.status==='pass')text+='[PASS]    '+label+'\n';
   else{text+='[FAIL]    '+label+'\n';text+=row.status==='read-error'?'          read error: '+row.read_error+'\n':'          expected sha256: '+row.expected_sha256+'\n          got      sha256: '+row.actual_sha256+'\n';}
  }
 }
 text+='\nSummary: '+result.pass+' PASS, '+result.fail+' FAIL, '+result.missing+' MISSING\n\n';
 return text+(result.fail?'Data has drifted from tutorial baseline. Either:\n  (a) re-fetch with the pinned commands above\n  (b) re-run `bash examples/'+tutorial+'-walkthrough/run.sh`\n  (c) if the upstream legitimately changed, re-pin checksums.json\n':'[OK] No drift detected.\n');
}

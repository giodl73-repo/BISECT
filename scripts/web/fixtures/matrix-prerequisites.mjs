import fs from 'node:fs/promises';import path from 'node:path';
// Check every declared external prerequisite before starting any expensive work.
// Missing data is different from invalid bytes or a failed engine invocation.
export async function preflightMatrixPrerequisites(root,paths,rows=[]){
 const missing=[];
 for(const name of [...new Set(paths)]){try{const stat=await fs.stat(name);if(!stat.isFile())throw Error('Matrix prerequisite is not a file: '+name);}catch(error){if(error.code!=='ENOENT')throw error;missing.push(path.resolve(name));}}
 if(!missing.length)return true;
 await fs.mkdir(root,{recursive:true});const result={schema_version:'bisect-matrix-prerequisites-v1',status:'not-run',reason:'Missing declared integration prerequisites',missing,rows:rows.map(row=>({id:row.id,status:'not-run',attempted_execution:false})),execution_performed:false,full_parity:false};
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(result,null,2)+'\n');process.exitCode=2;console.log(JSON.stringify({root,status:'not-run',missing}));return false;
}

export async function preflightMatrixCapacity(root,requiredBytes,rows=[]){
 if(!Number.isSafeInteger(requiredBytes)||requiredBytes<1)throw Error('Invalid matrix evidence capacity requirement.');
 const stat=await fs.statfs(root,{bigint:true}),available=stat.bavail*stat.bsize;
 if(available>=BigInt(requiredBytes))return true;
 const result={schema_version:'bisect-matrix-prerequisites-v1',status:'not-run',reason:'Insufficient workspace capacity for owned validation evidence',required_bytes:requiredBytes,available_bytes:available.toString(),rows:rows.map(r=>({id:r.id,status:'not-run',attempted_execution:false})),execution_performed:false,scope:'Validation-host evidence storage prerequisite, not a browser product memory/storage minimum.',full_parity:false};
 let recorded=true;try{await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(result,null,2)+'\n');}catch{recorded=false;}
 process.exitCode=2;console.log(JSON.stringify({root,status:'not-run',reason:result.reason,evidence_recorded:recorded}));return false;
}

import {validateSourceFile,exactNativeJsonText,readNativeJsonSource} from './package-files.js';
export const MAX_MANIFEST_BYTES=1024*1024;
export function validateManifestSource(source){validateSourceFile(source,{maxBytes:MAX_MANIFEST_BYTES,allowEmpty:true});exactNativeJsonText(source,{allowEmpty:true});return source;}
export function manifestRequest(source){if(!source)throw new Error('Select a run manifest JSON.');validateManifestSource(source);return {operation:'verify-run-manifest',source_name:source.name,manifest_json:exactNativeJsonText(source,{allowEmpty:true})};}
export async function readManifestSource(file){
 return validateManifestSource(await readNativeJsonSource(file,{maxBytes:MAX_MANIFEST_BYTES}));
}

// Reproduce native Doctor's observations while naming the actual execution target.
// No native executable metadata or native process execution is inferred.
export function manifestConsoleText(result){
 if(result?.schema_version!=='bisect-manifest-compatibility-v1')throw Error('Run the manifest comparison first.');
 let out='=== bisect doctor --verify-manifest (WASM) ===\n\nManifest: '+result.source_name+'\n\n';
 if(result.status==='invalid-json')return out+'[FAIL] Manifest is not valid JSON: '+result.parse_error+'\n';
 const build=result.current_build,assessment=result.assessment,recorded=assessment.recorded;
 const date=new Date(Number(build.build_unix)*1000).toISOString().replace('.000Z','Z');
 out+='Running WASM module:\n  version:      '+build.version+'\n  build_commit: '+build.build_commit+'\n  bisect_build_date:   '+date+'\n  rustc_version:       '+build.rustc_version+'\n\n';
 out+='Manifest records:\n  binary_version:      '+(recorded.binary_version||'(missing)')+'\n';
 for(const [key,pad]of [['binary_sha256','       '],['adjacency_sha256','    ']])if(recorded[key]!==null)out+='  '+key+':'+pad+(recorded[key]||'(not recorded)')+'\n';
 if(recorded.created_at!==null)out+='  created_at:          '+recorded.created_at+'\n';
 out+='\n';
 if(assessment.version_status==='missing')out+='[WARN] manifest has no binary_version field — cannot verify provenance.\n';
 else if(assessment.version_status==='matches')out+='[PASS] binary_version matches running binary.\n';
 else out+='[FAIL] '+assessment.version_error+'\n';
 if(recorded.adjacency_file)out+='[INFO] Adjacency file referenced: '+recorded.adjacency_file+'\n       To verify integrity: sha256sum <path>/'+recorded.adjacency_file+' and compare to manifest.adjacency_sha256\n';
 if(build.build_commit.startsWith('unknown'))out+="[WARN] Running binary's build commit is 'unknown' — built outside a git checkout. Source provenance cannot be attested.\n";
 else if(build.build_commit.endsWith('-dirty'))out+="[WARN] Running binary was built from a dirty working tree (commit suffix '-dirty'). Source provenance is degraded.\n";
 return out+'\nResult: '+(assessment.hard_failures?'FAIL':'OK')+' ('+assessment.hard_failures+' hard failures)\n';
}

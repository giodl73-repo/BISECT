export function validateConfiguredRuntimeArtifacts(value){
 if(!value||Object.keys(value).sort().join()!=='export,run,schema_version'||value.schema_version!=='bisect-configured-runtime-v1')throw new Error('Invalid configured runtime identity.');
 for(const key of ['run','export']){const a=value[key];if(key==='export'&&a===null)continue;if(!a||Object.keys(a).sort().join()!=='byte_length,sha256'||!/^[a-f0-9]{64}$/.test(a.sha256||'')||!Number.isSafeInteger(a.byte_length)||a.byte_length<1)throw new Error('Invalid measured WASM artifact.');}
 return value;
}
export function configuredRuntimeText(recorded,loaded){
 if(!recorded)return 'No recorded configured run engine identity.';
 validateConfiguredRuntimeArtifacts(recorded);const comparison=!loaded?'no loaded engine identity':loaded.sha256===recorded.run.sha256&&loaded.byte_length===recorded.run.byte_length?'matches loaded engine':'differs from loaded engine';
 return `Recorded run WASM SHA-256: ${recorded.run.sha256} · ${recorded.run.byte_length} bytes · ${comparison}. Export WASM: ${recorded.export?.sha256??'not recorded'}. Imported identities are recorded claims; matching bytes do not authenticate generation.`;
}

import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
export function validatePartitionDiagnosticInputs(value){
 if(!value||typeof value!=='object'||Object.keys(value).sort().join()!=='max_lag,source'||!Number.isSafeInteger(value.max_lag)||value.max_lag<0||value.max_lag>100000)throw new Error('Supply an integer maximum lag from 0 through 100,000.');
 if(value.source!==null){validateSourceFile(value.source,{maxBytes:3*1024*1024});exactSourceText(value.source);}return value;
}
export async function readPartitionDiagnosticSource(file){if(!file||file.size>3*1024*1024)throw new Error('Select a partition trace JSON of at most 3 MiB.');const source=await readElectionSource(file);validatePartitionDiagnosticInputs({source,max_lag:0});return source;}
export function partitionDiagnosticRequest(value){validatePartitionDiagnosticInputs(value);if(!value.source)throw new Error('Select partition trace JSON first.');return{operation:'diagnose-partition-traces',source_json:exactSourceText(value.source),max_lag:value.max_lag};}

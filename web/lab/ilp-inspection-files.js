import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
export function validateIlpInspectionInputs(value){if(!value||typeof value!=='object'||Object.keys(value).join()!=='source')throw new Error('Invalid ILP inspection inputs.');if(value.source!==null){validateSourceFile(value.source);exactSourceText(value.source);}return value;}
export async function readIlpInspectionSource(file){const source=await readElectionSource(file);validateIlpInspectionInputs({source});return source;}
export function ilpInspectionRequest(value){validateIlpInspectionInputs(value);if(!value.source)throw new Error('Select an ILP engine request JSON first.');return{operation:'inspect-native-ilp-run',source_json:exactSourceText(value.source)};}

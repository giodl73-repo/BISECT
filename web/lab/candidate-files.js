import {validateSourceFile,readElectionSource} from './package-files.js';
import {readBlocAttestations} from './bloc-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const CANDIDATE_DEFAULTS={state:'AL',year:'2020',label:'',submitter:'',timestamp:''};
const bytes=s=>new TextEncoder().encode(s).length;
const validLabel=label=>typeof label==='string'&&/^[A-Za-z0-9_-]{1,100}$/.test(label)&&!['__proto__','constructor','prototype'].includes(label);
function resolvedLabel(settings){
 const label=settings.label||`candidate_race_${settings.year}_${settings.state.toLowerCase()}`;
 if(!validLabel(label))throw new Error('Use a package label with 1–100 ASCII letters, digits, underscores or hyphens.');
 return label;
}
export function validateCandidateInputs(v){
 if(!v||Array.isArray(v)||Object.keys(v).sort().join()!=='csv,documents,settings,staged_path'||typeof v.staged_path!=='string'||!v.documents||Array.isArray(v.documents)||Object.keys(v.documents).length>256)throw new Error('Invalid candidate-race inputs.');
 const s=v.settings;if(!s||Array.isArray(s)||Object.keys(s).sort().join()!=='label,state,submitter,timestamp,year'||typeof s.state!=='string'||!s.state||bytes(s.state)>100||typeof s.year!=='string'||!s.year||bytes(s.year)>20||(s.label!==''&&!validLabel(s.label))||typeof s.submitter!=='string'||bytes(s.submitter)>200||typeof s.timestamp!=='string'||bytes(s.timestamp)>100)throw new Error('Invalid candidate package settings.');
 const safe=p=>typeof p==='string'&&bytes(p)<=200&&/^[A-Za-z0-9_./ -]+$/.test(p)&&!p.split('/').some(x=>!x||['.','..','__proto__','constructor','prototype'].includes(x));
 if(v.csv!==null){validateSourceFile(v.csv,{maxBytes:1024*1024});if(!safe(v.csv.name)||v.csv.name.includes('/'))throw new Error('Annotation source needs a safe plain filename.');}
 let total=0;for(const [p,b]of Object.entries(v.documents)){if(!safe(p)||p===v.csv?.name)throw new Error('Invalid or colliding document path.');validateSourceFile({name:p,base64:b},{maxBytes:8*1024*1024});total+=atob(b).length;}if(total>8*1024*1024)throw new Error('Documents exceed 8 MiB.');if(v.staged_path&&!Object.hasOwn(v.documents,v.staged_path))throw new Error('Primary attestation is not selected.');return v;
}
export function candidateRequest(v){validateCandidateInputs(v);if(!v.csv||!v.staged_path||!v.settings.timestamp)throw new Error('Select annotation CSV, primary attestation and recorded timestamp.');return {operation:'ingest-candidate-race',csv_base64:v.csv.base64,source_name:v.csv.name,documents:v.documents,staged_path:v.staged_path,settings:{...v.settings,label:resolvedLabel(v.settings)}};}
export async function readCandidateCsv(f){if(!f||!Number.isSafeInteger(f.size)||f.size<1||f.size>1024*1024)throw new Error('Select nonempty annotation CSV up to 1 MiB.');const s=await readElectionSource(f);if(atob(s.base64).length!==f.size)throw new Error('Annotation file size changed while reading.');return s;}
export {readBlocAttestations as readCandidateDocuments};
export function candidateZip(result,inputs){
 const {settings:{label}}=candidateRequest(inputs);if(result?.schema_version!=='bisect-candidate-ingest-v1'||result.manifest?.label!==label||result.staged_path!==inputs.staged_path)throw new Error('Run candidate-race ingestion with these sources first.');const core=['annotations.json','provenance.json','manifest.json'];if(Object.keys(result.files||{}).sort().join()!==core.sort().join()||core.some(n=>typeof result.files[n]!=='string'))throw new Error('Invalid native candidate package files.');const prefix=`v1/civic_inputs/${label}/`,doc=inputs.staged_path.split('/').at(-1);if(core.includes(doc.toLowerCase())||doc.toLowerCase()==='candidate_race.csv')throw new Error('Attestation filename collides with generated package files.');const decode=b=>Uint8Array.from(atob(b),c=>c.charCodeAt(0));return storedZipBytes([[prefix+'candidate_race.csv',decode(inputs.csv.base64)],[prefix+doc,decode(inputs.documents[inputs.staged_path])],...core.map(n=>[prefix+n,new TextEncoder().encode(result.files[n])])]);
}

import {validateSourceFile,exactSourceText,encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const BLOC_DEFAULTS={state:'RI',year:2020,election:'presidential-primary',party:'DEM',minority_group:'black',bootstrap_seed:'42',bootstrap_samples:10000,ci_level:.95,alpha:.05,min_precincts:50};
export function validateBlocSettings(s){
 if(!s||typeof s!=='object'||Array.isArray(s)||Object.keys(s).length!==Object.keys(BLOC_DEFAULTS).length||!Object.keys(BLOC_DEFAULTS).every(k=>Object.hasOwn(s,k)))throw new Error('Invalid bloc-voting settings.');
 if(typeof s.state!=='string'||!/^[A-Z]{2}$/.test(s.state)||!Number.isInteger(s.year)||s.year<1900||s.year>2100)throw new Error('Supply state and year 1900–2100.');
 for(const key of ['election','party','minority_group'])if(typeof s[key]!=='string'||!s[key].trim()||new TextEncoder().encode(s[key]).length>200||/[\x00-\x1f\x7f]/.test(s[key]))throw new Error('Supply a nonempty run label of at most 200 bytes.');
 if(typeof s.bootstrap_seed!=='string'||!/^(0|[1-9][0-9]*)$/.test(s.bootstrap_seed)||s.bootstrap_seed.length>20||BigInt(s.bootstrap_seed)>18446744073709551615n)throw new Error('Supply a canonical unsigned 64-bit seed.');
 if(!Number.isInteger(s.bootstrap_samples)||s.bootstrap_samples<1||s.bootstrap_samples>10000||!Number.isInteger(s.min_precincts)||s.min_precincts<4||s.min_precincts>100000||![s.ci_level,s.alpha].every(x=>Number.isFinite(x)&&x>0&&x<1))throw new Error('Use 1–10,000 replicates, at least four precincts and confidence/alpha strictly between zero and one.');return s;
}
function validPath(name){return typeof name==='string'&&name.length<=240&&/^[A-Za-z0-9_./ -]+$/.test(name)&&!name.split('/').some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p));}
export function validateBlocInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==4||!['precincts','annotations','attestations','settings'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid bloc-voting inputs.');
 validateBlocSettings(value.settings);let bytes=0;for(const key of ['precincts','annotations'])if(value[key]!==null){validateSourceFile(value[key]);exactSourceText(value[key]);bytes+=atob(value[key].base64).length;}
 if(bytes>3*1024*1024)throw new Error('Precinct TSV and annotation CSV exceed 3 MiB.');
 if(value.annotations&&(!value.annotations.name||/[\/\\:\x00-\x1f\x7f]/.test(value.annotations.name)))throw new Error('Annotation source name must be a plain filename.');
 const docs=value.attestations;if(!docs||typeof docs!=='object'||Array.isArray(docs)||Object.keys(docs).length>256)throw new Error('Select at most 256 attestation files.');bytes=0;
 if(value.annotations&&Object.hasOwn(docs,value.annotations.name))throw new Error('Annotation CSV filename collides with an attestation path.');
 for(const [name,base64]of Object.entries(docs)){if(!validPath(name))throw new Error('Invalid relative attestation path.');validateSourceFile({name:name.slice(-200),base64});bytes+=atob(base64).length;}
 if(bytes>8*1024*1024)throw new Error('Attestation files exceed 8 MiB.');return value;
}
export function blocRequest(inputs){validateBlocInputs(inputs);if(!inputs.precincts||!inputs.annotations||!Object.keys(inputs.attestations).length)throw new Error('Select precinct TSV, candidate annotation CSV and attestation files.');return {operation:'analyze-bloc-voting',precincts_tsv:exactSourceText(inputs.precincts),annotation_csv:exactSourceText(inputs.annotations),annotation_source_name:inputs.annotations.name,attestation_files:inputs.attestations,settings:inputs.settings};}
export async function readBlocAttestations(selected){
 const files=Array.from(selected);if(files.length>256)throw new Error('Select at most 256 attestation files.');let total=0;const paths=files.map(f=>{let p=f.webkitRelativePath||f.name;if(f.webkitRelativePath)p=p.slice(p.indexOf('/')+1);return p;});
 if(new Set(paths).size!==paths.length||paths.some(p=>!validPath(p)))throw new Error('Duplicate or invalid relative attestation paths.');
 for(const f of files){if(!Number.isSafeInteger(f.size)||f.size<0)throw new Error('Invalid attestation file size.');total+=f.size;if(total>8*1024*1024)throw new Error('Attestation files exceed 8 MiB.');}
 const out=Object.create(null);total=0;for(let i=0;i<files.length;i++){const b=new Uint8Array(await files[i].arrayBuffer());if(b.length!==files[i].size)throw new Error('Attestation size changed while reading.');total+=b.length;if(total>8*1024*1024)throw new Error('Attestation files exceed 8 MiB.');out[paths[i]]=encodeBytes(b);}return out;
}
export function blocEvidenceZip(result,inputs){
 blocRequest(inputs);
 validateBlocInputs(inputs);if(result?.schema_version!=='bisect-bloc-voting-browser-v1'||typeof result.files?.['bloc_voting.json']!=='string'||typeof result.files?.['bloc_voting_summary.md']!=='string')throw new Error('Run bloc analysis before exporting evidence.');
 const bytes=b64=>Uint8Array.from(atob(b64),c=>c.charCodeAt(0));const entries=[['inputs/precincts.tsv',bytes(inputs.precincts.base64)],['inputs/annotations/'+inputs.annotations.name,bytes(inputs.annotations.base64)],...Object.entries(inputs.attestations).map(([name,b64])=>['inputs/annotations/'+name,bytes(b64)]),...Object.entries(result.files).map(([name,text])=>['reports/'+name,new TextEncoder().encode(text)]),['run-record.json',new TextEncoder().encode(JSON.stringify(result,null,2))]];return storedZipBytes(entries);
}

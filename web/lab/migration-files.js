import {validateSourceFile,encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const MIGRATION_DEFAULTS={label:'legacy_plan',state_identifier:'WA',source_dir_name:'washington',created_at:''};
const safe=p=>typeof p==='string'&&p.length>0&&p.length<=200&&/^[A-Za-z0-9_./ -]+$/.test(p)&&!p.split('/').some(x=>!x||['.','..','__proto__','constructor','prototype'].includes(x));
export function validateMigrationInputs(v){
 if(!v||Object.keys(v).sort().join()!=='files,settings'||!v.files||Array.isArray(v.files)||Object.keys(v.files).length>256)throw new Error('Select up to 256 migration files.');
 const s=v.settings;if(!s||Object.keys(s).sort().join()!=='created_at,label,source_dir_name,state_identifier'||typeof s.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(s.label)||['__proto__','constructor','prototype'].includes(s.label)||!safe(s.state_identifier)||s.state_identifier.includes('/')||!safe(s.source_dir_name)||s.source_dir_name.includes('/')||typeof s.created_at!=='string'||new TextEncoder().encode(s.created_at).length>100)throw new Error('Invalid migration settings.');
 let total=0;const folded=new Set();for(const [p,b]of Object.entries(v.files)){if(!safe(p)||p.length+s.label.length+7>240||folded.has(p.toLowerCase()))throw new Error('Unsafe or colliding migration paths.');folded.add(p.toLowerCase());if(b!=='')validateSourceFile({name:p,base64:b},{maxBytes:16*1024*1024});total+=atob(b).length;}if(total>16*1024*1024)throw new Error('Migration files exceed 16 MiB.');return v;
}
export function migrationRequest(v){validateMigrationInputs(v);if(!v.settings.created_at)throw new Error('Supply a recorded migration timestamp.');return {operation:'migrate-legacy-plan',files_base64:v.files,settings:v.settings};}
export async function readMigrationFolder(selected){
 const list=Array.from(selected);if(!list.length||list.length>256)throw new Error('Select a legacy folder containing 1–256 files.');let total=0,root;const paths=[];
 for(const f of list){const parts=(f.webkitRelativePath||'').split('/');if(parts.length<2||!safe(parts[0]))throw new Error('Select a folder with safe relative paths.');const r=parts.shift();if(root&&root!==r)throw new Error('Select one legacy state folder.');root=r;const p=parts.join('/');if(!safe(p)||paths.some(x=>x.toLowerCase()===p.toLowerCase())||!Number.isSafeInteger(f.size)||f.size<0)throw new Error('Invalid migration path or file size.');paths.push(p);total+=f.size;if(total>16*1024*1024)throw new Error('Migration files exceed 16 MiB.');}
 const files={};for(let i=0;i<list.length;i++){const bytes=new Uint8Array(await list[i].arrayBuffer());if(bytes.length!==list[i].size)throw new Error('Migration file size changed while reading.');files[paths[i]]=encodeBytes(bytes);}return {files,source_dir_name:root};
}
export function migrationZip(result,inputs){
 migrationRequest(inputs);if(result?.schema_version!=='bisect-legacy-migration-v1'||result.manifest?.label!==inputs.settings.label||typeof result.manifest_json!=='string')throw new Error('Run migration with these sources first.');const prefix=`plans/${inputs.settings.label}/`,decode=b=>Uint8Array.from(atob(b),c=>c.charCodeAt(0));const entries=Object.entries(inputs.files).filter(([p])=>!['manifest.json','manifest.tmp'].includes(p)).map(([p,b])=>[prefix+p,decode(b)]);entries.push([prefix+'manifest.json',new TextEncoder().encode(result.manifest_json)]);for(const p of ['analysis/','maps/','intermediate/'])if(!entries.some(([n])=>n.startsWith(prefix+p)))entries.push([prefix+p,new Uint8Array()]);return storedZipBytes(entries);
}

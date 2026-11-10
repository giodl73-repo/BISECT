import {encodeBytes,isBase64Encoding,exactCeremonyText} from './package-files.js';
export const CEREMONY_SOURCE_LIMIT=3*1024*1024;
export const CEREMONY_START_DEFAULTS=Object.freeze({rule_profile:null,settings:Object.freeze({genesis_unix_seconds:'',review_seconds:'10',minimum_witnesses:2,witness_receipts:Object.freeze([])})});
const utf8=value=>new TextEncoder().encode(value),sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
function shape(value,keys,label){if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(key=>!Object.hasOwn(value,key)))throw Error('Invalid '+label+'.');}
function exactInteger(value,signed,label,complete){
 if(value===''&&!complete)return value;
 if(typeof value!=='string'||!new RegExp(signed?'^(0|-?[1-9][0-9]{0,18})$':'^(0|[1-9][0-9]{0,19})$').test(value))throw Error(label+' must be canonical exact integer text.');
 const n=BigInt(value);if(n<(signed?-(1n<<63n):0n)||n>(signed?((1n<<63n)-1n):((1n<<64n)-1n)))throw Error(label+' exceeds its native 64-bit integer range.');return value;
}
export function ceremonyProfileBytes(source){
 shape(source,['name','base64'],'original rule-profile source');
 if(typeof source.name!=='string'||source.name.length>200||typeof source.base64!=='string'||source.base64.length>4*Math.ceil(CEREMONY_SOURCE_LIMIT/3)||!isBase64Encoding(source.base64))throw Error('Invalid rule-profile source or 3 MiB limit exceeded.');
 const raw=atob(source.base64);if(raw.length>CEREMONY_SOURCE_LIMIT||btoa(raw)!==source.base64)throw Error('Rule-profile encoding is not canonical or exceeds 3 MiB.');
 // Empty profile files are valid native hash inputs. Generic election-source
 // validators intentionally retain their nonempty-source requirement.
 return Uint8Array.from(raw,c=>c.charCodeAt(0));
}
export function validateCeremonyStartInputs(value,{complete=false}={}){
 shape(value,['rule_profile','settings'],'ceremony source inputs');shape(value.settings,['genesis_unix_seconds','review_seconds','minimum_witnesses','witness_receipts'],'ceremony start settings');
 if(value.rule_profile!==null)ceremonyProfileBytes(value.rule_profile);else if(complete)throw Error('Select the original rule-profile file.');
 const s=value.settings;exactInteger(s.genesis_unix_seconds,true,'Genesis timestamp',complete);exactInteger(s.review_seconds,false,'Review interval',complete);
 if(!Number.isSafeInteger(s.minimum_witnesses)||Object.is(s.minimum_witnesses,-0)||s.minimum_witnesses<0||s.minimum_witnesses>100||!Array.isArray(s.witness_receipts)||s.witness_receipts.length>100||s.witness_receipts.some(v=>typeof v!=='string'||utf8(v).length>2000))throw Error('Ceremony witness settings exceed the bounded native browser contract.');
 return value;
}
export function ceremonyStartSourceRequest(treeSource,inputs){
 validateCeremonyStartInputs(inputs,{complete:true});
 if(!treeSource)throw Error('Select the certified complete tree.');
 return {operation:'start-ceremony-from-sources',input:{tree_json:exactCeremonyText(treeSource),rule_profile:{...inputs.rule_profile},settings:{...inputs.settings,witness_receipts:[...inputs.settings.witness_receipts]}}};
}
export async function startCeremonyFromSources(engine,input){
 shape(input,['tree_json','rule_profile','settings'],'ceremony source request');
 validateCeremonyStartInputs({rule_profile:input.rule_profile,settings:input.settings},{complete:true});
 const tree_json=input.tree_json,profile=ceremonyProfileBytes(input.rule_profile),settings={...input.settings,witness_receipts:[...input.settings.witness_receipts]};
 if(typeof tree_json!=='string'||utf8(tree_json).length>CEREMONY_SOURCE_LIMIT)throw Error('Ceremony tree exceeds the 3 MiB text limit.');
 if(tree_json.charCodeAt(0)===0xfeff)throw Error('Ceremony tree has a UTF-8 BOM. Native CLI JSON requires the original file without a BOM.');
 // These are candidate fields only. Native Start validates the entire original
 // tree and its root/count binding before any accepted result is returned.
 // Never reserialize the tree or read integer populations through JavaScript.
 const tree=JSON.parse(tree_json),root=tree?.nodes?.[0]?.certificate?.instance_hash;
 if(!Number.isSafeInteger(tree?.k)||tree.k<2||tree.k>24||typeof root!=='string'||!/^sha256:[0-9a-f]{64}$/.test(root))throw Error('Tree root/count unavailable.');
 const [profileHash,treeHash]=await Promise.all([sha(profile),sha(utf8(tree_json))]);
 const config_json='{"rule_profile_hash":'+JSON.stringify('sha256:'+profileHash)+',"root_instance_hash":'+JSON.stringify(root)+',"district_count":'+tree.k+',"genesis_unix_seconds":'+settings.genesis_unix_seconds+',"minimum_review_seconds":'+settings.review_seconds+',"minimum_witness_receipts":'+settings.minimum_witnesses+',"genesis_witness_receipts":'+JSON.stringify(settings.witness_receipts)+'}';
 const result=engine.execute({operation:'start-ceremony',tree_json,config_json});
 return {...result,source_preparation:{schema_version:'bisect-ceremony-start-sources-v1',config_json,rule_profile_sha256:profileHash,rule_profile_bytes:profile.length,tree_sha256:treeHash,...settings,scope:'Original supplied bytes hashed locally; native bounded tree/config validation and Start. No publication, timestamp/witness authentication or legal certification.'}};
}
export async function readCeremonyRuleProfile(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>CEREMONY_SOURCE_LIMIT)throw Error('Select an original rule-profile file of at most 3 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw Error('Rule-profile size changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};ceremonyProfileBytes(source);return source;
}

import {readJsonFile,validateJsonTree} from './project.js';
import {characterStateFips} from './character-input.js';
import {nativeCharacterRequest} from './native-character-files.js';
export const NATIVE_CHARACTER_WEIGHT_DEFAULTS={target:null,alpha:.5,allow_absent_fallback:false};
export function validateNativeCharacterWeightInputs(input){
 validateJsonTree(input);
 if(!input||Object.keys(input).sort().join()!=='allow_absent_fallback,alpha,target'||!Number.isFinite(input.alpha)||Object.is(input.alpha,-0)||input.alpha<0||input.alpha>1||typeof input.allow_absent_fallback!=='boolean')throw new Error('Choose native blend alpha in [0,1] and explicit fallback permission.');
 const t=input.target;if(t!==null){
  validateNativeCharacterTarget(t);
  const seen=new Set();for(const e of t.edges){if(!Array.isArray(e)||e.length!==3||!Number.isSafeInteger(e[0])||!Number.isSafeInteger(e[1])||e[0]<0||e[0]>=e[1]||e[1]>=t.geoids.length||!Number.isFinite(e[2])||e[2]<=0||seen.has(e[0]+':'+e[1]))throw new Error('Target boundaries must be unique, canonical and positive.');seen.add(e[0]+':'+e[1]);}
 }
 return input;
}
export function validateNativeCharacterTarget(t){
 const keys=['state','year','geoids','edges','unit_kind','county_ids'],kind=t?.unit_kind??'tract',custom=['precinct','imported'].includes(kind),fips=characterStateFips(t?.state);
 if(!t||Object.keys(t).some(k=>!keys.includes(k))||!fips||typeof t.year!=='string'||!/^\d{4}$/.test(t.year)||!['county','tract','block-group','block','precinct','imported'].includes(kind)||!Array.isArray(t.geoids)||!t.geoids.length||t.geoids.length>100000||new Set(t.geoids).size!==t.geoids.length||!Array.isArray(t.edges)||t.edges.length>1000000)throw new Error('Select a scoped target with a supported unit kind, unique IDs and boundary edges.');
 if(custom){
  const valid=id=>typeof id==='string'&&id.length>0&&new TextEncoder().encode(id).length<=200&&!/[\u0000-\u001f\u007f-\u009f]/.test(id)&&!['__proto__','constructor','prototype'].includes(id);
  if(!Array.isArray(t.county_ids)||t.county_ids.length!==t.geoids.length||!t.geoids.every(valid)||!t.county_ids.every(valid))throw new Error('Custom targets require safe exact unit IDs and one explicit county ID per unit.');
 }else{
  const width={county:5,tract:11,'block-group':12,block:15}[kind];
  if(t.county_ids!=null||!['2000','2010','2020'].includes(t.year)||t.geoids.some(id=>typeof id!=='string'||id.length!==width||!/^\d+$/.test(id)||!id.startsWith(fips)))throw new Error('Census target IDs must match their declared kind, selected state and supported Census year.');
 }
 return t;
}
export async function readNativeCharacterWeightTarget(file){const value=await readJsonFile(file);const target={state:value.state,year:value.year,...(value.unit_kind!=null?{unit_kind:value.unit_kind}:{}),...(value.county_ids!=null?{county_ids:value.county_ids}:{}),geoids:value.geoids,edges:value.edges};validateNativeCharacterWeightInputs({...NATIVE_CHARACTER_WEIGHT_DEFAULTS,target});return target;}
export function nativeCharacterWeightRequest(source,inputs){
 validateNativeCharacterWeightInputs(inputs);if(!inputs.target)throw new Error('Select target prepared graph JSON before evaluating native weights.');
 const request=nativeCharacterRequest(source);if(inputs.target.state!==source.settings.state||inputs.target.year!==source.settings.year)throw new Error('Native source and target state/unit year must match.');
 return {operation:'evaluate-native-character-weights',input:{source:request.input,...inputs}};
}

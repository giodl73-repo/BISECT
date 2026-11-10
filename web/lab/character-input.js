import {countyPreLength} from './county-augmentation.js';
import {nativeCharacterLabRequest} from './native-character-lab.js';
import {validateJsonTree,readJsonFile,readValidatedFile} from './project.js';
export const usesCharacterWeights=weights=>['economic-character','housing-character'].includes(weights);
const exact=(value,keys)=>value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===keys.length&&keys.every(key=>Object.hasOwn(value,key));
const number=(value,max=1)=>Number.isFinite(value)&&value>=0&&value<=max&&!Object.is(value,-0);
const fields={economic:['commercial_intensity','industrial_fraction','jobs_per_resident'],housing:['pct_single_family','pct_multifamily','pct_owner','housing_vintage']};
const rawFields=['c000','cns01','cns02','cns05','cns07','cns08','cns09','cns10','cns11'];
const states='AL01 AK02 AZ04 AR05 CA06 CO08 CT09 DE10 DC11 FL12 GA13 HI15 ID16 IL17 IN18 IA19 KS20 KY21 LA22 ME23 MD24 MA25 MI26 MN27 MS28 MO29 MT30 NE31 NV32 NH33 NJ34 NM35 NY36 NC37 ND38 OH39 OK40 OR41 PA42 RI44 SC45 SD46 TN47 TX48 UT49 VT50 VA51 WA53 WV54 WI55 WY56';
export const characterStateFips=code=>states.split(' ').find(state=>state.slice(0,2)===code)?.slice(2);
export function validateCharacterInput(input,graph,kind){
  validateJsonTree(input);
  if(!exact(input,['schema_version','state','year','data_year','source_label','data',...(Object.hasOwn(input,'alignment')?['alignment']:[])])||input.schema_version!=='bisect-character-v1'||typeof input.state!=='string'||!['2000','2010','2020'].includes(input.year)||typeof input.data_year!=='string'||!/^\d{4}$/.test(input.data_year)||typeof input.source_label!=='string'||!input.source_label.trim()||new TextEncoder().encode(input.source_label).length>200)throw new Error('Invalid character schema, scope, observation year or source label.');
  const fips=states.split(' ').find(state=>state.slice(0,2)===input.state)?.slice(2),data=input.data;
  if(!fips||!data||!['economic','housing'].includes(data.kind)||kind&&data.kind!==kind||!exact(data,data.kind==='economic'?['kind','raw_counts','characters']:['kind','characters']))throw new Error('Character input kind must match the selected weights.');
  const values=data.characters;
  if(!values||typeof values!=='object'||Array.isArray(values)||!Object.keys(values).length||Object.keys(values).length>100000)throw new Error('Character input requires 1–100,000 tract observations.');
  for(const [id,value]of Object.entries(values)){
    if(!/^\d{11}$/.test(id)||!id.startsWith(fips)||!exact(value,fields[data.kind])||fields[data.kind].some(field=>!number(value[field])))throw new Error('Invalid tract character fractions or state GEOIDs.');
  }
  if(data.kind==='economic'){
    const raw=data.raw_counts;
    if(!raw||typeof raw!=='object'||Array.isArray(raw)||Object.keys(raw).length!==Object.keys(values).length)throw new Error('Economic input requires complete retained raw counts.');
    for(const [id,value]of Object.entries(values)){
      const row=raw[id];if(!exact(row,rawFields)||rawFields.some(field=>!number(row[field],Number.MAX_SAFE_INTEGER)))throw new Error('Invalid economic raw counts or coverage.');
      const expected=row.c000<1e-10?[0,0,0]:[Math.min(1,(row.cns07+row.cns09+row.cns10+row.cns11)/row.c000),Math.min(1,(row.cns01+row.cns02+row.cns05+row.cns08)/row.c000),Math.min(1,row.c000/10000)];
      if(fields.economic.some((field,i)=>value[field]!==expected[i]))throw new Error('Economic raw counts and derived character disagree.');
    }
  }
  if(graph&&(input.state!==graph.state||input.year!==graph.year||Object.keys(values).length!==graph.geoids.length||graph.geoids.some(id=>!Object.hasOwn(values,id))))throw new Error('Character input must match the graph state, Census year and complete tract coverage.');
  if(Object.hasOwn(input,'alignment')){
    const a=input.alignment;if(!exact(a,['policy','source_data','matched_tracts','missing_tract_ids','ignored_tract_ids'])||a.policy!=='explicit-native-nonempty-source-alignment-v1')throw new Error('Invalid character alignment receipt.');
    const original={...input,data:a.source_data};delete original.alignment;validateCharacterInput(original,null,data.kind);
    const expected=alignCharacterInput(original,{state:input.state,year:input.year,geoids:Object.keys(values)});
    if(!sameTree(expected.data,input.data)||!sameTree(expected.alignment,a))throw new Error('Character alignment defaults or receipt disagree with original observations.');
  }
  return input;
}
function sameTree(a,b){if(a===b)return true;if(!a||!b||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;const keys=Object.keys(a);return keys.length===Object.keys(b).length&&keys.every(k=>Object.hasOwn(b,k)&&sameTree(a[k],b[k]));}
export function alignCharacterInput(input,graph){
  validateCharacterInput(input);if(input.alignment)throw new Error('Align original character observations, not an already aligned input.');
  if(!graph||input.state!==graph.state||input.year!==graph.year||!Array.isArray(graph.geoids)||!graph.geoids.length||graph.geoids.length>100000||new Set(graph.geoids).size!==graph.geoids.length||graph.geoids.some(id=>!/^\d{11}$/.test(id)||!id.startsWith(characterStateFips(input.state))))throw new Error('Character alignment requires matching canonical state tract GEOIDs.');
  const source=input.data,kind=source.kind,target=new Set(graph.geoids),missing=graph.geoids.filter(id=>!Object.hasOwn(source.characters,id)).sort(),ignored=Object.keys(source.characters).filter(id=>!target.has(id)).sort();
  const zero=Object.fromEntries(fields[kind].map(f=>[f,kind==='housing'?.5:0])),characters=Object.fromEntries(graph.geoids.map(id=>[id,structuredClone(source.characters[id]??zero)]));
  const data={kind,...(kind==='economic'?{raw_counts:Object.fromEntries(graph.geoids.map(id=>[id,structuredClone(source.raw_counts[id]??Object.fromEntries(rawFields.map(f=>[f,0])))]))}:{}),characters};
  return {...structuredClone(input),data,alignment:{policy:'explicit-native-nonempty-source-alignment-v1',source_data:structuredClone(source),matched_tracts:graph.geoids.length-missing.length,missing_tract_ids:missing,ignored_tract_ids:ignored}};
}
export async function readCharacterFile(file,{csvMetadata,wasmSha256,signal}={}){
  if(/\.csv$/i.test(file?.name||'')){
    if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select a nonempty character CSV of at most 8 MiB.');
    if(!csvMetadata||! /^[a-f0-9]{64}$/.test(wasmSha256||''))throw new Error('Character CSV import needs explicit scope and a verified browser engine.');
    return validateCharacterInput(await readValidatedFile({size:file.size,file,metadata:csvMetadata,sha256:wasmSha256},new URL('./character-csv-worker.js',import.meta.url),{signal}),null,csvMetadata.kind);
  }
  return validateCharacterInput(await readJsonFile(file,{signal}));
}
export function characterWeights(graph,config){
  if(config.character_absent_file_fallback===true&&!Object.hasOwn(config.characters??{},graph.state))return {input:null,kind:config.weights.replace('-character',''),alpha:config.character_alpha,absent:true,weight:(u,v,length)=>countyPreLength(graph,config,u,v,length)};
  const input=validateCharacterInput(config.characters[graph.state],graph,config.weights.replace('-character','')),kind=input.data.kind,alpha=config.character_alpha;
  if(!number(alpha))throw new Error('Character blend alpha must be finite in [0,1].');
  const vectors=graph.geoids.map(id=>fields[kind].map(field=>input.data.characters[id][field]));
  return {input,kind,alpha,weight(u,v,length){
    const a=vectors[u],b=vectors[v],dot=a.reduce((sum,value,i)=>sum+value*b[i],0),ma=Math.sqrt(a.reduce((sum,value)=>sum+value*value,0)),mb=Math.sqrt(b.reduce((sum,value)=>sum+value*value,0));
    const similarity=ma<1e-15||mb<1e-15?(kind==='housing'||ma<1e-15&&mb<1e-15?1:.5):Math.min(1,Math.max(0,dot/(ma*mb)));
    return countyPreLength(graph,config,u,v,length)*(alpha+(1-alpha)*similarity);
  }};
}
export async function characterIdentity(input){
  validateCharacterInput(input);
  const kind=input.data.kind,header=new TextEncoder().encode(`BISECT_CHARACTER_V1\0${input.state}\0${input.year}\0${input.data_year}\0${kind}\0`),ids=Object.keys(input.data.characters).sort(),size=11+fields[kind].length*8;
  const bytes=new Uint8Array(header.length+ids.length*size);bytes.set(header);const view=new DataView(bytes.buffer);let offset=header.length;
  for(const id of ids){bytes.set(new TextEncoder().encode(id),offset);offset+=11;for(const field of fields[kind]){view.setFloat64(offset,input.data.characters[id][field],true);offset+=8;}}
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
}

export function characterRequest(config,graph){
 if(!usesCharacterWeights(config.weights))return {};
 const native=nativeCharacterLabRequest(config,graph);if(native)return {native_character_source:native};
 if(config.character_absent_file_fallback===true&&!Object.hasOwn(config.characters??{},graph.state))return {};
 return {character:validateCharacterInput(config.characters?.[graph.state],graph,config.weights.replace('-character',''))};
}
export function characterAbsenceEvidence(config,units,scope='edge-weighting'){return {method:'native-character-absent-geographic-v1',kind:config.weights.replace('-character',''),character_sha256:null,data_year:null,source_policy:'explicit-absent-input-geographic',baseline:'geographic-boundary-m',tracts:units,scope};}

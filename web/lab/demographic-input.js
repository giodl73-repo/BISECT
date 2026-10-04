import {validateJsonTree,readJsonFile,readValidatedFile} from './project.js';

export const DEMOGRAPHIC_SCHEMA='bisect-demographic-fractions-v1';
const keys=['schema_version','state','year','basis','source_label','minority_fractions'];
export const DEMOGRAPHIC_COUNTS_SCHEMA='bisect-demographic-counts-v2';
const bases=['total-population','voting-age-population','citizen-voting-age-population'];
export function validateDemographicInput(input,graph){
  validateJsonTree(input);
  const counts=input?.schema_version===DEMOGRAPHIC_COUNTS_SCHEMA,fields=counts?[...keys,'counts']:keys;
  if(!input||typeof input!=='object'||Array.isArray(input)||Object.keys(input).length!==fields.length||fields.some(key=>!Object.hasOwn(input,key))||![DEMOGRAPHIC_SCHEMA,DEMOGRAPHIC_COUNTS_SCHEMA].includes(input.schema_version)||typeof input.state!=='string'||! /^[A-Z]{2}$/.test(input.state)||typeof input.year!=='string'||! /^\d{4}$/.test(input.year)||!bases.includes(input.basis)||typeof input.source_label!=='string'||!input.source_label.length||new TextEncoder().encode(input.source_label).length>200)throw new Error('Invalid demographic schema, scope, basis or source label.');
  const values=input.minority_fractions;
  if(!values||typeof values!=='object'||Array.isArray(values)||!Object.keys(values).length||Object.entries(values).some(([id,value])=>! /^\d{11}$/.test(id)||!Number.isFinite(value)||value<0||value>1||Object.is(value,-0)))throw new Error('Supply finite minority fractions from zero through one, keyed by complete GEOIDs.');
  if(counts){
    if(!input.counts||typeof input.counts!=='object'||Array.isArray(input.counts)||Object.keys(input.counts).length!==Object.keys(values).length)throw new Error('Demographic count coverage mismatch.');
    for(const [id,fraction]of Object.entries(values)){
      const c=input.counts[id];
      if(!c||typeof c!=='object'||Array.isArray(c)||Object.keys(c).length!==2||!Object.hasOwn(c,'total')||!Object.hasOwn(c,'minority')||!Number.isFinite(c.total)||!Number.isFinite(c.minority)||c.total<0||Object.is(c.total,-0)||c.total>Number.MAX_SAFE_INTEGER||c.minority<0||Object.is(c.minority,-0)||c.minority>c.total||fraction!==(c.total===0?0:c.minority/c.total))throw new Error('Demographic counts and fractions disagree.');
    }
  }
  if(graph&&(input.state!==graph.state||input.year!==graph.year||Object.keys(values).length!==graph.geoids.length||graph.geoids.some(id=>!Object.hasOwn(values,id))))throw new Error('Demographic data must match the graph state, year and complete tract coverage.');
  return input;
}
export async function readDemographicFile(file,{csvMetadata,wasmSha256,signal}={}){
  if(/\.csv$/i.test(file?.name||'')){
    if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select a nonempty demographic CSV of at most 8 MiB.');
    if(!csvMetadata||! /^[a-f0-9]{64}$/.test(wasmSha256||''))throw new Error('CSV import needs explicit scope, basis and a verified browser engine.');
    return validateDemographicInput(await readValidatedFile({size:file.size,file,metadata:csvMetadata,sha256:wasmSha256},new URL('./demographic-csv-worker.js',import.meta.url),{signal}));
  }
  return validateDemographicInput(await readJsonFile(file,{signal}));
}
export function alignDemographicFractions(input,graph){validateDemographicInput(input,graph);return graph.geoids.map(id=>input.minority_fractions[id]);}

// Identity of the operational fractions, rather than their JSON spelling or
// user-supplied source label. Matches the native little-endian f64 protocol.
export async function demographicIdentity(input){
  validateDemographicInput(input);
  const header=new TextEncoder().encode(`BISECT_DEMOGRAPHICS_V1\0${input.state}\0${input.year}\0${input.basis}\0`),ids=Object.keys(input.minority_fractions).sort();
  const bytes=new Uint8Array(header.length+ids.length*19);bytes.set(header);
  const view=new DataView(bytes.buffer);let offset=header.length;
  for(const id of ids){bytes.set(new TextEncoder().encode(id),offset);view.setFloat64(offset+11,input.minority_fractions[id],true);offset+=19;}
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(byte=>byte.toString(16).padStart(2,'0')).join('');
}

// This is the existing native heuristic: an unweighted mean of tract fractions.
// It deliberately does not substitute an aggregation of raw VAP/CVAP counts.
export function tractMeanFractions(graph,assignments,input,k){
  const fractions=alignDemographicFractions(input,graph),sums=Array(k).fill(0),counts=Array(k).fill(0);
  graph.geoids.forEach((id,i)=>{const d=assignments[id];if(!Number.isSafeInteger(d)||d<1||d>k)throw new Error('Invalid demographic district assignment.');sums[d-1]+=fractions[i];counts[d-1]++;});
  if(counts.some(count=>!count))throw new Error('Demographic evidence has an empty district.');
  return sums.map((sum,i)=>sum/counts[i]);
}

// Descriptive count aggregation; the fraction-based search does not use this.
export function districtCountShares(graph,assignments,input,k){
  validateDemographicInput(input,graph);if(!input.counts)return null;
  const totals=Array(k).fill(0),minorities=Array(k).fill(0);
  for(const id of graph.geoids){const d=assignments[id];if(!Number.isSafeInteger(d)||d<1||d>k)throw new Error('Invalid demographic district assignment.');totals[d-1]+=input.counts[id].total;minorities[d-1]+=input.counts[id].minority;if(totals[d-1]>Number.MAX_SAFE_INTEGER)throw new Error('District demographic count exceeds the exact browser range.');}
  return totals.map((total,i)=>({district:i+1,total,minority:minorities[i],share:total===0?null:minorities[i]/total}));
}

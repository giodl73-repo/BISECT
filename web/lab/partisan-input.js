import {validateJsonTree,readJsonFile} from './project.js';
export function validatePartisanInput(input,graph){
  validateJsonTree(input);
  const fields=['schema_version','state','year','source_label','dem_shares'];
  if(!input||typeof input!=='object'||Array.isArray(input)||Object.keys(input).length!==fields.length||fields.some(k=>!Object.hasOwn(input,k))||input.schema_version!=='bisect-partisan-shares-v1'||typeof input.state!=='string'||! /^[A-Z]{2}$/.test(input.state)||typeof input.year!=='string'||! /^(2000|2010|2020)$/.test(input.year)||typeof input.source_label!=='string'||!input.source_label.length||new TextEncoder().encode(input.source_label).length>200)throw new Error('Invalid partisan schema, scope or source label.');
  const values=input.dem_shares;
  if(!values||typeof values!=='object'||Array.isArray(values)||!Object.keys(values).length||Object.entries(values).some(([id,v])=>! /^\d{11}$/.test(id)||!Number.isFinite(v)||v<0||v>1||Object.is(v,-0)))throw new Error('Supply complete GEOIDs with finite Democratic shares from zero through one.');
  if(graph&&(input.state!==graph.state||input.year!==graph.year||Object.keys(values).length!==graph.geoids.length||graph.geoids.some(id=>!Object.hasOwn(values,id))))throw new Error('Partisan shares must match the graph state, year and complete tract coverage.');
  return input;
}
export async function readPartisanFile(file){return validatePartisanInput(await readJsonFile(file));}
export function partisanWeights(graph,config){
  const input=validatePartisanInput(config.partisans[graph.state],graph),shares=graph.geoids.map(id=>input.dem_shares[id]);
  const strong=shares.filter(v=>v>=config.dem_threshold||v<=config.rep_threshold).length;
  const alpha=Math.max(3,10*(1-.7*(strong/shares.length)));
  const weight=(u,v)=>((shares[u]>=config.dem_threshold&&shares[v]>=config.dem_threshold)||(shares[u]<=config.rep_threshold&&shares[v]<=config.rep_threshold))?alpha:1;
  return {weight,strong,alpha};
}
export async function partisanIdentity(input){
  validatePartisanInput(input);
  const header=new TextEncoder().encode(`BISECT_PARTISAN_SHARES_V1\0${input.state}\0${input.year}\0`),ids=Object.keys(input.dem_shares).sort();
  const bytes=new Uint8Array(header.length+ids.length*19);bytes.set(header);const view=new DataView(bytes.buffer);let offset=header.length;
  for(const id of ids){bytes.set(new TextEncoder().encode(id),offset);view.setFloat64(offset+11,input.dem_shares[id],true);offset+=19;}
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
}

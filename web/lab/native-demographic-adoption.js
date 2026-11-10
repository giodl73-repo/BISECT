import {validateSourceFile,encodeBytes} from './package-files.js';
import {parsePreparedGraphJson} from './project.js';
import {validateDemographicInput} from './demographic-input.js';

const digest=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
const decodeBytes=text=>Uint8Array.from(atob(text),c=>c.charCodeAt(0));
const jsonSource=(name,value)=>({name,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(value)))});

export async function prepareNativeDemographicAdoption({inspection,source,graphSource}){
 validateSourceFile(source,{maxBytes:8*1024*1024});validateSourceFile(graphSource);
 if(inspection?.schema_version!=='bisect-native-demographic-inspection-v1'||!Array.isArray(inspection.records)||inspection.records.length>100000)throw new Error('Inspect this native CSV first.');
 const sourceHash=await digest(decodeBytes(source.base64));
 if(sourceHash!==inspection.source_sha256)throw new Error('Demographic CSV changed since inspection. Inspect it again.');
 const graph=parsePreparedGraphJson(new TextDecoder('utf-8',{fatal:true}).decode(decodeBytes(graphSource.base64)));
 if(graph.schema_version!==1||!Array.isArray(graph.geoids)||!graph.geoids.length||new Set(graph.geoids).size!==graph.geoids.length)throw new Error('Select a scoped prepared graph with unique unit IDs.');
 const values=new Map();for(const row of inspection.records){if(typeof row.geoid!=='string'||values.has(row.geoid))throw new Error('Inspection contains duplicate or invalid unit IDs.');values.set(row.geoid,row);}
 const fractions=Object.create(null),counts=Object.create(null),missing=[],retainCounts=inspection.basis!=='total-population';
 for(const id of graph.geoids){const row=values.get(id);if(!row){fractions[id]=0;missing.push(id);continue;}if(row.non_finite_fraction||!Number.isFinite(row.fraction)||row.fraction<0||row.fraction>1||Object.is(row.fraction,-0))throw new Error('A graph unit has an unusable native fraction: '+id);fractions[id]=row.fraction;}
 if(retainCounts)for(const id of graph.geoids){const row=values.get(id);counts[id]=row?{total:row.total,minority:row.minority}:{total:0,minority:0};}
 const input=validateDemographicInput({schema_version:retainCounts?'bisect-demographic-counts-v2':'bisect-demographic-fractions-v1',state:graph.state,year:graph.year,basis:inspection.basis,source_label:inspection.source_label,minority_fractions:fractions,...(retainCounts?{counts}:{})},graph);
 const graphHash=await digest(decodeBytes(graphSource.base64));
 const receipt={schema_version:'bisect-native-demographic-adoption-v1',source_name:source.name,source_sha256:sourceHash,graph_source_sha256:graphHash,state:graph.state,year:graph.year,basis:inspection.basis,source_diagnostics:inspection.diagnostics,matched_units:graph.geoids.length-missing.length,missing_units_defaulted_to_zero:missing.length,missing_unit_ids:missing,ignored_source_records:[...values.keys()].filter(id=>!Object.hasOwn(fractions,id)).length,policy:'Explicit adoption of native fractions. Missing graph units receive the historical native zero default. Original CSV remains retained. Fraction heuristics do not use count-weighted shares.'};
 const demographics=jsonSource('native-demographic-fractions.json',input);receipt.demographics_source_sha256=await digest(decodeBytes(demographics.base64));
 return {input,receipt,demographics,receiptSource:jsonSource('native-demographic-adoption.json',receipt)};
}

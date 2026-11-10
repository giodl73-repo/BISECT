import {validateTigerInputs} from './tiger-files.js';
import {validatePl94Inputs} from './pl94-files.js?v=pl94-large-v1';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {parseGeometryJson} from './project.js';
export function validateTractPreparationSettings(v){if(!v||!['projection,state','geometry_profile,projection,state'].includes(Object.keys(v).sort().join())||v.geometry_profile!==undefined&&!['display-simplified','display-topology','source-exact','source-exact-large'].includes(v.geometry_profile)||typeof v.state!=='string'||!/^[A-Z]{2}$/.test(v.state))throw new Error('Select a tract preparation state.');if(v.projection!==null)validateSourceFile(v.projection,{maxBytes:65536});return v;}
export function tractPreparationRequest(files){const settings=validateTractPreparationSettings(files.tractPreparationSettings);validateTigerInputs(files.tigerInputs);validatePl94Inputs(files.pl94Inputs);if(!settings.projection||files.tigerInputs.kind!=='tracts'||!files.tigerInputs.shp||!files.tigerInputs.dbf||!files.pl94Inputs.geography||files.pl94Inputs.year!==2000&&!files.pl94Inputs.population)throw new Error('Select tract SHP/DBF, NAD83 PRJ, and PL94 sources.');return {operation:'prepare-tract-engine-input',input:{state:settings.state,projection:settings.projection,tiger:files.tigerInputs,pl94:files.pl94Inputs,...(settings.geometry_profile?{geometry_profile:settings.geometry_profile}:{})}};}
export async function readTractProjection(file){if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>65536)throw new Error('Select NAD83 PRJ up to 64 KiB.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('PRJ changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};validateTractPreparationSettings({state:'RI',projection:source});return source;}
export function tractPreparationDownload(result,kind){if(!['bisect-tract-input-preparation-v1','bisect-census-input-preparation-v1'].includes(result?.schema_version)||result.status!=='prepared'||result.execution_performed!==false||!['graph','geometry'].includes(kind))throw new Error('Prepare fresh tract inputs first.');const text=result[kind+'_json'];if(typeof text!=='string'||text.length>(kind==='graph'?8:['source-exact-large','display-topology'].includes(result.geometry_profile)?64:16)*1024*1024)throw new Error('Invalid prepared tract artifact.');return {name:result.state+'-'+result.year+'-'+kind+'.json',type:'application/json',bytes:new TextEncoder().encode(text)};}
export function preparedTractRunSources(result){const file=tractPreparationDownload(result,'graph'),map=tractPreparationDownload(result,'geometry'),graph={name:file.name,base64:encodeBytes(file.bytes)},geometry={name:map.name,base64:encodeBytes(map.bytes)};parseGeometryJson(new TextDecoder().decode(map.bytes));validateSourceFile(graph);validateSourceFile(geometry,{maxBytes:64*1024*1024});return {graph,geometry,settings:{chamber:'congressional',districts:result.suggested_districts||2}};}

export function censusPreparationRequest(files){
 const settings=validateTractPreparationSettings(files.tractPreparationSettings);validateTigerInputs(files.tigerInputs);validatePl94Inputs(files.pl94Inputs);
 if(!settings.projection||!['counties','tracts','block-groups','blocks'].includes(files.tigerInputs.kind)||!files.tigerInputs.shp||!files.tigerInputs.dbf||!files.pl94Inputs.geography||files.pl94Inputs.year!==2000&&!files.pl94Inputs.population)throw new Error('Select Census polygon SHP/DBF, NAD83 PRJ and matching PL94 sources.');
 return {operation:'prepare-census-engine-input',input:{state:settings.state,projection:settings.projection,tiger:files.tigerInputs,pl94:files.pl94Inputs,...(settings.geometry_profile?{geometry_profile:settings.geometry_profile}:{})}};
}

// After a handoff, exact derived bytes live in project source fields once.
// Retain source hashes and preparation metadata without a second large copy.
export function preparedTractHandoffReceipt(prepared){
 if(!['source-exact-large','display-topology'].includes(prepared.geometry_profile))return prepared;
 const {graph_json,geometry_json,...receipt}=prepared;
 return {...receipt,graph_project_field:'configRunInputs.graph',geometry_project_field:'planGeometry'};
}

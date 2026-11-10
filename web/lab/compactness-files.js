import {validateSourceFile,exactSourceText,encodeBytes} from './package-files.js?v=compactness-batch-v1';
import {parseGeometryJson} from './project.js';
export const MAX_COMPACTNESS_BYTES=64*1024*1024;
export function compactnessGeometryText(source){
 validateSourceFile(source,{maxBytes:MAX_COMPACTNESS_BYTES});const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));
 if(bytes.length>MAX_COMPACTNESS_BYTES)throw new Error('Geometry exceeds 64 MiB.');
 const text=new TextDecoder('utf-8',{fatal:true}).decode(bytes);if(bytes.length>18*1024*1024&&!['source-exact-large','display-topology'].includes(parseGeometryJson(text).geometry_profile))throw new Error('Large geometry requires the explicit root source profile.');return text;
}
export async function readCompactnessGeometry(file){
 if(!file)return null;
 if(!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_COMPACTNESS_BYTES)throw new Error('Select nonempty geometry of at most 64 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Geometry size changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};
 compactnessGeometryText(source);return source;
}
export function validateCompactnessInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(k=>!['geometry','coordinate_basis','crs_label','include_exact_reock','population','dissolve_method'].includes(k))||!['geometry','coordinate_basis','crs_label'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid compactness inputs.');
 if(Object.hasOwn(value,'dissolve_method')&&!['sequential','balanced','spatial-balanced','batch'].includes(value.dissolve_method))throw new Error('Select a supported dissolve method.');
 if(Object.hasOwn(value,'include_exact_reock')&&typeof value.include_exact_reock!=='boolean')throw new Error('Exact Reock option must be boolean.');
 if(!['wgs84-degrees','nad83-degrees','projected-metres'].includes(value.coordinate_basis))throw new Error('Select a coordinate basis.');
 if(typeof value.crs_label!=='string'||!value.crs_label.trim()||new TextEncoder().encode(value.crs_label).length>200)throw new Error('Supply a recorded CRS label of 1–200 bytes.');
 if(value.coordinate_basis==='wgs84-degrees'&&value.crs_label!=='EPSG:4326')throw new Error('WGS84 degrees require CRS label EPSG:4326.');
 if(value.coordinate_basis==='nad83-degrees'&&value.crs_label!=='EPSG:4269')throw new Error('NAD83 degrees require CRS label EPSG:4269.');
 if(value.population!==undefined&&value.population!==null){validateSourceFile(value.population);if(atob(value.population.base64).length>3*1024*1024)throw new Error('Population centroid CSV exceeds 3 MiB.');exactSourceText(value.population);}
 if(value.geometry!==null){compactnessGeometryText(value.geometry);}return value;
}
export function compactnessRequest(files,inputs){
 validateCompactnessInputs(inputs);if(!files.auditPlanText||!inputs.geometry)throw new Error('Select an exact RPLAN and unit geometry source.');
 const geometryText=compactnessGeometryText(inputs.geometry),large=/"geometry_profile"\s*:\s*"(?:source-exact-large|display-topology)"/.test(geometryText)&&['source-exact-large','display-topology'].includes(parseGeometryJson(geometryText).geometry_profile),cap=(large?64:18)*1024*1024;
 if(atob(files.auditPlanText.base64).length+atob(inputs.geometry.base64).length+(inputs.population?atob(inputs.population.base64).length:0)>cap)throw new Error('Plan, geometry and optional population sources exceed the selected 18/64 MiB profile.');
 return {operation:'analyze-plan-compactness',document_json:exactSourceText(files.auditPlanText),unit_geometry_json:compactnessGeometryText(inputs.geometry),coordinate_basis:inputs.coordinate_basis,crs_label:inputs.crs_label,...(inputs.dissolve_method&&inputs.dissolve_method!=='sequential'?{dissolve_method:inputs.dissolve_method}:{}),...(inputs.include_exact_reock?{include_exact_reock:true}:{}),...(inputs.population?{population_centroids_csv:exactSourceText(inputs.population)}:{})};
}

import {validateTigerInputs} from './tiger-files.js';
import {validateSourceFile,encodeBytes} from './package-files.js';
import {parseGeometryJson,parseNativeMapPlanJson} from './project.js';
const bytes=source=>Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));
const text=source=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes(source));
function raw(source,max=3*1024*1024){validateSourceFile(source?.base64===''?{...source,base64:'AA=='}:source,{maxBytes:max});return source;}
export function validateMapTigerSources(v){
 if(v.geometry_source!==undefined&&!['geojson','tiger'].includes(v.geometry_source))throw new Error('Select GeoJSON or original TIGER map geometry.');
 if(v.tiger!=null){validateTigerInputs(v.tiger);const size=Object.values(v.tiger).filter(p=>p&&typeof p==='object'&&typeof p.base64==='string').reduce((sum,p)=>sum+p.base64.length/4*3-(p.base64.endsWith('==')?2:p.base64.endsWith('=')?1:0),0);if(size>90*1024*1024)throw new Error('Native TIGER state parts exceed 90 MiB combined.');}
 return v;
}
export function validateRawMapInputs(v){
 validateMapTigerSources(v);
 if(v.geometry_source==='tiger'&&(v.dissolve_method&&v.dissolve_method!=='sequential'||v.tiger&&(!['tracts','block-groups'].includes(v.tiger.kind)||v.tiger.year!==v.raw_year)))throw new Error('Native TIGER state maps require matching tract/block-group vintage and native input-order dissolve.');
 if(v.analysis_policy!==undefined)throw new Error('Raw native assignment maps use native analysis semantics; clear the separate state analysis policy.');
 if(!/^[A-Z]{2}$/.test(v.raw_state)||![2000,2010,2020].includes(v.raw_year)||!['unit-id','native-index-map'].includes(v.assignment_key_basis||'unit-id'))throw new Error('Select raw assignment state/year and explicit key basis.');
 if(v.round_assignments||v.round_index||v.round_directory||v.round_key_basis&&v.round_key_basis!=='unit-id')throw new Error('Clear recorded-round sources before selecting raw assignments.');
 for(const key of ['assignments','assignment_index','analysis'])if(v[key]!=null)raw(v[key]);
 if(v.geometry){raw(v.geometry,64*1024*1024);parseGeometryJson(text(v.geometry));}
 if(v.plan){raw(v.plan);parseNativeMapPlanJson(text(v.plan));}return v;
}
export function rawMapRequest(v){validateRawMapInputs(v);if(v.geometry_source==='tiger'){if(!v.assignments||!v.tiger?.shp||!v.tiger?.dbf)throw new Error('Select original assignments and matching TIGER SHP/DBF parts.');return {operation:'render-native-tiger-assignment-map',input:{state:v.raw_state,tiger:v.tiger,key_basis:v.assignment_key_basis||'unit-id',assignments_base64:v.assignments.base64,map_type:v.map_type||'districts',dpi:v.dpi,...(v.assignment_index?{unit_index_base64:v.assignment_index.base64}:{}),...(v.analysis?{analysis_base64:v.analysis.base64}:{})}};}if(!v.assignments||!v.geometry)throw new Error('Select original native assignments and matching geographic unit geometry.');return {operation:'render-native-assignment-map',input:{state:v.raw_state,year:v.raw_year,key_basis:v.assignment_key_basis||'unit-id',assignments_base64:v.assignments.base64,unit_geometry_json:text(v.geometry),map_type:v.map_type||'districts',dpi:v.dpi,dissolve_method:v.dissolve_method||'sequential',...(v.assignment_index?{unit_index_base64:v.assignment_index.base64}:{}),...(v.analysis?{analysis_base64:v.analysis.base64}:{})}};}
export async function readRawMapSource(file){if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>3*1024*1024)throw new Error('Native assignment/index sources are limited to 3 MiB each.');const body=new Uint8Array(await file.arrayBuffer());if(body.length!==file.size)throw new Error('Native map source changed while reading.');return raw({name:file.name,base64:encodeBytes(body)});}

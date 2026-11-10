import {encodeBytes,validateSourceFile} from './package-files.js';
import {characterStateFips,validateCharacterInput} from './character-input.js';
export const NATIVE_CHARACTER_DEFAULTS={state:'RI',year:'2020',data_year:'2020',kind:'housing',source_label:'Native tract character CSV'};
export function validateNativeCharacterInputs(v){
 if(!v||Object.keys(v).sort().join()!=='settings,source'||!v.settings||Object.keys(v.settings).sort().join()!==Object.keys(NATIVE_CHARACTER_DEFAULTS).sort().join())throw new Error('Invalid native character source selection.');const s=v.settings;
 if(!characterStateFips(s.state)||typeof s.year!=='string'||!/^\d{4}$/.test(s.year)||typeof s.data_year!=='string'||!/^\d{4}$/.test(s.data_year)||!['economic','housing'].includes(s.kind)||typeof s.source_label!=='string'||!s.source_label.trim()||new TextEncoder().encode(s.source_label).length>200)throw new Error('Choose native character kind, state, unit/observation year and label.');
 if(v.source!==null){if(typeof v.source?.name!=='string'||!v.source.name||new TextEncoder().encode(v.source.name).length>200)throw new Error('Invalid native source name.');if(v.source?.base64===''){if(Object.keys(v.source).sort().join()!=='base64,name')throw new Error('Invalid empty native source.');}else validateSourceFile(v.source);}
 return v;
}
export function nativeCharacterRequest(v){validateNativeCharacterInputs(v);if(!v.source)throw new Error('Select a native tract CSV file.');return {operation:'prepare-native-character-source',input:{...v.settings,source_name:v.source.name,source_base64:v.source.base64}};}
export async function readNativeCharacterSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>8*1024*1024)throw new Error('Select a native tract CSV up to8MiB; empty files may be inspected.');const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Native character file changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};validateNativeCharacterInputs({source,settings:NATIVE_CHARACTER_DEFAULTS});return source;
}
export function nativeCharacterDownload(result,kind){
 if(result?.schema_version!=='bisect-native-character-source-v1'||result.execution_performed!==false)throw new Error('Inspect a fresh native character CSV first.');
 if(kind==='character'){if(result.character_status!=='ready')throw new Error('Native observations are unavailable for strict engine input.');validateCharacterInput(result.character_input,null,result.kind);return {name:result.kind+'-native-character.json',bytes:new TextEncoder().encode(JSON.stringify(result.character_input,null,2)+'\n')};}
 if(kind!=='report')throw new Error('Unknown native character download.');return {name:'native-character-source-report.json',bytes:new TextEncoder().encode(JSON.stringify(result,null,2)+'\n')};
}

import {validateProject} from './project.js';
import {validateNativeCharacterInputs,readNativeCharacterSource} from './native-character-files.js';
import {MAX_PROJECT_BYTES,readJsonFile,validateJsonTree} from './project.js';
import {readCharacterFile,validateCharacterInput,alignCharacterInput} from './character-input.js';
import {preparedCharacterInput,verifyCharacterPreparation} from './character-handoff.js';
import {isNativeCharacterFallback,nativeCharacterFallbackInfo,verifyNativeCharacterFallback} from './character-fallback-source.js';

// Resolve a saved selection without running preparation, alignment or a plan.
export async function characterSelection(value,{kind,year,allowFallback=false,nativeSourceMode=false}={}){
 validateJsonTree(value);
 let input=null,preparation=null,fallback=null,native=null,info;
 if(nativeSourceMode&&(value?.schema_version==='bisect-project-v1'&&value.files?.nativeCharacterInputs||value?.source&&value?.settings)){if(value?.schema_version==='bisect-project-v1')validateProject(value);native=structuredClone(validateNativeCharacterInputs(value.files?.nativeCharacterInputs??value));if(!native.source)throw new Error('Select a retained original native CSV.');info=native.settings;}
 else if(value?.schema_version==='bisect-project-v1'){
  if(isNativeCharacterFallback(value)){
   if(!allowFallback)throw new Error('Explicitly allow geographic fallback before selecting an empty/error source project.');
   info=nativeCharacterFallbackInfo(value);await verifyNativeCharacterFallback(value);fallback=structuredClone(value);
  }else{input=preparedCharacterInput(value);await verifyCharacterPreparation(value,input);preparation=structuredClone(value);info={...input,kind:input.data.kind};}
 }else{input=validateCharacterInput(value);info={...input,kind:input.data.kind};}
 if(info.kind!==kind||info.year!==year)throw new Error('Character selection must match the selected weights and Census year.');
 return {state:info.state,input:input?structuredClone(input):null,preparation,fallback,...(native?{native}:{})};
}

export async function readCharacterSelections(files,options){
 if(!Array.isArray(files)||!files.length||files.length>50||files.some(f=>!f||!Number.isSafeInteger(f.size)||f.size<0)||files.reduce((n,f)=>n+f.size,0)>MAX_PROJECT_BYTES)throw new Error('Select 1–50 character files totaling at most 25 MiB.');
 const selections=Object.create(null);
 for(const file of files){
  const rawCsv=options.nativeSourceMode&&/\.csv$/i.test(file.name||'');
  const value=rawCsv?{source:await readNativeCharacterSource(file),settings:{...options.csvMetadata,source_label:file.name}}:/\.csv$/i.test(file.name||'')?await readCharacterFile(file,{csvMetadata:{...options.csvMetadata,source_label:file.name},wasmSha256:options.wasmSha256,signal:options.signal}):await readJsonFile(file,{signal:options.signal});
  const selection=await characterSelection(value,options);
  if(Object.hasOwn(selections,selection.state))throw new Error('Select one character file or preparation project per state.');
  selections[selection.state]=selection;
 }
 return selections;
}

export function applyCharacterSelections(current,selections){
 const next={characters:{...current.characters},preparations:{...current.preparations},fallbacks:{...current.fallbacks},...(current.native||Object.values(selections).some(s=>s.native)?{native:{...current.native}}:{})};
 for(const [state,selection]of Object.entries(selections)){
  delete next.characters[state];delete next.preparations[state];delete next.fallbacks[state];if(next.native)delete next.native[state];
  if(selection.input)next.characters[state]=selection.input;
  if(selection.preparation)next.preparations[state]=selection.preparation;
  if(selection.fallback)next.fallbacks[state]=selection.fallback;if(selection.native)next.native[state]=selection.native;
 }
 return next;
}

export async function alignCharacterSelections(characters,{states,kind,year,allowFallback=false,nativeSources={},graphFor}){
 const next={...characters};let aligned=0;
 for(const code of states){
  if(nativeSources[code]){const s=validateNativeCharacterInputs(nativeSources[code]).settings;if(s.state!==code||s.year!==year||s.kind!==kind)throw new Error('Native source scope must match alignment selection.');continue;}
  if(!Object.hasOwn(characters,code)&&allowFallback)continue;
  const input=validateCharacterInput(characters[code],null,kind);
  if(input.year!==year||input.state!==code)throw new Error('Character input scope must match the selected state and Census year.');
  const graph=await graphFor(code),original={...input,data:input.alignment?.source_data??input.data};delete original.alignment;
  next[code]=validateCharacterInput(alignCharacterInput(original,graph),graph,kind);aligned++;
 }
 return {characters:next,aligned};
}

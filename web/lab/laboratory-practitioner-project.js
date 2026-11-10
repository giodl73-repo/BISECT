import {createProject} from './project.js';
import {createStoredWorkbenchArchive,parseWorkbenchArchiveBytes} from './workbench-archive.js';

// Ordinary projects keep their existing limit and schema. Larger selected-state
// handoffs use the existing bounded, explicitly opened workbench archive format.
export async function preparePractitionerProject(data){
 try{return {project:createProject(data)};}
 catch(error){
  const capacity=error.message==='Project exceeds the 25 MB import limit.'||
   error.message==='Project has too many values.'||
   error.message==='JSON is too complex (maximum 64 levels and 500,000 values).';
  if(!capacity)throw error;
 }
 const archive=await createStoredWorkbenchArchive(data);
 const bytes=typeof archive==='string'?new TextEncoder().encode(archive):archive;
 return {project:await parseWorkbenchArchiveBytes(bytes),archive};
}

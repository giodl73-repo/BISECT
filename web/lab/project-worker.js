import {readFileTextStrict} from './project.js?v=census-acquire-zip-v1';
import {parseProject} from './project.js?v=census-acquire-zip-v1';
self.onmessage = async ({data:file}) => {
  try { self.postMessage({ok:true,project:parseProject(await readFileTextStrict(file))}); }
  catch (error) { self.postMessage({ok:false,error:error.message || 'Invalid project file.'}); }
};

import {readFileTextStrict} from './project.js';
import {parseProject} from './project.js';
self.onmessage = async ({data:file}) => {
  try { self.postMessage({ok:true,project:parseProject(await readFileTextStrict(file))}); }
  catch (error) { self.postMessage({ok:false,error:error.message || 'Invalid project file.'}); }
};

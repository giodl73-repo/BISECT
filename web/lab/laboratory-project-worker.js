import {readFileTextStrict} from './project.js';
import {parseLabProject} from './laboratory-project.js';
self.onmessage = async ({data:file}) => {
  try { self.postMessage({ok:true,project:parseLabProject(await readFileTextStrict(file))}); }
  catch (error) { self.postMessage({ok:false,error:error.message || 'Invalid laboratory project.'}); }
};

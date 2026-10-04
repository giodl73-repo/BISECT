import {parseSafeJson,readFileTextStrict} from './project.js';
self.onmessage = async ({data:file}) => {
  try { self.postMessage({ok:true,project:parseSafeJson(await readFileTextStrict(file))}); }
  catch(error) { self.postMessage({ok:false,error:error.message || 'Invalid JSON input.'}); }
};

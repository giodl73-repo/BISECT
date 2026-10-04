// Portable project snapshots. Browser storage is a convenience; downloaded files are backups.
import {validatePackageFiles,validateSourceFile,validateImportSettings,validateRiSources,validateAggregationSettings,validateDemographicSettings} from './package-files.js';
export const PROJECT_SCHEMA = 'bisect-project-v1';
export const MAX_PROJECT_BYTES = 25 * 1024 * 1024;
const fields = ['schema_version','name','saved_at_utc','files','operation','constraints','result','lastOperation'];
const inputs = ['plan', 'context', 'profile', 'certificate', 'package','packageFiles','sourceFile','importSettings','riSources','crosswalkFile','aggregationSettings','demographicSource','demographicSettings'];
const operations = ['attach-demographic-csv','verify-history-files', 'validate-rplan', 'audit-plan', 'verify-certificate', 'verify-count','verify-count-files','replay-count-audits','import-statement-csv','import-nist-cdf-json','import-ri2024-rep28-rla','aggregate-districts'];
const checks = ['plan-shape', 'population', 'contiguity', 'splits', 'vra', 'geometry'];
export function createProject({name, files, operation, constraints, result, lastOperation}) {
  return parseProject(JSON.stringify({schema_version: PROJECT_SCHEMA,
    name: name || 'Untitled project', saved_at_utc: new Date().toISOString(),
    files, operation, constraints, result: result ?? null, lastOperation: lastOperation ?? null}));
}
export function validateProject(project) {
  validateJsonTree(project);
  if (!isRecord(project) || Object.keys(project).length !== fields.length || fields.some(key => !Object.hasOwn(project,key)) || Object.keys(project).some(key => !fields.includes(key))) throw new Error('Invalid project fields.');
  if (!project || project.schema_version !== PROJECT_SCHEMA) throw new Error('Unsupported project format.');
  if (typeof project.name !== 'string' || project.name.length > 200) throw new Error('Invalid project name.');
  if (typeof project.saved_at_utc !== 'string' || project.saved_at_utc.length > 40 || !Number.isFinite(Date.parse(project.saved_at_utc))) throw new Error('Invalid project save date.');
  if (!operations.includes(project.operation)) throw new Error('Unsupported project operation.');
  if (!Array.isArray(project.constraints) || project.constraints.length > checks.length || new Set(project.constraints).size !== project.constraints.length || project.constraints.some(c => !checks.includes(c))) throw new Error('Invalid project checks.');
  if (!isRecord(project.files)) throw new Error('Invalid project inputs.');
  for (const [key, value] of Object.entries(project.files)) {
    if (!inputs.includes(key) || !isRecord(value)) throw new Error('Invalid project input: ' + key);
    if(key==='packageFiles')validatePackageFiles(value);
    if(key==='sourceFile')validateSourceFile(value);
    if(key==='importSettings')validateImportSettings(value,{requireComplete:false});
    if(key==='riSources')validateRiSources(value,{requireComplete:false});
    if(key==='crosswalkFile')validateSourceFile(value);
    if(key==='aggregationSettings')validateAggregationSettings(value,{requireComplete:false});
    if(key==='demographicSource')validateSourceFile(value);
    if(key==='demographicSettings')validateDemographicSettings(value,{requireComplete:false});
  }
  if (project.lastOperation !== null && !operations.includes(project.lastOperation)) throw new Error('Invalid result operation.');
  if (project.result !== null && (typeof project.result !== 'object' || Array.isArray(project.result) || !project.lastOperation)) throw new Error('Invalid project result.');
  if (project.result?.schema_version === 'bisect-election-import-v1') validatePackageFiles(project.result.package_files);
  if (project.result?.schema_version === 'bisect-district-aggregation-v1') validateSourceFile({name:'aggregation.json',base64:project.result.transcript_base64});
  return project;
}
export function parseProject(text) {
  return validateProject(parseSafeJson(text));
}
export function parseSafeJson(text) {
  if (typeof text !== 'string' || text.length > MAX_PROJECT_BYTES || new TextEncoder().encode(text).byteLength > MAX_PROJECT_BYTES) throw new Error('Project exceeds the 25 MB import limit.');
  inspectJsonText(text);
  const value = JSON.parse(text); validateJsonTree(value); return value;
}
// JSON.parse accepts duplicate fields by keeping the last one. Reject that
// ambiguity, including escaped keys. Bound depth and values before JSON.parse
// allocates an object tree; syntax validation still belongs to JSON.parse.
function inspectJsonText(text) {
  const containers=[];let nodes=0;
  function value(depth) {
    if(++nodes>500000 || depth>64)throw new Error('Project is too complex (maximum 64 levels and 500,000 values).');
  }
  for(let i=0;i<text.length;i++) {
    const char=text[i];
    if(char==='{' || char==='[') { value(containers.length);containers.push(char==='{'?new Set():null); }
    else if(char==='}' || char===']')containers.pop();
    else if(char==='"') {
      const start=i;
      while(++i<text.length) { if(text[i]==='\\')i++; else if(text[i]==='"')break; }
      let next=i+1;while(/\s/.test(text[next]||'') && next<text.length)next++;
      if(text[next]===':') {
        const key=JSON.parse(text.slice(start,i+1)),keys=containers[containers.length-1];
        if(keys instanceof Set) {
          if(['__proto__','constructor','prototype'].includes(key))throw new Error('Unsafe project property.');
          if(keys.has(key))throw new Error('Duplicate JSON property.');
          keys.add(key);
        }
      } else value(containers.length);
    } else if(!/[\s,:]/.test(char)) {
      value(containers.length);
      while(i+1<text.length && !/[\s,\[\]{}:"]/.test(text[i+1]))i++;
    }
  }
}
export async function readFileTextStrict(file) {
  if(!file || !Number.isSafeInteger(file.size) || file.size<0 || file.size>MAX_PROJECT_BYTES)throw new Error('Project exceeds the 25 MB import limit.');
  const bytes=await file.arrayBuffer();
  if(bytes.byteLength>MAX_PROJECT_BYTES)throw new Error('Project exceeds the 25 MB import limit.');
  try { return new TextDecoder('utf-8',{fatal:true}).decode(bytes); }
  catch { throw new Error('File is not valid UTF-8.'); }
}
export function readJsonFile(file, options) {
  return readValidatedFile(file,new URL('./json-worker.js',import.meta.url),options);
}
function isRecord(value) {
  return value !== null && typeof value === 'object' && (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null);
}
export function validateJsonTree(root) {
  const stack = [[root,0]], seen = new Set();
  let nodes = 0, stringBytes = 0;
  while (stack.length) {
    const [value,depth] = stack.pop();
    if (++nodes > 500000 || depth > 64) throw new Error('Project is too complex (maximum 64 levels and 500,000 values).');
    if (typeof value === 'string') { stringBytes += new TextEncoder().encode(value).byteLength; }
    else if (typeof value === 'number') {
      if (!Number.isFinite(value)) throw new Error('Invalid project number.');
      // JSON.parse rounds larger integers before schema or certificate checks.
      // Reject them rather than silently verifying different imported data.
      if (Number.isInteger(value) && !Number.isSafeInteger(value)) throw new Error('Project integer exceeds the exact JavaScript range.');
    }
    else if (value !== null && typeof value === 'object') {
      if (seen.has(value)) throw new Error('Repeated or cyclic project object.');
      seen.add(value);
      if (!Array.isArray(value) && !isRecord(value)) throw new Error('Project contains a non-JSON object.');
      const keys = Object.keys(value);
      if (keys.length > 500000 - nodes - stack.length) throw new Error('Project has too many values.');
      for (const key of keys) {
        if (['__proto__','constructor','prototype'].includes(key)) throw new Error('Unsafe project property.');
        const property = Object.getOwnPropertyDescriptor(value,key);
        if (!property || !Object.hasOwn(property,'value')) throw new Error('Project contains an accessor.');
        stringBytes += new TextEncoder().encode(key).byteLength;
        stack.push([property.value,depth+1]);
      }
    } else if (value !== null && typeof value !== 'boolean') throw new Error('Project contains a non-JSON value.');
    if (stringBytes > MAX_PROJECT_BYTES) throw new Error('Project exceeds the 25 MB import limit.');
  }
}

// Parse off the UI thread. Untrusted files never replace current state until validated.
export function readProjectFile(file, {signal} = {}) {
  return readValidatedFile(file, new URL('./project-worker.js',import.meta.url), {signal});
}
export function readValidatedFile(file, workerUrl, {signal} = {}) {
  if (!file || !Number.isSafeInteger(file.size) || file.size < 0 || file.size > MAX_PROJECT_BYTES) return Promise.reject(new Error('Project exceeds the 25 MB import limit.'));
  if (signal?.aborted) return Promise.reject(new Error('Project import cancelled. Current project was kept.'));
  return new Promise((resolve,reject) => {
    const worker = new Worker(workerUrl,{type:'module'});
    const timer = setTimeout(() => finish(new Error('Project import timed out. Current project was kept.')),10000);
    let settled=false;
    function abort() { finish(new Error('Project import cancelled. Current project was kept.')); }
    function finish(error,project) {
      if(settled)return;settled=true;
      clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error ? reject(error) : resolve(project);
    }
    signal?.addEventListener('abort',abort,{once:true});
    worker.onmessage = ({data}) => {
      if(data?.ok === true && Object.hasOwn(data,'project')) finish(null,data.project);
      else finish(new Error(typeof data?.error === 'string' ? data.error : 'Project import failed. Current project was kept.'));
    };
    worker.onerror = () => finish(new Error('Project import failed. Current project was kept.'));
    worker.onmessageerror = () => finish(new Error('Project import failed. Current project was kept.'));
    try { worker.postMessage(file); } catch { finish(new Error('Project import failed. Current project was kept.')); }
  });
}
export async function writeAutosave(project) {
  return withStore('readwrite', store => store.put(validateProject(project), 'current'));
}
export async function readAutosave() {
  const project = await withStore('readonly', store => store.get('current'));
  return project ? validateProject(project) : null;
}
function withStore(mode, action) {
  return new Promise((resolve, reject) => {
    const open = indexedDB.open('bisect-workbench', 1);
    open.onupgradeneeded = () => open.result.createObjectStore('projects');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const db = open.result;
      const transaction = db.transaction('projects', mode);
      const request = action(transaction.objectStore('projects'));
      transaction.oncomplete = () => { db.close(); resolve(request.result); };
      transaction.onabort = transaction.onerror = () => { db.close(); reject(transaction.error || request.error); };
    };
  });
}

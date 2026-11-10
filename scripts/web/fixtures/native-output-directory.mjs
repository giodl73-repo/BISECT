import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';
export function nativeOutputDirectoryHarness(root){
 assert(path.isAbsolute(root)&&root.startsWith(path.resolve('target')+path.sep));const log=[];
class Directory{
 constructor(dir,options={}){this.dir=path.resolve(dir);this.options=options;this.kind='directory';this.name=path.basename(dir);assert(this.dir.startsWith(root+path.sep));}
 child(name){assert(!['.','..'].includes(name)&&!name.includes('/')&&!name.includes('\\'));const p=path.resolve(this.dir,name);assert(p.startsWith(root+path.sep));return p;}
 async getDirectoryHandle(name,{create=false}={}){const p=this.child(name);try{if(!(await fs.stat(p)).isDirectory())throw Error('Not directory');}catch(e){if(e.code!=='ENOENT')throw e;if(!create)throw Object.assign(Error('Missing'),{name:'NotFoundError'});await fs.mkdir(p);}return new Directory(p,this.options);}
 async getFileHandle(name,{create=false}={}){const p=this.child(name);try{if(!(await fs.stat(p)).isFile())throw Error('Not file');}catch(e){if(e.code!=='ENOENT')throw e;if(!create)throw Object.assign(Error('Missing'),{name:'NotFoundError'});await fs.writeFile(p,new Uint8Array());}const options=this.options;return {kind:'file',name,async getFile(){const data=await fs.readFile(p);return {size:data.length,lastModified:(await fs.stat(p)).mtimeMs,async arrayBuffer(){return Uint8Array.from(data).buffer;}};},async createWritable(){let pending;return {async write(data){if(options.fail?.(p))throw Error('Synthetic disk failure');pending=data;},async close(){if(options.corrupt?.(p))pending=new TextEncoder().encode('corrupt');await fs.writeFile(p,pending);log.push('write:'+path.relative(root,p).replaceAll('\\','/'));await options.afterClose?.(p);},async abort(){pending=undefined;}};}};}
 async *entries(){for(const e of await fs.readdir(this.dir,{withFileTypes:true})){if(e.isDirectory())yield[e.name,new Directory(this.child(e.name),this.options)];else if(e.isFile())yield[e.name,await this.getFileHandle(e.name)];else throw Error('Unsupported test link');}}
 async removeEntry(name,{recursive=false}={}){const p=this.child(name);if(this.options.removeFail?.(p))throw Error('Synthetic marker removal failure');assert(p.startsWith(root+path.sep)&&p!==root);await fs.rm(p,{recursive});log.push('remove:'+path.relative(root,p).replaceAll('\\','/'));}
}
async function directory(name,options={}){const p=path.join(root,name+'-'+crypto.randomUUID());await fs.mkdir(p);return new Directory(p,options);}
return {Directory,directory,log};}

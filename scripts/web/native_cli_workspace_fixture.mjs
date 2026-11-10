import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const repo=path.resolve(process.argv[3]??'.');
const {nativeWorkspaceCase}=await import(pathToFileURL(path.join(repo,'scripts/web/fixtures/native-workspace-case.mjs')));
const root=path.resolve(process.argv[2]);await fs.mkdir(root,{recursive:true});
const fixture=nativeWorkspaceCase(),input=path.join(root,'inputs');
await fs.mkdir(input,{recursive:true});
for(const [name,bytes] of fixture.files){const file=path.join(input,name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,bytes);}
const manifest=JSON.parse(await fs.readFile(path.join(repo,'data/manifest.json'),'utf8'));manifest.local_outputs_dir=input;
await fs.writeFile(path.join(input,'manifest.json'),JSON.stringify(manifest,null,2));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const pins={};for(const [name,bytes]of fixture.files)pins[name]=sha(bytes);
await fs.writeFile(path.join(root,'fixture.json'),JSON.stringify({scope:'Synthetic four-unit RI tract chain, population 100 per unit, two congressional districts. Physical values are fixture data; no real geography or legal validity claim.',ids:fixture.ids,stem:fixture.stem,input,manifest:path.join(input,'manifest.json'),files:pins,
 fixture_builder_sha256:sha(await fs.readFile(path.join(repo,'scripts/web/fixtures/native-workspace-case.mjs'))),
 fixture_dependency_sha256:sha(await fs.readFile(path.join(repo,'scripts/web/fixtures/cffi-engine-case.mjs')))},null,2));
console.log(JSON.stringify({status:'prepared',root,units:4,districts:2}));

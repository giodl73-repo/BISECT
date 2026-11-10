import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
const root=path.resolve(process.argv[2]||'target/toolkit-asset-check');
const files=await fs.readdir(root);assert.ok(files.includes('index.html'));
let references=0;
for(const file of files.filter(name=>name.endsWith('.js')||name==='index.html')){
 const source=await fs.readFile(path.join(root,file),'utf8');
 const refs=[...source.matchAll(/(?:from\s*|import\s*\(|new URL\s*\()(['"])(\.\.?\/[^'"]+)\1/g)].map(match=>match[2]);
 if(file==='index.html')refs.push(...[...source.matchAll(/(?:src|href)="(\.\/[^"?#]+\.(?:js|css))(?:[?#][^"]*)?"/g)].map(match=>match[1]));
 for(const ref of refs){const target=path.resolve(root,ref.split(/[?#]/)[0]);assert.ok(target.startsWith(root+path.sep),`${file}: asset escapes package: ${ref}`);await fs.access(target).catch(()=>assert.fail(`${file}: missing packaged dependency ${ref}`));references++;}
}
assert.ok(references>100,'Check must cover the toolkit module/worker dependency graph');
console.log(`Toolkit package: ${files.length} files and ${references} local module/worker/HTML dependencies resolved.`);

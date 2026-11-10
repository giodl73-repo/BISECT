// Real Cargo rebuild checks for the exact native asset generator; no JS executes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const repository=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const sha=b=>createHash('sha256').update(b).digest('hex');
await fs.mkdir(path.resolve('target'),{recursive:true});
const out=await fs.mkdtemp(path.resolve('target/native-asset-build-')),fixture=path.join(out,'fixture'),crate=path.join(fixture,'crates/bisect-web'),web=path.join(fixture,'web/lab'),target=path.join(out,'cargo-target');
await fs.mkdir(path.join(crate,'src'),{recursive:true});await fs.mkdir(web,{recursive:true});
const buildSource=await fs.readFile(path.join(repository,'crates/bisect-web/build.rs')),registrySource=await fs.readFile(path.join(repository,'crates/bisect-web/src/input_assets.rs'));
await fs.writeFile(path.join(crate,'build.rs'),buildSource);await fs.writeFile(path.join(crate,'src/input_assets.rs'),registrySource);
await fs.writeFile(path.join(crate,'Cargo.toml'),'[package]\nname="native-asset-build-probe"\nversion="0.0.0"\nedition="2021"\n[workspace]\n');
await fs.writeFile(path.join(crate,'src/main.rs'),'mod input_assets;\nuse std::io::Write;\nfn main(){let name=std::env::args().nth(1);if let Some(name)=name {let (_,source)=input_assets::INPUT_ASSETS.iter().find(|(key,_)|*key==name).expect("asset exists");std::io::stdout().write_all(source.as_bytes()).unwrap();}else{for (name,_) in input_assets::INPUT_ASSETS {println!("{name}");}}}\n');
const base="export const base = 'fixed';\n",worker="export const worker = 'worker';\n";
for(const [name,text]of [['base.js',base],['worker.js',worker],['lab.js','root route'],['static.js','root route']])await fs.writeFile(path.join(web,name),text);
const phases=[],cargo=process.env.CARGO||'cargo',env={...process.env,CARGO_INCREMENTAL:'0',CARGO_PROFILE_DEV_DEBUG:'0'};
delete env.CARGO_TARGET_DIR;
function run(name,args,expected=0){const r=spawnSync(cargo,args,{cwd:crate,env,maxBuffer:8*1024*1024});assert.ifError(r.error);return fs.writeFile(path.join(out,name+'.stdout.bin'),r.stdout).then(()=>fs.writeFile(path.join(out,name+'.stderr.bin'),r.stderr)).then(()=>{assert.equal(r.status,expected,r.stderr.toString());phases.push({name,args,exit_code:r.status,stdout_sha256:sha(r.stdout),stderr_sha256:sha(r.stderr)});return r;});}
const args=['build','--locked','--offline','-j','1','--target-dir',target],binary=path.join(target,'debug',process.platform==='win32'?'native-asset-build-probe.exe':'native-asset-build-probe');
async function sourceIdentity(){const values={};for(const n of ['Cargo.toml','build.rs','src/main.rs','src/input_assets.rs'])values[n]=sha(await fs.readFile(path.join(crate,n)));return values;}
function execute(name){const r=spawnSync(binary,name?[name]:[],{maxBuffer:1024*1024});assert.ifError(r.error);assert.equal(r.status,0,r.stderr.toString());return r.stdout;}
await run('lock',['generate-lockfile','--offline']);const identity=await sourceIdentity();
await run('initial',args);assert.equal(execute().toString(),'base.js\nworker.js\n');assert.deepEqual(execute('base.js'),Buffer.from(base));
const added="export const added = 'alpha';\n";await fs.writeFile(path.join(web,'added.js'),added);await run('addition',args);assert.equal(execute().toString(),'added.js\nbase.js\nworker.js\n');assert.deepEqual(execute('added.js'),Buffer.from(added));assert.deepEqual(await sourceIdentity(),identity);
const changed=added.replace('alpha','bravo');assert.equal(changed.length,added.length);await fs.writeFile(path.join(web,'added.js'),changed);await run('same-size-content-change',args);assert.deepEqual(execute('added.js'),Buffer.from(changed));assert.deepEqual(await sourceIdentity(),identity);
// Retain the removed fixture outside the watched web directory. No evidence is deleted.
const from=path.resolve(web,'added.js'),retained=path.resolve(out,'retained-added.js');assert(from.startsWith(out+path.sep)&&retained.startsWith(out+path.sep));await fs.rename(from,retained);await run('removal',args);assert.equal(execute().toString(),'base.js\nworker.js\n');assert.deepEqual(await fs.readFile(retained),Buffer.from(changed));assert.deepEqual(await sourceIdentity(),identity);
await fs.writeFile(path.join(web,'unsafe name.js'),'not evaluated');const rejected=await run('unsafe-filename',args,101);assert(rejected.stderr.toString().includes('Browser asset name is not portable.'));
const invalid=path.resolve(web,'unsafe name.js'),retainedInvalid=path.resolve(out,'retained-unsafe-name.js');assert(invalid.startsWith(out+path.sep)&&retainedInvalid.startsWith(out+path.sep));await fs.rename(invalid,retainedInvalid);await fs.mkdir(path.join(web,'directory.js'));const directory=await run('directory-with-js-extension',args,101);assert(directory.stderr.toString().includes('Browser JavaScript assets must be regular files.'));
assert.deepEqual(await sourceIdentity(),identity);
await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed',build_source_sha256:sha(buildSource),registry_source_sha256:sha(registrySource),fixture_source_identity:identity,phases,scope:'Exact production generator/registry compiled in a standalone Cargo probe. New files, same-size byte edits and removal recompile without Rust/manifest edits; unsafe names/nonfiles reject. All mutation inputs/logs/targets retained. Not a native server/browser or complete host/profile acceptance claim.'},null,2));
console.log('PASS native asset Cargo addition/content/removal and refusal controls: '+out);

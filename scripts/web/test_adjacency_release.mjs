import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {ADJACENCY_RELEASE_DEFAULTS,discoverAdjacencyRelease,acquireAdjacencyRelease,retireAdjacencyRelease} from '../../web/lab/adjacency-release.js';
import {nativeWorkspaceCase} from './fixtures/native-workspace-case.mjs';
const root=path.resolve(process.env.BISECT_RELEASE_TEST_OUTPUT||'target/adjacency-release/unit-'+Date.now());await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));assert.equal(ADJACENCY_RELEASE_DEFAULTS.repository,manifest.github_repo);assert.equal(ADJACENCY_RELEASE_DEFAULTS.tag,manifest.releases.data_inputs);
const scope={...ADJACENCY_RELEASE_DEFAULTS,state:'RI',year:'2020'},base='https://api.github.com/repos/'+scope.repository,fixture=nativeWorkspaceCase();
const sources=[...fixture.files].filter(([name])=>/\.adj\.bin$|_geoids\.json$|_centroids\.json$/.test(name)).map(([name,bytes],i)=>({id:i+1,name:path.basename(name),bytes}));
const assets=sources.map(s=>({id:s.id,name:s.name,size:s.bytes.length,state:'uploaded',url:base+'/releases/assets/'+s.id,digest:'sha256:'+createHash('sha256').update(s.bytes).digest('hex')}));
const json=value=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}}),requests=[];
function transport({assetRows=assets,release={id:99,tag_name:scope.tag,draft:false,prerelease:false},mutateBytes=null,paginated=false}={}){return async(url,options)=>{
 requests.push({url,credential_present:!!options.headers.Authorization});assert.equal(options.credentials,'omit');assert.equal(options.cache,'no-store');assert.equal(options.referrerPolicy,'no-referrer');assert(url.startsWith(base+'/'));assert(!url.includes('test_secret'));
 if(url.includes('/releases/tags/'))return json(release);
 if(url.includes('/releases/99/assets?')){if(paginated&&url.endsWith('page=1'))return json(Array.from({length:100},(_,i)=>({id:1000+i,name:'unrelated-'+i})));return json(assetRows);}
 const item=sources.find(s=>url===base+'/releases/assets/'+s.id);if(!item)return new Response('',{status:404});assert.equal(options.headers['X-GitHub-Api-Version'],undefined);assert.equal(options.headers.Accept,'application/octet-stream');const bytes=mutateBytes?mutateBytes(item):item.bytes;return new Response(bytes,{headers:{'content-type':'application/octet-stream','content-length':String(bytes.length)}});
};}
const fetcher=transport({paginated:true}),plan=await discoverAdjacencyRelease(scope,{fetcher,token:'test_secret'});assert.equal(plan.assets.length,3);assert.equal(plan.published_digests,3);assert(!JSON.stringify(plan).includes('test_secret'));assert(Object.isFrozen(plan.assets));
await assert.rejects(acquireAdjacencyRelease(structuredClone(plan),{fetcher}),/Discover/);
const imported=await acquireAdjacencyRelease(plan,{fetcher});assert.equal(imported.release_receipt.files.length,3);assert(imported.release_receipt.files.every(f=>f.published_digest_verified));assert(!JSON.stringify(imported).includes('test_secret'));assert.equal(imported.sources.geometry,null);assert(requests.some(r=>r.url.endsWith('page=2')));assert(requests.every(r=>r.credential_present));
await assert.rejects(acquireAdjacencyRelease(plan,{fetcher}),/Discover/);
await fs.writeFile(root+'/receipt.json',JSON.stringify(imported.release_receipt,null,2));
let refuses=0;
for(const overrides of [{release:{id:99,tag_name:'wrong',draft:false,prerelease:false}},{assetRows:[...assets,assets[0]]},{assetRows:assets.filter(a=>!a.name.endsWith('.adj.bin'))},{assetRows:assets.map((a,i)=>i? a:{...a,url:'https://localhost/private'})},{assetRows:assets.map((a,i)=>i? a:{...a,size:64*1024*1024+1})}]){await assert.rejects(discoverAdjacencyRelease(scope,{fetcher:transport(overrides)}));refuses++;}
const invalidDigest=await discoverAdjacencyRelease(scope,{fetcher:transport({assetRows:assets.map((a,i)=>i?a:{...a,digest:'sha256:'+'0'.repeat(64)})})});await assert.rejects(acquireAdjacencyRelease(invalidDigest,{fetcher:transport()}),/SHA-256/);refuses++;
const badLength=await discoverAdjacencyRelease(scope,{fetcher:transport()});await assert.rejects(acquireAdjacencyRelease(badLength,{fetcher:transport({mutateBytes:item=>item.bytes.subarray(1)})}),/length/);refuses++;
const retired=await discoverAdjacencyRelease(scope,{fetcher:transport()});retireAdjacencyRelease(retired);const count=requests.length;await assert.rejects(acquireAdjacencyRelease(retired,{fetcher:transport()}));assert.equal(requests.length,count);
const abort=new AbortController();abort.abort(new Error('cancelled'));await assert.rejects(discoverAdjacencyRelease(scope,{fetcher:transport(),signal:abort.signal}),/cancelled/);assert.equal(requests.length,count);
await assert.rejects(discoverAdjacencyRelease(scope,{fetcher:()=>new Promise(()=>{}),timeoutMs:10}),/timed out/);
await assert.rejects(discoverAdjacencyRelease(scope,{fetcher:async()=>new Response('',{status:404})}),/404/);
await assert.rejects(discoverAdjacencyRelease({...scope,repository:'https://localhost/x'},{fetcher:transport()}));
const noDigest=await discoverAdjacencyRelease(scope,{fetcher:transport({assetRows:assets.map(a=>({...a,digest:null}))})}),observed=await acquireAdjacencyRelease(noDigest,{fetcher:transport()});assert(observed.release_receipt.files.every(f=>f.published_digest===null&&!f.published_digest_verified));
await fs.writeFile(root+'/evidence.json',JSON.stringify({status:'passed',synthetic_transport:true,private_header_memory_only:true,pagination:true,complete_original_sources:true,digest_and_length_checked:true,no_digest_disclosed:true,refusals:refuses,retire_copy_reuse_abort_timeout_404_refused:true,partition_execution_performed:false,live_provider_success_claim:false},null,2)+'\n');
console.log('PASS release adapter: pagination, exact bounded bytes, digests, private-header isolation and refusal/cancellation boundaries. '+root);

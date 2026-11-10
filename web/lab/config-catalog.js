import {parseSafeJson,parseGeometryJson,parsePreparedJobsJson} from './project.js?v=label-build-archive-v1';
import {encodeBytes} from './package-files.js';
const MAX=8*1024*1024;
async function bytes(response,limit){
 if(!response.ok)throw new Error('Published input unavailable. Local graph import remains available.');
 const declared=response.headers.get('content-length');if(declared!==null&&(!/^\d+$/.test(declared)||Number(declared)>limit))throw new Error('Published input exceeds the portable size limit.');
 const reader=response.body.getReader(),chunks=[];let size=0;
 try{while(true){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>limit)throw new Error('Published input exceeds the portable size limit.');chunks.push(value);}}catch(error){await reader.cancel();throw error;}finally{reader.releaseLock();}
 const out=new Uint8Array(size);let offset=0;for(const chunk of chunks){out.set(chunk,offset);offset+=chunk.length;}return out;
}
const text=raw=>new TextDecoder('utf-8',{fatal:true}).decode(raw);
async function getBytes(fetcher,path,limit){
 const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),30000);
 try{return await bytes(await fetcher(path,{signal:controller.signal}),limit);}finally{clearTimeout(timer);}
}
export async function readConfigCatalog(fetcher=fetch){
 const manifest=parseSafeJson(text(await getBytes(fetcher,'../catalog.json',1024*1024)));
 if(manifest.schema_version!==1||!manifest.graphs||!manifest.assets||!manifest.geometries||!Array.isArray(manifest.catalog?.states))throw new Error('Invalid published input catalog.');
 const entries=[];
 for(const [key,entry] of Object.entries(manifest.graphs)){
  if(!/^[A-Z]{2}:\d{4}$/.test(key))throw new Error('Invalid published graph scope.');
  const [state,year]=key.split(':'),info=manifest.catalog.states.find(s=>s.code===state),geometry=manifest.geometries[key];
  for(const ref of [entry.graph_ref,geometry])if(typeof ref!=='string'||!/^assets\/[a-f0-9]{64}\.json$/.test(ref)||manifest.assets[ref]!==ref.slice(7,-5))throw new Error('Invalid published asset identity.');
  const allocation=info?.years?.find(y=>String(y.year)===year)?.districts;
  if(!info||typeof info.name!=='string'||!allocation)throw new Error('Missing published district allocation.');
  entries.push({key,state,year,name:info.name,districts:allocation});
 }
 if(!entries.length||entries.length>150)throw new Error('No supported published state inputs.');
 return {manifest,entries:entries.sort((a,b)=>a.name.localeCompare(b.name)||a.year.localeCompare(b.year))};
}
export async function loadConfigCatalogInputs(catalog,key,chamber,fetcher=fetch){
 const selected=catalog.entries.find(e=>e.key===key),manifest=catalog.manifest;
 if(!selected||!['congressional','house','senate'].includes(chamber))throw new Error('Choose an available state, year and chamber.');
 const districts=selected.districts[chamber];if(!Number.isSafeInteger(districts)||districts<1||districts>500)throw new Error('District allocation unavailable for this chamber.');
 const refs=[manifest.graphs[key].graph_ref,manifest.geometries[key]],raw=[];
 for(const [i,ref] of refs.entries()){const data=await getBytes(fetcher,'../'+ref,i===0?MAX:16*1024*1024);const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',data)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==manifest.assets[ref])throw new Error('Published input failed its SHA-256 identity check.');raw.push(data);}
 const graph=parseSafeJson(text(raw[0])),geometry=parseGeometryJson(text(raw[1]));
 if(graph.schema_version!==1||graph.state!==selected.state||graph.year!==selected.year||!Array.isArray(graph.geoids))throw new Error('Published graph scope mismatch.');
 // Require a complete matching geometry join before replacing current sources.
 const ids=geometry.features.map(f=>String(f.properties?.GEOID??f.properties?.geoid??f.id));
 const set=new Set(ids);if(ids.length!==graph.geoids.length||set.size!==ids.length||new Set(graph.geoids).size!==graph.geoids.length||!graph.geoids.every(id=>set.has(id)))throw new Error('Published map does not match graph units.');
 return {graph:{name:`${selected.state}-${selected.year}-graph.json`,base64:encodeBytes(raw[0])},geometry:{name:`${selected.state}-${selected.year}-map.geojson`,base64:encodeBytes(raw[1])},settings:{chamber,districts},selected};
}

// Build preparation needs graphs and chamber allocations, without downloading map geometry.
export async function loadConfigCatalogJobs(catalog,keys,chamber,fetcher=fetch){
 if(!Array.isArray(keys)||!keys.length||keys.length>150||new Set(keys).size!==keys.length||!['congressional','house','senate'].includes(chamber))throw new Error('Choose distinct published state/year jobs and a chamber.');
 const entries=keys.map(key=>catalog.entries.find(e=>e.key===key));
 if(entries.some(e=>!e||!Number.isSafeInteger(e.districts[chamber])||e.districts[chamber]<1||e.districts[chamber]>500))throw new Error('District allocation unavailable for a selected job.');
 const jobs=[];let sourceBytes=2;
 for(const selected of entries){
  const ref=catalog.manifest.graphs[selected.key].graph_ref;
  if(typeof ref!=='string'||!/^assets\/[a-f0-9]{64}\.json$/.test(ref)||catalog.manifest.assets[ref]!==ref.slice(7,-5))throw new Error('Invalid published graph identity.');
  const raw=await getBytes(fetcher,'../'+ref,MAX);
  const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',raw)),b=>b.toString(16).padStart(2,'0')).join('');
  if(hash!==catalog.manifest.assets[ref])throw new Error('Published graph failed its SHA-256 identity check.');
  const graph=parseSafeJson(text(raw));
  if(graph.schema_version!==1||graph.state!==selected.state||graph.year!==selected.year||!Array.isArray(graph.geoids))throw new Error('Published graph scope mismatch.');
  const job={graph,districts:selected.districts[chamber],chamber,demographics:null};
  sourceBytes+=new TextEncoder().encode(JSON.stringify(job)).length+(jobs.length?1:0);
  if(sourceBytes>16*1024*1024)throw new Error('Selected jobs exceed the 16 MiB prepared-source limit.');
  jobs.push(job);
 }
 const json=JSON.stringify(jobs);parsePreparedJobsJson(json);
 return {source:{name:'published-'+chamber+'-jobs.json',base64:encodeBytes(new TextEncoder().encode(json))},years:[...new Set(entries.map(e=>e.year))],selected:entries};
}

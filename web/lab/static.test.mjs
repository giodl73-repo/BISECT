import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {StaticCatalog,effectiveConfig} from './static.js';

const configuration = {name:'Fixture',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,structure:'nway',weights:'geographic',search:'single',seed:42,seeds:8,steps:200,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,timeout_seconds:300};
function fixture(corrupt=false) {
  const data=new Map(), assets={};
  function asset(value) {const bytes=JSON.stringify(value),hash=createHash('sha256').update(bytes).digest('hex'),ref=`assets/${hash}.json`;assets[ref]=hash;data.set(ref,bytes);return ref;}
  const geometry_ref=asset({type:'FeatureCollection',state:'RI',year:'2020',features:[{properties:{geoid:'01001000100',state:'RI',county:'01001'},geometry:{type:'MultiPolygon',coordinates:[]}}]});
  const assignments_ref=asset({'01001000100':1});
  const state={code:'RI',status:'completed',metrics:{units:1,district_count:2},geometry_ref,assignments_ref,effective_config:effectiveConfig(configuration,2)};
  const manifest={schema_version:1,assets,geometries:{'RI:2020':geometry_ref},catalog:{states:[{code:'RI',years:[{year:'2020',districts:{congressional:2}}]}]},runs:[{id:'original',created_unix:1,config:configuration,states:[state],status:'completed'}]};
  return new StaticCatalog(manifest,async ref=>new Response(corrupt ? '{}' : data.get(ref)));
}
test('saved map uses a shared geometry and preserves string GEOIDs',async()=>{
  const client=fixture(),map=await client.api('/api/runs/original/RI/map');
  assert.equal(map.features[0].properties.geoid,'01001000100');assert.equal(map.features[0].properties.district,1);
  assert.equal((await client.api('/api/geometry?state=RI&year=2020')).features[0].properties.district,undefined);
});
test('exact matching does not substitute another seed or hide missing states',()=>{
  const client=fixture();assert.equal(client.coverage({...configuration,seeds:10000,steps:10000}),1);
  assert.equal(client.coverage({...configuration,seed:43}),0);
  const selected=client.select({...configuration,mode:'national',states:['RI','IA']});
  assert.equal(selected.status,'partial');assert.equal(selected.states[1].status,'unavailable');assert.equal(selected.states[0].source_run_id,'original');
});
test('asset corruption and unsupported schema fail closed',async()=>{
  await assert.rejects(fixture(true).api('/api/runs/original/RI/map'),/integrity/);
  assert.throws(()=>new StaticCatalog({schema_version:2}),/version/);
  const client=fixture();assert.throws(()=>new StaticCatalog({...client.manifest,assets:{'../private.json':'wrong'}}),/reference/);
});
test('static lookup performs no remote mutation',async()=>{
  const client=fixture();client.fetcher=()=>{throw new Error('Unexpected request');};
  const saved=await client.api('/api/runs',{method:'POST',body:JSON.stringify(configuration)});
  assert.equal(saved.status,'completed');await assert.rejects(client.api('/api/runs/original/cancel',{method:'POST',body:'{}'}),/cannot execute/);
});

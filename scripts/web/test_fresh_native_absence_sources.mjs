import assert from 'node:assert/strict';
import {freshNativeAuxiliaryKinds,freshNativeAuxiliaryJson,freshNativeAuxiliarySources,validateFreshNativeMissingSource} from '../../web/lab/fresh-native-state-auxiliary.js';
import {validateFreshNativeStateSession} from '../../web/lab/fresh-native-state-session.js';
let positives=0,refusals=0;
const absent={name:'recorded-missing.csv',missing:true};
const session=sources=>({schema_version:'bisect-fresh-native-state-session-v1',fields:{arguments_json:'[]',state:'AL',year:'2020',label:'Missing source test',chamber:'congressional',created_at:'2026-11-03T00:00:00Z'},sources,display_geometry_json:null,retained_run:null,runtime_artifact:null});
for(const kind of freshNativeAuxiliaryKinds){
 const sources={[kind]:absent};assert.deepEqual(freshNativeAuxiliarySources(freshNativeAuxiliaryJson(sources)),sources);assert.deepEqual(validateFreshNativeStateSession(session(sources)).sources,sources);positives+=2;
 const selected={[kind]:{name:'selected.csv',base64:'eA=='}};assert.deepEqual(freshNativeAuxiliarySources(freshNativeAuxiliaryJson(selected)),selected);positives++;
}
for(const value of [null,{},[],{name:'missing.csv'},{missing:true},{name:'',missing:true},{name:'  ',missing:true},{name:'a\0b',missing:true},{name:'x',missing:false},{name:'x',missing:'true'},{name:'x',missing:true,base64:'eA=='},{name:'x',missing:true,grant:true},{name:'😀'.repeat(1025),missing:true}]){
 assert.throws(()=>validateFreshNativeMissingSource(value));refusals++;
}
for(const kind of ['adjacency','geoids','centroids','fine_adjacency','fine_geoids','fine_centroids']){assert.throws(()=>validateFreshNativeStateSession(session({[kind]:absent})));refusals++;}
for(const value of [{name:'missing.csv',missing:false},{name:'missing.csv',missing:true,base64:'eA=='},{name:'',missing:true}]){
 assert.throws(()=>freshNativeAuxiliaryJson({economic:value}));assert.throws(()=>validateFreshNativeStateSession(session({economic:value})));refusals+=2;
}
for(const aux of [{economic:{source_name:'',bytes_base64:null}},{economic:{source_name:'missing.csv',bytes_base64:null,grant:true}},{economic:{source_name:'missing.csv'}},{unknown:{source_name:'missing.csv',bytes_base64:null}}]){
 assert.throws(()=>freshNativeAuxiliarySources(JSON.stringify({native_auxiliary:aux})));refusals++;
}
assert.equal(freshNativeAuxiliaryJson({}),null);
console.log(JSON.stringify({status:'passed-explicit-absence-source-boundaries',positives,refusals,absence_inferred:false}));

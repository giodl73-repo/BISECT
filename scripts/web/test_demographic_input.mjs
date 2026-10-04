import assert from 'node:assert/strict';
import {validateDemographicInput,alignDemographicFractions,demographicIdentity,tractMeanFractions,districtCountShares} from '../../web/lab/demographic-input.js';
const graph={state:'RI',year:'2020',geoids:['44001000002','44001000001']};
const input={schema_version:'bisect-demographic-fractions-v1',state:'RI',year:'2020',basis:'total-population',source_label:'Explicit synthetic fixture',minority_fractions:{'44001000001':0,'44001000002':1}};
assert.deepEqual(alignDemographicFractions(input,graph),[1,0]);
assert.deepEqual(tractMeanFractions(graph,{'44001000001':1,'44001000002':1},input,1),[0.5]);
const identity=await demographicIdentity(input);
assert.equal(await demographicIdentity({...input,source_label:'Relabeled',minority_fractions:{'44001000002':1,'44001000001':0}}),identity);
assert.notEqual(await demographicIdentity({...input,basis:'voting-age-population'}),identity);
for(const change of [{schema_version:'unknown'},{state:'XX'},{year:'2010'},{basis:'unverified'},{source_label:'é'.repeat(101)},{unknown:1},{minority_fractions:{'44001000001':0}},{minority_fractions:{'44001000001':0,'44001000003':1}}])assert.throws(()=>validateDemographicInput({...input,...change},graph));
for(const fraction of [-1,-0,1.01,NaN,Infinity])assert.throws(()=>validateDemographicInput({...input,minority_fractions:{...input.minority_fractions,'44001000001':fraction}},graph));
assert.throws(()=>tractMeanFractions(graph,{'44001000001':1,'44001000002':1},input,2),/empty/);
console.log('Demographic inputs: strict schema/scope/coverage, fraction bounds, ordered alignment, unweighted means, canonical identity and user-label independence passed.');

const counted={...input,schema_version:'bisect-demographic-counts-v2',counts:{'44001000001':{total:800,minority:0},'44001000002':{total:400,minority:400}}};
assert.deepEqual(validateDemographicInput(counted,graph),counted);
assert.deepEqual(districtCountShares(graph,{'44001000001':1,'44001000002':1},counted,1),[{district:1,total:1200,minority:400,share:1/3}]);
assert.equal(districtCountShares(graph,{},input,1),null);
assert.equal(await demographicIdentity(counted),identity,'Counts do not change the fraction-based search identity');
for(const change of [c=>delete c.counts['44001000001'],c=>c.counts['44001000001'].minority=1,c=>c.counts['44001000002'].total=-0,c=>c.counts['44001000001'].extra=1,c=>c.counts['44001000002'].total=Infinity,c=>c.counts['44001000002'].minority=401,c=>c.schema_version='bisect-demographic-fractions-v1']){const bad=structuredClone(counted);change(bad);assert.throws(()=>validateDemographicInput(bad,graph));}
console.log('Count inputs: backward-compatible fraction identity, complete raw-count retention and count/fraction mismatch rejection passed.');

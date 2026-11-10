import {parseSafeJson} from './project.js';
import {validateDemographicInput} from './demographic-input.js';
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
const same=(a,b)=>Array.isArray(b)?Array.isArray(a)&&a.length===b.length&&b.every((v,i)=>same(a[i],v)):b&&typeof b==='object'?exact(a,Object.keys(b))&&Object.keys(b).every(k=>same(a[k],b[k])):a===b;
export function validateNativeStateVra(v){
 if(!exact(v,['schema_version','basis','minority_mass','threshold','comparison','subgroup_inputs','demographics_sha256','analysis','analysis_json'])||v.schema_version!=='bisect-native-state-vra-v1'||v.basis!=='total-population'||v.minority_mass!=='fraction-times-prepared-unit-population'||v.threshold!==.5||v.comparison!=='strictly-greater'||v.subgroup_inputs!=='native-zero-placeholders'||!/^[a-f0-9]{64}$/.test(v.demographics_sha256)||!exact(v.analysis,['mm_count','mm_districts','districts'])||!Array.isArray(v.analysis.districts)||v.analysis.districts.length<2||v.analysis.districts.length>500)throw new Error('Invalid native State minority summary.');
 const a=v.analysis;
 for(const [i,d]of a.districts.entries())if(!exact(d,['district','pct_minority','pct_black','pct_hispanic','is_mm'])||d.district!==i+1||!Number.isFinite(d.pct_minority)||d.pct_minority<0||d.pct_minority>1||d.pct_black!==0||d.pct_hispanic!==0||d.is_mm!==(d.pct_minority>.5))throw new Error('Native State minority district summary disagrees with its threshold.');
 const ids=a.districts.filter(d=>d.is_mm).map(d=>d.district);if(a.mm_count!==ids.length||!same(a.mm_districts,ids)||typeof v.analysis_json!=='string'||new TextEncoder().encode(v.analysis_json).length>256*1024||!same(parseSafeJson(v.analysis_json),a))throw new Error('Native State minority JSON disagrees with its summary.');return v;
}
export function verifyNativeStateVra(v,graph,config,assignments,k){
 if(v===undefined)return; // Older projects predate the native State summary.
 validateNativeStateVra(v);if(config.structure!=='nway'||config.weights!=='vra-aligned'||config.search!=='single'||k<2)throw new Error('Unexpected native State minority summary.');
 const input=validateDemographicInput(config.demographics[graph.state],graph);if(input.basis!=='total-population')throw new Error('Native State summary requires total-population fractions.');
 const total=Array(k).fill(0),minority=Array(k).fill(0);for(const [i,id]of graph.geoids.entries()){const d=assignments[id]-1;if(!Number.isSafeInteger(d)||d<0||d>=k)throw new Error('Invalid native State minority assignment.');total[d]+=graph.population[i];minority[d]+=input.minority_fractions[id]*graph.population[i];}
 const districts=total.map((population,i)=>{const share=population>0?minority[i]/population:0;return {district:i+1,pct_minority:share,pct_black:0,pct_hispanic:0,is_mm:share>.5};}),mm_districts=districts.filter(d=>d.is_mm).map(d=>d.district);
 if(!same(v.analysis,{mm_count:mm_districts.length,mm_districts,districts}))throw new Error('Native State minority summary differs from saved populations, fractions or assignments.');
}

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const root='target/partisan-vra-boundaries';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(source+'/catalog.json','utf8'));
let cases=0;
for(const code of ['RI','IA'])for(const mode of ['absent','coi'])for(const search of ['single','multi']){
 const graph=JSON.parse(await fs.readFile(source+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8'));
 const partisan={schema_version:'bisect-partisan-shares-v1',state:code,year:'2020',source_label:'SYNTHETIC boundary shares',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%4<2?.8:.2]))};
 const shares=root+'/shares-'+cases+'.tsv';await fs.writeFile(shares,'geoid\tdem_share\n'+graph.geoids.map(id=>id+'\t'+partisan.dem_shares[id]).join('\n')+'\n');
 const request={graph,partisan,options:{structure:'ratio-optimal-vra',search,weights:'partisan',districts:3,seed:'18446744073709551615',seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45,w_vra:.4}};
 const fixture={graph,state_args:['--state',code,'--year','2020','--partition-mode','partisan-weighted','--structure','ratio-optimal-vra','--partisan-shares',path.resolve(shares),'--districts','3','--search',search,'--seeds','3','--niter','10','--balance-tolerance','25','--w-vra','.4','--metis-engine','metis-core','--seed','18446744073709551615']};
 if(mode==='absent')request.options.vra_absent_file_fallback=true;
 else{
  const csv=root+'/demographics-'+cases+'.csv';await fs.writeFile(csv,'GEOID,total_pop,white_non_hispanic\n'+graph.geoids.map(id=>id+',100,40').join('\n')+'\n');fixture.demographics_path=path.resolve(csv);
  request.demographics={schema_version:'bisect-demographic-fractions-v1',state:code,year:'2020',basis:'total-population',source_label:'SYNTHETIC minority fractions',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,.6]))};
  const weights_json=JSON.stringify({[graph.geoids[0]]:9,[graph.geoids[1]]:0,'unknown-parity-unit':4})+'\n',coi=root+'/coi-'+cases+'.json';await fs.writeFile(coi,weights_json);fixture.coi_path=path.resolve(coi);request.coi={source_name:'SYNTHETIC COI.json',weights_json};
 }
 const file=root+'/native-'+cases+'.json';await fs.writeFile(file,JSON.stringify(fixture));const n=spawnSync('target/release/examples/partisan_structure_reference.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(n.status,0,n.stderr);
 const out=engine.execute(request),expected=JSON.parse(n.stdout);assert.deepEqual(out.assignments,expected.assignments);assert(Math.abs(out.metrics.root_split.native_candidate_cut-expected.root.cut)<1e-8);assert(Math.abs(out.metrics.weighted_boundary-expected.weighted_boundary)<1e-8);
 if(mode==='absent'){assert.equal(out.metrics.structure_evidence.method,'vra-section-absent-file-fallback');assert.equal(out.metrics.structure_evidence.alignment_bonus_used,false);}
 else assert.equal(out.metrics.coi_weighting_evidence.ignored_units,1);
 cases++;
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_absent_or_coi_state_comparisons:cases,explicit_absence_no_alignment_bonus:true,coi_sparse_weight_composition:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Partisan VRA boundaries: ${cases} exact native absent-demographics/COI comparisons with preserved fallback and sparse cost semantics.`);

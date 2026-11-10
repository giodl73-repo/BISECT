import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {stateCommandPreparationRequest,preparedEngineFromStateCommand} from '../../web/lab/state-command-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const oracleAt=process.argv.indexOf('--native-oracle'),nativeOracle=oracleAt>=0?process.argv[oracleAt+1]:'target/release/examples/state_signal_preset_reference.exe';
const root=path.resolve('target/native-state-run');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const old=JSON.parse(await fs.readFile('target/native-signal-presets/records.json','utf8'));
const base=structuredClone(old.find(r=>r.mode==='partisan-weighted'&&r.districts===4).request);delete base.partisan;
base.options={structure:'standard-bisect',weights:'geographic',search:'single',districts:4,seed:42,seeds:1,steps:200,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
base.graph.areas=Array(36).fill(10000);base.graph.exterior_perimeters=Array(36).fill(50);base.graph.centroids=Array.from({length:36},(_,i)=>[-71+(i%6)*.01,41+Math.floor(i/6)*.01]);
const settings={label:'Native State flags',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true};
const common=['--state','RI','--metis-engine','metis-core','--districts','4','--balance-tolerance','25','--niter','10'];
const prepared=request=>({source:{name:'original.json',text:JSON.stringify(request)},settings});
const command=args=>({arguments_json:JSON.stringify(args)});
const records=[];let hosts=0,native=0,executed=0;
function runHost(request,name){const file=root+'/'+name+'.json';return fs.writeFile(file,JSON.stringify(request)).then(()=>{const p=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(p.status,0,p.stderr);return JSON.parse(p.stdout);});}
async function compile(name,args,template=base,run=false){
 const inputs=prepared(template),source=command(args),request=stateCommandPreparationRequest(source,inputs),result=engine.execute(request);
 assert.equal(result.execution_performed,false);assert.equal(result.filesystem_reads_performed,false);
 assert.deepEqual(result.native_command.arguments,args);assert.equal(result.prepared_review.execution_performed,false);
 const selected=await preparedEngineFromStateCommand(source,inputs,result),compiled=JSON.parse(selected.source.text);
 assert.equal(selected.settings.inspect_only,true);assert.equal(compiled.operation,compiled.operation.includes('multiscale')?'run-multiscale-build':'run-engine-build');
 if(!process.argv.includes('--prepare')){const h=await runHost(request,name+'.compile');compareNative(request,result,h);hosts++;}
 let execution=null;
 if(run){const request={operation:'run-prepared-engine',input:{request_json:selected.source.text,...settings,inspect_only:false}};execution=engine.execute(request);assert.equal(execution.execution_performed,true);assert(execution.result.build_report);assert(execution.export?.document_json,execution.export_error);if(!process.argv.includes('--prepare')){compareNative(request,execution,await runHost(request,name+'.run'));hosts++;}executed++;}
 records.push({name,request,result,execution});return {result,compiled,source,inputs,selected,execution};
}
// Every legacy mode and structure is bound, including defaults specific to it.
const modes=['unweighted','edge-weighted','metis-vra','partisan-weighted','proportional','compact-bisect','geosection','areasection','proportional-section','apportion-regions','vra-section','simulated-annealing','bfs-growth','centroidal-voronoi','ilp','moving-knife','capacity-clustering','spectral','regionalization','flow-construction'];
for(const mode of modes){
 const template=structuredClone(base),args=[...common,'--partition-mode',mode,'--seeds','2','--geosection-seeds','2','--compact-seeds','2','--sa-steps-per-tract','1','--spectral-iters','5','--mka-orientations','4'];
 if(mode==='apportion-regions')args[args.indexOf('--balance-tolerance')+1]='3';
 if(['capacity-clustering','regionalization','flow-construction'].includes(mode)){const n=6;template.graph={...template.graph,geoids:template.graph.geoids.slice(0,n),population:Array(n).fill(100),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(j=>j>=0&&j<n)),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100]),areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(50),centroids:template.graph.centroids.slice(0,n)};args[args.indexOf('--districts')+1]='2';}
 if(mode==='metis-vra')template.demographics=structuredClone(old.find(r=>r.mode==='metis-vra').request.demographics);
 if(['partisan-weighted','proportional'].includes(mode)){template.partisan=structuredClone(old[0].request.partisan);args.push('--partisan-shares',template.partisan.source_label);}
 if(mode==='proportional-section')template.elections={schema_version:'bisect-election-counts-v1',state:'RI',year:'2020',election_year:'2020',source_label:'Synthetic votes',counts:Object.fromEntries(template.graph.geoids.map((id,i)=>[id,{democratic:50,two_party:100}]))};
 await compile('mode-'+mode,args,template,true);
}
const structures=['standard-bisect','nway','ratio-optimal','ratio-optimal-area','ratio-optimal-vra','prime-factor','compact-polsby','centroidal-voronoi','bfs-growth','moving-knife','capacity-clustering','spectral','regionalization','flow-construction'];
for(const structure of structures){
 const mode=({'prime-factor':'apportion-regions','capacity-clustering':'capacity-clustering','regionalization':'regionalization','flow-construction':'flow-construction'})[structure];
 const template=mode?JSON.parse(records.find(r=>r.name==='mode-'+mode).request.input.prepared.request_json):base;
 const args=[...common,'--structure',structure,'--search','single','--seed','18446744073709551615','--mka-orientations','4','--spectral-iters','5'];
 if(mode==='apportion-regions')args[args.indexOf('--balance-tolerance')+1]='3';
 if(['capacity-clustering','regionalization','flow-construction'].includes(structure))args[args.indexOf('--districts')+1]='2';
 await compile('structure-'+structure,args,template,true);
}
const searches=['single','multi','convergence','percentile','bisection-ensemble','flip','short-burst','short-burst-forest','short-burst-merge-split','forest-recom','merge-split','parallel-tempering','vra-recom','smc-percentile'];
for(const search of searches){
 const args=[...common,'--search',search,'--seeds','2','--convergence-threshold','2','--percentile','.5','--ensemble-steps','2','--flip-steps','2','--forest-steps','2','--merge-split-steps','2','--burst-length','2','--n-bursts','2','--pt-replicas','2','--pt-swap-interval','1','--particles','4'];
 await compile('search-'+search,args,base,true);
}
for(const search of ['multiscale','multiscale-adaptive']){
 const template=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-24-0.json','utf8'));
 template.fine_edges=template.input.block_groups.adjacency.flatMap((ns,u)=>ns.filter(v=>u<v).map(v=>[u,v,1])); // Explicit synthetic fine boundaries for export.
 const args=[...common,'--districts','3'];args.splice(args.indexOf('--districts'),2);args[args.indexOf('--state')+1]=template.input.request.graph.state; // Native clap forbids duplicate flags.
 args.push('--search',search,'--seeds','2','--multiscale-steps','2','--multiscale-fine','bg','--multiscale-coarse','tract','--multiscale-alpha','.2','--ms-target-accept','.3','--ms-adapt-interval','1');
 const c=await compile('search-'+search,args,template,true);assert.equal(c.compiled.input.alpha,search==='multiscale'?.2:.3);
}
for(const weights of ['unweighted','geographic','county','vra-aligned','proportional','economic-character','housing-character','zone-membership']){
 const template=structuredClone(base),args=[...common,'--search','single','--weights-override',weights,'--alpha-county','.125'];
 if(weights==='proportional'){template.partisan=structuredClone(old[0].request.partisan);args.push('--partisan-shares',template.partisan.source_label);}
 if(weights==='vra-aligned')template.demographics=structuredClone(old.find(r=>r.mode==='metis-vra').request.demographics);
 const c=await compile('weights-'+weights,args,template,true);if(weights==='county')assert.equal(c.compiled.request.options.county_augmentation,1);
}
// Independent native State parser, native file readers and actual CLI kernels.
for(const [i,row]of old.entries()){
 const raw=JSON.parse(await fs.readFile('target/native-signal-presets/'+i+'.json','utf8')),template=structuredClone(row.request);
 if(template.partisan)template.partisan.source_label=raw.state_args[raw.state_args.indexOf('--partisan-shares')+1];
 const c=await compile('native-'+i,raw.state_args,template,false),request=c.compiled.request,result=engine.execute(request);
 const file=root+'/native-'+i+'.reference';await fs.writeFile(file,JSON.stringify(raw));const p=spawnSync(path.resolve(nativeOracle),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:32*1024*1024});assert.equal(p.status,0,p.stderr);const reference=JSON.parse(p.stdout);assert.deepEqual(result.assignments,reference.assignments);assert.equal(result.metrics.native_state_vra?.analysis_json??null,reference.native_vra_json);native++;
}
// Graph scope, source binding, unsupported side effects and native profile policy.
for(const extra of [['--reset'],['--output-dir','never-read'],['--force'],['--print-only'],['--state','IA'],['--resolution','block'],['--profile','court','--partisan-shares','unread.tsv'],['--partition-mode','partisan-weighted','--partisan-shares','unread.tsv'],['--metis-engine','gpmetis']]){
 let args=[...common];if(extra[0]==='--state'||extra[0]==='--metis-engine'){const at=args.indexOf(extra[0]);args.splice(at,2);}args.push(...extra);assert.throws(()=>engine.execute(stateCommandPreparationRequest(command(args),prepared(base))),undefined,JSON.stringify(extra));
}
// C-capable preparation is inert and target-specific: the native host adapter
// has no browser bridge. Actual C execution is covered by test_cffi_engine.
const cArgs=[...common];cArgs[cArgs.indexOf('--metis-engine')+1]='c-ffi';
const cRequest=stateCommandPreparationRequest(command(cArgs),prepared(base));
if(engine.supportsCMetis){
 const review=engine.execute(cRequest),selected=await preparedEngineFromStateCommand(command(cArgs),prepared(base),review),compiled=JSON.parse(selected.source.text);
 assert.equal(review.prepared_review.execution_performed,false);assert.equal(compiled.request.options.metis_engine,'c-ffi');assert.equal(compiled.request.options.metis_runtime,'msvcrt-v1');
 await fs.writeFile(root+'/c-capability-preparation.json',JSON.stringify({original_request:cRequest,review,selected,execution_performed:false},null,2)+'\n');
}else assert.throws(()=>engine.execute(cRequest),/METIS backend/);
for(const year of ['2024','9999']){const template=structuredClone(base);template.graph.year=year;assert.throws(()=>engine.execute(stateCommandPreparationRequest(command([...common,'--year',year]),prepared(template))),/not available/);}
const inferred=[...common];inferred.splice(inferred.indexOf('--districts'),2);inferred[inferred.indexOf('--state')+1]='ri';const allocation=await compile('native-policy-count',inferred,base,false);assert.equal(allocation.compiled.request.options.districts,2);assert.equal(allocation.result.native_command.state,'ri');
const metadata=await compile('native-unused-and-electoral',[...common,'--search','single','--objtype','vol','--ufactor','27','--target-mm-districts','2','--seats-per-district','0','--total-seats','12'],base,false);assert.equal(metadata.compiled.request.options.metis_objective,undefined);assert.equal(metadata.compiled.request.options.metis_ufactor,undefined);assert.deepEqual(metadata.compiled.request.electoral,{seats_per_district:0,total_seats:12});
for(const [resolution,kind,suffix]of [['block_group','block-group','1'],['block','block','1000']]){const template=structuredClone(base);template.graph.unit_kind=kind;template.graph.geoids=template.graph.geoids.map(id=>id+suffix);const c=await compile('resolution-'+resolution,[...common,'--resolution',resolution,'--search','single'],template,true);assert.equal(c.execution.executed_request.graph.unit_kind,kind);}
const c=await compile('max-seed',[...common,'--search','single','--seed','18446744073709551615'],base,true);assert.equal(c.compiled.request.options.seed,'18446744073709551615');
await assert.rejects(preparedEngineFromStateCommand(c.source,c.inputs,{...c.result,prepared_request_json:c.result.prepared_request_json+' '}));
await assert.rejects(preparedEngineFromStateCommand(command([...common,'--search','multi']),c.inputs,c.result));
const project=createProject({name:'Native State flags to engine',files:{stateCommandInputs:c.source,preparedEngineInputs:c.inputs},operation:'prepare-native-state-run',constraints:[],result:null,lastOperation:null});assert.equal(parseProject(JSON.stringify(project)).operation,'prepare-native-state-run');await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/records.json',JSON.stringify(records));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,compiled:records.length,full_native_host_matches:hosts,executed_with_native_retries_and_exports:executed,independent_native_state_assignment_matches:native,all20_legacy_modes:true,all14_structure_overrides:true,all16_searches:true,all8_weight_overrides:true,inert_preparation:true,all3_native_resolutions_executed:true,unsafe_or_unbound_flags_refused:true,max_u64_seed:true,compiled_hash_and_source_guard:true,published:false},null,2)+'\n');console.log(`PASS native State Run: ${records.length} compilations, ${executed} runs/exports, ${native} independent native State matches, ${hosts} whole host matches`);

import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {execFileSync} from 'node:child_process';
import {createWorkbenchArchive,parseWorkbenchArchive,SHARED_WORKBENCH_ARCHIVE_SCHEMA} from '../../web/lab/workbench-archive.js';
import {tuiRunSources} from '../../web/lab/tui-run-files.js';import {analysisPackageRequest} from '../../web/lab/analysis-package.js';import {nativeMapRequest} from '../../web/lab/map-export-files.js';
const root='target/shared-workbench-archive';await fs.mkdir(root,{recursive:true});
const original=JSON.parse(await fs.readFile('target/large-source/handoff.bisect-workbench','utf8')),run=JSON.parse(await fs.readFile('target/large-source/run.json','utf8'));
const files={...original.files,...tuiRunSources(run),tuiRunWorkflowSettings:{analysis:true,map:true,report:true,dpi:'72',dissolve_method:'batch'}};
files.compactnessInputs={geometry:{...files.planGeometry,name:'analysis-original.geojson'},coordinate_basis:'nad83-degrees',crs_label:'EPSG:4269',dissolve_method:'batch'};
files.statisticsInputs={demographic:null,urban:null,election:null,tolerance_percent:run.configured_run.configuration.execution_tolerance_percent};
files.electionAnalysisSettings={bootstrap_seed:'0',bootstrap_replicates:1000};
files.nativeMapInputs={plan:{...files.auditPlanText,name:'map-original.rplan'},geometry:{...files.planGeometry,name:'map-original.geojson'},analysis:null,dpi:72,map_type:'districts',dissolve_method:'batch'};
const beforeAnalysis=analysisPackageRequest(files,files.statisticsInputs,files.compactnessInputs,files.electionAnalysisSettings),beforeMap=nativeMapRequest(files.nativeMapInputs),source={...original,files,result:run,operation:'analyze-plan-package',lastOperation:'run-tui-configured'};
const expandedBytes=Buffer.byteLength(JSON.stringify(source));assert(expandedBytes>128*1024*1024);
const text=createWorkbenchArchive(source);assert.equal(JSON.parse(text).schema_version,SHARED_WORKBENCH_ARCHIVE_SCHEMA);const restored=parseWorkbenchArchive(text);
assert.deepEqual(restored.files,files);assert.deepEqual(restored.result,run);assert.deepEqual(analysisPackageRequest(restored.files,restored.files.statisticsInputs,restored.files.compactnessInputs,restored.files.electionAnalysisSettings),beforeAnalysis);assert.deepEqual(nativeMapRequest(restored.files.nativeMapInputs),beforeMap);
await fs.writeFile(root+'/real-ri-analysis.bisect-workbench',text);await fs.writeFile(root+'/real-ri-analysis-request.json',JSON.stringify(beforeAnalysis));
if(process.argv.includes('--native-host')){
 const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/real-ri-analysis-request.json'],{encoding:'utf8',maxBuffer:128*1024*1024}));await fs.writeFile(root+'/real-ri-analysis.json',JSON.stringify(native));assert.equal(native.report_ready,true);assert.equal(native.steps.find(s=>s.name==='compactness').status,'computed');
 for(const name of ['compactness.json','summary.json','contiguity.json','splits.json','manifest.json'])assert.equal(native.files[name],Buffer.from(run.workflow.files_base64[name==='manifest.json'?name:'analysis/'+name],'base64').toString(),name);
 await fs.writeFile(root+'/real-ri-analysis.json',JSON.stringify(native));
}
console.log(JSON.stringify({original_units:25649,expanded_json_bytes:expandedBytes,shared_archive_bytes:Buffer.byteLength(text),all_inputs_results_source_names_and_requests_identical:true,native_analysis_comparison:process.argv.includes('--native-host')}));

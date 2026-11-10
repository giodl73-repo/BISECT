// Three source-generated native State runs; the collection itself is synthetic.
import {stateGraphRequest} from '../../../web/lab/state-graph-files.js';
import {nativeStateJobFromResult,executeNativeStateJob} from '../../../web/lab/native-state-job.js';
import {executeNativeStateOutput,nativeStateOutputSource} from '../../../web/lab/native-state-output.js';
export async function nativeOutputBatchRecoveryCase(engine){
 const years=[],runs=[];
 for(const [state,fips,year,districts]of [['RI','44','2020',2],['VT','50','2010',1],['WA','53','2000',2]]){
  const graph={schema_version:1,state,year,geoids:['001000300','001000100','001000200','001000400'].map(id=>fips+id),adjacency:[[1],[0,2],[1,3],[2]],population:[100,100,100,100],edges:[[0,1,1.5],[1,2,2.5],[2,3,3.5]],areas:[10,10,10,10],exterior_perimeters:[1,1,1,1],centroids:[[-120,47],[-120.1,47],[-120.2,47],[-120.3,47]]};
  const command={arguments_json:JSON.stringify(['--state',state,'--year',year,'--metis-engine','metis-core','--districts',String(districts),'--seed','42','--balance-tolerance','25','--profile','open'])};
  const inputs={graph:{name:state+'-'+year+'-synthetic.json',text:JSON.stringify(graph)},auxiliary:null,settings:{label:'Recovery_'+state+'_'+year,chamber:'congressional',created_at:'2026-10-22T00:00:00Z',inspect_only:true}};
  const compiled=engine.execute(stateGraphRequest(command,inputs)),job=await nativeStateJobFromResult(command,inputs,compiled,{graph:true}),run=await executeNativeStateJob(engine,job);runs.push(run);
  years.push({year,result:{schema_version:'bisect-state-run-batch-v1',selection:{year},status:'completed',states:[{state_code:state,status:'ok',result:run}]}});
 }
 const collection={schema_version:'bisect-state-run-years-v1',years},input={source:nativeStateOutputSource(collection),version:'vBatchRecovery',layout:'state'},output=await executeNativeStateOutput(engine,input);
 return {runs,collection,input,output};
}

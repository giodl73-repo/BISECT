// Regenerates from source; no retained target fixture is a prerequisite.
import {executeNativeStateOutput,nativeStateOutputSource} from '../../../web/lab/native-state-output.js';
export async function nativeOutputRecoveryCase(engine){
 const graph={schema_version:1,state:'WA',year:'2020',geoids:['53001000300','53001000100','53001000200','53001000400'],adjacency:[[1],[0,2],[1,3],[2]],population:[100,100,100,100],edges:[[0,1,1.5],[1,2,2.5],[2,3,3.5]],areas:[10,10,10,10],exterior_perimeters:[1,1,1,1],centroids:[[-120,47],[-120.1,47],[-120.2,47],[-120.3,47]]};
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:'18446744073709551615',seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
 const request={operation:'run-prepared-engine',input:{request_json:JSON.stringify({graph,options}),inspect_only:false,label:'Recovery',chamber:'congressional',created_at:'2026-10-22T00:00:00Z'}};
 const run=engine.execute(request),input={source:nativeStateOutputSource(run),version:'vRecovery'},output=await executeNativeStateOutput(engine,input);
 return {request,run,input,output};
}

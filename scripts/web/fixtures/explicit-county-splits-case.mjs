// Explicit custom subdivision fixture; regenerate from tracked source.
import {executeNativeStateOutput,nativeStateOutputSource} from '../../../web/lab/native-state-output.js';
export async function explicitCountySplitsCase(engine,{kind='imported',numeric=false}={}){
 const ids=numeric?['44001000100','44001000200','44001000300','44001000400']:['ward-A','ward-B','ward-C','ward-D'];
 const graph={schema_version:1,state:'RI',year:'2020',unit_kind:kind,county_ids:['supplied-A','44001','supplied-A','44001'],geoids:ids,adjacency:[[1],[0,2],[1,3],[2]],population:[100,100,100,100],edges:[[0,1,1.5],[1,2,2.5],[2,3,3.5]],areas:[10,10,10,10],exterior_perimeters:[1,1,1,1],centroids:[[-71.4,41.8],[-71.3,41.8],[-71.2,41.8],[-71.1,41.8]]};
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:'18446744073709551615',seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
 const request={operation:'run-prepared-engine',input:{request_json:JSON.stringify({graph,options}),inspect_only:false,label:'ExplicitCounties',chamber:'congressional',created_at:'2026-10-22T00:00:00Z'}};
 const run=engine.execute(request),input={source:nativeStateOutputSource(run),version:'vExplicit'},output=await executeNativeStateOutput(engine,input);
 return {request,run,input,output,plan:run.export.document_json,context:run.export.context_json};
}

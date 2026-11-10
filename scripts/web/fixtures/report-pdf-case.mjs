import {labelRepositoryCase} from './label-repository-case.mjs';
export async function reportPdfCase(engine){
 const fixture=await labelRepositoryCase(engine),state=fixture.stageRequest.input.states.washington;
 const ids=JSON.parse(state.document_json).plan.units.unit_ids;
 // Adjacent polygons share the same stored vertex values, avoiding floating-point boundary overlap.
 const xs=[-122,-121.99,-121.98,-121.97],ys=[47,47.01,47.02,47.03];
 const geometry={type:'FeatureCollection',features:ids.map((GEOID,i)=>{const col=i%3,row=Math.floor(i/3),x=xs[col],y=ys[row],right=xs[col+1],top=ys[row+1];return {type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[[[x,y],[right,y],[right,top],[x,top],[x,y]]]}};})};
 const mapRequest={operation:'render-native-district-map',input:{document_json:state.document_json,unit_geometry_json:JSON.stringify(geometry),dpi:72,map_type:'districts'}};
 const map=engine.execute(mapRequest),prefix='analysis/'+fixture.label+'/2020/washington/';
 const files={};
 for(const [path,text]of Object.entries(fixture.analysis.files))if(path.startsWith(prefix)){
  const name=path.slice(prefix.length);if(['manifest.json','plan.rplan','context.rctx','summary.json','contiguity.json','compactness.json'].includes(name))files[name.endsWith('.json')&&name!=='manifest.json'?'analysis/'+name:name]=[...Buffer.from(text)];
 }
 files['maps/districts.png']=[...Buffer.from(map.png_base64,'base64')];
 const request={operation:'assemble-commission-report',files,generated_at:'2026-10-23T00:00:00Z',delivery:{label:fixture.label,formats:['html','json'],audit_with_report:true}};
 const result=engine.execute(request);
 const projectFiles={commissionInputs:{files:Object.fromEntries(Object.entries(files).map(([path,data])=>[path,{name:path.split('/').at(-1),base64:Buffer.from(data).toString('base64')}])),generated_at:request.generated_at},commissionDeliverySettings:request.delivery};
 const comparisonFiles=Object.fromEntries(Object.entries(files).filter(([path])=>['manifest.json','analysis/compactness.json'].includes(path)));
 const comparisonSettings={label_a:'Synthetic A',label_b:'Synthetic B',leaning_threshold:0.55,close_call_band:0.05,generated_at:request.generated_at};
 const comparisonRequest={operation:'render-report-comparison',files_a:comparisonFiles,files_b:comparisonFiles,...comparisonSettings},comparisonResult=engine.execute(comparisonRequest);
 const comparisonProjectFiles={reportFiles:Object.fromEntries(Object.entries(comparisonFiles).map(([path,data])=>[path,Buffer.from(data).toString('base64')])),comparisonReportFiles:Object.fromEntries(Object.entries(comparisonFiles).map(([path,data])=>[path,Buffer.from(data).toString('base64')])),reportComparisonSettings:comparisonSettings};
 return {request,result,projectFiles,mapRequest,map,label:fixture.label,labelRequest:fixture.reportRequest,labelResult:fixture.report,comparisonRequest,comparisonResult,comparisonProjectFiles};
}

import {bindNativeReportExport} from './native-report-export-binding.js';

export function nativeReportSourceWarning(binding){
 const p=binding?.report_sources?.receipt;if(!p?.cvap_missing_warning)return null;
 const i=p.identity;
 return 'WARNING: CVAP data not found for '+i.state_code+' '+i.effective_year+'.\nCVAP requires a separate download: https://www.census.gov/programs-surveys/decennial-census/about/voting-rights/cvap.html\nFalling back to total population.';
}
export function emitNativeReportSourceWarning(binding,onProgress){
 const text=nativeReportSourceWarning(binding);
 // Native warnings are not STATUS lanes; position 999 must not hide them.
 if(text)onProgress?.({schema_version:'bisect-native-state-report-warning-v1',level:'warning',message:'CVAP is absent in the retained source snapshot. '+text,native_warning_text:text,scope:'Availability within supplied snapshot only. The native returned source is unused; balancing still uses selected graph populations.'});
}
export async function applyNativeStateReportSources(engine,result,binding,{onProgress}={}){
 const snapshot=binding?.report_sources;if(!snapshot)return result;
 let error=result.prepared.export_error;
 if(result.prepared.export&&error==null){
  try{onProgress?.({stage:'native-state-report-sources',message:'Binding retained report sources to the accepted plan/context'});result.prepared=await bindNativeReportExport(engine,result.prepared,snapshot);error=result.prepared.export_error;}
  catch(failure){error='Native report binding failed: '+String(failure.message).slice(0,2000);const p=result.prepared;result.prepared={...p,export:null,export_error:error,report_source_binding:{schema_version:'bisect-prepared-report-binding-v1',source_snapshot:structuredClone(snapshot),receipt:{status:'binding-error',partition_execution_performed:false},previous_export:p.export?{document_json:p.export.document_json,context_json:p.export.context_json}:null}};}
 }
 const completed=!!result.prepared.export&&error==null;
 result.native_report_sources={schema_version:'bisect-native-state-report-outcome-v1',status:completed?'bound':'failed',source_bundle_sha256:snapshot.receipt.source_bundle_sha256,requested_population_source:snapshot.receipt.requested_population_source,cvap_missing_warning:snapshot.receipt.cvap_missing_warning,native_warning_text:nativeReportSourceWarning(binding),balance_population_source:'selected-graph-populations',context_hash:completed?result.prepared.export.context_hash:null,...(!completed?{error:String(error??'Accepted engine output has no physical export.').slice(0,2000)}:{})};
 if(!completed)result.status='partial-output';
 return result;
}

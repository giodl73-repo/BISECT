import {commissionFormatAvailable} from './commission-files.js';
import {validatePackagePath,validateSourceFile,encodeBytes} from './package-files.js';
const MAX=25*1024*1024;
// Only known native result schemas expose reports. These descriptors confer no
// freshness authority: the workbench must still require its live result.
export function generatedReports(result){
 const reports=[];
 function add(key,label,path,source,encoding,hash,required=true){
  validatePackagePath(path);
  if(typeof source!=='string'||required&&!/^[a-f0-9]{64}$/.test(hash??'')||hash!=null&&!/^[a-f0-9]{64}$/.test(hash))throw new Error('Generated report is missing its recorded bytes or SHA-256.');
  if(reports.length>=512)throw new Error('Report preview accepts up to 512 generated HTML reports.');
  reports.push({key,label,path,source,encoding,hash});
 }
 function files(value,prefix,encoded=false){
  const sources=encoded?value.files_base64:value.files,hashes=value.file_sha256;
  if(!sources||typeof sources!=='object'||Array.isArray(sources)||Object.keys(sources).length>20000)throw new Error('Generated report file inventory is unavailable or too large.');
  for(const name of Object.keys(sources).sort())if(name.endsWith('.html'))add(prefix+name,prefix+name,name,sources[name],encoded?'base64':'text',hashes?.[name]);
 }
 function workflow(value,prefix=''){
  if(['bisect-tui-run-v1','bisect-accepted-state-workflow-v1','bisect-prepared-state-run-v1'].includes(value?.schema_version)&&value.workflow?.schema_version==='bisect-state-workflow-v1'&&['completed','partial'].includes(value.workflow.status))files(value.workflow,prefix,true);
 }
 function batch(value,prefix=''){
  if(value?.schema_version!=='bisect-state-run-batch-v1'||!Array.isArray(value.states)||value.states.length>51)throw new Error('Generated state report inventory is invalid.');
  for(const row of value.states)if(row.status==='ok'){
   if(!/^[A-Z]{2}$/.test(row.state_code))throw new Error('Generated state report scope is invalid.');
   workflow(row.result,prefix+row.state_code+'/');
  }
 }
 switch(result?.schema_version){
  case 'bisect-commission-report-v1':
   if(commissionFormatAvailable(result,'html')){
    const path=result.delivery_settings?result.delivery_settings.label+'_report.html':'commission-report.html';
    const hash=result.report_file_sha256?.[path];
    if(result.delivery_settings&&result.report_files?.[path]!==result.report_html)throw new Error('Generated HTML differs from its native delivery file.');
    add('commission',path,path,result.report_html,'text',hash,!!result.delivery_settings);
   }break;
  case 'bisect-imported-report-comparison-v1':
   if(typeof result.html==='string')add('comparison','Draft comparison report','comparison-report.html',result.html,'text',result.receipt?.html_sha256);break;
  case 'bisect-label-report-v1':case 'bisect-label-report-years-v1':files(result,'');break;
  case 'bisect-tui-run-v1':case 'bisect-accepted-state-workflow-v1':case 'bisect-prepared-state-run-v1':workflow(result);break;
  case 'bisect-state-run-batch-v1':
   batch(result);break;
  case 'bisect-state-run-years-v1':{
   if(!Array.isArray(result.years)||result.years.length>3)throw new Error('Generated multi-year report inventory is invalid.');
   const seen=new Set();
   for(const row of result.years){
    if(!['2020','2010','2000'].includes(row.year)||seen.has(row.year))throw new Error('Generated report year scope is invalid.');seen.add(row.year);
    if(row.result){if(row.result.selection?.year!==row.year)throw new Error('Generated report child year differs.');batch(row.result,row.year+'/');}
   }break;
  }
 }
 if(new Set(reports.map(r=>r.key)).size!==reports.length)throw new Error('Duplicate generated report paths.');
 return reports;
}
export async function generatedReportText(result,key){
 const report=generatedReports(result).find(r=>r.key===key);if(!report)throw new Error('Select a freshly generated HTML report.');
 let bytes;
 if(report.encoding==='base64'){
  validateSourceFile({name:report.path,base64:report.source},{maxBytes:MAX});
  bytes=Uint8Array.from(atob(report.source),c=>c.charCodeAt(0));
  if(encodeBytes(bytes)!==report.source)throw new Error('Generated report encoding is not canonical base64.');
 }else{
  if(report.source.length>MAX)throw new Error('Report preview is limited to 25 MiB. Download the original HTML instead.');
  bytes=new TextEncoder().encode(report.source);
 }
 if(bytes.length>MAX)throw new Error('Report preview is limited to 25 MiB. Download the original HTML instead.');
 const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
 if(report.hash!=null&&hash!==report.hash)throw new Error('Generated report SHA-256 differs from its recorded native bytes.');
 return {name:report.path,text:new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes),sha256:hash};
}

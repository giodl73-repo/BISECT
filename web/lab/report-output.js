// Report files are downloaded as bytes; never insert imported HTML into the UI.
const encoder=new TextEncoder();
const MAX_REPORT_BYTES=2*1024*1024;
export function validateReportOutput(result){
  if(result?.schema_version!=='bisect-imported-report-comparison-v1'||result.draft!==true||result.receipt?.schema_version!=='bisect-report-byte-receipt-v1')throw new Error('Run a draft comparison report first.');
  for(const key of ['html','markdown']){
    if(typeof result[key]!=='string'||encoder.encode(result[key]).length>MAX_REPORT_BYTES||!(/^[a-f0-9]{64}$/).test(result.receipt[key+'_sha256']))throw new Error('Invalid report output or byte receipt.');
  }
  return result;
}
export async function reportDownload(result,kind){
  validateReportOutput(result);
  if(!['html','markdown','receipt'].includes(kind))throw new Error('Unsupported report download.');
  const payloads={};
  for(const key of ['html','markdown']){
    const bytes=encoder.encode(result[key]);
    const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),byte=>byte.toString(16).padStart(2,'0')).join('');
    if(hash!==result.receipt[key+'_sha256'])throw new Error('Report bytes do not match the receipt. Run again before downloading.');
    payloads[key]=bytes;
  }
  if(kind==='receipt')return {bytes:encoder.encode(JSON.stringify(result.receipt,null,2)),name:'bisect-comparison-report-receipt.json',mime:'application/json'};
  return {bytes:payloads[kind],name:'bisect-comparison-report.'+(kind==='html'?'html':'md'),mime:kind==='html'?'text/html;charset=utf-8':'text/markdown;charset=utf-8'};
}

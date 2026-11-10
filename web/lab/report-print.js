import {generatedReportText} from './generated-report-files.js';
import {commissionPreviewDocument} from './commission-preview.js';

export const REPORT_PRINT_DEFAULTS=Object.freeze({paper:'letter',orientation:'portrait'});
export const REPORT_PRINT_PROFILE='browser-report-print-v1';
const MAX_PDF_BYTES=25*1024*1024;
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const bytes=text=>new TextEncoder().encode(text);
export function validateReportPrintSettings(value){
 if(!value||Object.keys(value).sort().join()!=='orientation,paper'||!['letter','a4'].includes(value.paper)||!['portrait','landscape'].includes(value.orientation))throw new Error('Choose Letter or A4 and portrait or landscape.');
 return {...value};
}
export function reportPrintCss(settings){
 const value=validateReportPrintSettings(settings);
 return `@page { size: ${value.paper} ${value.orientation}; margin: 12mm;
 @bottom-right { content: "Page " counter(page) " of " counter(pages); font-family: Georgia, serif; font-size: 8pt; color: #666; }
}
@media print {
 body { max-width: none !important; margin: 0 !important; padding: 0 !important; font-size: 10pt !important; line-height: 1.35 !important; overflow-wrap: anywhere; }
 h1 { font-size: 18pt !important; } h2 { font-size: 14pt !important; } h3 { font-size: 11pt !important; }
 h1, h2, h3, h4 { break-after: avoid-page; }
 table { max-width: 100% !important; width: 100% !important; }
 thead { display: table-header-group; } tr { break-inside: avoid; }
 th, td { padding: 1.5mm !important; overflow-wrap: anywhere; }
 pre, code, .audit-box { white-space: pre-wrap !important; overflow: visible !important; overflow-wrap: anywhere; word-break: break-word; font-size: 8pt !important; }
 .meta-grid { display: block !important; } .meta-item { flex-wrap: wrap; } .meta-label { min-width: 0 !important; }
 img { max-width: 100% !important; max-height: ${value.orientation==='portrait'?'220':'145'}mm; object-fit: contain; break-inside: avoid; }
 footer { font-size: 8pt !important; }
}
@media screen and (max-width: 480px) {
 body { padding: 0.75rem !important; overflow-wrap: anywhere; }
 h1 { font-size: 1.4rem !important; overflow-wrap: anywhere; }
 pre, code, .audit-box { white-space: pre-wrap !important; overflow-wrap: anywhere; }
 .meta-grid { display: block !important; } .meta-item { flex-wrap: wrap; } .meta-label { min-width: 0 !important; }
 .provenance dl { display: block !important; } .provenance dd { margin: 0 0 0.5rem !important; }
 th, td { padding: 0.25rem !important; overflow-wrap: anywhere; }
}`;
}
export async function prepareReportPrint(result,key,document,{settings=REPORT_PRINT_DEFAULTS,runtimeArtifact,userAgent=globalThis.navigator?.userAgent??'unavailable'}={}){
 const selected=validateReportPrintSettings(settings),report=await generatedReportText(result,key);
 if(!runtimeArtifact||!/^[a-f0-9]{64}$/.test(runtimeArtifact.sha256)||!Number.isSafeInteger(runtimeArtifact.byte_length)||runtimeArtifact.byte_length<1)throw new Error('Loaded report engine identity is unavailable. Run the report again.');
 const css=reportPrintCss(selected),restricted=commissionPreviewDocument(report.text,document),html=restricted.replace('</head>','<style data-bisect-print-profile="'+REPORT_PRINT_PROFILE+'">'+css+'</style></head>');
 return {html,record:{schema_version:'bisect-report-print-record-v1',report_path:report.name,report_key:key,source_schema:result.schema_version,original_html_sha256:report.sha256,restricted_print_html_sha256:await hash(bytes(html)),print_css_sha256:await hash(bytes(css)),profile:REPORT_PRINT_PROFILE,settings:selected,dialog_settings:'User-controlled and not observed; may override CSS paper intent.',runtime_artifact:{sha256:runtimeArtifact.sha256,byte_length:runtimeArtifact.byte_length},reported_user_agent:userAgent,prepared_at_utc:new Date().toISOString(),print_call:'not-requested',beforeprint_observed:false,afterprint_observed:false,pdf_save_observed:false,selected_pdf:null,scope:'Local byte/print observations only. Print events do not prove a PDF was saved; selecting a PDF records a user-declared association, not a verified HTML-to-PDF match or certification.'}};
}
export async function reportPrintDomHash(document){return hash(bytes(document.documentElement.outerHTML));}
export async function verifyReportPrintDom(document,expected){if(await reportPrintDomHash(document)!==expected)throw new Error('Report preview changed. Prepare a new preview before printing or recording a PDF.');}
export async function observeSavedReportPdf(file,{signal}={}){
 signal?.throwIfAborted();
 if(!file||typeof file.name!=='string'||file.name.length>200||/[\u0000-\u001f]/.test(file.name)||!Number.isSafeInteger(file.size)||file.size<16||file.size>MAX_PDF_BYTES)throw new Error('Select the PDF you saved, up to 25 MiB.');
 const data=new Uint8Array(await file.arrayBuffer());signal?.throwIfAborted();
 if(data.length!==file.size||!/^%PDF-[12]\.[0-9][\r\n]/.test(new TextDecoder('latin1').decode(data.slice(0,16)))||!/%%EOF[\t\n\f\r ]*$/.test(new TextDecoder('latin1').decode(data.slice(-1024))))throw new Error('Selected bytes lack a bounded PDF header/end marker. No content was rendered or executed.');
 const sha256=await hash(data);signal?.throwIfAborted();
 return {name:file.name,byte_length:data.length,sha256,association:'user-selected for this report; not independently verified',checks:'byte count and PDF header/end markers only',content_match_verified:false,observed_at_utc:new Date().toISOString()};
}

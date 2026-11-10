import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {REPORT_PRINT_DEFAULTS,reportPrintCss,observeSavedReportPdf} from '../../web/lab/report-print.js';
for(const value of [null,{}, {...REPORT_PRINT_DEFAULTS,paper:'letter;script'}, {...REPORT_PRINT_DEFAULTS,orientation:'up'}, {...REPORT_PRINT_DEFAULTS,other:true}])assert.throws(()=>reportPrintCss(value));
const data=Buffer.from('%PDF-1.7\nNot parsed or rendered by the byte observer.\n%%EOF\n'),file={name:'observed.pdf',size:data.length,arrayBuffer:async()=>Uint8Array.from(data).buffer};
const observed=await observeSavedReportPdf(file);assert.equal(observed.sha256,createHash('sha256').update(data).digest('hex'));assert.equal(observed.content_match_verified,false);assert.match(observed.association,/not independently verified/);
for(const invalid of [{...file,size:26*1024*1024},{...file,name:'bad\u0000.pdf'},{...file,arrayBuffer:async()=>new TextEncoder().encode('<script>bad</script>').buffer},{...file,size:data.length+1},{...file,arrayBuffer:async()=>new TextEncoder().encode('%PDF-1.7\nNo end marker here').buffer}])await assert.rejects(observeSavedReportPdf(invalid));
const controller=new AbortController();controller.abort();await assert.rejects(observeSavedReportPdf(file,{signal:controller.signal}));
console.log('PASS report print: settings injection refusal, exact selected byte hash, explicit unverified association and size/header/end/cancellation guards.');

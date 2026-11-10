import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {configuredSourceBundle} from '../../web/lab/configured-source-bundle.js';
import {configRunRequest,configuredRunExportRequest} from '../../web/lab/config-run.js';
const root='target/configured-source-bundle';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let refusals=0;
for(const i of [0,1]){
 const p=JSON.parse(await fs.readFile(`target/native-demographic-adoption/case-${i}.bisect`,'utf8')),files=p.files,result=engine.execute(configRunRequest(files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings));result.practitioner_export=engine.execute(configuredRunExportRequest(result,{label:'Native source bundle',created_at:'2026-10-13T00:00:00Z'}));const bundle=await configuredSourceBundle(result,files);assert.equal(Object.keys(bundle.manifest.files).length,8);await fs.writeFile(`${root}/case-${i}.zip`,bundle.bytes);await fs.writeFile(`${root}/case-${i}-manifest.json`,JSON.stringify(bundle.manifest));await fs.writeFile(`${root}/case-${i}-result.json`,JSON.stringify(result));
 for(const key of ['demographicSource','nativeDemographicReceipt','configSource']){const bad=structuredClone(files);bad[key].base64=Buffer.from('changed').toString('base64');await assert.rejects(configuredSourceBundle(result,bad));refusals++;}
 for(const key of ['graph','demographics']){const bad=structuredClone(files);bad.configRunInputs[key].base64=Buffer.from('{}').toString('base64');await assert.rejects(configuredSourceBundle(result,bad));refusals++;}
 const bad=structuredClone(result);bad.practitioner_export.document_json=JSON.stringify({provenance:{producer:{demographic_input:{}}}});await assert.rejects(configuredSourceBundle(bad,files));refusals++;
 const old=structuredClone(files),receipt=JSON.parse(Buffer.from(old.nativeDemographicReceipt.base64,'base64').toString());delete receipt.demographics_source_sha256;old.nativeDemographicReceipt.base64=Buffer.from(JSON.stringify(receipt)).toString('base64');await assert.rejects(configuredSourceBundle(result,old),/adopt again/);refusals++;
 const noReceipt=structuredClone(files);delete noReceipt.nativeDemographicReceipt;const plain=await configuredSourceBundle(result,noReceipt);assert.equal(Object.keys(plain.manifest.files).length,6);assert(!plain.manifest.files['sources/native-demographics.csv']);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_export_cases:2,source_binding_refusals:refusals,legacy_unbound_receipt_requires_readoption:true,receipt_free_configured_export:true,no_native_plan_bytes_changed:true},null,2));console.log(`Configured source bundle: two native export packages, ${refusals} source/receipt/plan binding refusals; legacy receipts require readoption, native plan bytes remain exact.`);

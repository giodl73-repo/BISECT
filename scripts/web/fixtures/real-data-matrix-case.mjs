// Integration-tier recipe: exact retained graph, native template and TUI adapter.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {configTemplateRequest,CONFIG_TEMPLATE_DEFAULTS} from '../../../web/lab/config-template.js';
import {tuiRunRequest,TUI_RUN_FORM_DEFAULTS} from '../../../web/lab/tui-run-files.js';
export const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function realDataMatrixRequest(engine,row){
 const zipped=await fs.readFile(row.graph.path);assert.equal(digest(zipped),row.graph.sha256,'Retained graph archive changed');
 const bytes=gunzipSync(zipped);assert.equal(digest(bytes),row.graph.decoded_sha256,'Retained graph bytes changed');
 const graph=JSON.parse(bytes);assert.equal(graph.state,row.state);assert.equal(graph.year,row.year);assert.equal(graph.geoids.length,row.units);
 assert.equal(graph.population.reduce((a,b)=>a+BigInt(b),0n).toString(),row.population);
 const template=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,seed:row.seed,workers:1,years:[row.year]}));
 const source={name:template.generated_filename,base64:Buffer.from(template.yaml_text).toString('base64')};
 return tuiRunRequest({form:{...TUI_RUN_FORM_DEFAULTS,location:row.state,year:row.year,resolution:row.unit_kind,seed:row.seed,label:'original_'+row.state.toLowerCase()},created_at:'2026-10-12T00:00:00Z'},source,{profile:'open'},{graph:{name:row.state+'-'+row.year+'-graph.json',base64:bytes.toString('base64')}},{chamber:row.chamber,districts:row.districts,native_build:true,capture_rounds:false});
}

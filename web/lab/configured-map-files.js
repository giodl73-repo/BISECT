import {configuredRunMapDocument} from './config-run.js';
import {NATIVE_MAP_DEFAULTS,validateNativeMapInputs} from './map-export-files.js';
import {parseNativeMapPlanJson} from './project.js';
import {encodeBytes,validateSourceFile} from './package-files.js';

// Select the exact exported plan. Rendering and geographic validation remain an
// explicit native operation; no imported execution decision is inherited here.
export function configuredRunMapInputs(result,geometry=null){
 const expected=configuredRunMapDocument(result),exported=result.practitioner_export;
 if(exported?.schema_version!=='bisect-configured-plan-export-v1'||typeof exported.document_json!=='string')throw new Error('Prepare native files from this configured run first.');
 const plan={name:'configured.rplan',base64:encodeBytes(new TextEncoder().encode(exported.document_json))};
 validateSourceFile(plan,{maxBytes:3*1024*1024});
 const actual=parseNativeMapPlanJson(exported.document_json)?.plan,graph=result.engine_request.graph;
 if(!actual||actual.k!==expected.plan.k||actual.units?.state!==graph.state||actual.units?.year!==Number(graph.year)||actual.units?.unit_kind!==(graph.unit_kind||'tract')||JSON.stringify(actual.units?.unit_ids)!==JSON.stringify(expected.plan.units.unit_ids)||JSON.stringify(actual.assignment)!==JSON.stringify(expected.plan.assignment))throw new Error('Exported plan does not match this configured run scope, unit order or assignments.');
 // Start a districts map with no prior analysis, round, raw-assignment or TIGER
 // selections. Geometry is retained as selected source evidence, not certified.
 return structuredClone(validateNativeMapInputs({...NATIVE_MAP_DEFAULTS,plan,geometry}));
}

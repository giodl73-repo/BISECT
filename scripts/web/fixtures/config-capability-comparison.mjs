import assert from 'node:assert/strict';

// Native host reference has no browser sidecar. This contract permits only
// explicitly capability-dependent presentation fields, never algorithm data,
// original source, seeds, native resolution, assignments or artifacts.
export function compareConfigCapability(browser,native,supportsCMetis){
  const used=browser.metis_engine_used===true,backend=browser.resolution?.metis?.engine;
  const expected=!used||backend==='redist-metis'||supportsCMetis&&backend==='c-ffi';
  assert.equal(browser.browser_engine_available,expected);
  const differs=supportsCMetis&&used&&backend==='c-ffi';
  if(!differs){assert.deepEqual(browser,native);return {contract:'exact-host-reply-v1',browser,native,differing_fields:[]};}
  assert.equal(native.browser_engine_available,false);
  assert.equal(native.browser_execution_backend,'unavailable-declared-metis-engine');
  assert.equal(browser.browser_execution_backend,'c-metis-msvcrt-v1');
  const fields=['browser_engine_available','browser_execution_backend'];
  if(browser.schema_version==='bisect-algorithm-config-validation-v1'){
    fields.push('scope');assert.match(browser.scope,/optional C bridge/);assert.match(browser.scope,/No C module/);
  }
  const project=value=>Object.fromEntries(Object.entries(value).filter(([key])=>!fields.includes(key)));
  assert.deepEqual(project(browser),project(native));
  return {contract:'browser-c-metis-capability-v1',browser,native,differing_fields:fields};
}

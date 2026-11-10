import {compareSidecar} from './compare-sidecar.mjs';
// Source-owned experiment only; both paths must be chosen by the test harness.
self.onmessage=async({data})=>{
  try{
    if(!['unmodified','msvcrt'].includes(data.profile))throw new Error('Unknown explicit profile');
    const base='/artifacts/'+data.profile+'/';
    const create=(await import(base+'metis-probe.mjs')).default;
    const response=await fetch(base+'native-reference.json');
    if(!response.ok)throw new Error('Native reference unavailable');
    const expected=await response.json();
    const module=await create();
    self.postMessage({profile:data.profile,result:compareSidecar(module,expected)});
  }catch(error){self.postMessage({error:String(error),stack:error.stack});}
};

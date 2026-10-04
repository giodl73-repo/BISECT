import {verifyLabAssignments,verifyDemographicEvidence} from './laboratory-project.js';

// Evidence replay can be expensive. Keep imported histories off the UI thread
// and terminate them when cancelled or when they exceed the verification budget.
export function verifyAssignments(graph,state,assignments,config,{signal,workerFactory}={}) {
  if(signal?.aborted)return Promise.reject(new Error('Project import cancelled.'));
  if(!workerFactory && typeof Worker==='undefined') {
    // Node integration tests have no browser Worker; never fall back in the UI.
    if(typeof window!=='undefined')return Promise.reject(new Error('Project verification requires browser Worker support.'));
    return Promise.resolve().then(async()=>{verifyLabAssignments(graph,state,assignments,config);await verifyDemographicEvidence(graph,state,config,assignments);});
  }
  return new Promise((resolve,reject)=>{
    const worker=workerFactory ? workerFactory() : new Worker(new URL('./assignment-verification-worker.js',import.meta.url),{type:'module'});
    let settled=false;
    const timer=setTimeout(()=>finish(new Error('Project evidence verification timed out. Current project was kept.')),30000);
    function finish(error) {
      if(settled)return;settled=true;
      clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();
      error ? reject(error) : resolve();
    }
    function abort(){finish(new Error('Project import cancelled.'));}
    signal?.addEventListener('abort',abort,{once:true});
    worker.onmessage=({data})=>{
      if(data?.ok===true)finish();
      else finish(new Error(typeof data?.error==='string'?data.error:'Project evidence verification failed.'));
    };
    worker.onerror=worker.onmessageerror=()=>finish(new Error('Project evidence verification failed. Current project was kept.'));
    try{worker.postMessage({graph,state,assignments,config});}
    catch{finish(new Error('Project evidence verification failed. Current project was kept.'));}
  });
}

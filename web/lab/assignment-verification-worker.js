import {verifyLabAssignments,verifyDemographicEvidence} from './laboratory-project.js';
self.onmessage=async({data})=>{
  try {
    verifyLabAssignments(data.graph,data.state,data.assignments,data.config);
    await verifyDemographicEvidence(data.graph,data.state,data.config,data.assignments);
    self.postMessage({ok:true});
  } catch(error) {
    self.postMessage({ok:false,error:error.message || 'Invalid project evidence.'});
  }
};

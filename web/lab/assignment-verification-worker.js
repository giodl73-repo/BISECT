import {verifyLaboratoryCoiSource} from './laboratory-coi.js';
import {verifyLaboratoryElectoral} from './laboratory-electoral.js';
import {verifyLaboratoryIlpReports} from './laboratory-ilp-reports.js';
import {verifyLaboratoryCapture} from './laboratory-round-capture.js';
import {verifyLabAssignments,verifyDemographicEvidence} from './laboratory-project.js';
self.onmessage=async({data})=>{
  try {
    verifyLabAssignments(data.graph,data.state,data.assignments,data.config);
    await verifyDemographicEvidence(data.graph,data.state,data.config,data.assignments);await verifyLaboratoryCapture(data.graph,data.state,data.assignments,data.config);await verifyLaboratoryIlpReports(data.graph,data.state,data.assignments,data.config);await verifyLaboratoryElectoral(data.graph,data.state,data.config);await verifyLaboratoryCoiSource(data.state,data.config);
    self.postMessage({ok:true});
  } catch(error) {
    self.postMessage({ok:false,error:error.message || 'Invalid project evidence.'});
  }
};

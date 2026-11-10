import {loadConfigCatalogJobs} from './config-catalog.js?v=build-catalog-worker-v1';
import {validateLabelBuildInputs} from './label-build-files.js?v=label-build-checkpoint-v1';
let started=false;
self.onmessage=async({data})=>{
 try{
  if(started)throw new Error('Preparation worker already used.');
  started=true;
  if(!data||Object.keys(data).sort().join()!=='catalog,chamber,inputs,keys,type'||data.type!=='prepare-build-jobs')throw new Error('Invalid catalog preparation request.');
  const next=await loadConfigCatalogJobs(data.catalog,data.keys,data.chamber);
  const inputs=validateLabelBuildInputs({...data.inputs,jobs:next.source,settings:{...data.inputs.settings,years:next.years}});
  self.postMessage({ok:true,value:{...next,inputs}});
 }catch(error){self.postMessage({ok:false,error:error.message});}
};

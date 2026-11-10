import {nativeWorkspaceCase} from './native-workspace-case.mjs';
import {createNativeStateSourceJob} from '../../../web/lab/native-state-sources.js';
import {appendStateRunJob,prepareNativeRunQueue,prepareNativeStatesQueue} from '../../../web/lab/state-run-batch.js';
export async function nativeWorkspaceContinuationCase(engine,{allYears=false,states=false,geographies=[['VT','50'],['WY','56']]}={}){
 if(allYears&&states)throw new Error('Native States selects one year.');
 const fixture=nativeWorkspaceCase(),years=allYears?['2020','2010','2000']:['2020'];
 const command={arguments_json:JSON.stringify(['--year',allYears?'all':'2020','--states',...geographies.map(row=>row[0]),'--workers','1','--version','vContinue',...(states?['--partition-mode','spectral','--output-dir','recorded continuation output intent']:['--reset'])])};
 let inputs={jobs:null,workers:1,...(allYears?{all_years:true}:{})};
 for(const year of years)for(const [state,fips]of geographies){
  const sources=Object.fromEntries([['adjacency','.adj.bin'],['geoids','_geoids.json'],['centroids','_centroids.json']].map(([key,suffix])=>[key,{name:state+suffix,base64:fixture.files.get(fixture.stem+suffix).toString('base64')}]));
  const ids=JSON.parse(Buffer.from(sources.geoids.base64,'base64'));for(const index of Object.keys(ids))ids[index]=fips+ids[index].slice(2);sources.geoids.base64=Buffer.from(JSON.stringify(ids)).toString('base64');
  const files={nativeAdjacencyInputs:{...sources,geometry:null},nativeAdjacencySettings:{state,year,unit_kind:'tract'},nativeStateSourceSettings:{enabled:true,command_kind:states?'states':'run'},[states?'statesCommandInputs':'runCommandInputs']:command,stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Continue_'+state,chamber:'congressional',created_at:'2026-10-23T00:00:00Z',inspect_only:true}}};
  inputs=appendStateRunJob(inputs,await createNativeStateSourceJob(files,'native'));
 }
 const selection={year:'2020',states:geographies.map(row=>row[0]),manifest:null};
 return (states?prepareNativeStatesQueue:prepareNativeRunQueue)(command,inputs,selection,engine.execute({operation:states?'inspect-native-states-command':'inspect-native-run-command',arguments:JSON.parse(command.arguments_json)}));
}

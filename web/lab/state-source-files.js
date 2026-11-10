import {readCensusArchive,MAX_CENSUS_ZIP_BYTES} from './census-archive.js';
import {characterStateFips} from './character-input.js';
import {validateTigerInputs} from './tiger-files.js';
import {validatePl94Inputs} from './pl94-files.js';
import {validateTractPreparationSettings,tractPreparationRequest,censusPreparationRequest} from './tract-input-files.js';
export async function readStateCensusArchives(tigerFile,pl94File,{state,year,kind='tracts',signal,archiveReader=readCensusArchive}={}){
 if(!characterStateFips(state)||![2000,2010,2020].includes(year)||!['counties','tracts','block-groups','blocks'].includes(kind))throw new Error('Select a known state, matching Census vintage and polygon resolution.');
 for(const file of [tigerFile,pl94File])if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>MAX_CENSUS_ZIP_BYTES)throw new Error('Select both Census ZIPs, each up to 64 MiB.');
 signal?.throwIfAborted();const controller=new AbortController(),abort=()=>controller.abort(signal.reason);signal?.addEventListener('abort',abort,{once:true});
 try{
  const [tiger,pl94]=await Promise.all([archiveReader(tigerFile,{kind:'tiger',year},{signal:controller.signal}),archiveReader(pl94File,{kind:'pl94',year},{signal:controller.signal})]);signal?.throwIfAborted();
  if(tiger.kind!=='tiger'||pl94.kind!=='pl94'||tiger.year!==year||pl94.year!==year)throw new Error('Census archive selection changed its scope.');
  const files={tigerInputs:validateTigerInputs({kind,year,shp:tiger.sources.shp,dbf:tiger.sources.dbf,shx:tiger.sources.shx||null}),pl94Inputs:validatePl94Inputs({year,geography:pl94.sources.geography,population:pl94.sources.population||null}),tractPreparationSettings:validateTractPreparationSettings({state,projection:tiger.sources.prj||null})};
  const request=(kind==='tracts'?tractPreparationRequest:censusPreparationRequest)(files);
  return {files,operation:request.operation,archives:[tiger,pl94].map(a=>({kind:a.kind,sha256:a.archive_sha256,members:a.members.length}))};
 }catch(error){controller.abort();throw error;}finally{signal?.removeEventListener('abort',abort);}
}

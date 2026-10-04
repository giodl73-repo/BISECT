import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {encodeBytes,decodePackageFiles,validateRiSources,validatePackageArchive} from '../../web/lab/package-files.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/ri-rla-import-fixtures';await fs.mkdir(root,{recursive:true});
function sources(winner='6',loser='4',count='10'){
  return {
    auditReport:`######## CONTESTS ########\nContest Name,Targeted?,Number of Winners,Votes Allowed,Total Ballots Cast,Vote Totals\nRepresentative 28,Targeted,1,1,${count},Scott Guthrie: ${loser}; George A. Nardone: ${winner}; Write-in: 0\n\n######## AUDIT SETTINGS ########\nAudit Name,Audit Type,Audit Math Type,Risk Limit,Random Seed,Online Data Entry?\nRI Rep 28,BALLOT_POLLING,MINERVA,9%,34053800000000000000,No\n\n######## ROUNDS ########\nRound Number,Status,Started At,Sample Size,Risk Measurements\n1,Ended,2024-11-20 21:52:00+00:00,1,George A. Nardone / Scott Guthrie: 0.054\n\n######## SAMPLED BALLOTS ########\nDraw Number,Container,Tabulator,Batch Name,Ballot Position,Ticket Numbers,Audit Result\n1,0600,0315412524,EV Coventry,1,Round 1: 0.1,George A. Nardone\n`,
    ballotManifest:`Batch Name,Number of Ballots,Container,Tabulator\nEV Coventry,${count},0600,0315412524\n`,
    ballotRetrieval:'Container,Tabulator,Batch Name,Ballot Number,Ticket Numbers,Already Audited,Audit Board\n0600,0315412524,EV Coventry,1,0.1,N,Audit Board #1\n',
  };
}
const fields={auditReport:'audit_report_base64',ballotManifest:'ballot_manifest_base64',ballotRetrieval:'ballot_retrieval_base64'};
function requestFor(input){return {operation:'import-ri2024-rep28-rla',...Object.fromEntries(Object.entries(input).map(([key,text])=>[fields[key],Buffer.from(text).toString('base64')]))};}
let sequence=0;
async function native(request){const file=path.join(root,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));return spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:16*1024*1024});}
for(const [name,input] of [['ordinary',sources()],['large-count',sources('9007199254740993','4','9007199254740997')]]){
  const request=requestFor(input),reference=await native(request);assert.equal(reference.status,0,reference.stderr);
  const result=engine.execute(request);assert.deepEqual(result,JSON.parse(reference.stdout));
  assert.equal(result.adapter,'ri-2024-rep28-rla');assert.equal(result.verification.status,'pass');
  assert.equal(result.source_summary.public_seed,'34053800000000000000');assert.equal(result.source_summary.sampled_ballot_rows,1);assert.equal(result.source_summary.retrieval_rows,1);
  const riSources={};
  for(const [key,text]of Object.entries(input)){const bytes=Buffer.from(text),base64=encodeBytes(bytes);riSources[key]={name:key+'.csv',base64};await fs.writeFile(path.join(root,name+'-'+key+'.csv'),bytes);const sourcePath={'auditReport':'sources/ri-2024-rep28-audit-report.csv','ballotManifest':'sources/ri-2024-rep28-ballot-manifest.csv','ballotRetrieval':'sources/ri-2024-rep28-ballot-retrieval.csv'}[key];assert.equal(result.package_files[sourcePath],base64);}
  validateRiSources(riSources);
  const project=createProject({name:'RI '+name,files:{riSources},operation:request.operation,constraints:[],result,lastOperation:request.operation});
  assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));
  const archive=validatePackageArchive({schema_version:'bisect-rcount-files-v1',files:result.package_files});
  const replay=engine.execute({operation:'replay-count-audits',files:decodePackageFiles(archive)});
  assert.equal(replay.status,'boundary');assert.equal(replay.runs[0].status,'boundary');
  for(const sourcePath of Object.keys(archive).filter(key=>key.endsWith('.csv'))){const changed=structuredClone(archive);const bytes=Buffer.from(changed[sourcePath],'base64');bytes[0]^=1;changed[sourcePath]=bytes.toString('base64');assert.equal(engine.execute({operation:'verify-count-files',files:decodePackageFiles(changed)}).status,'fail');}
}
const good=sources();
for(const input of [{...good,ballotManifest:good.ballotManifest.replace('Coventry,10','Coventry,11')},{...good,ballotRetrieval:good.ballotRetrieval.replace('Coventry,1,','Coventry,2,')}, {...good,auditReport:good.auditReport.replace('Guthrie: 4','Guthrie: -1')},sources('9223372036854775807','4','10'),{...good,auditReport:'wrong report'}, {...good,ballotRetrieval:good.ballotRetrieval+good.ballotRetrieval.split('\n')[1]+'\n'}]){
  const request=requestFor(input);assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);
}
assert.throws(()=>validateRiSources({}));validateRiSources({},{requireComplete:false});
assert.throws(()=>validateRiSources({auditReport:{name:'x',base64:'AA=='}},{}));
console.log('RI RLA import: two exact native/WASM results including counts above 2^53; three source byte/hash checks, public seed, Save/Open, native replay boundary, six matching invalid-source rejections and tampering each source passed.');

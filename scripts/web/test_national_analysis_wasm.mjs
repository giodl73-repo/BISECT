import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {nationalRequest,readNationalReports} from '../../web/lab/national-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/national-analysis-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let matched=0,rejected=0,byteMatches=0;
const fixture=(state,extra={})=>({analyzer:'demographic',state,year:'2020',districts:[{district:1,total_population:'18446744073709551615',note:'a, "quote"\nnext',nested:{x:[1,2]}}],...extra});
async function exact(reports_json){const request={operation:'aggregate-national-analysis',reports_json};const result=engine.execute(request);const path=root+`/request-${matched}.json`;await fs.writeFile(path,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));assert.deepEqual(result,native);const reference=root+`/reference-${matched}`;execFileSync('target/release/examples/national_reference.exe',[path,reference],{encoding:'utf8',stdio:['ignore','pipe','pipe']});for(const [name,key]of [['national.json','report_json'],['national.csv','csv'],['records.json','array_json']]){assert.equal(result[key],await fs.readFile(reference+'/'+name,'utf8'));byteMatches++;}assert.equal(result.report_sha256,createHash('sha256').update(result.report_json).digest('hex'));matched++;return result;}
const base={'RI.json':JSON.stringify(fixture('RI')),'IA.json':JSON.stringify(fixture('IA'))};
const out=await exact(base);assert.equal(out.report.state_count,2);assert.equal(out.report.district_count,2);assert.deepEqual(out.provided_states,['IA','RI']);assert.equal(out.report.districts[0].state,'IA');
await exact({'RI.json':base['RI.json']});
await exact({'RI.json':base['RI.json'].replace('"18446744073709551615"','18446744073709551615')});
const negative=await exact({'RI.json':base['RI.json'].replace('"18446744073709551615"','-9223372036854775808')});assert.equal(negative.report.districts[0].total_population,'-9223372036854775808');
const stateLevel=await exact({'IA.json':JSON.stringify(fixture('IA',{analyzer:'proportionality',districts:undefined,total_dem_votes:123,note:'"x"\nline',per_district_dem_share_sorted:[.2,.8]})),'RI.json':JSON.stringify(fixture('RI',{analyzer:'proportionality',districts:undefined,total_dem_votes:321,per_district_dem_share_sorted:[.5]}))});assert.equal(stateLevel.report.states.length,2);assert.ok(!stateLevel.csv.includes('per_district_dem_share_sorted'));
await exact({'RI.json':JSON.stringify(fixture('RI',{year:2020}))});
async function reject(reports_json){const request={operation:'aggregate-national-analysis',reports_json};assert.throws(()=>engine.execute(request));const path=root+`/reject-${rejected}.json`;await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));rejected++;}
await reject({});await reject({'bad/path.json':base['RI.json']});await reject({'RI.json':'null'});await reject({'RI.json':'{"state":"RI","state":"IA"}'});await reject({'RI.json':base['RI.json'].replace('"district":1','"district":1,"district":2')});await reject({'RI.json':base['RI.json'].replace('"nested":','"__proto__":')});await reject({'RI.json':base['RI.json'].replace('"2020"','null')});await reject({'RI.json':JSON.stringify(fixture('RI',{analyzer:'other'}))});await reject({'RI.json':JSON.stringify(fixture('RI',{districts:[]}))});await reject({'RI.json':JSON.stringify(fixture('RI',{districts:[{district:1},{district:1}]}))});await reject({'RI.json':JSON.stringify(fixture('RI',{districts:[{district:0}]}))});
for(const edit of [{state:'ri'},{year:'2010'},{analyzer:'urban'},{coordinate_basis:'projected-metres'},{dissolve_method:'batch'},{bootstrap_protocol:'other'}])await reject({'RI.json':base['RI.json'],'other.json':JSON.stringify(fixture('IA',edit))});
let deep='0';for(let i=0;i<66;i++)deep='{"x":'+deep+'}';await reject({'RI.json':base['RI.json'].replace('{"x":[1,2]}',deep)});
await reject(Object.fromEntries(Array.from({length:61},(_,i)=>['state'+i+'.json',base['RI.json']])));
await reject({'RI.json':' '.repeat(3*1024*1024+1)});
let read=false;await assert.rejects(readNationalReports([{name:'big.json',size:3*1024*1024+1,arrayBuffer:async()=>{read=true;}}]));assert.equal(read,false);
await assert.rejects(readNationalReports([{name:'same.json',size:1},{name:'same.json',size:1}]));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const files={nationalInputs:{reports:Object.fromEntries(Object.entries(base).map(([n,t])=>[n,source(n,t)]))}};assert.deepEqual(nationalRequest(files.nationalInputs),{operation:'aggregate-national-analysis',reports_json:base});
assert.deepEqual(parseProject(JSON.stringify(createProject({name:'National synthetic evidence',files,operation:'aggregate-national-analysis',constraints:[],result:out,lastOperation:'aggregate-national-analysis'}))).files,files);
assert.equal(countResultTables(out)[0].rows[2][1],'2');
try{const realReports={'IA.json':await fs.readFile('target/full-state-batch-wasm/IA-native.json','utf8'),'RI.json':await fs.readFile('target/full-state-batch-wasm/RI-native.json','utf8')};const real=await exact(realReports);assert.equal(real.report.district_count,6);const realFiles={nationalInputs:{reports:Object.fromEntries(Object.entries(realReports).map(([n,t])=>[n,source(n,t)]))}};await fs.writeFile(root+'/RI-IA.bisect',JSON.stringify(createProject({name:'National aggregation: supplied RI and IA batch reports',files:realFiles,operation:'aggregate-national-analysis',constraints:[],result:real,lastOperation:'aggregate-national-analysis'})));await fs.writeFile(root+'/real-evidence.json',JSON.stringify({states:real.provided_states,districts:6,report_sha256:real.report_sha256,source_sha256:real.source_sha256},null,2));}catch(error){if(error.code!=='ENOENT')throw error;console.log('Real state aggregation not run: prepared report artifacts unavailable.');}
console.log(`National aggregation: ${matched} native/WASM matches, ${byteMatches} identical native JSON/CSV/array files, ${rejected} rejected requests; exact counts, lexical order, incompatibility and project/source guards verified.`);

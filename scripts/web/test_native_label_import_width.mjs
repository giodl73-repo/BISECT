// Actual desktop import versus the browser package, export and selected-root application.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';

const arg=k=>{const i=process.argv.indexOf(k);assert(i>=0&&process.argv[i+1],k);return process.argv[i+1];};
const jsRoot=path.resolve(process.argv.includes('--js-root')?arg('--js-root'):'web/lab');
const module=name=>import(pathToFileURL(path.join(jsRoot,name)).href);
const {labelImportFiles,labelImportRequest,labelImportZip}=await module('label-import-files.js');
const {reviewLabelImportApplication,applyLabelImportApplication}=await module('label-import-application.js');
const cli=await fs.realpath(arg('--cli')),wasm=await fs.readFile(arg('--wasm')),out=path.resolve(arg('--output'));
const legacy=process.argv.includes('--expect-legacy-width-failure'),sha=b=>createHash('sha256').update(b).digest('hex');
const source=(name,b)=>({name,base64:Buffer.from(b).toString('base64')});
const settings={label:'width_import',year:'2020',format:'auto',created:'2026-11-06T00:00:00Z'};
const max='18446744073709551615',over='18446744073709551616',plus32='4294967296',unsafe='9007199254740993';
const csv=(a,b='2')=>`GEOID,district\n50001000100,${a}\n50001000200,${b}\n`;
const geo=(a,b='2')=>`{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"GEOID":"50001000100","district_id":${a}},"geometry":null},{"type":"Feature","properties":{"GEOID":"50001000200","district_id":${b}},"geometry":null}]}`;
const text=(name,value)=>({source:source(name,value),dbf:null,shx:null,registry:null,settings});
const fixture='docs/fixtures/import-label/positive/vermont_two_tracts';
function characterDistricts(dbf,values){
  const header=dbf.readUInt16LE(8),width=dbf.readUInt16LE(10),records=dbf.readUInt32LE(4);
  assert.equal(records,values.length);
  let offset=1,descriptor=-1;
  for(let p=32;p+32<=header;p+=32){
    const name=dbf.subarray(p,p+11).toString('ascii').split('\0')[0];
    if(/^(DISTRICT|DISTRICTID|DISTRICTNO|DIST|CD)$/i.test(name)){descriptor=p;break;}
    offset+=dbf[p+16];
  }
  assert(descriptor>=0);
  const old=dbf[descriptor+16],n=20,next=width-old+n,end=header+records*width;
  assert(end<=dbf.length);
  const result=Buffer.alloc(header+records*next+dbf.length-end);
  dbf.copy(result,0,0,header);result.writeUInt16LE(next,10);result[descriptor+11]=67;result[descriptor+16]=n;result[descriptor+17]=0;
  for(let i=0;i<records;i++){
    const start=header+i*width,dst=header+i*next;dbf.copy(result,dst,start,start+offset);
    result.write(values[i].padEnd(n,' '),dst+offset,n,'ascii');
    dbf.copy(result,dst+offset+n,start+offset+old,start+width);
  }
  dbf.copy(result,header+records*next,end);
  return result;
}
const shape=await fs.readFile(fixture+'.shp'),index=await fs.readFile(fixture+'.shx'),dbf=await fs.readFile(fixture+'.dbf');
const shp=value=>({source:source('input.shp',shape),dbf:source('input.dbf',characterDistricts(dbf,[value,'2'])),shx:source('input.shx',index),registry:null,settings});
const context=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const ids=JSON.parse(context).units.unit_ids;
const aligned=`GEOID,district\n${ids.map((id,i)=>id+','+(i===0?max:'1')).join('\n')}\n`;
const cases=[
  ['csv-control',text('input.csv',csv('4294967295')),'ordinary'],
  ['csv-beyond-wasm32',text('input.csv',csv(plus32)),'wide',plus32],
  ['csv-beyond-js-integer',text('input.csv',csv(unsafe)),'wide',unsafe],
  ['csv-u64-max',text('input.csv',csv(max)),'wide',max],
  ['csv-u64-overflow',text('input.csv',csv(over)),'invalid'],
  ['csv-negative',text('input.csv',csv('-1')),'invalid'],
  ['csv-fraction',text('input.csv',csv('1.5')),'invalid'],
  ['geojson-wide',text('input.geojson',geo(max,unsafe)),'wide',max],
  ['geojson-overflow-row-omitted',text('input.geojson',geo(over)),'ordinary'],
  ['shapefile-character-wide',shp(max),'wide',max],
  ['shapefile-character-overflow',shp(over),'invalid'],
  ['context-alignment-refuses-large-district',text('input.csv',aligned),'wide',max],
];
const engine=await instantiateEngine(wasm);
await fs.mkdir(out);const harness=nativeOutputDirectoryHarness(out),rows=[];
for(const [id,inputs,kind,token] of cases){
  const caseDir=path.join(out,id);await fs.mkdir(caseDir);
  for(const f of [inputs.source,inputs.dbf,inputs.shx])if(f)await fs.writeFile(path.join(caseDir,f.name),Buffer.from(f.base64,'base64'));
  const argv=['label-import',settings.label,'--from',inputs.source.name,'--year',settings.year];
  const native=spawnSync(cli,argv,{cwd:caseDir,encoding:'utf8',windowsHide:true,timeout:25000});
  assert.ifError(native.error);assert.equal(native.status,kind==='invalid'?1:0,native.stderr);
  await fs.writeFile(path.join(caseDir,'native.stdout'),native.stdout);
  await fs.writeFile(path.join(caseDir,'native.stderr'),native.stderr);
  const request=labelImportRequest(inputs);
  await fs.writeFile(path.join(caseDir,'request.json'),JSON.stringify(request));
  const folder=`runs/${settings.label}/${settings.year}/`;
  const readNative=async name=>fs.readFile(path.join(caseDir,name),'utf8');
  if(kind==='invalid'){
    assert.throws(()=>engine.execute(request));
    await assert.rejects(fs.stat(path.join(caseDir,folder,'index.json')),{code:'ENOENT'});
  }else if(kind==='wide'&&legacy){
    assert.throws(()=>engine.execute(request),/u32::MAX/);
    const state=id.startsWith('context-')?'washington':'vermont';
    assert((await readNative(folder+state+'/assignments.json')).includes(token));
  }else{
    const result=engine.execute(request),files=await labelImportFiles(result,inputs);
    assert.equal(result.execution_performed,false);
    for(const [name,bytes] of files){
      const text=Buffer.from(bytes).toString('utf8'),expected=await readNative(name);
      if(name==='.bisect')assert.deepEqual(JSON.parse(text),JSON.parse(expected));
      else if(name.endsWith('/index.json')){
        const a=JSON.parse(text),b=JSON.parse(expected);assert.equal(a.created,settings.created);assert.ok(b.created);
        assert.deepEqual({...a,created:null},{...b,created:null});
      }else assert.equal(text,expected);
    }
    const archive=await labelImportZip(result,inputs);
    await fs.writeFile(path.join(caseDir,'package.zip'),archive);
    const root=await harness.directory('reviewed-import');
    const {review}=await reviewLabelImportApplication(engine,result,inputs,root);
    const receipt=await applyLabelImportApplication(review,{confirmed:true});
    assert.equal(receipt.status,'completed');assert.equal(receipt.registry_readback_verified,true);
    for(const [name,bytes] of files)assert.deepEqual(await fs.readFile(path.join(root.dir,name)),Buffer.from(bytes));
    if(token)assert(Object.entries(result.files).some(([n,text])=>n.endsWith('/assignments.json')&&text.includes(token)));
    if(id==='csv-u64-max'){
      const name=folder+'vermont/assignments.json';
      const changed=value=>{
        const copy=structuredClone(result);copy.files[name]=copy.files[name].replace(max,value);
        assert.notEqual(copy.files[name],result.files[name]);copy.file_sha256[name]=sha(copy.files[name]);return copy;
      };
      for(const value of [over,JSON.stringify(max),'1e0','-1','1.5']){
        await assert.rejects(()=>labelImportFiles(changed(value),inputs));
      }
      const unchanged=await harness.directory('tampered-import');
      const writes=harness.log.length;
      await assert.rejects(()=>reviewLabelImportApplication(engine,changed('18446744073709551614'),inputs,unchanged));
      assert.equal(harness.log.length,writes);
    }
    if(id==='context-alignment-refuses-large-district'){
      const align={operation:'prepare-label-import-plan',input:{input:request.input,state_name:'washington',context_json:context,
        expected_index_sha256:sha(result.index_json),district_basis:'one-based'}};
      assert.throws(()=>engine.execute(align),/at most 500 districts/);
      await fs.writeFile(path.join(caseDir,'alignment-refusal-request.json'),JSON.stringify(align));
    }
    await fs.writeFile(path.join(caseDir,'result.json'),JSON.stringify(result));
  }
  rows.push({id,kind,native_exit:native.status,argv,legacy_width_refusal:legacy&&kind==='wide'});
}
const status=legacy?'reproduced-native-label-import-width-gap':'passed-native-label-import-width';
await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status,rows,native:{path:cli,sha256:sha(await fs.readFile(cli))},
  wasm_sha256:sha(wasm),full_parity:false},null,2));
console.log(`${status}: ${rows.length} actual CLI cases.`);

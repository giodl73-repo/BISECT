// Recheck the complete accepted 50-state display map without rerunning a partition.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNationalMapBatch} from '../../web/lab/national-map-batch.js';

const historicalRoot = path.resolve('target/national-matrix/map-1793562621520');
const snapshotSha = 'd005a4c140a507744a755b30fb433268d700bcbb75a5747dc7d6f689e62909bb';
const options = (key, fallback) => {
  const i = process.argv.indexOf(key);
  const result = i >= 0 ? process.argv[i+1] : fallback;
  assert(result && !result.startsWith('--'), key);return result;
};
const verifyOnly = process.argv.includes('--verify-fixtures-only');
const root = path.resolve(options('--reference',historicalRoot));
const snapshot = options('--snapshot','target/parity-current-matrix-20261101/verification-manifest.json');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const snapBytes = await fs.readFile(snapshot);
assert.equal(sha(snapBytes), snapshotSha);
const pins = new Map(JSON.parse(snapBytes).artifacts.map(r => [path.resolve(r.path), r]));
async function retained(file) {
  const pin = pins.get(path.join(historicalRoot,path.basename(file)));
  assert.ok(pin, 'Missing prior independent file identity: ' + file);
  const bytes = await fs.readFile(file);
  assert.equal(bytes.length, pin.byte_length, file);assert.equal(sha(bytes), pin.sha256, file);
  return bytes;
}
const within = name => path.join(root, name);
const declaration = JSON.parse(await retained(within('acceptance-contract.json')));
assert.equal(declaration.id, 'national-full-roster-standard-core-42-display-map-v1');
assert.deepEqual([declaration.required_states, declaration.required_units, declaration.required_districts, declaration.required_population], [50,84208,435,'330759736']);
const old = JSON.parse(await retained(within('evidence.json')));
assert.equal(old.exact_native_stream_replies, 52);
const nativeRecordBytes = await fs.readFile(options('--native-engine-record',old.native_engine_record.path));
assert.equal(sha(nativeRecordBytes),old.native_engine_record.sha256);
assert.equal(nativeRecordBytes.length,old.native_engine_record.byte_length);
assert.deepEqual(JSON.parse(nativeRecordBytes).native_sequence,old.native_reference);
const native = path.resolve(options('--native-sequence',old.native_reference.path));
const nativeBytes = await fs.readFile(native);
assert.equal(nativeBytes.length,old.native_reference.byte_length);
assert.equal(sha(nativeBytes), old.native_reference.sha256);
// Stream the original native requests through a hash before invoking the owned executable.
const requestPath = path.resolve(options('--requests',within('requests.jsonl')));
const requestPin = pins.get(path.join(historicalRoot,'requests.jsonl'));
assert.ok(requestPin);
const f = await fs.open(requestPath, 'r');
const requestHash = createHash('sha256');let bytesRead = 0;
for await (const bytes of f.createReadStream()) {requestHash.update(bytes);bytesRead += bytes.length;}
assert.equal(bytesRead, requestPin.byte_length);assert.equal(requestHash.digest('hex'), requestPin.sha256);
const historical = await retained(within('native-stdout.jsonl'));
const input = await retained(within('national-map-sources.json'));
const archive = await retained(within('national.bisect-workbench'));
const expectedBytes = await retained(within('result.json'));
if(verifyOnly){
  console.log(JSON.stringify({status:'passed-exact-retained-national-fixtures',states:50,execution_performed:false}));
  process.exit(0);
}
const output = path.resolve(options('--output'));
const sites = {core: path.resolve(options('--core-site')), c: path.resolve(options('--c-site'))};
await fs.mkdir(output);
const {stdout, stderr} = await promisify(execFile)(native,[requestPath],{encoding:null,timeout:1800000,maxBuffer:64*1024*1024,windowsHide:true});
await fs.writeFile(path.join(output,'native-stderr.txt'),stderr);
await fs.writeFile(path.join(output,'native-stdout.jsonl'),stdout);
assert.deepEqual(stdout,historical,'Fresh native stream differs from the accepted retained reference');
const rows = stdout.toString('utf8').trim().split('\n').map(x => JSON.parse(x));
assert.equal(rows.length, 52);
const expected = JSON.parse(expectedBytes);
const pack = JSON.parse(input);
assert.equal(Object.keys(pack.states).length,50);
const packages={}, profiles={};
for(const [profile,site] of Object.entries(sites)) {
  const folder=path.join(output,profile);await fs.mkdir(folder);
  const wasmFile=path.join(site,'bisect_wasm.wasm'), wasm=await fs.readFile(wasmFile), engine=await instantiateEngine(wasm);
  const image=JSON.parse(await fs.readFile(path.resolve(site,'../image.json'),'utf8'));
  assert.equal(sha(wasm),image.wasm_sha256);assert.equal(wasm.length,image.wasm_bytes);
  const checks=[];
  const actual=await executeNationalMapBatch({execute(request){
    const ref=rows[checks.length];assert.equal(ref?.ok,true);
    const result=engine.execute(request);assert.deepEqual(result,ref.result);
    checks.push({operation:request.operation,result_sha256:sha(JSON.stringify(result))});return result;
  }},pack,{onProgress:p=>console.log(profile+' '+p.state+' '+p.states+'/50')});
  assert.equal(checks.length,52);assert.deepEqual(actual,expected);
  assert.deepEqual([actual.provided_states.length,actual.units,actual.districts,actual.execution_performed],[50,84208,435,false]);
  const resultHash=sha(expectedBytes), mapHash={png:sha(Buffer.from(actual.png_base64,'base64')),svg:sha(actual.svg_text)};
  assert.deepEqual(mapHash,{png:old.map.png.sha256,svg:old.map.svg.sha256});
  await fs.writeFile(path.join(folder,'national.bisect-workbench'),archive);
  await fs.writeFile(path.join(folder,'result.json'),expectedBytes);
  const current={status:'passed',states:50,units:84208,districts:435,exact_native_stream_replies:52,
    wasm:{path:wasmFile,sha256:sha(wasm),byte_length:wasm.length},native_reference:old.native_reference,
    native_execution_fresh:true,source_snapshot_sha256:snapshotSha,result_sha256:resultHash,map_hashes:mapHash,
    checks,partition_operations:0,full_parity:false};
  await fs.writeFile(path.join(folder,'evidence.json'),JSON.stringify(current,null,2));
  profiles[profile]=current;
  packages[profile]=[current.wasm];
}
assert.notEqual(profiles.core.wasm.sha256,profiles.c.wasm.sha256);
await fs.writeFile(path.join(output,'engine-validation.json'),JSON.stringify({status:'passed',source_unchanged:true,
  source_snapshot_sha256:snapshotSha,packages,profiles,full_parity:false},null,2));
console.log(JSON.stringify({status:'passed-current-retained-national-map-native-and-wasm',output,states:50,profiles:2}));

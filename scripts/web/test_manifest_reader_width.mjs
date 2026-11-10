// Compare selected read-only manifest consumers with an actual native 64-bit tool.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const option = name => process.argv.find(a => a.startsWith(`--${name}=`))?.slice(name.length + 3);
const wasm = option('wasm'), native = option('native'), cli = option('cli');
if (!wasm || !native || !cli) throw Error('Pass explicit --wasm, --native and --cli paths.');
const root = await fs.mkdtemp(path.resolve('target/manifest-reader-width/run-'));
const hash = b => createHash('sha256').update(b).digest('hex');
const pin = async p => ({path:p, sha256:hash(await fs.readFile(p))});
const engine = await instantiateEngine(await fs.readFile(wasm));
// Repository fixtures make the registered gate independent of historical target outputs.
const fixtureRoot='docs/examples/rplan-golden-packages/T.14+spectral-partitioning';
const fixtureManifest=JSON.parse(await fs.readFile(fixtureRoot+'/manifest.json','utf8'));
const fixtureFiles={};
for(const entry of fixtureManifest.files){
    const bytes=await fs.readFile(fixtureRoot+'/'+entry.path);
    assert.equal(hash(bytes),entry.sha256);
    fixtureFiles[entry.path]=bytes.toString('base64');
}
const role=name=>fixtureManifest.files.find(entry=>entry.role===name).path;
const certificate=JSON.parse(Buffer.from(fixtureFiles[role('certificate')],'base64').toString('utf8'));
const base={label:'certificate-test',state_code:'RI',year:'2020',chamber:'congressional',
    num_districts:2,population_source:'fixture',partition_mode:'fixture',seed:42,
    binary_version:'fixture',binary_sha256:'',binary_download_url:'',adjacency_file:'fixture',
    adjacency_sha256:'',adjacency_build_command:'',adjacency_build_version:'',tiger_source_url:'',
    created_at:'2026-10-08T00:00:00Z',balance_tolerance_pct:0.5,population_balance_valid:true,
    audit_certificate_path:role('certificate'),audit_certificate_sha256:hash(Buffer.from(fixtureFiles[role('certificate')],'base64')),
    audit_certificate_content_hash:certificate.content_hash,audit_result:certificate.result,
    legal_profile_id:certificate.legal_profile.profile_id,context_hash:certificate.context_hash,
    rplan_path:role('plan'),rctx_path:role('context')};
const baseline={manifest_json:JSON.stringify(base),files_base64:fixtureFiles};
const fields = ['num_districts','seats_per_district','total_seats','split_seeds','spectral_iters',
    'ilp_solve_report_count','flip_steps','flip_visited_count','flip_selected_plan_rank',
    'burst_length','n_bursts','selected_burst_idx','n_units'];
const b64 = text => Buffer.from(text).toString('base64');
const records = [], cliRecords = [];
async function verifyCli(name, input, status, diagnostic) {
    const directory = path.join(root,`cli-${cliRecords.length}`);
    await fs.mkdir(directory);
    for (const [name,encoded] of Object.entries(input.files_base64)) {
        assert(!name.includes('..') && !path.isAbsolute(name));
        const file = path.join(directory,name);
        await fs.mkdir(path.dirname(file),{recursive:true});
        await fs.writeFile(file,Buffer.from(encoded,'base64'));
    }
    const manifestPath = path.join(directory,'manifest.json');
    await fs.writeFile(manifestPath,input.manifest_json);
    const args = ['verify','--manifest',manifestPath,'--dry-run'];
    const proc = spawnSync(cli,args,{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024});
    assert.equal(proc.status,status,`${name}: ${proc.stderr}`);
    assert(proc.stderr.includes(diagnostic),`${name}: ${proc.stderr}`);
    cliRecords.push({name,args,status:proc.status,stdout:proc.stdout,stderr:proc.stderr});
}
async function exact(name, operation, input) {
    const request = {operation,input}, requestPath = path.join(root,`${records.length}.json`);
    await fs.writeFile(requestPath, JSON.stringify(request));
    const proc = spawnSync(native,[requestPath],{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024});
    assert.equal(proc.status,0,`${name}: ${proc.stderr}`);
    const expected = JSON.parse(proc.stdout), actual = engine.execute(request);
    // Native 64-bit usize and the read-only u64 adapter name the same domain differently.
    // Preserve the raw native output; normalize only this parser type spelling.
    if (name.startsWith('invalid-') && typeof expected.error === 'string') {
        expected.error = expected.error.replaceAll('expected usize','expected u64');
    }
    assert.deepEqual(actual,expected,name);
    records.push({name,operation,input,actual,native:{status:proc.status,stdout:proc.stdout,stderr:proc.stderr}});
    return actual;
}
function manifest(changes, rawField, token) {
    const value = {...base,year:'2020',label:'fallback',...changes};
    if (rawField) value[rawField] = '__RAW_COUNT__';
    return JSON.stringify(value).replace('"__RAW_COUNT__"',token);
}
const variants = [['baseline',manifest({})],...fields.map(field =>
    [`wide-${field}`,manifest({},field,'18446744073709551615')])];
for (const [name,text] of variants) {
    const cert = await exact(`${name}-certificate`,'verify-manifest-certificate',{
        ...baseline,manifest_json:text});
    assert.equal(cert.status,'verified');
    await verifyCli(`${name}-certificate`,{...baseline,manifest_json:text},0,'certificate verified');
    const prechecks = await exact(`${name}-prechecks`,'verify-manifest-prechecks',{
        ...baseline,manifest_json:manifest({ilp_audit_summary_path:null},
            name.startsWith('wide-') ? name.slice(5) : null,'18446744073709551615'),manifest_path:'manifest.json'});
    assert.equal(prechecks.certificate.status,'verified');
    const directory = 'v1/2020/plans/chosen';
    const discovery = await exact(`${name}-discovery`,'discover-original-assignments',{
        manifest_json:text,explicit_label:'chosen',files_base64:{
            [`${directory}/manifest.json`]:b64(text),
            [`${directory}/data/final_assignments.json`]:b64(' {"unit":2} \n')}});
    assert.equal(discovery.status,'found');
    assert.equal(discovery.source_json,' {"unit":2} \n');
    const summary = await exact(`${name}-summary`,'summarize-plan-labels',{
        labels:['one','two'],plans:{one:{'manifest.json':text},two:{'manifest.json':text}}});
    assert(summary.availability.every(row => row.manifest));
}
// A wide claim must reach the actual comparison, rather than fail at wasm32 parsing.
const report = await fs.readFile('docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/ilp-solve-report.json','utf8');
const model = await fs.readFile('docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/node_root.lp');
const summary = engine.execute({operation:'audit-ilp-artifacts',input:{reports:{'node.json':report},models_base64:{'node_root.lp':model.toString('base64')},summary_json:null}}).summary_json;
for (const token of ['1','4294967296','9007199254740993','18446744073709551615']) {
    const text = manifest({ilp_audit_summary_path:'nodes/audit-summary.json',
        ilp_audit_summary_sha256:hash(summary),ilp_solve_report_dir:'nodes'},'ilp_solve_report_count',token);
    const input = {
        manifest_json:text,manifest_path:'manifest.json',files_base64:{...baseline.files_base64,
            'nodes/node.json':b64(report),'nodes/node_root.lp':model.toString('base64'),
            'nodes/audit-summary.json':b64(summary)}};
    const result = await exact(`audited-count-${token}`,'verify-manifest-prechecks',input);
    assert.equal(result.computed_ilp_summary.checked,1);
    assert.equal(result.status,token === '1' ? 'verified' : 'failed');
    if (token !== '1') {
        assert.equal(result.error,`ILP report count mismatch: manifest=${token}, audited=1`);
        assert.equal(result.certificate.status,'not-run');
    }
    await verifyCli(`audited-count-${token}`,input,token === '1' ? 0 : 1,
        token === '1' ? 'certificate verified' : `ILP report count mismatch: manifest=${token}, audited=1`);
}
for (const token of ['-1','1.5','18446744073709551616','"2"']) {
    const text = manifest({},'num_districts',token);
    for (const operation of ['verify-manifest-certificate','verify-manifest-prechecks']) {
        const result = await exact(`invalid-${token}-${operation}`,operation,{
            ...baseline,manifest_json:text,...(operation.endsWith('prechecks') ? {manifest_path:'manifest.json'} : {})});
        assert.equal(result.status,'failed');
        assert.match(result.error,/invalid manifest JSON/);
    }
    const result = await exact(`invalid-${token}-summary`,'summarize-plan-labels',{
        labels:['one','two'],plans:{one:{'manifest.json':text}}});
    assert.equal(result.availability[0].manifest,false);
    assert.throws(() => engine.execute({operation:'discover-original-assignments',input:{manifest_json:text,explicit_label:null,files_base64:{}}}),/Invalid run manifest/);
}
const sources = await Promise.all(['crates/bisect-runner/src/ilp_audit.rs',
    'crates/bisect-report/src/manifest_schema.rs',...['manifest_certificate','manifest_prechecks','assignment_discovery','multi_plan_summary'].map(n=>`crates/bisect-wasm/src/${n}.rs`),
    'scripts/web/test_manifest_reader_width.mjs',fixtureRoot+'/manifest.json',
    ...fixtureManifest.files.map(entry=>fixtureRoot+'/'+entry.path),
    'docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/ilp-solve-report.json',
    'docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/node_root.lp'].map(pin));
const evidence = {status:'passed-manifest-reader-width',sources,native:await pin(native),cli:await pin(cli),wasm:await pin(wasm),records,cli_records:cliRecords,
    scope:'Actual native 64-bit tool versus WASM selected-byte operations, including valid certificate binding, selected assignment-context discovery, summary manifest availability and real ILP report-count mismatch ordering. Native Verify dry-run checks cover certificate and ILP precheck branches. Full Verify/Report orchestration, browser UI, OS grants and full parent acceptance require separate evidence.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({root,status:evidence.status,records:records.length,cli_records:cliRecords.length}));

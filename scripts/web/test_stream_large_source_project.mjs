// Validate retained original state recipes without acquisition or engine execution.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {stateSourceWorkbench} from '../../web/lab/local-state-output.js';
import {parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const topology=process.argv.includes('--topology'),suffix=topology?'-topology':'';
const root='target/stream-state-source-batch',evidence=JSON.parse(await fs.readFile(root+'/original-ca'+suffix+'-evidence.json','utf8')),stateRoot=root+'/'+evidence.output_folder+'/CA';
const recipe=evidence.states[0].output.artifacts.find(a=>['state-run-jobs.json','state-source-recipe.json'].includes(a.name));
const [job]=JSON.parse(await fs.readFile(stateRoot+'/'+recipe.name,'utf8')),result=JSON.parse(await fs.readFile(root+'/native-oracles/CA-52'+suffix+'.json','utf8'));
const bytes=stateSourceWorkbench(job,{result}),project=parseWorkbenchArchive(new TextDecoder().decode(bytes));
assert.equal(project.files.tractPreparationSettings.geometry_profile,topology?'display-topology':'display-simplified');assert.equal(project.operation,'prepare-census-engine-input');assert.equal(project.result,null);assert.equal(project.lastOperation,null);assert.deepEqual(project.files.tigerInputs,job.source_preparation.tiger);assert.deepEqual(project.files.pl94Inputs,job.source_preparation.pl94);assert.deepEqual(project.files.configSource,job.configuration.source);assert.deepEqual(project.files.configRunInputs,job.configuration.inputs);assert.equal(project.files.configRunSettings.districts,52);assert.equal(project.files.tuiRunSettings.form.seed,'18446744073709551615');assert.equal(project.files.tuiRunWorkflowSettings.report,true);assert(bytes.length<=128*1024*1024);
await fs.mkdir('target/stream-source-projects',{recursive:true});await fs.writeFile('target/stream-source-projects/CA'+suffix+'.bisect-workbench',bytes);await fs.writeFile('target/stream-source-projects/evidence'+suffix+'.json',JSON.stringify({state:'CA',byte_length:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),sources_exact:true,native_districts:52,seed:project.files.tuiRunSettings.form.seed,execution_performed:false},null,2));
console.log('Original CA source project: exact selected parts/YAML/options/follow-up/max-u64 seed, native 52-seat allocation, bounded archive and inert roundtrip passed.');

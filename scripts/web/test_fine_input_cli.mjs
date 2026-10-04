// Optional local RI Census preparation gate. Requires downloaded TIGER files.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {validateMultiscaleBundle} from '../../web/lab/multiscale-input.js';
const source='target/authentic-bg/RI',dir=await fs.mkdtemp('target/authentic-bg/cli-');
const paths={tract_graph:`${source}/tract.json`,shp:`${source}/tiger/tl_2020_44_bg.shp`,dbf:`${source}/tiger/tl_2020_44_bg.dbf`,shx:`${source}/tiger/tl_2020_44_bg.shx`,prj:`${source}/tiger/tl_2020_44_bg.prj`,geography:'data/2020/redistricting/rhode_island/rigeo2020.pl',population:'data/2020/redistricting/rhode_island/ri000012020.pl'};
const base=['--state','RI','--year','2020','--tract-graph',paths.tract_graph,'--shape',paths.shp,'--geography',paths.geography,'--population',paths.population];
const execute=args=>execFileSync('target/release/wasm-fine-input.exe',args,{encoding:'utf8',stdio:'pipe'});
for(const name of ['first','second'])execute([...base,'--output',`${dir}/${name}.json`,'--native-island-bridges']);
const first=await fs.readFile(`${dir}/first.json`),second=await fs.readFile(`${dir}/second.json`);assert.deepEqual(first,second,'Preparation must be byte-identical across executions.');
const bundle=JSON.parse(first),report=JSON.parse(await fs.readFile(`${dir}/first.sources.json`,'utf8'));
validateMultiscaleBundle(bundle,JSON.parse(await fs.readFile(paths.tract_graph,'utf8')));
for(const [key,file]of Object.entries(paths)){const h=createHash('sha256');for await(const chunk of createReadStream(file))h.update(chunk);assert.equal(report.sources[key].sha256,h.digest('hex'));}
let rejected=0;const reject=args=>{assert.throws(()=>execute(args));rejected++;};
reject([...base,'--output',`${dir}/first.json`,'--native-island-bridges']);
reject([...base,'--output',`${dir}/no-bridges.json`]);await assert.rejects(fs.access(`${dir}/no-bridges.json`));
const graph=JSON.parse(await fs.readFile(paths.tract_graph,'utf8'));
for(const [name,mutate]of [['scope',g=>g.state='WA'],['schema',g=>g.schema_version=99],['population',g=>g.population[0]++],['quotient',g=>g.adjacency[0]=[]]]){const bad=structuredClone(graph);mutate(bad);const file=`${dir}/${name}-tract.json`;await fs.writeFile(file,JSON.stringify(bad));reject([...base,'--tract-graph',file,'--output',`${dir}/${name}.json`,'--native-island-bridges']);}
for(const ext of ['shp','dbf','shx'])await fs.copyFile(paths[ext],`${dir}/wrong-crs.${ext}`);
await fs.writeFile(`${dir}/wrong-crs.prj`,'PROJCS["unsupported"]');reject([...base,'--shape',`${dir}/wrong-crs.shp`,'--output',`${dir}/wrong-crs.json`,'--native-island-bridges']);
await fs.writeFile('target/authentic-bg/cli-verification.json',JSON.stringify({units:bundle.graph.geoids.length,bridges:report.bridges.length,identical_preparations:2,verified_source_hashes:7,rejected,scope:'Local RI 2020 Census sources'},null,2));
console.log(`Fine preparation CLI: two identical ${bundle.graph.geoids.length}-unit bundles, seven source hashes verified, ${rejected} rejected requests.`);

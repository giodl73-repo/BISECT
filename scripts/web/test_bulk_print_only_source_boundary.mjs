// Use the real WASM parser and packaged jobs; record every engine dispatch.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';

const toolkit=path.resolve(process.argv[2]);
const load=name=>import(pathToFileURL(path.join(toolkit,name)).href);
const {instantiateEngine}=await load('wasm-engine.js');
const wasm=await instantiateEngine(await readFile(path.join(toolkit,'bisect_wasm.wasm')));
const settings={label:'Print only',chamber:'congressional',created_at:'',inspect_only:true};
const invalid={name:'not-a-source.json',text:'{'};
const checks=[];
for(const [family,exportName] of [['run','executeNativeRunJob'],['states','executeNativeStatesJob']]){
  const execute=(await load(`native-${family}-job.js`))[exportName];
  const base=['--year','2020','--states','RI','--version','vPrint','--output-dir','outputs/vPrint'];
  const print=[...base,'--print-only'];
  for(const graph of [true,false]){
    for(const selection of [null,invalid]){
      const source=graph?{graph_inputs:{graph:selection,auxiliary:null,settings}}:
        {prepared_inputs:{source:selection,settings:{...settings,inspect_only:false}}};
      const calls=[],progress=[];
      const run=args=>execute({execute(request){calls.push(request.operation);return wasm.execute(request);}},
        {...source,command:{arguments_json:JSON.stringify(args)}},{onProgress:event=>progress.push(event.stage)});
      const result=await run(print);
      assert.equal(result.execution_performed,false);
      assert.equal(result[`native_${family}_print_only`],true);
      assert.deepEqual(result.native_command.arguments,print);
      assert(!('prepared_request_json' in result)&&!('prepared_review' in result));
      assert.deepEqual(calls.splice(0),[`inspect-native-${family}-command`]);
      await assert.rejects(run([...print,'--not-a-flag']));
      assert.deepEqual(calls.splice(0),[`inspect-native-${family}-command`]);
      assert.deepEqual(progress,[]);
      if(selection===null){
        await assert.rejects(run(base),/Select (prepared graph|ordinary\/build\/multiscale engine request) JSON first/);
        assert.deepEqual(calls,[]);
      }
      checks.push({family,mode:graph?'graph':'template',selection:selection===null?'none':'invalid JSON'});
    }
  }
}
console.log(JSON.stringify({status:'passed',checks}));

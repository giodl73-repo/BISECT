import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
export async function acquisitionRecoveryCase(engine,root){
 await fs.mkdir(root,{recursive:true});const shapes=path.join(root,'shapes'),writer=path.join(root,'native-tiger-fixture.exe');await fs.copyFile('target/debug/examples/tiger_reference.exe',writer);execFileSync(writer,['fixtures',shapes],{windowsHide:true});
 const python=String.raw`import pathlib,zipfile,sys
r=pathlib.Path(sys.argv[1])
for state,fips in [('RI','44'),('VT','50')]:
 with zipfile.ZipFile(r/(state+'-tiger.zip'),'w',compression=8) as z:
  for part in ['shp','dbf','shx']:
   b=(r/'shapes'/'tracts'/('units.'+part)).read_bytes()
   if part=='dbf':b=b.replace(b'44',fips.encode('ascii'))
   z.writestr('units.'+part,b)
  z.writestr('units.prj',b'GEOGCS["North_American_1983",UNIT["Degree",1]]')
 with zipfile.ZipFile(r/(state+'-pl94.zip'),'w',compression=8) as z:
  z.writestr('rigeo2020.pl',('PLST|AA|750|||||0000001|7500000US'+fips+'0010001001001|unused\nPLST|AA|750|||||0000002|7500000US'+fips+'0010002001001|unused\n').encode())
  z.writestr('ri000012020.pl',b'PLST|AA|000|01|0000001|11|unused\nPLST|AA|000|01|0000002|7|unused\n')
`;
 execFileSync('python',['-c',python,root],{windowsHide:true});
 const selection={manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]},plan=engine.execute({operation:'plan-fetch-sources',input:selection});
 const files={fetchPlanInputs:{source:{name:'acquisition-recovery-selection.json',base64:Buffer.from(JSON.stringify(selection)).toString('base64')}},nativeStateSourceSettings:{enabled:true,command_kind:'state'},stateCommandInputs:{arguments_json:JSON.stringify(['--state','RI','--year','2020','--metis-engine','metis-core','--seed','18446744073709551615'])},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Recovered_sources',chamber:'congressional',created_at:'2026-10-23T00:00:00Z',inspect_only:true}}};
 const bodies=new Map();for(const item of plan.items.filter(i=>['tiger','pl94171'].includes(i.kind)))bodies.set(item.url,await fs.readFile(path.join(root,item.state_code+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip')));
 return {plan,files,inputs:{jobs:null,workers:1},bodies};
}

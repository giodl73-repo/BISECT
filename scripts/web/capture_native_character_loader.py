import json,subprocess
from pathlib import Path
root=Path('target/native-character-loader');root.mkdir(parents=True,exist_ok=True)
economic='geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11\n'
housing='geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n'
fixtures=[]
for kind,header,row in [('economic',economic,'44001000100,1000,10,20,30,100,200,100,50,50'),('housing',housing,'44001000100,.8,.1,.6,.2')]:
 cases={'valid':header+row+'\n','empty':'','header-only':header,'missing-column':'geoid,x\n44001000100,1\n','short':header+'44001000100,1\n','bad-number':header+row.replace('1000','bad',1).replace('.8','bad',1)+'\n','bom':'\ufeff'+header+row+'\n','duplicate-row':header+row+'\n'+row.replace('1000','2000',1).replace('.8','.2',1)+'\n','foreign':header+row.replace('44001000100','19001000100')+'\n','quoted-geoid':header+row.replace('44001000100','"44001000100"')+'\n','unsafe-key':header+row.replace('44001000100','__proto__')+'\n','negative':header+row.replace('1000','-1',1).replace('.8','-2',1)+'\n','nonfinite':header+row.replace('1000','NaN',1).replace('.8','NaN',1)+'\n'}
 if kind=='housing':cases['aliases-precedence']='geoid,pct_sf,pct_single_family,pct_mf,pct_multifamily,pct_owner,housing_vintage\n44001000100,.1,.8,.2,.7,.9,2\n'
 for name,text in cases.items():
  cwd=root/(kind+'-'+name);folder=cwd/'data/2020'/('lodes' if kind=='economic' else 'acs_housing');folder.mkdir(parents=True,exist_ok=True)
  path=folder/('fixture_wac_tract.csv' if kind=='economic' else 'fixture_housing_2020.csv');path.write_bytes(text.encode('utf-8'))
  result=subprocess.run([str(Path('target/release/examples/native_character_loader_reference.exe').resolve()),kind],cwd=cwd,capture_output=True,text=True,check=True)
  value=json.loads(result.stdout);relative=str(path.relative_to(cwd));value['native_error']=value.get('native_error','').replace(relative,'fixture.csv') if 'native_error' in value else None
  fixture={'kind':kind,'name':name,'text':text,'expected':value};fixtures.append(fixture)
(root/'original-native-golden.json').write_bytes((json.dumps(fixtures,indent=2,ensure_ascii=False)+'\n').encode('utf-8'))
print('Captured',len(fixtures),'original native CSV reader cases before parser extraction.')

import base64,json,zipfile
from pathlib import Path
root=Path('target/paired-census-resolution');root.mkdir(parents=True,exist_ok=True)
for year in [2000,2010,2020]:
 for kind in ['county','tract','block-group','block']:
  src=Path('target/census-preparation-wasm')/f'{year}-{kind}'
  project=json.loads((src/'project.bisect').read_text(encoding='utf-8'));files=project['files'];out=root/f'{year}-{kind}';out.mkdir(exist_ok=True)
  with zipfile.ZipFile(out/'tiger.zip','w',zipfile.ZIP_DEFLATED) as archive:
   for part in ['shp','dbf','shx']:archive.writestr('units.'+part,base64.b64decode(files['tigerInputs'][part]['base64']))
   archive.writestr('units.prj',base64.b64decode(files['tractPreparationSettings']['projection']['base64']))
  with zipfile.ZipFile(out/'pl94.zip','w',zipfile.ZIP_DEFLATED) as archive:
   archive.writestr('rigeo'+str(year)+'.pl',base64.b64decode(files['pl94Inputs']['geography']['base64']))
   if files['pl94Inputs']['population']:archive.writestr('ri00001'+str(year)+'.pl',base64.b64decode(files['pl94Inputs']['population']['base64']))
  (out/'expected.json').write_bytes(json.dumps(project['result']).encode('utf-8'))
print('Prepared 12 synthetic paired ZIP fixtures for four resolutions and three vintages.')

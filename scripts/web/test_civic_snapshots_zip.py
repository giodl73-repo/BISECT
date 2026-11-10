import base64,hashlib,json
from pathlib import Path
from zipfile import ZipFile
root=Path('target/civic-snapshots');archives=0;files=0;body_bytes=0
pairs=[(p,p.with_name(p.stem+'-result.json')) for p in root.glob('*.zip')]
browser=Path('target/civic-snapshot-browser');pairs.append((browser/'package.zip',browser/'package-result.json'))
for archive,source in pairs:
 result=json.loads(source.read_text(encoding='utf-8'));expected=result['files_base64'];label=result['manifest']['label'];prefix=f'v1/civic_inputs/{label}/'
 with ZipFile(archive) as z:
  assert z.testzip() is None;assert set(z.namelist())==set(expected)
  for item in z.infolist():
   assert item.filename.startswith(prefix);assert not any(p in ('','..','.') for p in item.filename.split('/'));assert not ((item.external_attr>>16)&0o111);assert z.read(item)==base64.b64decode(expected[item.filename],validate=True);files+=1
  for record in result['manifest']['url_snapshots']:
   if record['http_status']==0:assert not record['snapshot_path'];continue
   body=z.read(prefix+'snapshots/'+record['snapshot_path']);assert len(body)==record['content_length_bytes'];assert hashlib.sha256(body).hexdigest()==record['body_sha256'];assert len(body)<=5*1024*1024;body_bytes+=len(body)
 archives+=1
receipt={'archives':archives,'exact_files':files,'body_bytes_checked_including_shared_paths':body_bytes,'crc_and_path_checks':True,'manifest_body_hashes_and_byte_counts':True};(root/'zip-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
evidence=browser/'evidence.json'
if evidence.exists():v=json.loads(evidence.read_text());v['zip_check_pending']=False;evidence.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
print('Civic ZIP validation:',archives,'archives,',files,'exact files and manifest body hashes passed.')

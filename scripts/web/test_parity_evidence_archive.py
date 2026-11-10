from pathlib import Path
import hashlib,json,os,stat,struct,subprocess,tempfile,unittest,zipfile
from unittest.mock import patch
import parity_evidence_archive as archive

class EvidenceArchiveTests(unittest.TestCase):
    def setUp(self):
        # Retained test fixtures live in ignored target; never touch a user project.
        base=Path('target/parity-evidence-archive-tests');base.mkdir(parents=True,exist_ok=True)
        self.root=Path(tempfile.mkdtemp(prefix='case-',dir=base)).resolve();self.dest=self.root/'checkout';self.dest.mkdir()

    def capsule(self,rows=None,objects=None,extra=None,raw=None,stem='evidence'):
        data=b'owned fixture bytes';key=hashlib.sha256(data).hexdigest()
        rows=rows if rows is not None else [{'path':archive.PREFIX+'/blobs/'+key+'.bin','sha256':key,'bytes':len(data)}]
        manifest={'schema':'bisect-parity-child-evidence-v1','files':rows,'total_bytes':sum(r['bytes'] for r in rows)}
        raw=raw if raw is not None else archive.canonical(manifest);objects=objects if objects is not None else {key:data}
        zipped=self.root/(stem+'.zip');lock=self.root/(stem+'.lock.json')
        with zipfile.ZipFile(zipped,'x') as z:
            z.writestr('manifest.json',raw)
            for name,value in objects.items():z.writestr('objects/'+name,value)
            if extra:
                for name,value in extra:z.writestr(name,value)
        self.relock(zipped,lock,raw,len(rows),manifest['total_bytes']);return zipped,lock

    def relock(self,zipped,lock,raw,count,total):
        lock.write_bytes(archive.canonical({'schema':'bisect-parity-child-evidence-lock-v1','archive_sha256':archive.digest(zipped),'archive_bytes':zipped.stat().st_size,'manifest_sha256':hashlib.sha256(raw).hexdigest(),'files':count,'total_bytes':total}))

    def refused(self,zip_file,lock):
        with self.assertRaises((ValueError,zipfile.BadZipFile,RuntimeError)):archive.import_archive(self.dest,zip_file,lock)
        self.assertFalse((self.dest/'target').exists())

    def test_exact_roundtrip_and_no_execution(self):
        zipped,lock=self.capsule();result=archive.import_archive(self.dest,zipped,lock);self.assertEqual(result['executed_files'],0)
        values=list((self.dest/archive.PREFIX/'blobs').iterdir());self.assertEqual(len(values),1);self.assertEqual(values[0].read_bytes(),b'owned fixture bytes');self.assertEqual(values[0].stat().st_nlink,1)

    def test_existing_proofs_preserved(self):
        zipped,lock=self.capsule();directory=self.dest/archive.PREFIX;directory.mkdir(parents=True);f=directory/'keep.json';f.write_bytes(b'keep')
        with self.assertRaises(ValueError):archive.import_archive(self.dest,zipped,lock)
        self.assertEqual(f.read_bytes(),b'keep')

    def test_publication_failure_preserves_verified_stage(self):
        zipped,lock=self.capsule()
        with patch.object(archive.os,'rename',side_effect=PermissionError('owned simulated publication lock')):
            with self.assertRaisesRegex(PermissionError,'publication lock'):archive.import_archive(self.dest,zipped,lock)
        self.assertFalse((self.dest/archive.PREFIX).exists());stages=list((self.dest/'target').glob('.parity-evidence-import-*'));self.assertEqual(len(stages),1)
        self.assertEqual(json.loads((stages[0]/'failure.json').read_text(encoding='utf-8'))['status'],'failed-restore');self.assertEqual(next((stages[0]/'payload/blobs').iterdir()).read_bytes(),b'owned fixture bytes')

    def test_transient_windows_sharing_failure_retries_same_stage(self):
        zipped,lock=self.capsule();rename=archive.os.rename;seen=[]
        def locked_first(src,dst):
            seen.append((src,dst))
            if len(seen)==1:
                error=PermissionError('owned simulated Windows sharing lock');error.winerror=32;raise error
            return rename(src,dst)
        with patch.object(archive.os,'rename',side_effect=locked_first):result=archive.import_archive(self.dest,zipped,lock)
        self.assertEqual(seen,[seen[0],seen[0]]);self.assertEqual(result['executed_files'],0)
        self.assertEqual(next((self.dest/archive.PREFIX/'blobs').iterdir()).read_bytes(),b'owned fixture bytes')

    def test_windows_retry_preserves_new_destination(self):
        zipped,lock=self.capsule()
        def destination_appears(src,dst):
            dst.mkdir();(dst/'keep').write_bytes(b'other writer')
            error=PermissionError('owned simulated Windows access lock');error.winerror=5;raise error
        with patch.object(archive.os,'rename',side_effect=destination_appears) as rename:
            with self.assertRaisesRegex(ValueError,'appeared'):archive.import_archive(self.dest,zipped,lock)
        self.assertEqual(rename.call_count,1);self.assertEqual((self.dest/archive.PREFIX/'keep').read_bytes(),b'other writer')

    def test_receipt_failure_preserves_successful_publication(self):
        zipped,lock=self.capsule();original=Path.write_bytes
        def write(file,value):
            if file.name=='receipt.json':raise OSError('owned simulated receipt storage failure')
            return original(file,value)
        with patch.object(Path,'write_bytes',write):result=archive.import_archive(self.dest,zipped,lock)
        self.assertEqual(result['status'],'restored-verified-local-child-evidence');self.assertFalse(result['receipt_persisted']);self.assertEqual(next((self.dest/archive.PREFIX/'blobs').iterdir()).read_bytes(),b'owned fixture bytes')

    def test_failure_receipt_error_does_not_mask_primary_error(self):
        zipped,lock=self.capsule();original=Path.write_bytes
        def write(file,value):
            if file.name=='failure.json':raise OSError('secondary diagnostic failure')
            return original(file,value)
        with patch.object(archive.os,'rename',side_effect=PermissionError('primary publication failure')),patch.object(Path,'write_bytes',write):
            with self.assertRaisesRegex(PermissionError,'primary publication failure'):archive.import_archive(self.dest,zipped,lock)
        self.assertFalse((self.dest/archive.PREFIX).exists());self.assertEqual(len(list((self.dest/'target').glob('.parity-evidence-import-*/payload'))),1)

    def test_ensure_explicit_archive_and_present_outcome(self):
        zipped,lock=self.capsule();script=Path(archive.__file__).resolve();args=[os.sys.executable,str(script),'ensure','--root',str(self.dest),'--archive',str(zipped),'--lock',str(lock)]
        first=subprocess.run(args,capture_output=True,check=True);self.assertEqual(json.loads(first.stdout)['status'],'restored-verified-local-child-evidence')
        second=subprocess.run(args,capture_output=True,check=True);self.assertEqual(json.loads(second.stdout)['status'],'present-requires-ledger-validation')

    def test_ensure_missing_archive_refuses_without_writes(self):
        _,lock=self.capsule();env={k:v for k,v in os.environ.items() if k!='BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE'}
        q=subprocess.run([os.sys.executable,str(Path(archive.__file__).resolve()),'ensure','--root',str(self.dest),'--lock',str(lock)],env=env,capture_output=True)
        self.assertNotEqual(q.returncode,0);self.assertIn(b'Missing child evidence',q.stderr);self.assertFalse((self.dest/'target').exists())

    def test_export_deterministic_content_addressed_objects(self):
        source=self.root/'source';source.mkdir();data=b'shared source bytes';key=hashlib.sha256(data).hexdigest();rows=[]
        for name in ['one.bin','two.bin']:
            f=source/archive.PREFIX/name;f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(data);rows.append({'destination':archive.PREFIX+'/'+name,'source':'never-read-this-recorded-label','sha256':key,'byte_length':len(data)})
        declaration=self.root/'transfer.json';declaration.write_bytes(archive.canonical({'transfer':rows}))
        for i in (1,2):archive.export_archive(source,declaration,self.root/f'out{i}.zip',self.root/f'out{i}.lock')
        self.assertEqual((self.root/'out1.zip').read_bytes(),(self.root/'out2.zip').read_bytes());self.assertEqual((self.root/'out1.lock').read_bytes(),(self.root/'out2.lock').read_bytes())
        with archive.verified_archive(self.root/'out1.zip',self.root/'out1.lock') as (z,manifest,_):self.assertEqual(len(z.namelist()),2);self.assertEqual(len(manifest['files']),2)

    def test_export_changed_source_refuses(self):
        source=self.root/'source';f=source/archive.PREFIX/'changed.bin';f.parent.mkdir(parents=True);f.write_bytes(b'changed')
        declaration=self.root/'transfer.json';declaration.write_bytes(archive.canonical({'transfer':[{'destination':archive.PREFIX+'/changed.bin','sha256':hashlib.sha256(b'original').hexdigest(),'byte_length':8}]}))
        with self.assertRaises(ValueError):archive.export_archive(source,declaration,self.root/'out.zip',self.root/'out.lock')
        self.assertFalse((self.root/'out.zip').exists());self.assertFalse((self.root/'out.lock').exists())

    def test_archive_tamper(self):
        zipped,lock=self.capsule();zipped.write_bytes(zipped.read_bytes()+b'x');self.refused(zipped,lock)

    def test_manifest_digest_tamper(self):
        zipped,lock=self.capsule();value=json.loads(lock.read_text(encoding='utf-8'));value['manifest_sha256']='0'*64;lock.write_bytes(archive.canonical(value));self.refused(zipped,lock)

    def test_object_digest_tamper(self):
        data=b'owned fixture bytes';key=hashlib.sha256(data).hexdigest();zipped,lock=self.capsule(objects={key:b'wrong fixture bytes'});self.refused(zipped,lock)

    def test_traversal_and_windows_paths(self):
        for name in ['../escape','/absolute','C:/escape',archive.PREFIX+'/../escape',archive.PREFIX+'//file',archive.PREFIX+'/AUX.json',archive.PREFIX+'/stream:secret',archive.PREFIX+'/x\\y']:
            with self.subTest(name=name):
                with self.assertRaises(ValueError):archive.safe_name(name)

    def test_case_aliased_destinations(self):
        key=hashlib.sha256(b'x').hexdigest();rows=[{'path':archive.PREFIX+'/'+name,'sha256':key,'bytes':1} for name in ['file.json','FILE.json']];zipped,lock=self.capsule(rows,{key:b'x'});self.refused(zipped,lock)

    def test_duplicate_zip_entries(self):
        zipped,lock=self.capsule(extra=[('manifest.json',b'{}')]);self.refused(zipped,lock)

    def test_unknown_zip_entry(self):
        zipped,lock=self.capsule(extra=[('run-me.exe',b'unused')]);self.refused(zipped,lock)

    def test_missing_object(self):
        zipped,lock=self.capsule(objects={});self.refused(zipped,lock)

    def test_encrypted_entry_refused_without_password_prompt(self):
        zipped,lock=self.capsule();data=bytearray(zipped.read_bytes());central=struct.unpack_from('<L',data,len(data)-22+16)[0]
        struct.pack_into('<H',data,central+8,struct.unpack_from('<H',data,central+8)[0]|1);struct.pack_into('<H',data,6,struct.unpack_from('<H',data,6)[0]|1);zipped.write_bytes(data)
        value=json.loads(lock.read_text(encoding='utf-8'));value['archive_sha256']=archive.digest(zipped);lock.write_bytes(archive.canonical(value));self.refused(zipped,lock)

    def test_symlink_zip_entry(self):
        key=hashlib.sha256(b'owned fixture bytes').hexdigest();entry=zipfile.ZipInfo('objects/'+key);entry.create_system=3;entry.external_attr=(stat.S_IFLNK|0o777)<<16
        zipped,lock=self.capsule(objects={},extra=[(entry,b'owned fixture bytes')]);self.refused(zipped,lock)

    def test_duplicate_json_keys(self):
        zipped,lock=self.capsule(raw=b'{"schema":"x","schema":"bisect-parity-child-evidence-v1","files":[],"total_bytes":0}');self.refused(zipped,lock)

    def test_size_budget_before_writes(self):
        key=hashlib.sha256(b'x').hexdigest();rows=[{'path':archive.PREFIX+'/big.bin','sha256':key,'bytes':archive.MAX_FILE+1}];zipped,lock=self.capsule(rows,{key:b'x'});self.refused(zipped,lock)

    def test_false_central_count_before_zip_allocation(self):
        zipped,lock=self.capsule();raw=zipped.read_bytes();header=bytearray(raw);struct.pack_into('<H',header,len(header)-22+8,65535);zipped.write_bytes(header)
        value=json.loads(lock.read_text(encoding='utf-8'));value['archive_sha256']=archive.digest(zipped);lock.write_bytes(archive.canonical(value));self.refused(zipped,lock)

    def test_directory_symlink_refused(self):
        zipped,lock=self.capsule();outside=self.root/'outside';outside.mkdir()
        try:(self.dest/'target').symlink_to(outside,target_is_directory=True)
        except OSError:
            if os.name!='nt':raise
            command="$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:BISECT_TEST_LINK_PATH -Target $env:BISECT_TEST_LINK_TARGET | Out-Null"
            subprocess.run(['pwsh','-NoProfile','-Command',command],env={**os.environ,'BISECT_TEST_LINK_PATH':str(self.dest/'target'),'BISECT_TEST_LINK_TARGET':str(outside)},check=True,capture_output=True)
            self.assertTrue((self.dest/'target').is_junction())
        with self.assertRaises(ValueError):archive.import_archive(self.dest,zipped,lock)
        self.assertEqual(list(outside.iterdir()),[])

    def shard(self,stem,name,data):
        key=hashlib.sha256(data).hexdigest()
        return self.capsule([{'path':archive.PREFIX+'/'+name,'sha256':key,'bytes':len(data)}],{key:data},stem=stem)

    def set_lock(self,*parts):
        value=archive.archive_set([{'name':zipped.name,'lock':json.loads(lock.read_bytes())} for zipped,lock in parts])
        lock=self.root/'set.lock';lock.write_bytes(archive.canonical(value));return lock

    def test_set_cli_roundtrip_uses_one_publication(self):
        parts=[self.shard('base','old/base.bin',b'preserved'),self.shard('national','new/map.bin',b'national')]
        lock=self.root/'set.lock'
        script=str(Path(archive.__file__).resolve());cmd=[os.sys.executable,script]
        subprocess.run([*cmd,'compose-set','--archive',str(self.root),'--lock',str(lock),
                        *[item for z,l in parts for item in ('--part',f'{z.name}={l}')]],check=True,capture_output=True)
        env={**os.environ,'BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE':str(self.root)}
        def check(action,extra):
            return json.loads(subprocess.run([*cmd,action,'--root',str(self.dest),'--lock',str(lock),*extra],
                                             check=True,capture_output=True,env=env).stdout)
        verified=check('verify',['--archive',str(self.root)]);self.assertFalse((self.dest/'target').exists())
        result=check('ensure',[])
        self.assertEqual(result['set_sha256'],archive.digest(lock));self.assertEqual(result['set_sha256'],verified['set_sha256'])
        self.assertEqual((result['shards'],result['files'],result['bytes'],result['executed_files']),(2,2,17,0))
        self.assertEqual((self.dest/archive.PREFIX/'old/base.bin').read_bytes(),b'preserved')
        self.assertEqual((self.dest/archive.PREFIX/'new/map.bin').read_bytes(),b'national')
        self.assertEqual(len(list((self.dest/'target').glob('.parity-evidence-import-*'))),1)

    def test_set_refuses_tamper_in_later_shard_before_any_writes(self):
        a=self.shard('a','first',b'1');b=self.shard('b','second',b'2');lock=self.set_lock(a,b)
        data=bytearray(b[0].read_bytes());data[4]^=1;b[0].write_bytes(data)
        self.refused(self.root,lock)

    def test_set_refuses_cross_shard_file_and_directory_aliases(self):
        for i,(first,second) in enumerate([('same','SAME'),('dir/a','DIR/b'),('parent','parent/child')]):
            with self.subTest(first=first,second=second):
                a=self.shard(f'a{i}',first,b'1');b=self.shard(f'b{i}',second,b'2')
                self.refused(self.root,self.set_lock(a,b))

    def test_set_rejects_malformed_lock_before_opening_shards(self):
        a=self.shard('a','first',b'1');b=self.shard('b','second',b'2')
        lock=self.set_lock(a,b);good=json.loads(lock.read_bytes())
        from copy import deepcopy
        for change in [lambda v:v.update(files=1),lambda v:v['shards'][0].update(name='../a.zip'),
                       lambda v:v['shards'][0].update(name='con.zip'),lambda v:v['shards'].reverse(),
                       lambda v:v['shards'][1].update(lock=v['shards'][0]['lock'])]:
            bad=deepcopy(good);change(bad);lock.write_bytes(archive.canonical(bad))
            self.refused(self.root,lock)

    def test_set_enforces_aggregate_file_budget(self):
        a=self.shard('a','first',b'1');b=self.shard('b','second',b'2');lock=self.set_lock(a,b)
        with patch.object(archive,'MAX_SET_FILES',1):self.refused(self.root,lock)

if __name__=='__main__':unittest.main()

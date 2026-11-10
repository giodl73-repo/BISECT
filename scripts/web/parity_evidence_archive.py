"""Local, bounded transport of declared child-proof bytes. Never executes them."""
from pathlib import Path,PurePosixPath
from contextlib import ExitStack,contextmanager
import argparse,hashlib,json,os,re,stat,struct,uuid,zipfile

PREFIX='target/parity-child-acceptance-20261103'
MAX_ARCHIVE=512*1024*1024
MAX_TOTAL=256*1024*1024
MAX_FILE=128*1024*1024
MAX_FILES=4096
MAX_SHARDS=8
MAX_SET_FILES=16384
MAX_SET_TOTAL=2*1024*1024*1024
SINGLE_LOCK='bisect-parity-child-evidence-lock-v1'
SET_LOCK='bisect-parity-child-evidence-set-v1'
SHA=re.compile(r'[a-f0-9]{64}')

def require(ok,message):
    if not ok:raise ValueError(message)

def digest(file):
    with file.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()

def canonical(value):return (json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True)+'\n').encode()

def unique_pairs(pairs):
    value={}
    for k,v in pairs:
        require(k not in value,'Duplicate JSON key');value[k]=v
    return value

def parse(raw):return json.loads(raw.decode('utf-8'),object_pairs_hook=unique_pairs)

def safe_name(name):
    require(isinstance(name,str) and len(name)<=512,'Invalid proof path')
    p=PurePosixPath(name)
    require(str(p)==name and name.startswith(PREFIX+'/') and '\\' not in name and not p.is_absolute(),'Proof path escaped its namespace')
    for part in p.parts:
        require(part not in ('.','..') and re.fullmatch(r'[A-Za-z0-9._-]{1,200}',part) and not part.endswith(('.', ' ')),'Unsafe proof path component')
        require(not re.fullmatch(r'(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])',part.split('.')[0],re.I),'Reserved proof path')
    return p

def destination(root,name):
    p=safe_name(name);root=Path(root).resolve();current=root
    for part in p.parts:
        current=current/part
        require(not current.is_symlink() and not current.is_junction(),'Linked proof destination')
    require(current.resolve().is_relative_to(root),'Proof path escaped repository')
    return current

def validate_tree(rows):
    paths=(safe_name(row['path']).parts for row in rows)
    ordered=sorted((tuple(part.casefold() for part in path),path) for path in paths)
    # Equal prefixes form contiguous runs in component order. Comparing neighbors
    # detects aliases without retaining a separate map of all possible prefixes.
    for (previous_key,previous),(key,path) in zip(ordered,ordered[1:]):
        require(previous_key!=key,'Duplicate or case-aliased proof path')
        require(previous_key!=key[:len(previous_key)],'Proof file also used as a directory')
        for a,b in zip(previous,path):
            if a.casefold()!=b.casefold():break
            require(a==b,'Case-aliased proof directory')

def validate_manifest(value):
    require(isinstance(value,dict) and set(value)=={'schema','files','total_bytes'} and value['schema']=='bisect-parity-child-evidence-v1','Invalid evidence manifest')
    rows=value['files'];require(isinstance(rows,list) and 0<len(rows)<=MAX_FILES,'Evidence file-count limit')
    objects={};total=0
    for row in rows:
        require(isinstance(row,dict) and set(row)=={'path','sha256','bytes'},'Invalid evidence file record')
        require(isinstance(row['sha256'],str) and SHA.fullmatch(row['sha256']),'Invalid evidence digest')
        require(type(row['bytes']) is int and 0<=row['bytes']<=MAX_FILE,'Evidence file-size limit')
        previous=objects.setdefault(row['sha256'],row['bytes']);require(previous==row['bytes'],'Conflicting object sizes');total+=row['bytes']
    require(type(value['total_bytes']) is int and total==value['total_bytes'] and total<=MAX_TOTAL,'Evidence decoded-size limit')
    validate_tree(rows)
    return objects

def preflight_zip(stream,size):
    # Bound central-directory allocation before ZipFile constructs ZipInfo objects.
    require(size>=22,'Truncated ZIP');stream.seek(size-22);footer=stream.read(22)
    signature,disk,cd_disk,on_disk,count,central_size,central_offset,comment=struct.unpack('<4s4H2LH',footer)
    require(signature==b'PK\x05\x06' and disk==cd_disk==comment==0 and on_disk==count and 0<count<=MAX_FILES+1,'Unsupported ZIP footer')
    require(central_size<=4*1024*1024 and central_offset+central_size==size-22,'ZIP central-directory bound')
    stream.seek(central_offset);central=stream.read(central_size);offset=0;actual=0
    while offset<len(central):
        require(central[offset:offset+4]==b'PK\x01\x02' and offset+46<=len(central),'Invalid ZIP central directory')
        names,extras,comments=struct.unpack_from('<3H',central,offset+28);offset+=46+names+extras+comments;actual+=1
        require(offset<=len(central) and actual<=MAX_FILES+1,'Excessive ZIP central records')
    require(actual==count,'ZIP record-count mismatch');stream.seek(0)

def export_archive(root,transfer,archive,lock_file):
    root=Path(root).resolve();archive=Path(archive);lock_file=Path(lock_file)
    require(not archive.exists() and not lock_file.exists(),'Preserve existing archive or lock')
    require(not archive.resolve().is_relative_to((root/PREFIX).resolve()),'Archive output cannot be an input')
    require(Path(transfer).stat().st_size<=4*1024*1024,'Oversized transfer declaration')
    source=parse(Path(transfer).read_bytes());require(isinstance(source,dict) and isinstance(source.get('transfer'),list) and 0<len(source['transfer'])<=MAX_FILES,'Invalid transfer declaration');rows=[];by_hash={};total=0
    for item in source['transfer']:
        require(type(item.get('byte_length')) is int and 0<=item['byte_length']<=MAX_FILE,'Export file-size limit');total+=item['byte_length'];require(total<=MAX_TOTAL,'Export decoded-size limit')
        f=destination(root,item['destination']);require(f.is_file() and f.stat().st_nlink==1,'Missing or linked proof source')
        require(f.stat().st_size==item['byte_length'] and digest(f)==item['sha256'],'Changed proof source')
        rows.append({'path':item['destination'],'sha256':item['sha256'],'bytes':item['byte_length']});by_hash.setdefault(item['sha256'],f)
    manifest={'schema':'bisect-parity-child-evidence-v1','files':sorted(rows,key=lambda r:r['path']),'total_bytes':sum(r['bytes'] for r in rows)};validate_manifest(manifest);raw=canonical(manifest)
    def info(name):
        v=zipfile.ZipInfo(name,(1980,1,1,0,0,0));v.compress_type=zipfile.ZIP_DEFLATED;v.create_system=3;v.external_attr=(stat.S_IFREG|0o644)<<16;return v
    with archive.open('xb') as output,zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9,allowZip64=False) as z:
        z.writestr(info('manifest.json'),raw)
        for key in sorted(by_hash):
            h=hashlib.sha256()
            with by_hash[key].open('rb') as src,z.open(info('objects/'+key),'w') as dst:
                while chunk:=src.read(1024*1024):h.update(chunk);dst.write(chunk)
            require(h.hexdigest()==key,'Proof changed during export')
    require(archive.stat().st_size<=MAX_ARCHIVE,'Evidence encoded-size limit')
    lock={'schema':'bisect-parity-child-evidence-lock-v1','archive_sha256':digest(archive),'archive_bytes':archive.stat().st_size,'manifest_sha256':hashlib.sha256(raw).hexdigest(),'files':len(rows),'total_bytes':manifest['total_bytes']}
    with lock_file.open('xb') as stream:stream.write(canonical(lock))
    return lock

def read_lock(lock_file):
    with Path(lock_file).open('rb') as f:raw=f.read(64*1024+1)
    require(len(raw)<=64*1024,'Oversized trusted lock')
    lock=parse(raw);require(isinstance(lock,dict),'Invalid trusted lock')
    return raw,lock

def validate_lock(lock):
    require(isinstance(lock,dict) and set(lock)=={'schema','archive_sha256','archive_bytes','manifest_sha256','files','total_bytes'} and lock['schema']==SINGLE_LOCK,'Invalid trusted lock')
    require(all(isinstance(lock[k],str) and SHA.fullmatch(lock[k]) for k in ('archive_sha256','manifest_sha256')),'Invalid lock digest')
    require(type(lock['archive_bytes']) is int and 0<lock['archive_bytes']<=MAX_ARCHIVE,'Evidence encoded-size mismatch')
    require(type(lock['files']) is int and 0<lock['files']<=MAX_FILES,'Evidence file-count limit')
    require(type(lock['total_bytes']) is int and 0<=lock['total_bytes']<=MAX_TOTAL,'Evidence decoded-size limit')

@contextmanager
def verified_locked_archive(archive,lock):
    archive=Path(archive);validate_lock(lock)
    require(archive.is_file() and archive.stat().st_size==lock['archive_bytes'],'Evidence encoded-size mismatch')
    with archive.open('rb') as stream:
        require(os.fstat(stream.fileno()).st_size==lock['archive_bytes'],'Archive changed before open')
        require(hashlib.file_digest(stream,'sha256').hexdigest()==lock['archive_sha256'],'Evidence archive hash mismatch');preflight_zip(stream,lock['archive_bytes'])
        with zipfile.ZipFile(stream) as z:
            entries=z.infolist();names=[e.filename for e in entries];require(len(names)==len(set(names)) and len(names)<=MAX_FILES+1,'Duplicate or excessive ZIP entries')
            for e in entries:
                require(e.filename=='manifest.json' or re.fullmatch(r'objects/[a-f0-9]{64}',e.filename),'Unexpected ZIP entry')
                require(not e.is_dir() and not (e.flag_bits&1) and e.compress_type in (zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED),'Unsupported ZIP entry')
                mode=(e.external_attr>>16)&0xffff;require(stat.S_IFMT(mode) in (0,stat.S_IFREG),'Linked or special ZIP entry')
            require('manifest.json' in names and z.getinfo('manifest.json').file_size<=4*1024*1024,'Missing or oversized manifest')
            raw=z.read('manifest.json');require(hashlib.sha256(raw).hexdigest()==lock['manifest_sha256'],'Manifest hash mismatch');manifest=parse(raw);objects=validate_manifest(manifest)
            require(type(lock['files']) is int and lock['files']==len(manifest['files']) and type(lock['total_bytes']) is int and lock['total_bytes']==manifest['total_bytes'],'Lock inventory mismatch')
            require(set(names)=={'manifest.json',*('objects/'+key for key in objects)},'Missing or undeclared evidence object')
            for key,size in objects.items():
                require(z.getinfo('objects/'+key).file_size==size,'ZIP object size mismatch');h=hashlib.sha256();count=0
                with z.open('objects/'+key) as src:
                    while chunk:=src.read(1024*1024):count+=len(chunk);require(count<=size,'Decoded object overflow');h.update(chunk)
                require(count==size and h.hexdigest()==key,'Evidence object digest mismatch')
            yield z,manifest,lock

@contextmanager
def verified_archive(archive,lock_file):
    _,lock=read_lock(lock_file)
    with verified_locked_archive(archive,lock) as value:yield value

def archive_set(shards):
    require(isinstance(shards,list) and 2<=len(shards)<=MAX_SHARDS,'Evidence shard-count limit')
    files=total=0;names=set();identities=set()
    for row in shards:
        require(isinstance(row,dict) and set(row)=={'name','lock'},'Invalid evidence shard')
        name=row['name'];lock=row['lock'];validate_lock(lock)
        require(isinstance(name,str) and re.fullmatch(r'[a-z0-9][a-z0-9_-]{0,63}\.zip',name),'Invalid evidence shard name')
        safe_name(PREFIX+'/'+name)
        require(name not in names and lock['archive_sha256'] not in identities,'Duplicate evidence shard')
        names.add(name);identities.add(lock['archive_sha256']);files+=lock['files'];total+=lock['total_bytes']
    require(files<=MAX_SET_FILES and total<=MAX_SET_TOTAL,'Evidence archive set budget')
    return {'schema':SET_LOCK,'shards':sorted(shards,key=lambda row:row['name']),'files':files,'total_bytes':total}

@contextmanager
def verified_set(directory,lock):
    require(set(lock)=={'schema','shards','files','total_bytes'} and lock['schema']==SET_LOCK and
            type(lock['files']) is int and type(lock['total_bytes']) is int and
            archive_set(lock['shards'])==lock,'Invalid trusted evidence set')
    directory=Path(directory).resolve();require(directory.is_dir(),'Pass the local directory of archive shards')
    with ExitStack() as stack:
        parts=[]
        for row in lock['shards']:
            p=directory/row['name'];require(not p.is_symlink() and not p.is_junction(),'Linked evidence shard')
            z,manifest,_=stack.enter_context(verified_locked_archive(p,row['lock']));parts.append((z,manifest))
        validate_tree(r for _,manifest in parts for r in manifest['files'])
        yield parts

@contextmanager
def verified_capsule(archive,lock_file):
    raw,lock=read_lock(lock_file)
    if lock.get('schema')==SET_LOCK:
        with verified_set(archive,lock) as parts:
            yield parts,{'files':lock['files'],'bytes':lock['total_bytes'],'shards':len(parts),'set_sha256':hashlib.sha256(raw).hexdigest()}
    else:
        with verified_locked_archive(archive,lock) as (z,manifest,lock):
            yield [(z,manifest)],{'files':lock['files'],'bytes':lock['total_bytes'],'archive_sha256':lock['archive_sha256']}

def compose_set(directory,parts,output):
    require(2<=len(parts)<=MAX_SHARDS,'Evidence shard-count limit')
    output=Path(output);require(not output.exists(),'Preserve existing trusted lock')
    shards=[]
    for part in parts:
        name,separator,source=part.partition('=')
        require(separator and source,'Use --part name.zip=path-to-single-lock')
        _,lock=read_lock(source);shards.append({'name':name,'lock':lock})
    lock=archive_set(shards)
    with verified_set(directory,lock):
        with output.open('xb') as f:f.write(canonical(lock))
    return lock

def publish_verified_directory(root,payload,final):
    # A Windows file scanner can briefly hold freshly written contents open.
    # Retry only these OS failures, with the same verified stage and no destination.
    from time import sleep
    signature=payload.stat(follow_symlinks=False)
    for delay in (0.05,0.15,0.35,0.75,None):
        observed=payload.stat(follow_symlinks=False)
        require((observed.st_dev,observed.st_ino)==(signature.st_dev,signature.st_ino) and
                payload.resolve()==payload and not payload.is_symlink() and not payload.is_junction(),
                'Linked or changed staging payload')
        require(destination(root,PREFIX+'/placeholder').parent==final and not final.exists(),
                'Evidence destination appeared during restore')
        try:
            os.rename(payload,final)
            return
        except PermissionError as error:
            if delay is None or getattr(error,'winerror',None) not in (5,32,33):raise
            sleep(delay)

def import_archive(root,archive,lock_file):
    root=Path(root).resolve();require(root.is_dir(),'Repository root must already exist')
    final=destination(root,PREFIX+'/placeholder').parent;require(not final.exists(),'Existing evidence is preserved; import into a fresh worktree')
    with verified_capsule(archive,lock_file) as (parts,identity):
        parent=root/'target';require(not parent.is_symlink() and not parent.is_junction(),'Linked target directory');parent.mkdir(exist_ok=True)
        stage=parent/('.parity-evidence-import-'+uuid.uuid4().hex);stage.mkdir();payload=stage/'payload';payload.mkdir()
        try:
            for z,manifest in parts:
                for row in manifest['files']:
                    relative=PurePosixPath(row['path']).relative_to(PREFIX);f=payload/str(relative);f.parent.mkdir(parents=True,exist_ok=True);h=hashlib.sha256();count=0
                    with z.open('objects/'+row['sha256']) as src,f.open('xb') as dst:
                        while chunk:=src.read(1024*1024):count+=len(chunk);require(count<=row['bytes'],'Restore overflow');h.update(chunk);dst.write(chunk)
                    require(count==row['bytes'] and h.hexdigest()==row['sha256'] and f.stat().st_nlink==1,'Restored proof mismatch')
            publish_verified_directory(root,payload,final)
            result={'status':'restored-verified-local-child-evidence',**identity,'executed_files':0,'scope':'Byte restoration only; original source-owned tests, ledger checks and complete parity acceptance remain required.'}
            result['receipt_persisted']=True
            try:(stage/'receipt.json').write_bytes(canonical(result))
            except OSError as error:result['receipt_persisted']=False;result['receipt_error']=str(error)
            return result
        except Exception as error:
            try:(stage/'failure.json').write_bytes(canonical({'status':'failed-restore','error':str(error),'partial_files_retained':True,'executed_files':0}))
            except OSError:pass
            raise

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=('export','verify','import','ensure','compose-set'));p.add_argument('--root',type=Path,default=Path.cwd());p.add_argument('--archive',type=Path);p.add_argument('--lock',type=Path,required=True);p.add_argument('--transfer',type=Path);p.add_argument('--part',action='append',default=[]);a=p.parse_args()
    if a.action=='ensure':
        final=destination(a.root,PREFIX+'/placeholder').parent
        if final.is_dir():print(json.dumps({'status':'present-requires-ledger-validation','executed_files':0}));return
        a.archive=a.archive or (Path(os.environ['BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE']) if os.environ.get('BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE') else None)
        require(a.archive is not None,'Missing child evidence. Set BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE to the local capsule or shard directory matching the tracked lock, or import it explicitly. Source-owned fixture regeneration and the ledger remain required.');a.action='import'
    require(a.archive is not None,'Pass --archive')
    if a.action=='compose-set':result=compose_set(a.archive,a.part,a.lock)
    elif a.action=='export':require(a.transfer is not None,'Pass --transfer');result=export_archive(a.root,a.transfer,a.archive,a.lock)
    elif a.action=='import':result=import_archive(a.root,a.archive,a.lock)
    else:
        with verified_capsule(a.archive,a.lock) as (_,identity):result={'status':'verified-local-child-evidence-capsule',**identity,'executed_files':0}
    print(json.dumps(result))

if __name__=='__main__':main()

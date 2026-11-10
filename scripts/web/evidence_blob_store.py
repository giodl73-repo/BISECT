"""Read immutable evidence blobs, including versioned inherited snapshots.

This is developer artifact storage, never imported-project or filesystem-write
authority. Blob names are derived solely from verified SHA-256 identities.
"""
import hashlib,json,re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def local(name):
    value=Path(name);value=(value if value.is_absolute() else ROOT/value).resolve()
    if not value.is_relative_to(ROOT):raise ValueError('Evidence path is outside this workspace')
    return value
def digest(path):
    value=hashlib.sha256()
    with Path(path).open('rb') as stream:
        while chunk:=stream.read(1024*1024):value.update(chunk)
    return value.hexdigest()

class SnapshotBlobs:
    def __init__(self,path,expected=None,*,ancestors=()):
        self.path=local(path)
        if self.path in ancestors or len(ancestors)>=8:raise ValueError('Cyclic or excessive snapshot inheritance')
        if expected is not None and digest(self.path)!=expected:raise ValueError('Snapshot manifest digest differs')
        self.identity=digest(self.path);self.data=json.loads(self.path.read_text(encoding='utf-8-sig'))
        self.rows={};self.verified={};self.base=None
        rows=self.data.get('files')
        if not isinstance(rows,list) or len(rows)>50000:raise ValueError('Invalid evidence file inventory')
        paths=set()
        for row in rows:
            identity=row.get('sha256');name=row.get('path');size=row.get('byte_length')
            if not isinstance(identity,str) or not re.fullmatch('[0-9a-f]{64}',identity):raise ValueError('Invalid blob identity')
            if not isinstance(name,str) or not name or '\\' in name or ':' in name or '\0' in name or Path(name).is_absolute() or any(part in ['', '.', '..'] for part in name.split('/')) or name in paths:raise ValueError('Invalid/duplicate evidence path')
            if not isinstance(size,int) or isinstance(size,bool) or size<0:raise ValueError('Invalid blob size')
            paths.add(name)
            if identity in self.rows and self.rows[identity]['byte_length']!=size:raise ValueError('Conflicting blob sizes')
            self.rows[identity]=row
        mode=self.data.get('storage_format')
        if mode=='verified-inherited-blobs-v1':
            reference=self.data.get('base_snapshot')
            if not isinstance(reference,dict):raise ValueError('Missing inherited snapshot identity')
            self.base=SnapshotBlobs(reference['path'],reference['sha256'],ancestors=(*ancestors,self.path))
        elif mode!='content-addressed-blobs-v1':raise ValueError('Unsupported snapshot blob format')

    def blob(self,identity,size=None):
        if not isinstance(identity,str) or not re.fullmatch('[0-9a-f]{64}',identity) or identity not in self.rows:raise ValueError('Unknown blob identity')
        row=self.rows[identity]
        if size is not None and row['byte_length']!=size:raise ValueError('Inherited blob size differs')
        if self.data['storage_format']=='verified-inherited-blobs-v1':
            if row.get('blob')!=identity or row.get('storage') not in ['base','local']:raise ValueError('Invalid inherited blob locator')
            if row['storage']=='base':return self.base.blob(identity,row['byte_length'])
        elif row.get('blob')!='blobs/'+identity:raise ValueError('Invalid blob locator')
        path=local(self.path.parent/'blobs'/identity)
        if not path.is_relative_to(self.path.parent):raise ValueError('Blob escaped snapshot root')
        if identity not in self.verified:
            if path.stat().st_size!=row['byte_length'] or digest(path)!=identity:raise ValueError('Evidence blob bytes differ')
            self.verified[identity]=path
        return self.verified[identity]

    def verify(self):
        if self.base:self.base.verify()
        self.verified.clear()
        for identity in self.rows:self.blob(identity)
        return dict(files=len(self.data['files']),unique_blobs=len(self.rows),logical_bytes=sum(row['byte_length'] for row in self.data['files']))

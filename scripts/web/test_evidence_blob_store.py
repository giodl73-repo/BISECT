import json,tempfile,unittest
from pathlib import Path
from evidence_blob_store import ROOT,SnapshotBlobs,digest

class EvidenceBlobsTest(unittest.TestCase):
    def setUp(self):
        parent=ROOT/'target/evidence-blob-tests';parent.mkdir(parents=True,exist_ok=True)
        self.temp=tempfile.TemporaryDirectory(dir=parent);self.root=Path(self.temp.name);self.base=self.root/'base';self.base.mkdir();(self.base/'blobs').mkdir()
        source=self.root/'bytes';source.write_bytes(b'Exact native artifact bytes\x00\xff');self.sha=digest(source);self.size=source.stat().st_size;(self.base/'blobs'/self.sha).write_bytes(source.read_bytes())
        self.base_manifest=self.base/'snapshot.json';self.base_manifest.write_text(json.dumps(dict(storage_format='content-addressed-blobs-v1',files=[dict(path='original/native.exe',sha256=self.sha,byte_length=self.size,blob='blobs/'+self.sha)])))
        self.child=self.root/'child';self.child.mkdir();(self.child/'blobs').mkdir();self.manifest=self.child/'snapshot.json'
        self.child_data=dict(storage_format='verified-inherited-blobs-v1',base_snapshot=dict(path=self.base_manifest.relative_to(ROOT).as_posix(),sha256=digest(self.base_manifest)),files=[dict(path='new/native.exe',sha256=self.sha,byte_length=self.size,blob=self.sha,storage='base')]);self.write()
    def tearDown(self):
        assert self.root.resolve().is_relative_to((ROOT/'target/evidence-blob-tests').resolve())
        self.temp.cleanup()
    def write(self):self.manifest.write_text(json.dumps(self.child_data))
    def test_reuse_exact_bytes_without_copy(self):
        store=SnapshotBlobs(self.manifest);self.assertEqual(store.blob(self.sha),self.base/'blobs'/self.sha);self.assertEqual(store.verify()['logical_bytes'],self.size);self.assertEqual(list((self.child/'blobs').iterdir()),[])
    def test_mutated_inherited_bytes_refused(self):
        (self.base/'blobs'/self.sha).write_bytes(b'changed');self.assertRaises(ValueError,SnapshotBlobs(self.manifest).blob,self.sha)
    def test_verification_rechecks_previously_read_bytes(self):
        store=SnapshotBlobs(self.manifest);store.blob(self.sha);(self.base/'blobs'/self.sha).write_bytes(b'changed');self.assertRaises(ValueError,store.verify)
    def test_manifest_identity_refused(self):
        self.base_manifest.write_text(self.base_manifest.read_text()+' ');self.assertRaises(ValueError,SnapshotBlobs,self.manifest)
    def test_false_size_refused(self):
        self.child_data['files'][0]['byte_length']+=1;self.write();self.assertRaises(ValueError,SnapshotBlobs(self.manifest).blob,self.sha)
    def test_traversal_locator_refused(self):
        self.child_data['files'][0]['storage']='local';self.child_data['files'][0]['blob']='../outside';self.write();self.assertRaises(ValueError,SnapshotBlobs(self.manifest).blob,self.sha)
    def test_cycles_and_unbounded_depth_refused(self):
        self.assertRaises(ValueError,SnapshotBlobs,self.manifest,ancestors=(self.manifest,))
        self.assertRaises(ValueError,SnapshotBlobs,self.manifest,ancestors=tuple(self.root/str(n) for n in range(8)))
    def test_conflicting_sizes_for_same_hash_refused(self):
        self.child_data['files'].append(dict(self.child_data['files'][0],path='another.exe',byte_length=1));self.write();self.assertRaises(ValueError,SnapshotBlobs,self.manifest)

if __name__=='__main__':unittest.main()

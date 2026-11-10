"""Independent verification of accepted-workflow archives and retained native text."""
from pathlib import Path
import json,zipfile,hashlib
roots=[Path('target/accepted-state-workflow'),Path('target/accepted-state-workflow-browser')]
for root in roots:
    project=json.loads((root/'complete.bisect').read_text(encoding='utf-8'))
    workflow=project['result']['workflow']
    with zipfile.ZipFile(root/'complete.zip') as archive:
        assert archive.testzip() is None
        names=archive.namelist()
        assert len(names)==len(set(names))
        assert all(not name.startswith(('/', '\\')) and '..' not in name.split('/') and '\\' not in name for name in names)
        assert 'effective-config.yml' not in names
        assert len(names)==len(workflow['files_base64'])+1
        for name,expected in workflow['file_sha256'].items():
            assert hashlib.sha256(archive.read(name)).hexdigest()==expected,(root,name)
        receipt=json.loads(archive.read('workflow-receipt.json'))
        assert receipt['file_sha256']==workflow['file_sha256'] and 'files_base64' not in receipt
        for name,field in [('plan.rplan','accepted_plan_source_sha256'),('context.rctx','accepted_context_source_sha256')]:
            assert hashlib.sha256(archive.read(name)).hexdigest()==workflow[field]
        assert b'18446744073709551615' in archive.read('plan.rplan')
        assert 'report/report.html' in names and 'maps/political.png' in names
print('PASS accepted State workflow ZIPs: two exact native file sets, source/receipt hashes, maximum-u64 seed, paths, uniqueness and CRCs')

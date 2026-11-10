import json
from pathlib import Path
from zipfile import ZipFile
result=json.loads(Path('target/civic-inventory-wasm/browser-project.bisect').read_bytes())['result']
with ZipFile('target/civic-inventory-wasm/inventory.zip') as z:
    assert z.testzip() is None
    assert set(z.namelist())=={'list.txt','inventory.json','manifest-1.json','manifest-2.json'}
    assert z.read('list.txt').decode()==result['native_list_text']
    assert json.loads(z.read('inventory.json'))==result
    for i in (1,2):assert z.read(f'manifest-{i}.json').decode()==result['entries'][i-1]['native_show_json']
    assert all(not ((e.external_attr>>16)&0o111) for e in z.infolist())
print('Civic inventory ZIP: exact list/show/report bytes, entry paths, CRC and permissions passed.')

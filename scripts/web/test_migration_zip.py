import json,pathlib,zipfile,stat
root=pathlib.Path('target/migration-wasm')
cases=json.loads((root/'evidence.json').read_text())['cases']
for n in range(1,cases+1):
    native=root/f'native-{n}'/'plans'/'legacy_plan'
    with zipfile.ZipFile(root/f'package-{n}.zip') as z:
        assert z.testzip() is None
        names=z.namelist(); assert len(names)==len(set(names))
        for i in z.infolist():
            p=pathlib.PurePosixPath(i.filename)
            assert not p.is_absolute() and '..' not in p.parts
            assert p.parts[:2]==('plans','legacy_plan')
            relative=pathlib.Path(*p.parts[2:])
            if i.is_dir():
                assert (native/relative).is_dir() and stat.S_ISDIR(i.external_attr>>16)
            elif relative.as_posix()=='manifest.json':
                browser=json.loads(z.read(i)); actual=json.loads((native/relative).read_text())
                actual['created_at']=browser['created_at']; assert actual==browser
            else:
                assert z.read(i)==(native/relative).read_bytes()
                assert not (i.external_attr>>16)&0o111
        for directory in ['analysis','maps','intermediate']:
            assert any(p.startswith(f'plans/legacy_plan/{directory}/') for p in names)
        actual_files={p.relative_to(native).as_posix() for p in native.rglob('*') if p.is_file()}
        zip_files={pathlib.PurePosixPath(i.filename).relative_to('plans/legacy_plan').as_posix() for i in z.infolist() if not i.is_dir()}
        assert actual_files==zip_files
print(f'Migration ZIP: {cases} native packages, CRC, binary bytes, exact file sets, standard directories and safe paths passed.')

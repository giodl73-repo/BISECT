"""Vendor the pinned VeriPB libraries with auditable browser boundary changes."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess

COMMIT = 'a50f0483d65aa019918d03ccc590b71dc23c666c'
SOURCE = Path('target/proof-toolchain/veripb-current')
DEST = Path('third_party/veripb')
LIBS = ['veripb-checker', 'veripb-formula', 'veripb-rules', 'veripb-parser', 'veripb-propagator']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    if subprocess.check_output(['git', '-C', str(SOURCE), 'rev-parse', 'HEAD'], text=True).strip() != COMMIT:
        raise ValueError('Wrong upstream revision')
    if subprocess.check_output(['git', '-C', str(SOURCE), 'status', '--porcelain'], text=True).strip():
        raise ValueError('Upstream checkout must be clean')
    files = {}
    for library in LIBS:
        for path in [SOURCE / library / 'Cargo.toml', SOURCE / library / 'README.md', *(SOURCE / library / 'src').rglob('*')]:
            if path.is_file():
                name = path.relative_to(SOURCE).as_posix()
                files[name] = subprocess.check_output(['git', '-C', str(SOURCE), 'show', COMMIT + ':' + name])
    for name in ['LICENSE-APACHE', 'LICENSE-MIT', 'README.md']:
        files[name] = subprocess.check_output(['git', '-C', str(SOURCE), 'show', COMMIT + ':' + name])
    upstream = {name: digest(data) for name, data in files.items()}
    # Remove native benchmark/dev dependency metadata from this library-only
    # distribution. Kernel sources are retained verbatim.
    for library in LIBS:
        name = library + '/Cargo.toml'
        text = files[name].decode('utf-8').replace('\r\n', '\n').split('[dev-dependencies]')[0]
        text = text.replace('ahash = "0.8.12"', 'ahash = { version = "0.8.12", default-features = false, features = ["std", "no-rng"] }')
        if library == 'veripb-parser':
            text = text.replace('zstd = "0.13"\n', '').replace('xz2 = "0.1.7"\n', '').replace('flate2 = "1.1.9"\n', '')
        files[name] = text.encode('utf-8')
    files['Cargo.toml'] = ('[workspace]\nresolver = "3"\nmembers = ' + json.dumps(LIBS) + '\n[workspace.package]\nauthors = ["Andy Oertel <andy.oertel@cs.lth.se>"]\nedition = "2024"\nlicense = "MIT OR Apache-2.0"\nrust-version = "1.92.0"\nrepository = "https://gitlab.com/MIAOresearch/software/VeriPB"\n').encode()
    # Preserve the OPB parsing loop and grammar. Expose it to any BufRead,
    # instead of requiring a filename and native filesystem access.
    name = 'veripb-parser/src/opb_parser.rs'
    text = files[name].decode().replace('\r\n', '\n').replace('use std::path::Path;', 'use std::{path::Path, io::BufRead};')
    needle = '    let mut formula = Formula::default();\n    let mut labels = AHashMap::new();\n    let mut preserved_vars = None;\n    let lines = get_lines(&filename)?;'
    replacement = '''    let lines = get_lines(&filename)?;
    parse_opb_lines(filename, lines, var_name_manager)
}

/// Browser adapter: parse already acquired, bounded UTF-8 bytes in memory.
pub fn parse_opb_reader<R: BufRead>(reader: R) -> Result<(Formula, VarNameManager, AHashMap<String, isize>, PreservedSetType), ParserError> {
    let mut variables = VarNameManager::default();
    let (formula, labels, preserved) = parse_opb_lines("input.opb", reader.lines(), &mut variables)?;
    Ok((formula, variables, labels, preserved))
}

fn parse_opb_lines<P: AsRef<Path>, I: Iterator<Item = std::io::Result<String>>>(filename: P, lines: I, var_name_manager: &mut VarNameManager) -> Result<(Formula, AHashMap<String, isize>, PreservedSetType), ParserError> {
    let mut formula = Formula::default();
    let mut labels = AHashMap::new();
    let mut preserved_vars = None;'''
    assert text.count(needle) == 1
    files[name] = text.replace(needle, replacement).encode()
    # Retain existing native file API for compilation, without linked native
    # compression libraries. The browser wrapper never calls this API.
    files['veripb-parser/src/io.rs'] = b'''//! Plain-text file helpers for the browser library distribution.
use std::{fs::File, io::{BufReader, BufRead, Lines}, path::Path};
use crate::error::ParserError;
pub fn get_lines<P: AsRef<Path>>(filename: P) -> Result<Lines<BufReader<File>>, ParserError> {
    Ok(open_maybe_compressed(filename)?.0.lines())
}
pub fn open_maybe_compressed<P: AsRef<Path>>(filename: P) -> Result<(BufReader<File>, Option<u64>), ParserError> {
    let file = File::open(filename)?;
    let size = file.metadata()?.len();
    Ok((BufReader::new(file), Some(size)))
}
'''
    name = 'veripb-parser/src/pbp_parser/parser.rs'
    text = files[name].decode().replace('\r\n', '\n').replace('use crate::io::MaybeCompressed;\n', '')
    start, end = text.index('pub fn parse_proof<'), text.index('\nimpl<', text.index('pub fn parse_proof<'))
    text = text[:start] + '''pub fn parse_proof<P: ProofProcessor, R: BufRead>(file_size: Option<u64>, reader: &mut R, processor: &mut P, formula_labels: AHashMap<String, isize>, args: ParserArgs) -> Result<(), P::Error> {
    Parser::new(file_size, reader, processor, formula_labels, args).parse()
}
''' + text[end:]
    files[name] = text.encode()
    manifest = {'schema': 'bisect-vendored-veripb-v1', 'upstream': 'https://gitlab.com/MIAOresearch/software/VeriPB', 'commit': COMMIT,
                'changes': 'In-memory parser entry points, plain-text I/O only, deterministic nonrandom AHash seeds, library-only manifests. Checker, formula, rules and propagator kernel sources are unchanged.',
                'files': {name: {'upstream_sha256': upstream.get(name), 'vendored_sha256': digest(data)} for name, data in sorted(files.items())}}
    previous = json.loads((DEST / 'provenance.json').read_text(encoding='utf-8')) if (DEST / 'provenance.json').exists() else None
    for name, data in files.items():
        target = DEST / name
        if target.exists() and target.read_bytes() != data:
            old = previous and previous['files'].get(name)
            if not old or digest(target.read_bytes()) != old['vendored_sha256']:
                raise ValueError('Refusing to overwrite locally changed vendor file: ' + name)
    for name, data in files.items():
        target = DEST / name; target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists() or target.read_bytes() != data: target.write_bytes(data)
    (DEST / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(f'Vendored {len(files)} source/metadata files at {COMMIT}')


if __name__ == '__main__': main()

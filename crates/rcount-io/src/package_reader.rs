//! Shared package readers for filesystem and browser-selected file bytes.
use crate::*;

pub type PackageFiles = BTreeMap<String, Vec<u8>>;

pub fn read_package_files(files: &PackageFiles) -> Result<(RcountManifest, RcountPackage), RcountIoError> {
    read_package_with(&mut |path| Ok(files.get(path).cloned()))
}

pub fn read_package_with(read: &mut impl FnMut(&str) -> Result<Option<Vec<u8>>, RcountIoError>) -> Result<(RcountManifest, RcountPackage), RcountIoError> {
    let manifest: RcountManifest = json_record(read, "manifest.json")?;
    if manifest.rcount_version != RCOUNT_VERSION { return Err(RcountIoError::UnsupportedVersion(manifest.rcount_version)); }
    let package = RcountPackage {
        rcount_version: manifest.rcount_version.clone(),
        contests: ndjson_records(read,"normalized/contests.ndjson",false)?,
        reporting_units: ndjson_records(read,"normalized/reporting-units.ndjson",false)?,
        batches: ndjson_records(read,"normalized/batches.ndjson",true)?,
        lineage: ndjson_records(read,"normalized/lineage.ndjson",true)?,
        rhist_refs: ndjson_records(read,"normalized/rhist-refs.ndjson",true)?,
        rctx_refs: ndjson_records(read,"normalized/rctx-refs.ndjson",true)?,
        inclusion_proofs: ndjson_records(read,"proofs/inclusion-proofs.ndjson",true)?,
        cvr: ndjson_records(read,"normalized/cvr.ndjson",true)?,
        audit_algorithm_runs: ndjson_records(read,"audits/algorithm-runs.ndjson",true)?,
        rla_audits: ndjson_records(read,"audits/rla.ndjson",true)?,
        manual_audits: ndjson_records(read,"audits/manual.ndjson",true)?,
        batch_comparison_audits: ndjson_records(read,"audits/batch-comparison.ndjson",true)?,
        summaries: ndjson_records(read,"normalized/summaries.ndjson",false)?,
        status_events: ndjson_records(read,"status/events.ndjson",false)?,
    };
    let computed = package_content_hash(&package)?;
    if manifest.content_hash != computed { return Err(RcountIoError::ContentHashMismatch {declared:manifest.content_hash,computed}); }
    Ok((manifest,package))
}

pub fn verify_source_files(files: &PackageFiles) -> Result<Vec<SourceCheck>, RcountIoError> {
    verify_sources_with(&mut |path| Ok(files.get(path).cloned()))
}

pub fn verify_sources_with(read: &mut impl FnMut(&str) -> Result<Option<Vec<u8>>, RcountIoError>) -> Result<Vec<SourceCheck>, RcountIoError> {
    let index: SourceIndex = json_record(read,"sources/source-index.json")?;
    if index.sources.is_empty() { return Err(RcountIoError::EmptySourceIndex); }
    let mut checks=Vec::new();
    for source in index.sources {
        // The same source-path contract applies to disk and browser packages.
        let canonical=source.path.replace('\\',"/");
        package_relative_source_path(&canonical)?;
        let bytes=read(&canonical)?.ok_or_else(||RcountIoError::MissingSourceFile {path:source.path.clone()})?;
        let computed=source_bytes_hash(&bytes);
        if computed!=source.sha256 { return Err(RcountIoError::SourceHashMismatch {source_id:source.source_id,declared:source.sha256,computed}); }
        checks.push(SourceCheck {source_id:source.source_id,path:source.path,sha256:computed});
    }
    Ok(checks)
}

pub(crate) fn read_disk_optional(dir: &Path, path: &str) -> Result<Option<Vec<u8>>, RcountIoError> {
    match fs::read(dir.join(path)) {
        Ok(bytes)=>Ok(Some(bytes)),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),
        Err(error)=>Err(error.into()),
    }
}

fn required(bytes: Option<Vec<u8>>,path: &str)->Result<Vec<u8>,RcountIoError> {
    bytes.ok_or_else(||std::io::Error::new(std::io::ErrorKind::NotFound,format!("Missing package file: {path}")).into())
}
fn json_record<T: for<'de> Deserialize<'de>>(read: &mut impl FnMut(&str)->Result<Option<Vec<u8>>,RcountIoError>,path:&str)->Result<T,RcountIoError> {
    Ok(serde_json::from_slice(&required(read(path)?,path)?)?)
}
fn ndjson_records<T: for<'de> Deserialize<'de>>(read: &mut impl FnMut(&str)->Result<Option<Vec<u8>>,RcountIoError>,path:&str,optional:bool)->Result<Vec<T>,RcountIoError> {
    let bytes=read(path)?;
    if optional && bytes.is_none() {return Ok(Vec::new());}
    let bytes=required(bytes,path)?;
    let mut records=Vec::new();
    for line in BufReader::new(bytes.as_slice()).lines() {
        let line=line?;
        if !line.trim().is_empty() { records.push(serde_json::from_str(&line)?); }
    }
    Ok(records)
}

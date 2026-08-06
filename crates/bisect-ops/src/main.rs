use anyhow::{bail, Context, Result};
use bisect_data::{
    build_adjacency_graph, connect_island_components, read_pl94_block_populations,
    read_pl94_block_populations_for_year, read_tiger_block_centroids_projected,
    read_tiger_blocks_projected, read_tiger_blocks_projected_for_year, BlockRecord,
};
use bisect_map::CategoricalScheme;
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    fs::File,
    io::{BufWriter, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const SCREEN_TIMEOUT: Duration = Duration::from_secs(180);
const GENERATED_AT: &str = "2026-07-12T00:00:00Z";
const BUILDER_SNAPSHOT: &str = "builder-source.rs";
const NRS_SEED_PREFIX: &[u8] = b"NRS_BASELINE_V0_1";

#[derive(Parser)]
#[command(about = "Rust-native deterministic operational recursive-tree builder")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    Build {
        #[arg(long)]
        bisect: PathBuf,
        #[arg(long)]
        context: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        districts: usize,
        #[arg(long, default_value_t = 1)]
        root_seed: u64,
        #[arg(long, default_value_t = 2)]
        child_seed_0: u64,
        #[arg(long, default_value_t = 3)]
        child_seed_1: u64,
        #[arg(long, default_value_t = 16)]
        max_seed: u64,
    },
    Verify {
        package: PathBuf,
    },
    Batch {
        #[arg(long)]
        bisect: PathBuf,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        retry_failed: bool,
        #[arg(long, default_value_t = 16)]
        max_seed: u64,
    },
    AuditPython {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
    },
    AnalyzeTree {
        #[arg(long)]
        state: String,
        #[arg(long)]
        package: PathBuf,
        #[arg(long)]
        rctx_report: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    VerifyTreeReport {
        manifest: PathBuf,
    },
    VerifyNationalRctx {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(long)]
        out_dir: Option<PathBuf>,
        #[arg(long)]
        context_root: Option<PathBuf>,
        #[arg(long)]
        require_complete: bool,
    },
    VerifyNationalTrees {
        #[arg(long, default_value = "docs/experiments/nationwide-2020")]
        out_dir: PathBuf,
        #[arg(long, default_value = "data/2020/certified/operational-trees")]
        package_root: PathBuf,
        #[arg(long, default_value = "data/2020/certified")]
        context_root: PathBuf,
        #[arg(
            long,
            default_value = "docs/experiments/small-states-2020/one-district-states.json"
        )]
        one_district: PathBuf,
    },
    BuildNationalRelease {
        #[arg(long, default_value = "release_staging/nationwide-2020-operational-v1")]
        out_dir: PathBuf,
        #[arg(long)]
        created_at: String,
    },
    VerifyNationalRelease {
        #[arg(default_value = "release_staging/nationwide-2020-operational-v1")]
        bundle: PathBuf,
    },
    NrsSeed {
        #[arg(long)]
        context: PathBuf,
        #[arg(long)]
        districts: usize,
        #[arg(long, default_value = "configs/nrs_v0_1/standard_profile.json")]
        standard_profile: PathBuf,
        #[arg(long, default_value = "configs/nrs_v0_1/legal_profile.json")]
        legal_profile: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        generated_at: String,
    },
    VerifyNrsSeed {
        package: PathBuf,
        #[arg(long)]
        context: PathBuf,
    },
    BuildNrsState {
        #[arg(long)]
        bisect: PathBuf,
        #[arg(long)]
        context: PathBuf,
        #[arg(long)]
        districts: usize,
        #[arg(long)]
        seed_package: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        generated_at: String,
    },
    VerifyNrsState {
        package: PathBuf,
        #[arg(long)]
        context: PathBuf,
    },
    NrsBatch {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(long)]
        bisect: PathBuf,
        #[arg(
            long,
            default_value = "docs/experiments/nationwide-2020/inventory.json"
        )]
        inventory: PathBuf,
        #[arg(long)]
        standard_profile: Option<PathBuf>,
        #[arg(long)]
        legal_profile: Option<PathBuf>,
        #[arg(long, default_value = "runs/nrs-v0.1/national-2020")]
        out_dir: PathBuf,
        #[arg(long)]
        generated_at: String,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, value_delimiter = ',')]
        states: Vec<String>,
        #[arg(long)]
        retry_failed: bool,
    },
    VerifyNrsBatch {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(
            long,
            default_value = "docs/experiments/nationwide-2020/inventory.json"
        )]
        inventory: PathBuf,
        #[arg(long)]
        standard_profile: Option<PathBuf>,
        #[arg(long)]
        legal_profile: Option<PathBuf>,
        #[arg(long, default_value = "runs/nrs-v0.1/national-2020")]
        out_dir: PathBuf,
        #[arg(long)]
        require_complete: bool,
    },
    SummarizeNrsBatch {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(
            long,
            default_value = "docs/experiments/nationwide-2020/inventory.json"
        )]
        inventory: PathBuf,
        #[arg(long)]
        standard_profile: Option<PathBuf>,
        #[arg(long)]
        legal_profile: Option<PathBuf>,
        #[arg(long, default_value = "runs/nrs-v0.1/national-2020")]
        out_dir: PathBuf,
        #[arg(long, default_value = "docs/experiments/nrs-v0.1-national-2020")]
        report_dir: PathBuf,
    },
    RctxBatch {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(long)]
        inventory: Option<PathBuf>,
        #[arg(long, default_value_t = 2)]
        workers: usize,
        #[arg(long)]
        limit: Option<usize>,
    },
    BuildStateRctx {
        #[arg(long, default_value_t = 2020)]
        year: u16,
        #[arg(long)]
        state_code: String,
        #[arg(long)]
        state_fips: String,
        #[arg(long)]
        state_name: String,
        #[arg(long)]
        shapefile: PathBuf,
        #[arg(long)]
        tiger_archive: Option<PathBuf>,
        #[arg(long)]
        pl_geo: PathBuf,
        #[arg(long)]
        pl_population: PathBuf,
        #[arg(long)]
        rctx: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    CompareRctx {
        left: PathBuf,
        right: PathBuf,
    },
    VerifyRiFrontier {
        manifest: PathBuf,
        #[arg(long)]
        check_rctx: bool,
    },
    VerifyExactFrontier {
        manifest: PathBuf,
        #[arg(long)]
        check_sources: bool,
    },
}

#[derive(Clone)]
struct Candidate {
    discovery: Value,
    seed: u64,
    dir: PathBuf,
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn write_json(path: &Path, value: &Value, pretty: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };
    if pretty {
        bytes.push(b'\n');
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn sha256(path: &Path) -> Result<String> {
    let mut source = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 8 * 1024 * 1024];
    loop {
        let count = source
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn git_blob_sha256(commit: &str, path: &str) -> Result<String> {
    let object = format!("{commit}:{path}");
    let output = Command::new("git")
        .args(["cat-file", "blob", &object])
        .output()
        .with_context(|| format!("read frozen source {object}"))?;
    if !output.status.success() {
        bail!("frozen source object is unavailable: {object}");
    }
    Ok(format!("{:x}", Sha256::digest(output.stdout)))
}

fn tiger_archive_member_hashes(path: &Path) -> Result<BTreeMap<String, String>> {
    let source = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(source)
        .with_context(|| format!("open TIGER archive {}", path.display()))?;
    let mut hashes = BTreeMap::new();
    for index in 0..archive.len() {
        let mut member = archive.by_index(index)?;
        let Some(name) = Path::new(member.name())
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let extension = Path::new(&name)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(extension.as_str(), "shp" | "dbf" | "shx") {
            continue;
        }
        let mut digest = Sha256::new();
        let mut buffer = vec![0_u8; 8 * 1024 * 1024];
        loop {
            let count = member.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
        }
        hashes.insert(name, format!("sha256:{:x}", digest.finalize()));
    }
    Ok(hashes)
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        let matches = root
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(extension));
        return Ok(if matches {
            vec![root.to_path_buf()]
        } else {
            Vec::new()
        });
    }
    if !root.is_dir() {
        bail!("source path does not exist: {}", root.display());
    }
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("read source directory {}", directory.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(extension))
            {
                files.push(entry.path());
            }
        }
    }
    files.sort_by_key(|path| portable_path(path));
    Ok(files)
}

fn read_tiger_block_bundle(path: &Path, year: u16) -> Result<(Vec<BlockRecord>, Vec<PathBuf>)> {
    let shapefiles = files_with_extension(path, "shp")?;
    if shapefiles.is_empty() {
        bail!("TIGER input contains no shapefiles: {}", path.display());
    }
    let mut blocks = Vec::new();
    for shapefile in &shapefiles {
        blocks.extend(read_tiger_blocks_projected_for_year(shapefile, year)?);
    }
    blocks.sort_by(|left, right| left.geoid.cmp(&right.geoid));
    if let Some(pair) = blocks
        .windows(2)
        .find(|pair| pair[0].geoid == pair[1].geoid)
    {
        bail!("duplicate TIGER block GEOID {}", pair[0].geoid);
    }
    Ok((blocks, shapefiles))
}

fn hashed_source_map(paths: &[PathBuf]) -> Result<Value> {
    let mut hashes = Map::new();
    for path in paths {
        hashes.insert(
            portable_path(path),
            json!(format!("sha256:{}", sha256(path)?)),
        );
    }
    Ok(Value::Object(hashes))
}

fn governed_source_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("source custody path is not a safe relative path: {relative:?}");
    }
    Ok(root.join(relative))
}

fn custody_source(path: &Path) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    }
    let archived = Path::new("archive/legacy-python").join(path);
    if archived.is_file() {
        return archived;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let active = workspace.join(path);
    if active.is_file() {
        active
    } else {
        workspace.join("archive/legacy-python").join(path)
    }
}

fn portable_path(path: &Path) -> String {
    let root = std::env::current_dir().ok();
    let relative = root
        .as_deref()
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path);
    relative.to_string_lossy().replace('\\', "/")
}

fn canonical_hash(value: &Value) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}

fn canonical_sha256(value: &Value) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}

fn nrs_seed(input_manifest: &Value) -> Result<(String, u64, u32)> {
    let mut hasher = Sha256::new();
    hasher.update(NRS_SEED_PREFIX);
    hasher.update(serde_json::to_vec(input_manifest)?);
    let digest = hasher.finalize();
    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    let seed = u64::from_le_bytes(first);
    Ok((format!("{digest:x}"), seed, (seed % 2_147_483_647) as u32))
}

fn nrs_context_year(context: &Value) -> Result<u16> {
    context
        .pointer("/units/year")
        .and_then(Value::as_u64)
        .and_then(|year| u16::try_from(year).ok())
        .context("NRS context census year")
}

fn default_nrs_profiles(year: u16) -> (PathBuf, PathBuf) {
    let suffix = if year == 2020 {
        String::new()
    } else {
        format!("_{year}")
    };
    (
        PathBuf::from(format!("configs/nrs_v0_1/standard_profile{suffix}.json")),
        PathBuf::from(format!("configs/nrs_v0_1/legal_profile{suffix}.json")),
    )
}

fn resolve_nrs_profiles(
    year: u16,
    standard_profile: Option<&Path>,
    legal_profile: Option<&Path>,
) -> (PathBuf, PathBuf) {
    let defaults = default_nrs_profiles(year);
    (
        standard_profile.unwrap_or(&defaults.0).to_path_buf(),
        legal_profile.unwrap_or(&defaults.1).to_path_buf(),
    )
}

fn validate_nrs_profile_cycle(
    year: u16,
    standard_profile: &Value,
    legal_profile: &Value,
) -> Result<()> {
    let cycles = standard_profile["effective_census_cycles"]
        .as_array()
        .context("NRS standard profile effective census cycles")?;
    if !cycles
        .iter()
        .any(|cycle| cycle.as_u64() == Some(u64::from(year)))
    {
        bail!("NRS standard profile does not govern census year {year}");
    }
    if legal_profile["census_cycle"].as_u64() != Some(u64::from(year)) {
        bail!("NRS legal profile does not govern census year {year}");
    }
    Ok(())
}

fn build_nrs_seed_package(
    context_path: &Path,
    districts: usize,
    standard_profile_path: &Path,
    legal_profile_path: &Path,
    out: &Path,
    generated_at: &str,
) -> Result<()> {
    if out.exists() {
        bail!("NRS seed package already exists: {}", out.display());
    }
    if districts == 0 {
        bail!("NRS district count must be positive");
    }
    let context = read_json(context_path)?;
    let standard_profile = read_json(standard_profile_path)?;
    let legal_profile = read_json(legal_profile_path)?;
    if !matches!(
        standard_profile["schema_version"].as_str(),
        Some(
            "nrs-standard-profile-v0.1-v1"
                | "nrs-standard-profile-v0.2-v1"
                | "nrs-standard-profile-v0.3-v1"
        )
    ) || legal_profile["schema_version"] != "nrs-baseline-legal-profile-v1"
    {
        bail!("unknown NRS profile schema");
    }
    let year = nrs_context_year(&context)?;
    validate_nrs_profile_cycle(year, &standard_profile, &legal_profile)?;
    let unit_ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("NRS context unit ids")?;
    if unit_ids.is_empty()
        || unit_ids
            .windows(2)
            .any(|pair| pair[0].as_str() >= pair[1].as_str())
        || unit_ids.iter().any(|id| {
            id.as_str()
                .is_none_or(|id| id.len() != 15 || !id.bytes().all(|byte| byte.is_ascii_digit()))
        })
    {
        bail!("NRS unit index is not strictly sorted 15-digit GEOIDs");
    }
    let populations = context["populations"]
        .as_array()
        .context("NRS populations")?;
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("NRS adjacency")?;
    if populations.len() != unit_ids.len() || adjacency.len() != unit_ids.len() {
        bail!("NRS context universe lengths disagree");
    }
    let unit_index = json!({
        "schema_version":"nrs-unit-index-v1","unit_kind":"block",
        "canonical_order":"sorted-geoid","unit_ids":unit_ids
    });
    let reference_engine = standard_profile
        .get("reference_engine")
        .context("NRS reference engine")?;
    let input_manifest = json!({
        "adjacency_sha256":canonical_sha256(&Value::Array(adjacency.clone()))?,
        "algorithm_profile_sha256":canonical_sha256(&standard_profile)?,
        "canonicalization_version":"canonical-json-v1",
        "census_release":format!("{year}-PL94-171"),
        "district_count":districts,
        "geographic_vintage":format!("TIGER-Line-{year}-tabulation-blocks"),
        "legal_profile_sha256":canonical_sha256(&legal_profile)?,
        "population_sha256":canonical_sha256(&Value::Array(populations.clone()))?,
        "reference_engine_sha256":canonical_sha256(reference_engine)?,
        "unit_index_sha256":canonical_sha256(&unit_index)?
    });
    let (digest, seed_u64, seed_i32) = nrs_seed(&input_manifest)?;
    fs::create_dir_all(out)?;
    write_json(&out.join("standard_profile.json"), &standard_profile, true)?;
    write_json(&out.join("legal_profile.json"), &legal_profile, true)?;
    write_json(&out.join("unit_index.json"), &unit_index, true)?;
    write_json(&out.join("input_manifest.json"), &input_manifest, true)?;
    write_json(
        &out.join("seed_record.json"),
        &json!({
            "schema_version":"nrs-seed-record-v1",
            "generated_at":generated_at,
            "input_manifest_canonical_sha256":canonical_sha256(&input_manifest)?,
            "derivation":"SHA-256(ASCII(NRS_BASELINE_V0_1) || canonical-json-v1(input_manifest))",
            "digest_sha256":digest,"seed_u64_little_endian":seed_u64,
            "engine_seed_i32":seed_i32,"engine_conversion":"seed mod 2147483647"
        }),
        true,
    )?;
    let artifacts = [
        "standard_profile.json",
        "legal_profile.json",
        "unit_index.json",
        "input_manifest.json",
        "seed_record.json",
    ]
    .into_iter()
    .map(|path| Ok(json!({"path":path,"sha256":sha256(&out.join(path))?})))
    .collect::<Result<Vec<_>>>()?;
    write_json(
        &out.join("manifest.json"),
        &json!({
            "schema_version":"nrs-seed-package-v1","status":"seed-derived",
            "BISECT_version":env!("CARGO_PKG_VERSION"),
            "BISECT_build_commit":git_text(&["rev-parse","HEAD"]).unwrap_or_else(|_| "unknown".into()),
            "rustc_version":Command::new("rustc").arg("--version").output().ok().and_then(|output|String::from_utf8(output.stdout).ok()).map(|text|text.trim().to_owned()).unwrap_or_else(||"unknown".into()),
            "created_at":generated_at,
            "source_context_sha256":sha256(context_path)?,"district_count":districts,
            "artifacts":artifacts,
            "claim_boundary":"Seed and assignment-affecting input identities only; no baseline assignment or NRS conformance claim."
        }),
        true,
    )?;
    verify_nrs_seed_package(out, context_path)?;
    println!("NRS seed package: VERIFIED; seed_u64={seed_u64}; engine_seed_i32={seed_i32}");
    Ok(())
}

fn verify_nrs_seed_package(package: &Path, context_path: &Path) -> Result<()> {
    let manifest = read_json(&package.join("manifest.json"))?;
    if manifest["schema_version"] != "nrs-seed-package-v1" {
        bail!("unknown NRS seed package schema");
    }
    if manifest["source_context_sha256"] != sha256(context_path)? {
        bail!("NRS source context transport hash mismatch");
    }
    for artifact in manifest["artifacts"].as_array().context("NRS artifacts")? {
        let relative = artifact["path"].as_str().context("NRS artifact path")?;
        if relative.contains("..") || Path::new(relative).is_absolute() {
            bail!("nonportable NRS artifact path");
        }
        let path = package.join(relative);
        if sha256(&path)? != artifact["sha256"] {
            bail!("NRS artifact hash mismatch: {relative}");
        }
    }
    let input_manifest = read_json(&package.join("input_manifest.json"))?;
    let seed_record = read_json(&package.join("seed_record.json"))?;
    let (digest, seed_u64, seed_i32) = nrs_seed(&input_manifest)?;
    if seed_record["input_manifest_canonical_sha256"] != canonical_sha256(&input_manifest)?
        || seed_record["digest_sha256"] != digest
        || seed_record["seed_u64_little_endian"] != seed_u64
        || seed_record["engine_seed_i32"] != seed_i32
    {
        bail!("NRS seed derivation mismatch");
    }
    let standard_profile = read_json(&package.join("standard_profile.json"))?;
    let legal_profile = read_json(&package.join("legal_profile.json"))?;
    let unit_index = read_json(&package.join("unit_index.json"))?;
    for (field, expected) in [
        (
            "algorithm_profile_sha256",
            canonical_sha256(&standard_profile)?,
        ),
        ("legal_profile_sha256", canonical_sha256(&legal_profile)?),
        ("unit_index_sha256", canonical_sha256(&unit_index)?),
        (
            "reference_engine_sha256",
            canonical_sha256(&standard_profile["reference_engine"])?,
        ),
    ] {
        if input_manifest[field] != expected {
            bail!("NRS input manifest {field} mismatch");
        }
    }
    let context = read_json(context_path)?;
    let populations = context["populations"]
        .as_array()
        .context("NRS populations")?;
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("NRS adjacency")?;
    if input_manifest["population_sha256"] != canonical_sha256(&Value::Array(populations.clone()))?
        || input_manifest["adjacency_sha256"] != canonical_sha256(&Value::Array(adjacency.clone()))?
        || unit_index["unit_ids"] != context["units"]["unit_ids"]
    {
        bail!("NRS context canonical binding mismatch");
    }
    let source_commit = standard_profile
        .pointer("/reference_engine/source_commit")
        .and_then(Value::as_str)
        .context("NRS reference engine source commit")?;
    if let Some(git_blobs) = standard_profile
        .pointer("/reference_engine/source_git_blob_sha256")
        .and_then(Value::as_object)
    {
        for (relative, expected) in git_blobs {
            if git_blob_sha256(source_commit, relative)?
                != expected.as_str().context("NRS git blob hash")?
            {
                bail!("NRS reference engine Git blob mismatch: {relative}");
            }
        }
    } else {
        for (relative, expected) in standard_profile
            .pointer("/reference_engine/source_files")
            .and_then(Value::as_object)
            .context("NRS reference engine source files")?
        {
            if sha256(Path::new(relative))? != expected.as_str().context("NRS source hash")? {
                bail!("NRS reference engine source mismatch: {relative}");
            }
        }
    }
    println!("NRS seed package verification: PASS");
    Ok(())
}

fn field_u64(value: &Value, path: &[&str]) -> Result<u64> {
    let mut current = value;
    for key in path {
        current = current
            .get(*key)
            .with_context(|| format!("missing {}", path.join(".")))?;
    }
    current
        .as_u64()
        .with_context(|| format!("{} is not u64", path.join(".")))
}

fn objective(discovery: &Value) -> Result<&Value> {
    discovery
        .pointer("/objective/primary")
        .context("missing objective.primary")
}

fn discovery_seed(discovery: &Value) -> Result<u64> {
    discovery["method"]
        .as_str()
        .context("missing discovery method")?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("seed=")?.parse().ok())
        .context("discovery method has no seed")
}

fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    let key = |c: &Candidate, name| {
        field_u64(&c.discovery, &["objective", "primary", name]).unwrap_or(u64::MAX)
    };
    key(a, "max_population_deviation_scaled")
        .cmp(&key(b, "max_population_deviation_scaled"))
        .then_with(|| {
            key(a, "total_population_deviation_scaled")
                .cmp(&key(b, "total_population_deviation_scaled"))
        })
        .then_with(|| key(a, "weighted_boundary_cut").cmp(&key(b, "weighted_boundary_cut")))
        .then_with(|| a.seed.cmp(&b.seed))
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn is_transient_file_lock(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| {
                matches!(io_error.raw_os_error(), Some(32) | Some(33))
                    || io_error.kind() == std::io::ErrorKind::WouldBlock
            })
    })
}

fn prune(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.file_name().and_then(|v| v.to_str()) != Some("certified-discovery.json") {
            remove_path(&path)?;
        }
    }
    Ok(())
}

fn run_discovery(
    bisect: &Path,
    context: &Path,
    districts: usize,
    out: &Path,
    seed: u64,
    refinement: &str,
    timeout: Option<Duration>,
) -> Result<Option<Value>> {
    let mut child = Command::new(bisect)
        .args(["exact", "--context"])
        .arg(context)
        .args([
            "--districts",
            &districts.to_string(),
            "--method",
            "certified-discovery",
            "--out-dir",
        ])
        .arg(out)
        .args([
            "--generated-at",
            GENERATED_AT,
            "--discovery-seed",
            &seed.to_string(),
            "--discovery-refinement",
            refinement,
        ])
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("launch seed {seed}"))?;
    if let Some(limit) = timeout {
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    bail!("seed {seed} failed with {status}");
                }
                break;
            }
            if start.elapsed() >= limit {
                child.kill()?;
                let _ = child.wait();
                return Ok(None);
            }
            thread::sleep(Duration::from_millis(100));
        }
    } else if !child.wait()?.success() {
        bail!("seed {seed} discovery failed");
    }
    Ok(Some(read_json(&out.join("certified-discovery.json"))?))
}

fn floor_discovery(
    bisect: &Path,
    context: &Path,
    districts: usize,
    out: &Path,
    preferred: u64,
    floor: u64,
    max_seed: u64,
) -> Result<(Value, u64, Value)> {
    let completed_path = out.join("certified-discovery.json");
    if completed_path.is_file() {
        let completed = read_json(&completed_path)?;
        if field_u64(
            &completed,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            let report = if out.join("seed-screening.json").is_file() {
                read_json(&out.join("seed-screening.json"))?
            } else {
                json!([{"seed": discovery_seed(&completed)?, "status":"selected-node-reused", "objective": objective(&completed)?}])
            };
            println!(
                "{}: reused completed node at arithmetic floor {floor}",
                out.file_name().unwrap().to_string_lossy()
            );
            return Ok((completed.clone(), discovery_seed(&completed)?, report));
        }
    }
    let mut seeds = vec![preferred];
    seeds.extend((1..=max_seed).filter(|seed| *seed != preferred));
    let mut screened = Vec::new();
    let mut report = Vec::new();
    for seed in seeds {
        let screen = out.with_file_name(format!(
            "{}-screen-seed-{seed:02}",
            out.file_name().unwrap().to_string_lossy()
        ));
        let timeout_path = screen.with_extension("timeout.json");
        if timeout_path.is_file() {
            report
                .push(json!({"seed":seed,"status":"timeout","timeout_seconds":180,"reused":true}));
            continue;
        }
        let path = screen.join("certified-discovery.json");
        let reused = path.is_file();
        let discovery = if reused {
            let value = read_json(&path)?;
            prune(&screen)?;
            Some(value)
        } else {
            remove_path(&screen)?;
            match run_discovery(
                bisect,
                context,
                districts,
                &screen,
                seed,
                "metis",
                Some(SCREEN_TIMEOUT),
            ) {
                Ok(value) => value,
                Err(error) => {
                    remove_path(&screen)?;
                    report.push(json!({"seed":seed,"status":"failed","error":error.to_string()}));
                    continue;
                }
            }
        };
        let Some(discovery) = discovery else {
            remove_path(&screen)?;
            write_json(
                &timeout_path,
                &json!({"status":"timeout","timeout_seconds":180}),
                true,
            )?;
            report.push(json!({"seed":seed,"status":"timeout","timeout_seconds":180}));
            continue;
        };
        prune(&screen)?;
        report.push(json!({"seed":seed,"status":"completed","reused":reused,"objective":objective(&discovery)?}));
        screened.push(Candidate {
            discovery,
            seed,
            dir: screen,
        });
    }
    screened.sort_by(rank);
    let mut candidates = Vec::new();
    let mut selected = None;
    for screen in &screened {
        if field_u64(
            &screen.discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            selected = Some(screen.clone());
            break;
        }
        let dir = out.with_file_name(format!(
            "{}-seed-{:02}",
            out.file_name().unwrap().to_string_lossy(),
            screen.seed
        ));
        let path = dir.join("certified-discovery.json");
        let reused = path.is_file();
        let discovery = if reused {
            read_json(&path)?
        } else {
            remove_path(&dir)?;
            run_discovery(
                bisect,
                context,
                districts,
                &dir,
                screen.seed,
                "population",
                None,
            )?
            .context("unbounded discovery returned timeout")?
        };
        prune(&dir)?;
        if let Some(row) = report
            .iter_mut()
            .find(|row| row["seed"].as_u64() == Some(screen.seed))
        {
            row["refined_objective"] = objective(&discovery)?.clone();
            row["refined_reused"] = json!(reused);
        }
        let candidate = Candidate {
            discovery,
            seed: screen.seed,
            dir,
        };
        if field_u64(
            &candidate.discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            selected = Some(candidate.clone());
            candidates.push(candidate);
            break;
        }
        candidates.push(candidate);
    }
    let report_value = Value::Array(report);
    let Some(selected) = selected else {
        fs::create_dir_all(out)?;
        write_json(&out.join("seed-screening.json"), &report_value, true)?;
        let best = candidates
            .iter()
            .min_by(|a, b| rank(a, b))
            .context("no completed refinement")?;
        write_json(
            &out.join("unresolved-floor.json"),
            &json!({"population_floor":floor,"status":"unresolved-local-search-frontier","best_seed":best.seed,"best_objective":objective(&best.discovery)?,"claim_boundary":"No screened deterministic seed reached the arithmetic population floor; this is not a proof of infeasibility."}),
            true,
        )?;
        bail!(
            "no seed reached arithmetic population floor {floor}; best seed {} reached {}",
            best.seed,
            field_u64(
                &best.discovery,
                &["objective", "primary", "max_population_deviation_scaled"]
            )?
        );
    };
    remove_path(out)?;
    fs::rename(&selected.dir, out)?;
    write_json(&out.join("seed-screening.json"), &report_value, true)?;
    for candidate in candidates.iter().chain(screened.iter()) {
        if candidate.dir != selected.dir && candidate.dir.exists() {
            remove_path(&candidate.dir)?;
        }
    }
    for name in [
        "audit-certificate.json",
        "certified-discovery-manifest.json",
        "discovery.rctx",
        "discovery.rplan",
    ] {
        remove_path(&out.join(name))?;
    }
    Ok((selected.discovery, selected.seed, report_value))
}

fn ratio_floor(population: i64, seats: usize, right: usize) -> u64 {
    let rem = (right as i64 * population).rem_euclid(seats as i64);
    rem.min(seats as i64 - rem) as u64
}

fn nrs_generation_tolerance_scaled_bound(population: i64, smaller_child_seats: usize) -> u64 {
    // The established BISECT integer convention rounds a 0.5% allowance up.
    ((5_i128 * smaller_child_seats as i128 * population as i128 + 999) / 1_000) as u64
}

fn subset_context(context: &Value, selected: &[usize], source_id: String) -> Result<Value> {
    let chosen: BTreeSet<_> = selected.iter().copied().collect();
    let remap: std::collections::BTreeMap<_, _> = selected
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?;
    let pops = context["populations"].as_array().context("populations")?;
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    let mut units = context["units"].clone();
    units["unit_ids"] = Value::Array(selected.iter().map(|i| ids[*i].clone()).collect());
    units["source_id"] = json!(source_id);
    let mut projected = Vec::new();
    for old in selected {
        projected.push(Value::Array(
            adjacency[*old]
                .as_array()
                .context("edge list")?
                .iter()
                .filter_map(|edge| {
                    let to = edge["to"].as_u64()? as usize;
                    chosen.contains(&to).then(|| {
                        let mut e = edge.clone();
                        e["to"] = json!(remap[&to]);
                        e
                    })
                })
                .collect(),
        ));
    }
    let mut projection = json!({"units":units,"graph":{"edge_semantics":"undirected","adjacency":projected},"populations":selected.iter().map(|i|pops[*i].clone()).collect::<Vec<_>>(),"source_hashes":context["source_hashes"]});
    projection["units"]["unit_universe_hash"] = json!(canonical_hash(&projection["units"])?);
    let hash = canonical_hash(&projection)?;
    let mut result = Map::new();
    result.insert("rctx_version".into(), json!("0.1"));
    result.insert("context_hash".into(), json!(hash));
    for (key, value) in projection.as_object().unwrap() {
        result.insert(key.clone(), value.clone());
    }
    Ok(Value::Object(result))
}

struct BuildState<'a> {
    bisect: &'a Path,
    out: &'a Path,
    original: &'a Value,
    child_seeds: [u64; 2],
    max_seed: u64,
    assignment: Vec<i64>,
    nodes: Vec<Value>,
    leaves: Vec<Value>,
}

struct Visit {
    context: Value,
    context_path: PathBuf,
    global: Vec<usize>,
    seats: usize,
    path: String,
    offset: usize,
    seed: u64,
}

impl BuildState<'_> {
    fn visit(&mut self, visit: Visit) -> Result<()> {
        let Visit {
            context,
            context_path,
            global,
            seats,
            path,
            offset,
            seed,
        } = visit;
        let populations = context["populations"]
            .as_array()
            .context("node populations")?;
        if seats == 1 {
            for &unit in &global {
                self.assignment[unit] = offset as i64;
            }
            let population: i64 = global
                .iter()
                .map(|&unit| self.original["populations"][unit].as_i64().unwrap_or(0))
                .sum();
            self.leaves.push(json!({"path":path,"district":offset,"unit_count":global.len(),"population":population}));
            return Ok(());
        }
        let name = if path.is_empty() {
            "root".into()
        } else {
            format!("node-{path}")
        };
        let node_dir = self.out.join(name);
        let left = seats / 2;
        let right = seats - left;
        let parent_population: i64 = populations.iter().map(|v| v.as_i64().unwrap_or(0)).sum();
        let floor = ratio_floor(parent_population, seats, right);
        let (discovery, selected_seed, screening) = floor_discovery(
            self.bisect,
            &context_path,
            seats,
            &node_dir,
            seed,
            floor,
            self.max_seed,
        )?;
        let labels = discovery
            .pointer("/objective/canonical_assignment")
            .and_then(Value::as_array)
            .context("canonical assignment")?;
        self.nodes.push(json!({
            "path":path,"seats":seats,"parent_population":parent_population,
            "discovery_id":discovery["discovery_id"],"seed":selected_seed,
            "seed_screening":screening,"objective":objective(&discovery)?,
            "population_proof":{"kind":"ratio-arithmetic-floor","lower_bound":floor},
            "context_sha256":sha256(&context_path)?
        }));
        for label in 0..=1usize {
            let child_seats = if label == 0 { left } else { right };
            let child_offset = if label == 0 { offset } else { offset + left };
            let local: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(i, v)| (v.as_u64() == Some(label as u64)).then_some(i))
                .collect();
            if local.is_empty() {
                bail!("node {path} produced empty child {label}");
            }
            let child_global = local.iter().map(|&unit| global[unit]).collect();
            let child_path = format!("{path}{label}");
            let child_context = subset_context(
                &context,
                &local,
                format!(
                    "operational-tree-node-{}-{label}",
                    if path.is_empty() { "root" } else { &path }
                ),
            )?;
            let child_context_path = self.out.join(format!("context-{child_path}.rctx"));
            write_json(&child_context_path, &child_context, false)?;
            let child_seed = if path.is_empty() {
                self.child_seeds[label]
            } else {
                seed + label as u64 + 1
            };
            self.visit(Visit {
                context: child_context,
                context_path: child_context_path,
                global: child_global,
                seats: child_seats,
                path: child_path,
                offset: child_offset,
                seed: child_seed,
            })?;
        }
        Ok(())
    }
}

struct NrsBuildState<'a> {
    bisect: &'a Path,
    bisect_executable_sha256: &'a str,
    out: &'a Path,
    original: &'a Value,
    engine_seed: u64,
    discovery_refinement: String,
    assignment: Vec<i64>,
    nodes: Vec<Value>,
    leaves: Vec<Value>,
}

impl NrsBuildState<'_> {
    fn visit(&mut self, visit: Visit) -> Result<()> {
        let Visit {
            context,
            context_path,
            global,
            seats,
            path,
            offset,
            ..
        } = visit;
        let populations = context["populations"]
            .as_array()
            .context("NRS node populations")?;
        let unit_ids = context
            .pointer("/units/unit_ids")
            .and_then(Value::as_array)
            .context("NRS node unit ids")?;
        if seats == 1 {
            for &unit in &global {
                self.assignment[unit] = offset as i64;
            }
            let population: i64 = global
                .iter()
                .map(|&unit| self.original["populations"][unit].as_i64().unwrap_or(0))
                .sum();
            self.leaves.push(json!({
                "path":path,"district_zero_based":offset,"district_one_based":offset + 1,
                "unit_count":global.len(),"population":population,
                "minimum_geoid":unit_ids.first().and_then(Value::as_str)
            }));
            return Ok(());
        }
        let name = if path.is_empty() {
            "root".into()
        } else {
            format!("node-{path}")
        };
        let node_dir = self.out.join("nodes").join(name);
        let discovery = run_discovery(
            self.bisect,
            &context_path,
            seats,
            &node_dir,
            self.engine_seed,
            &self.discovery_refinement,
            None,
        )?
        .context("NRS discovery unexpectedly timed out")?;
        prune(&node_dir)?;
        if discovery_seed(&discovery)? != self.engine_seed
            || !discovery["method"].as_str().is_some_and(|method| {
                method.contains("niter=100")
                    && method.contains("partition-type=recursive")
                    && method.contains(
                        "candidate-initialization=minimum-geoid-rooted-sorted-dfs-tree-edge-cut",
                    )
                    && method.contains(&format!(
                        "refinement={}",
                        self.discovery_refinement.replace('-', "")
                    ))
            })
        {
            bail!("NRS discovery profile mismatch at node {path}");
        }
        let raw_labels = discovery
            .pointer("/objective/canonical_assignment")
            .and_then(Value::as_array)
            .context("NRS canonical assignment")?;
        if raw_labels.len() != global.len() {
            bail!("NRS node assignment length mismatch at {path}");
        }
        let reverse = raw_labels.first().and_then(Value::as_u64) == Some(1);
        let labels: Vec<u64> = raw_labels
            .iter()
            .map(|label| {
                let label = label.as_u64().context("NRS child label")?;
                if label > 1 {
                    bail!("NRS non-binary child label");
                }
                Ok(if reverse { 1 - label } else { label })
            })
            .collect::<Result<_>>()?;
        if labels.first() != Some(&0) {
            bail!("NRS minimum-GEOID orientation failed at {path}");
        }
        let floor_seats = seats / 2;
        let ceil_seats = seats - floor_seats;
        let child_seats = if reverse {
            [ceil_seats, floor_seats]
        } else {
            [floor_seats, ceil_seats]
        };
        let parent_population: i64 = populations.iter().map(|v| v.as_i64().unwrap_or(0)).sum();
        let arithmetic_floor = ratio_floor(parent_population, seats, ceil_seats);
        let achieved = field_u64(
            &discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )?;
        let tolerance_bound = nrs_generation_tolerance_scaled_bound(parent_population, floor_seats);
        self.nodes.push(json!({
            "path":path,"seats":seats,"child_seats":child_seats,
            "parent_population":parent_population,"minimum_geoid":unit_ids[0],
            "discovery_id":discovery["discovery_id"],"engine_seed_i32":self.engine_seed,
            "discovery_path":release_relative(self.out,&node_dir.join("certified-discovery.json"))?,
            "discovery_sha256":sha256(&node_dir.join("certified-discovery.json"))?,
            "objective":objective(&discovery)?,
            "population_floor":{"lower_bound":arithmetic_floor,"attained":achieved == arithmetic_floor},
            "generation_tolerance_scaled_bound":tolerance_bound,
            "orientation_reversed_from_engine":reverse,
            "context_canonical_sha256":canonical_sha256(&context)?
        }));
        if achieved > tolerance_bound {
            write_json(
                &self.out.join("nrs-failure.json"),
                &json!({
                    "schema_version":"nrs-baseline-failure-v2",
                    "status":"candidate-failed-population-tolerance","node_path":path,
                    "bisect_executable_sha256":self.bisect_executable_sha256,
                    "engine_seed_i32":self.engine_seed,"achieved_scaled_deviation":achieved,
                    "allowed_scaled_deviation":tolerance_bound,
                    "discovery_path":release_relative(self.out,&node_dir.join("certified-discovery.json"))?,
                    "discovery_sha256":sha256(&node_dir.join("certified-discovery.json"))?,
                    "claim_boundary":"The single canonical candidate failed the profile tolerance. This is a benchmark failure record, not a proof that no feasible partition exists."
                }),
                true,
            )?;
            bail!("NRS node {path} exceeded population tolerance: {achieved} > {tolerance_bound}");
        }
        for label in 0..=1usize {
            let local: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(unit, child)| (*child == label as u64).then_some(unit))
                .collect();
            if local.is_empty() {
                bail!("NRS node {path} produced empty child {label}");
            }
            let child_global = local.iter().map(|&unit| global[unit]).collect();
            let child_path = format!("{path}{label}");
            let child_context = subset_context(
                &context,
                &local,
                format!(
                    "nrs-v0.1-node-{}-{label}",
                    if path.is_empty() { "root" } else { &path }
                ),
            )?;
            let child_context_path = self.out.join(format!("context-{child_path}.rctx"));
            write_json(&child_context_path, &child_context, false)?;
            self.visit(Visit {
                context: child_context,
                context_path: child_context_path,
                global: child_global,
                seats: child_seats[label],
                path: child_path,
                offset: if label == 0 {
                    offset
                } else {
                    offset + child_seats[0]
                },
                seed: self.engine_seed,
            })?;
        }
        Ok(())
    }
}

fn build_nrs_state(
    bisect: &Path,
    context_path: &Path,
    districts: usize,
    seed_package: &Path,
    out: &Path,
    generated_at: &str,
) -> Result<()> {
    if out.exists() {
        bail!("NRS State package already exists: {}", out.display());
    }
    if !git_text(&["status", "--porcelain"])?.is_empty() {
        bail!("NRS State package must be generated from a clean working tree");
    }
    verify_nrs_seed_package(seed_package, context_path)?;
    let seed_record = read_json(&seed_package.join("seed_record.json"))?;
    let standard_profile = read_json(&seed_package.join("standard_profile.json"))?;
    let discovery_refinement = standard_profile
        .pointer("/search/discovery_refinement")
        .and_then(Value::as_str)
        .unwrap_or("nrs-v0-1")
        .to_owned();
    if !matches!(
        discovery_refinement.as_str(),
        "nrs-v0-1" | "nrs-v0-2" | "nrs-v0-3"
    ) {
        bail!("unsupported NRS discovery refinement: {discovery_refinement}");
    }
    let engine_seed = seed_record["engine_seed_i32"]
        .as_u64()
        .context("NRS engine seed")?;
    let seed_districts = read_json(&seed_package.join("manifest.json"))?["district_count"]
        .as_u64()
        .context("NRS seed district count")? as usize;
    if districts != seed_districts || districts == 0 {
        bail!("NRS State and seed-package district counts disagree");
    }
    fs::create_dir_all(out.join("seed"))?;
    for entry in fs::read_dir(seed_package)? {
        let path = entry?.path();
        if path.is_file() {
            copy_artifact(&path, &out.join("seed").join(path.file_name().unwrap()))?;
        }
    }
    let context = read_json(context_path)?;
    let year = nrs_context_year(&context)?;
    let bisect_executable_sha256 = sha256(bisect)?;
    let unit_ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("NRS unit ids")?;
    let mut state = NrsBuildState {
        bisect,
        bisect_executable_sha256: &bisect_executable_sha256,
        out,
        original: &context,
        engine_seed,
        discovery_refinement,
        assignment: vec![-1; unit_ids.len()],
        nodes: Vec::new(),
        leaves: Vec::new(),
    };
    state.visit(Visit {
        context: context.clone(),
        context_path: context_path.to_path_buf(),
        global: (0..unit_ids.len()).collect(),
        seats: districts,
        path: String::new(),
        offset: 0,
        seed: engine_seed,
    })?;
    if state.assignment.iter().any(|district| *district < 0) {
        bail!("NRS State tree left blocks unassigned");
    }
    for district in 0..districts {
        if !connected(&context, &state.assignment, district as i64)? {
            bail!("NRS district {district} is disconnected");
        }
    }
    let assignments = unit_ids
        .iter()
        .zip(&state.assignment)
        .map(|(geoid, district)| {
            Ok((
                geoid.as_str().context("NRS GEOID")?.to_owned(),
                json!(district + 1),
            ))
        })
        .collect::<Result<Map<String, Value>>>()?;
    let population_total: i64 = context["populations"]
        .as_array()
        .context("NRS populations")?
        .iter()
        .filter_map(Value::as_i64)
        .sum();
    let all_floors = state
        .nodes
        .iter()
        .all(|node| node["population_floor"]["attained"] == true);
    let tree = json!({
        "schema_version":"nrs-baseline-tree-v0.1-v1","state":context["units"]["state"],
        "year":year,"districts":districts,"engine_seed_i32":engine_seed,
        "unit_count":unit_ids.len(),"population_total":population_total,
        "nodes":state.nodes,"leaves":state.leaves,"assignment":state.assignment,
        "population_arithmetic_floor_all_nodes":all_floors,
        "claim_boundary":"Single-manifest-seed NRS v0.1 reference-engine baseline. Boundary and canonical global optimality are not claimed."
    });
    write_json(&out.join("baseline-tree.json"), &tree, true)?;
    write_json(
        &out.join("baseline_assignments.json"),
        &json!({
            "schema_version":"nrs-baseline-assignments-v0.1-v1","label_base":1,
            "canonical_order":"sorted-geoid","assignments":assignments
        }),
        true,
    )?;
    for entry in fs::read_dir(out)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("context-") && name.ends_with(".rctx"))
        {
            remove_path(&path)?;
        }
    }
    let artifact_paths = release_files(out)?;
    let artifacts = artifact_paths
        .iter()
        .map(|path| Ok(json!({"path":release_relative(out,path)?,"sha256":sha256(path)?})))
        .collect::<Result<Vec<_>>>()?;
    write_json(
        &out.join("baseline_manifest.json"),
        &json!({
            "schema_version":"nrs-baseline-package-v0.1-v2",
            "BISECT_version":env!("CARGO_PKG_VERSION"),
            "BISECT_build_commit":git_text(&["rev-parse","HEAD"]).unwrap_or_else(|_|"unknown".into()),
            "bisect_executable_sha256":bisect_executable_sha256,
            "rustc_version":Command::new("rustc").arg("--version").output().ok().and_then(|output|String::from_utf8(output.stdout).ok()).map(|text|text.trim().to_owned()).unwrap_or_else(||"unknown".into()),
            "created_at":generated_at,"status":"reference-baseline-candidate",
            "source_context_sha256":sha256(context_path)?,
            "input_manifest_canonical_sha256":seed_record["input_manifest_canonical_sha256"],
            "seed_u64_little_endian":seed_record["seed_u64_little_endian"],
            "engine_seed_i32":engine_seed,"artifacts":artifacts,
            "verification_status":"pass",
            "non_claims":["boundary global optimality","canonical global optimality","VRA compliance","partisan fairness","legal validity","official adoption"]
        }),
        true,
    )?;
    verify_nrs_state(out, context_path)?;
    println!("NRS State baseline package: VERIFIED ({})", out.display());
    Ok(())
}

fn verify_nrs_state(package: &Path, context_path: &Path) -> Result<()> {
    verify_nrs_seed_package(&package.join("seed"), context_path)?;
    let manifest = read_json(&package.join("baseline_manifest.json"))?;
    if manifest["schema_version"] != "nrs-baseline-package-v0.1-v2"
        || manifest["verification_status"] != "pass"
        || manifest["status"] != "reference-baseline-candidate"
        || manifest["source_context_sha256"] != sha256(context_path)?
    {
        bail!("NRS baseline manifest posture or context mismatch");
    }
    let executable_hash = manifest["bisect_executable_sha256"]
        .as_str()
        .context("NRS baseline executable hash")?;
    if !is_sha256_hex(executable_hash) {
        bail!("invalid NRS baseline executable hash");
    }
    for artifact in manifest["artifacts"]
        .as_array()
        .context("NRS baseline artifacts")?
    {
        let relative = artifact["path"]
            .as_str()
            .context("NRS baseline artifact path")?;
        if relative.contains("..") || Path::new(relative).is_absolute() {
            bail!("nonportable NRS baseline artifact path");
        }
        let path = package.join(relative);
        if sha256(&path)? != artifact["sha256"] {
            bail!("NRS baseline artifact hash mismatch: {relative}");
        }
    }
    let seed_record = read_json(&package.join("seed/seed_record.json"))?;
    let standard_profile = read_json(&package.join("seed/standard_profile.json"))?;
    let discovery_refinement = standard_profile
        .pointer("/search/discovery_refinement")
        .and_then(Value::as_str)
        .unwrap_or("nrs-v0-1");
    if !matches!(discovery_refinement, "nrs-v0-1" | "nrs-v0-2" | "nrs-v0-3") {
        bail!("unsupported NRS discovery refinement: {discovery_refinement}");
    }
    if manifest["input_manifest_canonical_sha256"] != seed_record["input_manifest_canonical_sha256"]
        || manifest["seed_u64_little_endian"] != seed_record["seed_u64_little_endian"]
        || manifest["engine_seed_i32"] != seed_record["engine_seed_i32"]
    {
        bail!("NRS baseline seed link mismatch");
    }
    let context = read_json(context_path)?;
    let tree = read_json(&package.join("baseline-tree.json"))?;
    let assignments = read_json(&package.join("baseline_assignments.json"))?;
    if tree["schema_version"] != "nrs-baseline-tree-v0.1-v1"
        || assignments["schema_version"] != "nrs-baseline-assignments-v0.1-v1"
        || assignments["label_base"] != 1
    {
        bail!("unknown NRS baseline tree or assignment schema");
    }
    let districts = tree["districts"].as_u64().context("NRS districts")? as usize;
    let unit_ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("NRS context unit ids")?;
    let assignment = tree["assignment"]
        .as_array()
        .context("NRS tree assignment")?
        .iter()
        .map(|value| value.as_i64().context("NRS district label"))
        .collect::<Result<Vec<_>>>()?;
    if assignment.len() != unit_ids.len()
        || assignment
            .iter()
            .any(|label| *label < 0 || *label >= districts as i64)
        || tree["unit_count"] != unit_ids.len()
    {
        bail!("NRS assignment universe mismatch");
    }
    let assignment_map = assignments["assignments"]
        .as_object()
        .context("NRS assignment object")?;
    if assignment_map.len() != unit_ids.len() {
        bail!("NRS assignment object coverage mismatch");
    }
    for (unit, district) in unit_ids.iter().zip(&assignment) {
        let geoid = unit.as_str().context("NRS GEOID")?;
        if assignment_map.get(geoid).and_then(Value::as_i64) != Some(district + 1) {
            bail!("NRS assignment representation mismatch for {geoid}");
        }
    }
    for district in 0..districts {
        if !connected(&context, &assignment, district as i64)? {
            bail!("NRS disconnected district {district}");
        }
    }
    let nodes = tree["nodes"].as_array().context("NRS nodes")?;
    let leaves = tree["leaves"].as_array().context("NRS leaves")?;
    if nodes.len() + 1 != districts || leaves.len() != districts {
        bail!("NRS recursive tree size mismatch");
    }
    let node_seats: BTreeMap<String, usize> = nodes
        .iter()
        .map(|node| {
            Ok((
                node["path"].as_str().context("NRS node path")?.to_owned(),
                node["seats"].as_u64().context("NRS node seats")? as usize,
            ))
        })
        .collect::<Result<_>>()?;
    let leaf_paths: BTreeMap<String, usize> = leaves
        .iter()
        .map(|leaf| {
            Ok((
                leaf["path"].as_str().context("NRS leaf path")?.to_owned(),
                leaf["district_zero_based"]
                    .as_u64()
                    .context("NRS leaf district")? as usize,
            ))
        })
        .collect::<Result<_>>()?;
    let root_valid = if districts == 1 {
        leaf_paths.get("") == Some(&0)
    } else {
        node_seats.get("") == Some(&districts)
    };
    if !root_valid
        || leaf_paths.values().copied().collect::<BTreeSet<_>>()
            != (0..districts).collect::<BTreeSet<_>>()
    {
        bail!("NRS recursive root or leaf labels mismatch");
    }
    let engine_seed = seed_record["engine_seed_i32"]
        .as_u64()
        .context("NRS seed")?;
    for node in nodes {
        let path = node["path"].as_str().context("NRS node path")?;
        let seats = node["seats"].as_u64().context("NRS node seats")? as usize;
        let child_seats = node["child_seats"]
            .as_array()
            .context("NRS child seats")?
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .context("NRS child seat")
                    .map(|value| value as usize)
            })
            .collect::<Result<Vec<_>>>()?;
        if child_seats.len() != 2 || child_seats.iter().sum::<usize>() != seats || {
            let mut sorted = child_seats.clone();
            sorted.sort_unstable();
            sorted != vec![seats / 2, seats - seats / 2]
        } {
            bail!("NRS child seat schedule mismatch at {path}");
        }
        for label in 0..=1 {
            let child = format!("{path}{label}");
            if node_seats
                .get(&child)
                .copied()
                .or_else(|| leaf_paths.contains_key(&child).then_some(1))
                != Some(child_seats[label])
            {
                bail!("NRS recursive child mismatch at {child}");
            }
        }
        if node["engine_seed_i32"] != engine_seed {
            bail!("NRS node seed mismatch at {path}");
        }
        let discovery_path = package.join(
            node["discovery_path"]
                .as_str()
                .context("NRS discovery path")?,
        );
        if sha256(&discovery_path)? != node["discovery_sha256"] {
            bail!("NRS discovery hash mismatch at {path}");
        }
        let discovery = read_json(&discovery_path)?;
        if discovery["discovery_id"] != node["discovery_id"]
            || discovery_seed(&discovery)? != engine_seed
            || !discovery["method"].as_str().is_some_and(|method| {
                method.contains("niter=100")
                    && method.contains("partition-type=recursive")
                    && method.contains(
                        "candidate-initialization=minimum-geoid-rooted-sorted-dfs-tree-edge-cut",
                    )
                    && method.contains(&format!(
                        "refinement={}",
                        discovery_refinement.replace('-', "")
                    ))
            })
            || objective(&discovery)? != &node["objective"]
        {
            bail!("NRS discovery record mismatch at {path}");
        }
        let lower_bound = node["population_floor"]["lower_bound"]
            .as_u64()
            .context("NRS population lower bound")?;
        let achieved = field_u64(
            &discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )?;
        if node["population_floor"]["attained"] != (achieved == lower_bound) {
            bail!("NRS population-floor classification mismatch at {path}");
        }
        let recorded_tolerance_bound = node["generation_tolerance_scaled_bound"]
            .as_u64()
            .context("NRS generation tolerance")?;
        let parent_population = node["parent_population"]
            .as_i64()
            .context("NRS node parent population")?;
        let smaller_child_seats = child_seats
            .iter()
            .copied()
            .min()
            .context("NRS node child seats")?;
        let tolerance_bound =
            nrs_generation_tolerance_scaled_bound(parent_population, smaller_child_seats);
        if recorded_tolerance_bound != tolerance_bound {
            bail!("NRS generation tolerance bound mismatch at {path}");
        }
        if achieved > tolerance_bound {
            bail!("NRS population tolerance exceeded at {path}");
        }
        let subtree_districts: BTreeSet<usize> = leaf_paths
            .iter()
            .filter_map(|(leaf_path, district)| leaf_path.starts_with(path).then_some(*district))
            .collect();
        let first_unit = assignment
            .iter()
            .enumerate()
            .find(|(_, district)| subtree_districts.contains(&(**district as usize)))
            .map(|(unit, _)| unit)
            .context("NRS empty node subtree")?;
        let first_district = assignment[first_unit] as usize;
        let first_leaf_path = leaf_paths
            .iter()
            .find_map(|(leaf_path, district)| (*district == first_district).then_some(leaf_path))
            .context("NRS missing first leaf")?;
        if !first_leaf_path.starts_with(&format!("{path}0"))
            || node["minimum_geoid"] != unit_ids[first_unit]
        {
            bail!("NRS minimum-GEOID child orientation mismatch at {path}");
        }
    }
    let population_total: i64 = context["populations"]
        .as_array()
        .context("NRS populations")?
        .iter()
        .filter_map(Value::as_i64)
        .sum();
    if tree["population_total"] != population_total {
        bail!("NRS population total mismatch");
    }
    println!("NRS State baseline verification: PASS");
    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn verify_nrs_failure(package: &Path, context_path: &Path) -> Result<()> {
    verify_nrs_seed_package(&package.join("seed"), context_path)?;
    let failure = read_json(&package.join("nrs-failure.json"))?;
    if failure["schema_version"] != "nrs-baseline-failure-v2"
        || failure["status"] != "candidate-failed-population-tolerance"
    {
        bail!("NRS failure posture mismatch");
    }
    let executable_hash = failure["bisect_executable_sha256"]
        .as_str()
        .context("NRS failure executable hash")?;
    if !is_sha256_hex(executable_hash) {
        bail!("invalid NRS failure executable hash");
    }
    let achieved = failure["achieved_scaled_deviation"]
        .as_u64()
        .context("NRS failure achieved deviation")?;
    let allowed = failure["allowed_scaled_deviation"]
        .as_u64()
        .context("NRS failure allowed deviation")?;
    if achieved <= allowed {
        bail!("NRS failure witness does not exceed tolerance");
    }
    let relative = failure["discovery_path"]
        .as_str()
        .context("NRS failure discovery path")?;
    if relative.contains("..") || Path::new(relative).is_absolute() {
        bail!("nonportable NRS failure discovery path");
    }
    if failure["discovery_sha256"] != sha256(&package.join(relative))? {
        bail!("NRS failure discovery hash mismatch");
    }
    Ok(())
}

fn write_nrs_batch_ledger(
    out: &Path,
    inventory: &Path,
    year: u16,
    generated_at: &str,
    standard_profile_sha256: &str,
    legal_profile_sha256: &str,
    bisect_executable_sha256: &str,
    rows: &BTreeMap<String, Value>,
) -> Result<()> {
    let results = rows.values().cloned().collect::<Vec<_>>();
    let verified = results
        .iter()
        .filter(|row| row["status"] == "verified")
        .count();
    let failed = results
        .iter()
        .filter(|row| row["status"] == "failed")
        .count();
    write_json(
        &out.join("ledger.json"),
        &json!({
            "schema_version":"nrs-national-batch-ledger-v3",
            "census_year":year,
            "generated_at":generated_at,
            "inventory_sha256":sha256(inventory)?,
            "standard_profile_canonical_sha256":standard_profile_sha256,
            "legal_profile_canonical_sha256":legal_profile_sha256,
            "bisect_executable_sha256":bisect_executable_sha256,
            "verified_count":verified,"failed_count":failed,"results":results,
            "claim_boundary":"Resumable NRS v0.1 reference-baseline execution ledger. State package verification is independent; complete national coverage is claimed only after verify-nrs-batch --require-complete passes."
        }),
        true,
    )
}

fn nrs_package_matches_execution_identity(
    package: &Path,
    standard_profile_sha256: &str,
    legal_profile_sha256: &str,
    bisect_executable_sha256: &str,
    failure: bool,
) -> Result<bool> {
    let packaged_profile = package.join("seed/standard_profile.json");
    if !packaged_profile.is_file()
        || canonical_sha256(&read_json(&packaged_profile)?)? != standard_profile_sha256
    {
        return Ok(false);
    }
    let packaged_legal_profile = package.join("seed/legal_profile.json");
    if !packaged_legal_profile.is_file()
        || canonical_sha256(&read_json(&packaged_legal_profile)?)? != legal_profile_sha256
    {
        return Ok(false);
    }
    let identity_record = if failure {
        package.join("nrs-failure.json")
    } else {
        package.join("baseline_manifest.json")
    };
    if !identity_record.is_file() {
        return Ok(false);
    }
    Ok(read_json(&identity_record)?["bisect_executable_sha256"] == bisect_executable_sha256)
}

fn nrs_seed_matches_profiles(
    seed: &Path,
    standard_profile_sha256: &str,
    legal_profile_sha256: &str,
) -> Result<bool> {
    let standard = seed.join("standard_profile.json");
    let legal = seed.join("legal_profile.json");
    Ok(standard.is_file()
        && legal.is_file()
        && canonical_sha256(&read_json(&standard)?)? == standard_profile_sha256
        && canonical_sha256(&read_json(&legal)?)? == legal_profile_sha256)
}

fn nrs_batch(
    year: u16,
    bisect: &Path,
    inventory_path: &Path,
    standard_profile_path: &Path,
    legal_profile_path: &Path,
    out: &Path,
    generated_at: &str,
    limit: Option<usize>,
    selected_states: &[String],
    retry_failed: bool,
) -> Result<()> {
    let inventory = read_json(inventory_path)?;
    if inventory["census_year"].as_u64() != Some(u64::from(year)) {
        bail!("NRS inventory census year does not match --year {year}");
    }
    let standard_profile = read_json(standard_profile_path)?;
    let legal_profile = read_json(legal_profile_path)?;
    validate_nrs_profile_cycle(year, &standard_profile, &legal_profile)?;
    let standard_profile_sha256 = canonical_sha256(&standard_profile)?;
    let legal_profile_sha256 = canonical_sha256(&legal_profile)?;
    let bisect_executable_sha256 = sha256(bisect)?;
    let selected: BTreeSet<String> = selected_states
        .iter()
        .map(|state| state.to_uppercase())
        .collect();
    let ledger_path = out.join("ledger.json");
    let mut rows: BTreeMap<String, Value> = if ledger_path.is_file() {
        let ledger = read_json(&ledger_path)?;
        if ledger["schema_version"] != "nrs-national-batch-ledger-v3"
            || ledger["census_year"].as_u64() != Some(u64::from(year))
            || ledger["inventory_sha256"] != sha256(inventory_path)?
            || ledger["generated_at"] != generated_at
            || ledger["standard_profile_canonical_sha256"] != standard_profile_sha256
            || ledger["legal_profile_canonical_sha256"] != legal_profile_sha256
            || ledger["bisect_executable_sha256"] != bisect_executable_sha256
        {
            bail!("NRS national ledger identity mismatch");
        }
        ledger["results"]
            .as_array()
            .context("NRS batch results")?
            .iter()
            .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
            .collect()
    } else {
        BTreeMap::new()
    };
    fs::create_dir_all(out.join("states"))?;
    let mut candidates = inventory["states"]
        .as_array()
        .context("NRS inventory states")?
        .iter()
        .filter(|row| {
            selected.is_empty()
                || row["state"]
                    .as_str()
                    .is_some_and(|state| selected.contains(state))
        })
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort_by_key(|row| {
        (
            row["block_count"].as_u64().unwrap_or(u64::MAX),
            row["state"].as_str().unwrap_or("").to_owned(),
        )
    });
    let found: BTreeSet<String> = candidates
        .iter()
        .filter_map(|row| row["state"].as_str().map(str::to_owned))
        .collect();
    if !selected.is_subset(&found) {
        bail!("unknown NRS --states selection");
    }
    let mut pending = Vec::new();
    for row in candidates {
        let state = row["state"].as_str().context("NRS state")?.to_owned();
        let lower = state.to_lowercase();
        let context_path =
            PathBuf::from(format!("data/{year}/certified/{lower}_blocks_{year}.rctx"));
        let state_root = out.join("states").join(&lower);
        let package = state_root.join("package");
        let staging = state_root.join("package.in-progress");
        if package.join("baseline_manifest.json").is_file()
            && verify_nrs_state(&package, &context_path).is_ok()
            && nrs_package_matches_execution_identity(
                &package,
                &standard_profile_sha256,
                &legal_profile_sha256,
                &bisect_executable_sha256,
                false,
            )?
        {
            let manifest_sha256 = sha256(&package.join("baseline_manifest.json"))?;
            let recovered = json!({
                "state":state,"districts":row["districts"],"block_count":row["block_count"],
                "status":"verified","recovered":true,
                "package_path":release_relative(out,&package)?,
                "manifest_sha256":manifest_sha256
            });
            let retained = rows.get(&state).filter(|prior| {
                prior["status"] == "verified"
                    && prior["manifest_sha256"] == recovered["manifest_sha256"]
                    && prior["package_path"] == recovered["package_path"]
            });
            rows.insert(state.clone(), retained.cloned().unwrap_or(recovered));
            continue;
        }
        if rows
            .get(&state)
            .is_some_and(|prior| prior["status"] == "failed" && prior["failure_path"].is_string())
            && !retry_failed
        {
            continue;
        }
        let failure = package.join("nrs-failure.json");
        if failure.is_file()
            && !retry_failed
            && verify_nrs_failure(&package, &context_path).is_ok()
            && nrs_package_matches_execution_identity(
                &package,
                &standard_profile_sha256,
                &legal_profile_sha256,
                &bisect_executable_sha256,
                true,
            )?
        {
            rows.insert(
                state.clone(),
                json!({
                    "state":state,"districts":row["districts"],"block_count":row["block_count"],
                    "status":"failed","recovered":true,"error":"Recovered retained NRS failure witness.",
                    "failure_path":release_relative(out,&failure)?,"failure_sha256":sha256(&failure)?
                }),
            );
            continue;
        }
        pending.push((row, context_path, state_root, package, staging));
    }
    if let Some(limit) = limit {
        pending.truncate(limit);
    }
    write_nrs_batch_ledger(
        out,
        inventory_path,
        year,
        generated_at,
        &standard_profile_sha256,
        &legal_profile_sha256,
        &bisect_executable_sha256,
        &rows,
    )?;
    for (row, context_path, state_root, package, staging) in pending {
        let state = row["state"].as_str().context("NRS state")?.to_owned();
        let districts = row["districts"].as_u64().context("NRS districts")? as usize;
        let seed = state_root.join("seed");
        if retry_failed {
            remove_path(&package)?;
            remove_path(&staging)?;
            if seed.exists()
                && (verify_nrs_seed_package(&seed, &context_path).is_err()
                    || !nrs_seed_matches_profiles(
                        &seed,
                        &standard_profile_sha256,
                        &legal_profile_sha256,
                    )?)
            {
                remove_path(&seed)?;
            }
        }
        let started = Instant::now();
        let mut attempts = 0_u8;
        let mut recovered_from_staging = false;
        let result = loop {
            attempts += 1;
            let attempt = (|| -> Result<()> {
                if staging.join("baseline_manifest.json").is_file()
                    && verify_nrs_state(&staging, &context_path).is_ok()
                    && nrs_package_matches_execution_identity(
                        &staging,
                        &standard_profile_sha256,
                        &legal_profile_sha256,
                        &bisect_executable_sha256,
                        false,
                    )?
                {
                    remove_path(&package)?;
                    fs::rename(&staging, &package).with_context(|| {
                        format!(
                            "promote recovered NRS package {} to {}",
                            staging.display(),
                            package.display()
                        )
                    })?;
                    recovered_from_staging = true;
                    return Ok(());
                }
                if staging.join("nrs-failure.json").is_file()
                    && verify_nrs_failure(&staging, &context_path).is_ok()
                    && nrs_package_matches_execution_identity(
                        &staging,
                        &standard_profile_sha256,
                        &legal_profile_sha256,
                        &bisect_executable_sha256,
                        true,
                    )?
                {
                    remove_path(&package)?;
                    fs::rename(&staging, &package).with_context(|| {
                        format!(
                            "promote recovered NRS failure {} to {}",
                            staging.display(),
                            package.display()
                        )
                    })?;
                    recovered_from_staging = true;
                    bail!("Recovered retained NRS failure witness from interrupted build");
                }
                remove_path(&staging)?;
                // A canonical directory without a verified manifest or retained
                // algorithm witness is a legacy interrupted build. It is safe to
                // replace because the recovery checks above rejected it.
                remove_path(&package)?;
                if seed.exists()
                    && (verify_nrs_seed_package(&seed, &context_path).is_err()
                        || !nrs_seed_matches_profiles(
                            &seed,
                            &standard_profile_sha256,
                            &legal_profile_sha256,
                        )?)
                {
                    remove_path(&seed)?;
                }
                if !seed.join("manifest.json").is_file() {
                    build_nrs_seed_package(
                        &context_path,
                        districts,
                        standard_profile_path,
                        legal_profile_path,
                        &seed,
                        generated_at,
                    )?;
                } else {
                    verify_nrs_seed_package(&seed, &context_path)?;
                }
                build_nrs_state(
                    bisect,
                    &context_path,
                    districts,
                    &seed,
                    &staging,
                    generated_at,
                )?;
                verify_nrs_state(&staging, &context_path)?;
                fs::rename(&staging, &package).with_context(|| {
                    format!(
                        "promote verified NRS package {} to {}",
                        staging.display(),
                        package.display()
                    )
                })?;
                Ok(())
            })();
            match attempt {
                Ok(()) => break Ok(()),
                Err(error) if staging.join("nrs-failure.json").is_file() => {
                    remove_path(&package)?;
                    fs::rename(&staging, &package).with_context(|| {
                        format!(
                            "promote retained NRS failure {} to {}",
                            staging.display(),
                            package.display()
                        )
                    })?;
                    break Err(error);
                }
                Err(error) if attempts < 3 && is_transient_file_lock(&error) => {
                    thread::sleep(Duration::from_millis(250 * u64::from(attempts)));
                }
                Err(error) => break Err(error),
            }
        };
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let result_row = match result {
            Ok(()) => json!({
                "state":state,"districts":districts,"block_count":row["block_count"],
                "status":"verified","recovered":recovered_from_staging,"attempts":attempts,
                "elapsed_seconds":elapsed_seconds,
                "package_path":release_relative(out,&package)?,
                "manifest_sha256":sha256(&package.join("baseline_manifest.json"))?
            }),
            Err(error) => {
                let failure = package.join("nrs-failure.json");
                json!({
                    "state":state,"districts":districts,"block_count":row["block_count"],
                    "status":"failed","attempts":attempts,"elapsed_seconds":elapsed_seconds,
                    "recovered":recovered_from_staging,
                    "error":format!("{error:#}"),
                    "failure_path":failure.is_file().then(||release_relative(out,&failure)).transpose()?,
                    "failure_sha256":failure.is_file().then(||sha256(&failure)).transpose()?
                })
            }
        };
        println!("{state}: {} ({elapsed_seconds:.3}s)", result_row["status"]);
        rows.insert(state, result_row);
        write_nrs_batch_ledger(
            out,
            inventory_path,
            year,
            generated_at,
            &standard_profile_sha256,
            &legal_profile_sha256,
            &bisect_executable_sha256,
            &rows,
        )?;
    }
    verify_nrs_batch(
        year,
        inventory_path,
        standard_profile_path,
        legal_profile_path,
        out,
        false,
    )
}

fn verify_nrs_batch(
    year: u16,
    inventory_path: &Path,
    standard_profile_path: &Path,
    legal_profile_path: &Path,
    out: &Path,
    require_complete: bool,
) -> Result<()> {
    let inventory = read_json(inventory_path)?;
    let ledger = read_json(&out.join("ledger.json"))?;
    if inventory["census_year"].as_u64() != Some(u64::from(year)) {
        bail!("NRS inventory census year does not match --year {year}");
    }
    let standard_profile = read_json(standard_profile_path)?;
    let legal_profile = read_json(legal_profile_path)?;
    validate_nrs_profile_cycle(year, &standard_profile, &legal_profile)?;
    let standard_profile_sha256 = canonical_sha256(&standard_profile)?;
    let legal_profile_sha256 = canonical_sha256(&legal_profile)?;
    if ledger["schema_version"] != "nrs-national-batch-ledger-v3"
        || ledger["census_year"].as_u64() != Some(u64::from(year))
        || ledger["inventory_sha256"] != sha256(inventory_path)?
        || ledger["standard_profile_canonical_sha256"] != standard_profile_sha256
        || ledger["legal_profile_canonical_sha256"] != legal_profile_sha256
    {
        bail!("NRS national batch ledger identity mismatch");
    }
    let inventory_rows: BTreeMap<String, Value> = inventory["states"]
        .as_array()
        .context("NRS inventory states")?
        .iter()
        .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
        .collect();
    let mut verified = 0_usize;
    let mut failed = 0_usize;
    let mut seen = BTreeSet::new();
    for row in ledger["results"].as_array().context("NRS batch results")? {
        let state = row["state"].as_str().context("NRS batch state")?;
        if !seen.insert(state.to_owned()) {
            bail!("duplicate NRS batch state {state}");
        }
        let source = inventory_rows
            .get(state)
            .context("unknown NRS batch state")?;
        if row["districts"] != source["districts"] || row["block_count"] != source["block_count"] {
            bail!("NRS inventory mismatch for {state}");
        }
        if row["status"] == "verified" {
            let relative = row["package_path"].as_str().context("NRS package path")?;
            if relative.contains("..") || Path::new(relative).is_absolute() {
                bail!("nonportable NRS batch package path for {state}");
            }
            let package = out.join(relative);
            let context_path = PathBuf::from(format!(
                "data/{year}/certified/{}_blocks_{year}.rctx",
                state.to_lowercase()
            ));
            verify_nrs_state(&package, &context_path)?;
            if !nrs_package_matches_execution_identity(
                &package,
                ledger["standard_profile_canonical_sha256"]
                    .as_str()
                    .context("NRS ledger standard profile hash")?,
                ledger["legal_profile_canonical_sha256"]
                    .as_str()
                    .context("NRS ledger legal profile hash")?,
                ledger["bisect_executable_sha256"]
                    .as_str()
                    .context("NRS ledger executable hash")?,
                false,
            )? {
                bail!("NRS execution identity mismatch for {state}");
            }
            if row["manifest_sha256"] != sha256(&package.join("baseline_manifest.json"))? {
                bail!("NRS package manifest hash mismatch for {state}");
            }
            verified += 1;
        } else if row["status"] == "failed" {
            failed += 1;
            if let Some(relative) = row["failure_path"].as_str() {
                if relative.contains("..") || Path::new(relative).is_absolute() {
                    bail!("nonportable NRS failure path for {state}");
                }
                if row["failure_sha256"] != sha256(&out.join(relative))? {
                    bail!("NRS failure witness hash mismatch for {state}");
                }
                let package = out
                    .join(relative)
                    .parent()
                    .context("NRS failure parent")?
                    .to_path_buf();
                let context_path = PathBuf::from(format!(
                    "data/{year}/certified/{}_blocks_{year}.rctx",
                    state.to_lowercase()
                ));
                verify_nrs_failure(&package, &context_path)?;
                if !nrs_package_matches_execution_identity(
                    &package,
                    ledger["standard_profile_canonical_sha256"]
                        .as_str()
                        .context("NRS ledger standard profile hash")?,
                    ledger["legal_profile_canonical_sha256"]
                        .as_str()
                        .context("NRS ledger legal profile hash")?,
                    ledger["bisect_executable_sha256"]
                        .as_str()
                        .context("NRS ledger executable hash")?,
                    true,
                )? {
                    bail!("NRS failure execution identity mismatch for {state}");
                }
            }
        } else {
            bail!("unknown NRS batch status for {state}");
        }
    }
    if ledger["verified_count"] != verified || ledger["failed_count"] != failed {
        bail!("NRS batch summary count mismatch");
    }
    if require_complete && (verified != inventory_rows.len() || failed != 0) {
        bail!(
            "NRS national batch incomplete: {verified}/{} verified, {failed} failed",
            inventory_rows.len()
        );
    }
    println!("NRS national batch verification: PASS ({verified} verified, {failed} failed)");
    Ok(())
}

fn summarize_nrs_batch(
    year: u16,
    inventory_path: &Path,
    standard_profile_path: &Path,
    legal_profile_path: &Path,
    out: &Path,
    report_dir: &Path,
) -> Result<()> {
    verify_nrs_batch(
        year,
        inventory_path,
        standard_profile_path,
        legal_profile_path,
        out,
        true,
    )?;
    let inventory = read_json(inventory_path)?;
    let standard_profile = read_json(standard_profile_path)?;
    let (profile_label, summary_schema, proof_schema, package_schema) =
        match standard_profile["schema_version"].as_str() {
            Some("nrs-standard-profile-v0.1-v1") => (
                "NRS v0.1",
                "nrs-national-summary-v0.1-v1",
                "nrs-national-proof-coverage-v0.1-v1",
                "nrs-national-summary-package-v0.1-v1",
            ),
            Some("nrs-standard-profile-v0.2-v1") => (
                "NRS v0.2",
                "nrs-national-summary-v0.2-v1",
                "nrs-national-proof-coverage-v0.2-v1",
                "nrs-national-summary-package-v0.2-v1",
            ),
            Some("nrs-standard-profile-v0.3-v1") => (
                "NRS v0.3",
                "nrs-national-summary-v0.3-v1",
                "nrs-national-proof-coverage-v0.3-v1",
                "nrs-national-summary-package-v0.3-v1",
            ),
            _ => bail!("unknown NRS standard profile schema for publication"),
        };
    let standard_profile_id = standard_profile["profile_id"]
        .as_str()
        .context("NRS standard profile id")?;
    let inventory_rows = inventory["states"]
        .as_array()
        .context("NRS inventory states")?;
    let expected_units: u64 = inventory_rows
        .iter()
        .map(|row| row["block_count"].as_u64().context("inventory block count"))
        .sum::<Result<_>>()?;
    let expected_districts: u64 = inventory_rows
        .iter()
        .map(|row| row["districts"].as_u64().context("inventory districts"))
        .sum::<Result<_>>()?;
    let expected_nodes: u64 = inventory_rows
        .iter()
        .map(|row| {
            row["districts"]
                .as_u64()
                .context("inventory districts")
                .map(|d| d - 1)
        })
        .sum::<Result<_>>()?;
    let single_district_states = inventory_rows
        .iter()
        .filter(|row| row["districts"].as_u64() == Some(1))
        .count();
    let ledger_path = out.join("ledger.json");
    let ledger = read_json(&ledger_path)?;
    let mut rows = Vec::new();
    let mut total_units = 0_u64;
    let mut total_population = 0_i64;
    let mut total_districts = 0_u64;
    let mut total_nodes = 0_u64;
    let mut arithmetic_floor_nodes = 0_u64;
    let mut measured_elapsed_seconds = 0_f64;
    let mut missing_elapsed_states = Vec::new();
    for row in ledger["results"].as_array().context("NRS batch results")? {
        let state = row["state"].as_str().context("NRS batch state")?;
        let package = out.join(
            row["package_path"]
                .as_str()
                .context("NRS batch package path")?,
        );
        let tree_path = package.join("baseline-tree.json");
        let tree = read_json(&tree_path)?;
        let nodes = tree["nodes"].as_array().context("NRS tree nodes")?;
        let node_count = nodes.len() as u64;
        let floors = nodes
            .iter()
            .filter(|node| node["population_floor"]["attained"] == true)
            .count() as u64;
        let districts = tree["districts"].as_u64().context("NRS districts")?;
        let units = tree["unit_count"].as_u64().context("NRS unit count")?;
        let population = tree["population_total"]
            .as_i64()
            .context("NRS population total")?;
        let max_tolerance_fraction = nodes
            .iter()
            .map(|node| {
                let achieved = node["objective"]["max_population_deviation_scaled"]
                    .as_u64()
                    .unwrap_or(u64::MAX);
                let allowed = node["generation_tolerance_scaled_bound"]
                    .as_u64()
                    .unwrap_or(0);
                if allowed == 0 {
                    0.0
                } else {
                    achieved as f64 / allowed as f64
                }
            })
            .fold(0.0_f64, f64::max);
        if let Some(elapsed) = row["elapsed_seconds"].as_f64() {
            measured_elapsed_seconds += elapsed;
        } else {
            missing_elapsed_states.push(state.to_owned());
        }
        total_units += units;
        total_population += population;
        total_districts += districts;
        total_nodes += node_count;
        arithmetic_floor_nodes += floors;
        rows.push(json!({
            "state":state,"districts":districts,"unit_count":units,
            "population_total":population,"recursive_nodes":node_count,
            "arithmetic_floor_nodes":floors,
            "max_generation_tolerance_fraction":max_tolerance_fraction,
            "elapsed_seconds":row["elapsed_seconds"],
            "package_manifest_sha256":row["manifest_sha256"],
            "baseline_tree_sha256":sha256(&tree_path)?,
            "assignment_coverage":"verified","contiguity":"independently-verified",
            "population_tolerance":"verified-all-nodes",
            "boundary_proof":"not-run","canonical_proof":"blocked-by-boundary"
        }));
    }
    rows.sort_by_key(|row| row["state"].as_str().unwrap_or("").to_owned());
    if total_units != expected_units
        || total_districts != expected_districts
        || total_nodes != expected_nodes
        || rows.len() != inventory_rows.len()
    {
        bail!("NRS national publication aggregate mismatch");
    }
    fs::create_dir_all(report_dir)?;
    let state_count = rows.len();
    let claim = format!("{profile_label} generated and independently verified complete {year} block assignments for all {state_count} States and all {total_districts} districts. All {total_nodes} recursive nodes satisfy the frozen population tolerance. Arithmetic-floor, weighted-boundary, and canonical proof coverage are reported separately; no legal validity, VRA, partisan-fairness, or official-adoption claim is made.");
    let summary_path = report_dir.join("national-summary.json");
    write_json(
        &summary_path,
        &json!({
            "schema_version":summary_schema,
            "status":"verified-national-reference-baseline",
            "standard_profile_id":standard_profile_id,
            "census_year":year,
            "state_count":state_count,"district_count":total_districts,
            "recursive_node_count":total_nodes,"unit_count":total_units,
            "population_total":total_population,"omitted_units":0,"duplicate_units":0,
            "disconnected_districts":0,"population_tolerance_failures":0,
            "measured_elapsed_seconds":measured_elapsed_seconds,
            "missing_elapsed_states":missing_elapsed_states,
            "ledger_sha256":sha256(&ledger_path)?,
            "standard_profile_canonical_sha256":ledger["standard_profile_canonical_sha256"],
            "legal_profile_canonical_sha256":ledger["legal_profile_canonical_sha256"],
            "bisect_executable_sha256":ledger["bisect_executable_sha256"],
            "states":rows,"claim_boundary":claim
        }),
        true,
    )?;
    let proof_path = report_dir.join("proof-coverage.json");
    write_json(
        &proof_path,
        &json!({
            "schema_version":proof_schema,
            "status":"classified","recursive_node_count":total_nodes,
            "standard_profile_id":standard_profile_id,
            "population_tolerance":{"verified_nodes":total_nodes,"failed_nodes":0,"coverage_rate":1.0},
            "population_exact":{"proved_nodes":arithmetic_floor_nodes,"unproved_nodes":total_nodes-arithmetic_floor_nodes,"coverage_rate":arithmetic_floor_nodes as f64/total_nodes as f64,"proof_kind":"ratio-arithmetic-floor-when-attained"},
            "boundary":{"proved_nodes":0,"unproved_nodes":total_nodes,"coverage_rate":0.0,"status":"not-run"},
            "canonical":{"proved_nodes":0,"unproved_nodes":total_nodes,"coverage_rate":0.0,"status":"blocked-by-boundary"},
            "single_district_states":{"count":single_district_states,"objective_proofs":"not-applicable"},
            "claim_boundary":claim
        }),
        true,
    )?;
    let manifest_path = report_dir.join("manifest.json");
    write_json(
        &manifest_path,
        &json!({
            "schema_version":package_schema,
            "status":"verified-national-reference-baseline",
            "standard_profile_id":standard_profile_id,
            "source_ledger_sha256":sha256(&ledger_path)?,
            "files":[
                {"path":"national-summary.json","sha256":sha256(&summary_path)?},
                {"path":"proof-coverage.json","sha256":sha256(&proof_path)?}
            ],
            "claim_boundary":claim
        }),
        true,
    )?;
    println!(
        "NRS national summary: VERIFIED ({} States, {} districts, {} nodes)",
        state_count, total_districts, total_nodes
    );
    Ok(())
}

fn connected(context: &Value, assignment: &[i64], label: i64) -> Result<bool> {
    let members: Vec<usize> = assignment
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| (v == label).then_some(i))
        .collect();
    let Some(&first) = members.first() else {
        return Ok(false);
    };
    let allowed: BTreeSet<_> = members.iter().copied().collect();
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    let mut seen = BTreeSet::from([first]);
    let mut queue = VecDeque::from([first]);
    while let Some(unit) = queue.pop_front() {
        for edge in adjacency[unit].as_array().context("edge list")? {
            let to = edge["to"].as_u64().context("edge.to")? as usize;
            if allowed.contains(&to) && seen.insert(to) {
                queue.push_back(to);
            }
        }
    }
    Ok(seen == allowed)
}

fn build(
    bisect: &Path,
    context_path: &Path,
    out: &Path,
    districts: usize,
    root_seed: u64,
    child_seeds: [u64; 2],
    max_seed: u64,
) -> Result<()> {
    let context = read_json(context_path)?;
    fs::create_dir_all(out)?;
    let unit_count = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?
        .len();
    let mut state = BuildState {
        bisect,
        out,
        original: &context,
        child_seeds,
        max_seed,
        assignment: vec![-1; unit_count],
        nodes: Vec::new(),
        leaves: Vec::new(),
    };
    state.visit(Visit {
        context: context.clone(),
        context_path: context_path.to_path_buf(),
        global: (0..unit_count).collect(),
        seats: districts,
        path: String::new(),
        offset: 0,
        seed: root_seed,
    })?;
    if state.assignment.iter().any(|&v| v < 0) {
        bail!("operational tree left units unassigned");
    }
    for district in 0..districts {
        if !connected(&context, &state.assignment, district as i64)? {
            bail!("district {district} is disconnected");
        }
    }
    let population_total: i64 = context["populations"]
        .as_array()
        .context("populations")?
        .iter()
        .map(|v| v.as_i64().unwrap_or(0))
        .sum();
    let claim="Complete connected wall-to-wall recursive assignment. Node objectives are discovery incumbents unless separately proved.";
    let tree = json!({"schema_version":"certified-operational-recursive-tree-v1","status":"operational-unproved-objectives","context_sha256":sha256(context_path)?,"context_hash":context["context_hash"],"districts":districts,"nodes":state.nodes,"leaves":state.leaves,"assignment":state.assignment,"unit_count":unit_count,"population_total":population_total,"claim_boundary":claim});
    let tree_path = out.join("operational-tree.json");
    write_json(&tree_path, &tree, true)?;
    let source = out.join(BUILDER_SNAPSHOT);
    fs::write(&source, include_bytes!("main.rs"))?;
    let state_name = context
        .pointer("/units/state")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_lowercase();
    let manifest = json!({"schema_version":"certified-operational-recursive-tree-package-v1","package_id":format!("operational-tree-{state_name}-2020"),"status":"operational-unproved-objectives","files":[{"path":"operational-tree.json","sha256":sha256(&tree_path)?}],"builder_path":BUILDER_SNAPSHOT,"builder_sha256":sha256(&source)?,"seed_frontier_max":max_seed,"claim_boundary":claim});
    write_json(&out.join("manifest.json"), &manifest, true)?;
    for entry in fs::read_dir(out)? {
        let path = entry?.path();
        let keep = path.file_name().and_then(|v| v.to_str()).is_some_and(|v| {
            v == "manifest.json" || v == "operational-tree.json" || v == BUILDER_SNAPSHOT
        });
        if !keep {
            remove_path(&path)?;
        }
    }
    println!("Operational recursive tree: VERIFIED");
    Ok(())
}

fn batch(bisect: &Path, limit: Option<usize>, retry_failed: bool, max_seed: u64) -> Result<()> {
    let root = std::env::current_dir()?;
    let inventory_path = root.join("docs/experiments/nationwide-2020/inventory.json");
    let ledger_path = root.join("docs/experiments/nationwide-2020/tree-build-ledger.json");
    let inventory = read_json(&inventory_path)?;
    let mut prior: BTreeMap<String, Value> = if ledger_path.is_file() {
        read_json(&ledger_path)?["results"]
            .as_array()
            .context("ledger results")?
            .iter()
            .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
            .collect()
    } else {
        BTreeMap::new()
    };
    let package_root = root.join("data/2020/certified/operational-trees");
    for source in inventory["states"].as_array().context("inventory states")? {
        let state = source["state"].as_str().context("state")?;
        let package = package_root.join(state.to_lowercase());
        if package.join("manifest.json").is_file() {
            prior.insert(state.into(), json!({"state":state,"districts":source["districts"],"block_count":source["block_count"],"status":"built","exit_code":0,"command":["recovered-from-package"],"output":"Recovered from verified package manifest."}));
        }
    }
    let failed: BTreeSet<String> = prior
        .iter()
        .filter_map(|(state, row)| (row["status"] == "failed").then_some(state.clone()))
        .collect();
    let mut states: Vec<Value> = inventory["states"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            let state = row["state"].as_str().unwrap_or("");
            row["districts"].as_u64().unwrap_or(0) > 1
                && (retry_failed || !failed.contains(state))
                && !package_root
                    .join(state.to_lowercase())
                    .join("manifest.json")
                    .is_file()
        })
        .cloned()
        .collect();
    states.sort_by_key(|row| {
        (
            row["block_count"].as_u64().unwrap_or(u64::MAX),
            row["state"].as_str().unwrap_or("").to_owned(),
        )
    });
    if let Some(limit) = limit {
        states.truncate(limit);
    }
    for row in states {
        let state = row["state"].as_str().context("state")?.to_owned();
        let lower = state.to_lowercase();
        let districts = row["districts"].as_u64().context("districts")? as usize;
        let context = root.join(format!("data/2020/certified/{lower}_blocks_2020.rctx"));
        let out = package_root.join(&lower);
        let result = build(bisect, &context, &out, districts, 1, [2, 3], max_seed);
        let (status, exit_code, output) = match result {
            Ok(()) => ("built", 0, "Built and verified by bisect-ops.".into()),
            Err(error) => ("failed", 1, format!("{error:#}")),
        };
        println!("{state}: {status}");
        prior.insert(state.clone(),json!({"state":state,"districts":districts,"block_count":row["block_count"],"status":status,"exit_code":exit_code,"command":["bisect-ops","batch",format!("--max-seed={max_seed}")],"output":output}));
        let results: Vec<_> = prior.values().cloned().collect();
        let built = results
            .iter()
            .filter(|row| row["status"] == "built")
            .count();
        let failed_count = results
            .iter()
            .filter(|row| row["status"] == "failed")
            .count();
        write_json(
            &ledger_path,
            &json!({"schema_version":"certified-national-tree-build-ledger-v1","results":results,"built_count":built,"failed_count":failed_count,"claim_boundary":"Resumable operational tree build ledger; national coverage verification is separate."}),
            true,
        )?;
    }
    Ok(())
}

fn audit_python(staged: bool, base: Option<&str>) -> Result<()> {
    let mut command = Command::new("git");
    command.args(["diff", "--name-only", "--diff-filter=AM"]);
    if staged {
        command.arg("--cached");
    } else if let Some(base) = base {
        command.arg(format!("{base}...HEAD"));
    } else {
        command.arg("HEAD");
    }
    let output = command.output().context("run git diff for Python policy")?;
    if !output.status.success() {
        bail!("git diff failed while enforcing Rust-first policy");
    }
    let blocked: Vec<_> = String::from_utf8(output.stdout)?
        .lines()
        .filter(|path| path.ends_with(".py"))
        .filter(|path| {
            *path != "setup_data.py"
                && !path.starts_with("python/bisect_py/")
                && !path.starts_with("scripts/data/")
        })
        .map(str::to_owned)
        .collect();
    if !blocked.is_empty() {
        bail!(
            "Rust-first policy blocks new or modified Python:\n{}",
            blocked.join("\n")
        );
    }
    println!("Rust-first Python boundary: PASS");
    Ok(())
}

fn write_source_snapshot(dir: &Path, name: &str) -> Result<(PathBuf, String)> {
    let path = dir.join(name);
    fs::create_dir_all(dir)?;
    let source = include_bytes!("main.rs");
    if fs::read(&path).ok().as_deref() != Some(source) {
        fs::write(&path, source)?;
    }
    Ok((path.clone(), sha256(&path)?))
}

fn write_content_addressed_source_snapshot(dir: &Path, stem: &str) -> Result<(PathBuf, String)> {
    let source = include_bytes!("main.rs");
    let digest = format!("{:x}", Sha256::digest(source));
    let path = dir.join(format!("{stem}-{}.rs", &digest[..16]));
    fs::create_dir_all(dir)?;
    if fs::read(&path).ok().as_deref() != Some(source) {
        fs::write(&path, source)?;
    }
    Ok((path, digest))
}

fn analyze_tree(
    state: &str,
    package: &Path,
    rctx_report_path: &Path,
    report_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    let package_manifest = read_json(&package.join("manifest.json"))?;
    let tree_path = package.join(
        package_manifest["files"][0]["path"]
            .as_str()
            .context("tree path")?,
    );
    if sha256(&tree_path)?
        != package_manifest["files"][0]["sha256"]
            .as_str()
            .context("tree hash")?
    {
        bail!("operational tree package hash mismatch");
    }
    let tree = read_json(&tree_path)?;
    let rctx = read_json(rctx_report_path)?;
    let leaves = tree["leaves"].as_array().context("leaves")?;
    let nodes = tree["nodes"].as_array().context("nodes")?;
    let leaf_units: u64 = leaves
        .iter()
        .map(|v| v["unit_count"].as_u64().unwrap_or(0))
        .sum();
    let leaf_population: i64 = leaves
        .iter()
        .map(|v| v["population"].as_i64().unwrap_or(0))
        .sum();
    let floors = nodes.iter().all(|node| {
        node.pointer("/objective/max_population_deviation_scaled")
            == node.pointer("/population_proof/lower_bound")
    });
    if tree["unit_count"] != rctx["unit_count"]
        || leaves.len() != tree["districts"].as_u64().unwrap_or(0) as usize
        || leaf_units != tree["unit_count"].as_u64().unwrap_or(u64::MAX)
        || leaf_population != tree["population_total"].as_i64().unwrap_or(i64::MIN)
        || !floors
    {
        bail!("operational tree coverage or population proof mismatch");
    }
    let claim="Complete connected wall-to-wall recursive tree with arithmetic population optimality at every node; boundary and canonical optimality are unproved.";
    let report = json!({"schema_version":"certified-operational-tree-frontier-v1","status":"operational-complete-population-proved","state":state,"year":2020,"districts":tree["districts"],"unit_count":tree["unit_count"],"population_total":tree["population_total"],"bridge_edge_count":rctx["bridge_edge_count"],"tree_sha256":sha256(&tree_path)?,"package_manifest_sha256":sha256(&package.join("manifest.json"))?,"nodes":nodes,"leaves":leaves,"boundary_proof":"not-run","canonical_proof":"blocked-by-boundary","claim_boundary":claim});
    write_json(report_path, &report, true)?;
    let parent = manifest_path.parent().context("manifest parent")?;
    let (source, source_hash) = write_source_snapshot(parent, "bisect-ops-analyzer-source.rs")?;
    let report_name = report_path
        .file_name()
        .context("report filename")?
        .to_string_lossy()
        .into_owned();
    let source_name = source
        .file_name()
        .context("source filename")?
        .to_string_lossy()
        .into_owned();
    let manifest = json!({"schema_version":"certified-operational-tree-frontier-package-v1","package_id":format!("{}-operational-tree-2020",state.to_lowercase()),"status":"operational-complete-population-proved","files":[{"path":report_name,"sha256":sha256(report_path)?}],"analyzer_path":source_name,"analyzer_sha256":source_hash,"claim_boundary":claim});
    write_json(manifest_path, &manifest, true)?;
    println!("{state} operational tree frontier: VERIFIED");
    Ok(())
}

fn verify_tree_report(manifest_path: &Path) -> Result<()> {
    let manifest = read_json(manifest_path)?;
    let parent = manifest_path.parent().context("manifest parent")?;
    let analyzer = PathBuf::from(
        manifest["analyzer_path"]
            .as_str()
            .context("analyzer path")?,
    );
    let source = if analyzer.components().count() == 1 {
        parent.join(analyzer)
    } else {
        custody_source(&analyzer)
    };
    if sha256(&source)?
        != manifest["analyzer_sha256"]
            .as_str()
            .context("analyzer hash")?
    {
        bail!("operational tree analyzer hash mismatch");
    }
    let report_path = parent.join(
        manifest["files"][0]["path"]
            .as_str()
            .context("report path")?,
    );
    if sha256(&report_path)?
        != manifest["files"][0]["sha256"]
            .as_str()
            .context("report hash")?
    {
        bail!("operational tree report hash mismatch");
    }
    let report = read_json(&report_path)?;
    let leaves = report["leaves"].as_array().context("leaves")?;
    let units: u64 = leaves
        .iter()
        .map(|v| v["unit_count"].as_u64().unwrap_or(0))
        .sum();
    if report["status"] != "operational-complete-population-proved"
        || leaves.len() != report["districts"].as_u64().unwrap_or(0) as usize
        || units != report["unit_count"].as_u64().unwrap_or(u64::MAX)
    {
        bail!("operational tree report posture drift");
    }
    println!("Operational tree frontier report verification: PASS");
    Ok(())
}

fn verify_national_rctx(
    year: u16,
    out: &Path,
    context_root: &Path,
    require_complete: bool,
) -> Result<()> {
    if !matches!(year, 2000 | 2010 | 2020) {
        bail!("national RCTX verification currently supports census years 2000, 2010, and 2020");
    }
    let root = std::env::current_dir()?;
    let inventory = read_json(&out.join("inventory.json"))?;
    if inventory["state_count"] != 50 || inventory["district_count"] != 435 {
        bail!("national {year} inventory must bind 50 States and 435 districts");
    }
    if inventory.get("census_year").is_some() && inventory["census_year"] != year {
        bail!("inventory census year does not match requested year");
    }
    let mut states: Vec<(String, String)> = inventory["states"]
        .as_array()
        .context("states")?
        .iter()
        .filter_map(|v| {
            Some((
                v["state"].as_str()?.to_owned(),
                v["fips"].as_str()?.to_owned(),
            ))
        })
        .collect();
    states.sort();
    let mut rows = Vec::new();
    let mut missing_states = Vec::new();
    for (state, fips) in states {
        let path = context_root.join(format!("{}_blocks_{year}.rctx", state.to_lowercase()));
        if !path.is_file() {
            missing_states.push(state);
            continue;
        }
        let context_text = fs::read_to_string(&path)
            .with_context(|| format!("read {state} RCTX {}", path.display()))?;
        rplan_io::read_rctx_str(&context_text)
            .with_context(|| format!("parse {state} RCTX through the public reader"))?;
        let context: Value = serde_json::from_str(&context_text)?;
        if context["rctx_version"] != "0.1"
            || context.pointer("/units/year") != Some(&json!(year))
            || context.pointer("/units/state") != Some(&json!(state))
        {
            bail!("{state} context identity mismatch");
        }
        let projection = json!({"units":context["units"],"graph":context["graph"],"populations":context["populations"],"source_hashes":context["source_hashes"]});
        if context["context_hash"] != canonical_hash(&projection)? {
            bail!("{state} context hash mismatch");
        }
        let unit_ids = context
            .pointer("/units/unit_ids")
            .and_then(Value::as_array)
            .context("unit_ids")?;
        let populations = context["populations"].as_array().context("populations")?;
        let adjacency = context
            .pointer("/graph/adjacency")
            .and_then(Value::as_array)
            .context("adjacency")?;
        if unit_ids.is_empty()
            || unit_ids.len() != populations.len()
            || unit_ids.len() != adjacency.len()
        {
            bail!("{state} context vector length mismatch");
        }
        let unit_id_text: Vec<&str> = unit_ids
            .iter()
            .map(|value| value.as_str().context("unit id"))
            .collect::<Result<_>>()?;
        if unit_id_text
            .iter()
            .any(|id| id.len() != 15 || !id.bytes().all(|byte| byte.is_ascii_digit()))
            || unit_id_text.windows(2).any(|pair| pair[0] >= pair[1])
        {
            bail!("{state} unit universe is not unique sorted 15-digit GEOIDs");
        }
        for (from, edges) in adjacency.iter().enumerate() {
            let mut edge_keys = BTreeSet::new();
            for edge in edges.as_array().context("edges")? {
                let to = edge["to"].as_u64().context("edge.to")? as usize;
                let kind = edge["kind"].as_str().context("edge.kind")?;
                if to >= adjacency.len() || to == from || !edge_keys.insert((to, kind)) {
                    bail!("{state} invalid or duplicate adjacency edge at unit {from}");
                }
                let reverse = adjacency[to]
                    .as_array()
                    .context("reverse edges")?
                    .iter()
                    .any(|candidate| {
                        candidate["to"] == from
                            && candidate["kind"] == edge["kind"]
                            && candidate["weight"] == edge["weight"]
                    });
                if !reverse {
                    bail!("{state} asymmetric adjacency edge {from}->{to}");
                }
            }
        }
        let mut seen = BTreeSet::from([0usize]);
        let mut queue = VecDeque::from([0usize]);
        while let Some(unit) = queue.pop_front() {
            for edge in adjacency[unit].as_array().context("edges")? {
                let to = edge["to"].as_u64().context("edge.to")? as usize;
                if seen.insert(to) {
                    queue.push_back(to);
                }
            }
        }
        if seen.len() != adjacency.len() {
            bail!("{state} context is disconnected");
        }
        let directed: usize = adjacency
            .iter()
            .map(|v| v.as_array().map_or(0, Vec::len))
            .sum();
        let bridges: usize = adjacency
            .iter()
            .flat_map(|v| v.as_array().into_iter().flatten())
            .filter(|edge| edge["kind"] == "bridge")
            .count()
            / 2;
        let population: i64 = populations.iter().map(|v| v.as_i64().unwrap_or(0)).sum();
        let suffix = if year == 2000 {
            "00"
        } else if year == 2010 {
            "10"
        } else if year == 2020 {
            "20"
        } else {
            ""
        };
        let lower = state.to_lowercase();
        let tiger_year = if year == 2000 { 2010 } else { year };
        let tiger_base = root.join(format!("data/{year}/tiger/blocks/tl_{tiger_year}_{fips}_tabblock{suffix}/tl_{tiger_year}_{fips}_tabblock{suffix}"));
        let source_checks = if year == 2000 {
            let geography = root.join(format!("data/2000/redistricting/{lower}geo.upl"));
            vec![("pl_geo", geography.clone()), ("pl_population", geography)]
        } else {
            let pl_dir = root.join(format!("data/{year}/redistricting/{lower}{year}.pl"));
            vec![
                ("tiger_block_shp", tiger_base.with_extension("shp")),
                ("tiger_block_dbf", tiger_base.with_extension("dbf")),
                ("tiger_block_shx", tiger_base.with_extension("shx")),
                ("pl_geo", pl_dir.join(format!("{lower}geo{year}.pl"))),
                (
                    "pl_population",
                    pl_dir.join(format!("{lower}00001{year}.pl")),
                ),
            ]
        };
        let mut rehashed_sources = 0usize;
        let mut rehashed_keys = BTreeSet::new();
        for (key, source_path) in &source_checks {
            if source_path.is_file() {
                let expected = context["source_hashes"][key]
                    .as_str()
                    .with_context(|| format!("{state} missing source hash {key}"))?;
                if expected != format!("sha256:{}", sha256(source_path)?) {
                    bail!("{state} source hash mismatch for {key}");
                }
                rehashed_sources += 1;
                rehashed_keys.insert(*key);
            }
        }
        let source_hashes = context["source_hashes"]
            .as_object()
            .context("RCTX source_hashes must be an object")?;
        if source_hashes.values().any(|value| !value.is_string()) {
            bail!("{state} RCTX source_hashes must be a flat string-to-string map");
        }
        let tiger_block_files: BTreeMap<String, String> = source_hashes
            .iter()
            .filter_map(|(key, value)| {
                key.strip_prefix("tiger_block_file:").map(|relative| {
                    (
                        relative.to_owned(),
                        value.as_str().unwrap_or_default().to_owned(),
                    )
                })
            })
            .collect();
        let tiger_archives: BTreeMap<String, String> = source_hashes
            .iter()
            .filter_map(|(key, value)| {
                key.strip_prefix("tiger_archive_file:").map(|relative| {
                    (
                        relative.to_owned(),
                        value.as_str().unwrap_or_default().to_owned(),
                    )
                })
            })
            .collect();
        if year == 2000 && (tiger_block_files.is_empty() || tiger_archives.is_empty()) {
            bail!("{state} Census 2000 RCTX is missing flat TIGER file custody hashes");
        }
        let mut missing_tiger_block_files = Vec::new();
        for (relative, expected) in &tiger_block_files {
            let source_path = governed_source_path(&root, relative)?;
            if !source_path.is_file() {
                missing_tiger_block_files.push(relative.clone());
                continue;
            }
            if expected != &format!("sha256:{}", sha256(&source_path)?) {
                bail!("{state} TIGER block source hash mismatch: {relative}");
            }
            rehashed_sources += 1;
        }
        let mut tiger_bundle_archive_verified = false;
        if !tiger_archives.is_empty() {
            let mut member_hashes = BTreeMap::new();
            for (relative, expected) in &tiger_archives {
                let archive_path = governed_source_path(&root, relative)?;
                if !archive_path.is_file()
                    || expected != &format!("sha256:{}", sha256(&archive_path)?)
                {
                    bail!("{state} TIGER archive missing or hash-mismatched: {relative}");
                }
                rehashed_sources += 1;
                for (name, hash) in tiger_archive_member_hashes(&archive_path)? {
                    if member_hashes.insert(name.clone(), hash).is_some() {
                        bail!("{state} duplicate TIGER archive member filename: {name}");
                    }
                }
            }
            for (relative, expected) in &tiger_block_files {
                let filename = Path::new(relative)
                    .file_name()
                    .and_then(|value| value.to_str())
                    .context("TIGER block source filename")?;
                if member_hashes.get(filename) != Some(expected) {
                    bail!("{state} TIGER archive member hash mismatch for {filename}");
                }
            }
            tiger_bundle_archive_verified = true;
        }
        if !missing_tiger_block_files.is_empty() && !tiger_bundle_archive_verified {
            bail!(
                "{state} has {} missing extracted TIGER files without verified archive custody",
                missing_tiger_block_files.len()
            );
        }
        if let Some(expected_archive_hash) = context["source_hashes"]["tiger_archive"].as_str() {
            let archive_path = root.join(format!(
                "data/{year}/tiger/archives/tl_{year}_{fips}_tabblock{suffix}.zip"
            ));
            if !archive_path.is_file()
                || format!("sha256:{}", sha256(&archive_path)?) != expected_archive_hash
            {
                bail!("{state} TIGER archive missing or hash-mismatched");
            }
            rehashed_sources += 1;
            let members = tiger_archive_member_hashes(&archive_path)?;
            for (key, source_path) in source_checks.iter().take(3) {
                let filename = source_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .context("TIGER source filename")?;
                let member_hash = members
                    .get(filename)
                    .with_context(|| format!("{state} TIGER archive missing {filename}"))?;
                if Some(member_hash.as_str()) != context["source_hashes"][key].as_str() {
                    bail!("{state} TIGER archive member hash mismatch for {filename}");
                }
                if rehashed_keys.insert(*key) {
                    rehashed_sources += 1;
                }
            }
        }
        rows.push(json!({"state":state,"year":year,"unit_count":unit_ids.len(),"population_total":population,"edge_count":directed/2,"bridge_edge_count":bridges,"rctx_bytes":fs::metadata(&path)?.len(),"rctx_sha256":sha256(&path)?,"context_hash":context["context_hash"],"source_files_rehashed":rehashed_sources,"status":"verified"}));
        println!(
            "{}: verified",
            rows.last().unwrap()["state"].as_str().unwrap()
        );
    }
    let sum = |key: &str| {
        rows.iter()
            .map(|v| v[key].as_u64().unwrap_or(0))
            .sum::<u64>()
    };
    let population: i64 = rows
        .iter()
        .map(|v| v["population_total"].as_i64().unwrap_or(0))
        .sum();
    if require_complete && !missing_states.is_empty() {
        bail!(
            "national {year} RCTX verification incomplete; missing {} States: {}",
            missing_states.len(),
            missing_states.join(",")
        );
    }
    let complete = missing_states.is_empty() && rows.len() == 50;
    let status = if complete { "verified" } else { "partial" };
    let claim=format!("Local {year} block contexts listed as verified are independently checked for hash validity, canonical unit coverage, symmetric connected adjacency, and available source hashes. Complete national context coverage is claimed only when all 50 States pass; no district assignments are claimed.");
    let report = json!({"schema_version":"certified-national-rctx-verification-v2","status":status,"year":year,"complete":complete,"state_count":rows.len(),"missing_state_count":missing_states.len(),"missing_states":missing_states,"unit_count":sum("unit_count"),"population_total":population,"edge_count":sum("edge_count"),"bridge_edge_count":sum("bridge_edge_count"),"rctx_bytes":sum("rctx_bytes"),"states":rows,"claim_boundary":claim});
    let report_path = out.join("rctx-verification.json");
    write_json(&report_path, &report, true)?;
    let (source, source_hash) =
        write_content_addressed_source_snapshot(out, "bisect-ops-rctx-verifier-source")?;
    let source_name = source
        .file_name()
        .context("source filename")?
        .to_string_lossy()
        .into_owned();
    let manifest = json!({"schema_version":"certified-national-rctx-verification-package-v2","package_id":format!("nationwide-{year}-rctx-verification"),"status":status,"year":year,"files":[{"path":"rctx-verification.json","sha256":sha256(&report_path)?}],"verifier_path":source_name,"verifier_sha256":source_hash,"claim_boundary":claim});
    write_json(&out.join("rctx-manifest.json"), &manifest, true)?;
    println!(
        "National RCTX verification: {} States, {} blocks, {} bridges",
        report["state_count"], report["unit_count"], report["bridge_edge_count"]
    );
    Ok(())
}

fn verify_connected_assignment(
    context: &Value,
    assignment: &[u64],
    districts: usize,
) -> Result<()> {
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    if adjacency.len() != assignment.len() {
        bail!("assignment and adjacency length mismatch");
    }
    let mut counts = vec![0usize; districts];
    let mut first = vec![None; districts];
    for (unit, &district) in assignment.iter().enumerate() {
        let district = usize::try_from(district).context("district label overflow")?;
        if district >= districts {
            bail!("assignment contains out-of-range district {district}");
        }
        counts[district] += 1;
        first[district].get_or_insert(unit);
    }
    let mut visited = vec![false; assignment.len()];
    for district in 0..districts {
        let start = first[district].context("assignment contains an empty district")?;
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        let mut reached = 0usize;
        while let Some(unit) = queue.pop_front() {
            reached += 1;
            for edge in adjacency[unit].as_array().context("edge list")? {
                let to = edge["to"].as_u64().context("edge.to")? as usize;
                if to >= assignment.len() {
                    bail!("adjacency endpoint outside unit universe");
                }
                if assignment[to] == district as u64 && !visited[to] {
                    visited[to] = true;
                    queue.push_back(to);
                }
            }
        }
        if reached != counts[district] {
            bail!("district {district} is disconnected");
        }
    }
    Ok(())
}

fn verify_recursive_schedule(tree: &Value) -> Result<()> {
    let districts = tree["districts"].as_u64().context("districts")?;
    let nodes = tree["nodes"].as_array().context("nodes")?;
    let leaves = tree["leaves"].as_array().context("leaves")?;
    if nodes.len() as u64 != districts - 1 || leaves.len() as u64 != districts {
        bail!("recursive node or leaf count mismatch");
    }
    let node_seats: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| {
            Ok((
                node["path"].as_str().context("node path")?.to_owned(),
                node["seats"].as_u64().context("node seats")?,
            ))
        })
        .collect::<Result<_>>()?;
    let leaf_paths: BTreeSet<_> = leaves
        .iter()
        .map(|leaf| {
            leaf["path"]
                .as_str()
                .context("leaf path")
                .map(str::to_owned)
        })
        .collect::<Result<_>>()?;
    if node_seats.get("") != Some(&districts) || leaf_paths.len() != leaves.len() {
        bail!("recursive root or leaf path mismatch");
    }
    for (path, seats) in &node_seats {
        if *seats < 2 {
            bail!("internal node {path} has fewer than two seats");
        }
        let left = seats / 2;
        let right = seats - left;
        for (label, child_seats) in [(0, left), (1, right)] {
            let child = format!("{path}{label}");
            let present = if child_seats == 1 {
                leaf_paths.contains(&child)
            } else {
                node_seats.get(&child) == Some(&child_seats)
            };
            if !present {
                bail!("recursive schedule missing {child} with {child_seats} seats");
            }
        }
    }
    Ok(())
}

fn verify_national_trees(
    out: &Path,
    package_root: &Path,
    context_root: &Path,
    one_district_path: &Path,
) -> Result<()> {
    let rctx_report = read_json(&out.join("rctx-verification.json"))?;
    if rctx_report["status"] != "verified" || rctx_report["state_count"] != 50 {
        bail!("national RCTX verification is not complete");
    }
    let rctx_states: BTreeMap<String, Value> = rctx_report["states"]
        .as_array()
        .context("RCTX states")?
        .iter()
        .map(|row| {
            Ok((
                row["state"].as_str().context("RCTX state")?.to_owned(),
                row.clone(),
            ))
        })
        .collect::<Result<_>>()?;
    let one_district = read_json(one_district_path)?;
    if one_district["status"] != "verified" || one_district["state_count"] != 6 {
        bail!("one-district verification is not complete");
    }
    let one_district_manifest_path = one_district_path
        .parent()
        .context("one-district parent")?
        .join("manifest.json");
    let one_district_manifest = read_json(&one_district_manifest_path)?;
    let one_district_name = one_district_path
        .file_name()
        .and_then(|value| value.to_str())
        .context("one-district filename")?;
    let one_district_file = one_district_manifest["files"]
        .as_array()
        .context("one-district manifest files")?
        .iter()
        .find(|file| file["path"] == one_district_name)
        .context("one-district manifest entry")?;
    if sha256(one_district_path)? != one_district_file["sha256"] {
        bail!("one-district report hash mismatch");
    }

    let mut rows = Vec::new();
    let mut state_names = BTreeSet::new();
    let mut total_units = 0u64;
    let mut total_population = 0i64;
    let mut total_leaves = 0u64;
    let mut total_nodes = 0u64;
    let mut builder_custody_gaps = Vec::new();

    for row in one_district["states"]
        .as_array()
        .context("one-district states")?
    {
        let state = row["state"].as_str().context("one-district state")?;
        let rctx = rctx_states.get(state).context("one-district RCTX row")?;
        if !state_names.insert(state.to_owned())
            || row["status"] != "verified"
            || row["final_component_count"] != 1
            || row["unit_count"] != rctx["unit_count"]
            || row["population_total"] != rctx["population_total"]
            || row["rctx_sha256"] != rctx["rctx_sha256"]
        {
            bail!("one-district evidence mismatch for {state}");
        }
        let units = row["unit_count"].as_u64().context("one-district units")?;
        let population = row["population_total"]
            .as_i64()
            .context("one-district population")?;
        total_units += units;
        total_population += population;
        total_leaves += 1;
        rows.push(json!({
            "state":state,"districts":1,"unit_count":units,"population_total":population,
            "recursive_nodes":0,"leaves":1,"assignment_coverage":"verified",
            "contiguity":"verified","population_proof":"not-applicable-single-district",
            "boundary_proof":"not-applicable-single-district",
            "canonical_proof":"not-applicable-single-district"
        }));
    }

    for entry in fs::read_dir(package_root)? {
        let package = entry?.path();
        if !package.is_dir() || !package.join("manifest.json").is_file() {
            continue;
        }
        let state = package
            .file_name()
            .and_then(|value| value.to_str())
            .context("package state")?
            .to_uppercase();
        let rctx = rctx_states.get(&state).context("multi-district RCTX row")?;
        let context_path = context_root.join(format!("{}_blocks_2020.rctx", state.to_lowercase()));
        let context = read_json(&context_path)?;
        let manifest_path = package.join("manifest.json");
        let manifest = read_json(&manifest_path)?;
        let builder = PathBuf::from(
            manifest["builder_path"]
                .as_str()
                .context("operational builder path")?,
        );
        let builder_source = if builder.components().count() == 1 {
            package.join(&builder)
        } else {
            custody_source(&builder)
        };
        let mut builder_custody =
            builder_source.is_file() && sha256(&builder_source)? == manifest["builder_sha256"];
        if let Some(base) = manifest["base_builder_path"].as_str() {
            let base_source = custody_source(Path::new(base));
            builder_custody = builder_custody
                && base_source.is_file()
                && sha256(&base_source)? == manifest["base_builder_sha256"];
        }
        if !builder_custody {
            builder_custody_gaps.push(state.clone());
        }
        let tree_path = package.join(
            manifest["files"][0]["path"]
                .as_str()
                .context("operational tree path")?,
        );
        if sha256(&tree_path)? != manifest["files"][0]["sha256"] {
            bail!("package hash mismatch for {state}");
        }
        let tree = read_json(&tree_path)?;
        if sha256(&context_path)? != tree["context_sha256"] {
            bail!("package or context hash mismatch for {state}");
        }
        if tree["context_hash"] != context["context_hash"]
            || tree["unit_count"] != rctx["unit_count"]
            || tree["population_total"] != rctx["population_total"]
        {
            bail!("tree/context identity mismatch for {state}");
        }
        verify_recursive_schedule(&tree).with_context(|| state.clone())?;
        let districts = tree["districts"].as_u64().context("districts")? as usize;
        let assignment: Vec<u64> = tree["assignment"]
            .as_array()
            .context("assignment")?
            .iter()
            .map(|value| value.as_u64().context("assignment label"))
            .collect::<Result<_>>()?;
        if assignment.len() as u64 != tree["unit_count"].as_u64().unwrap_or(u64::MAX) {
            bail!("assignment coverage mismatch for {state}");
        }
        verify_connected_assignment(&context, &assignment, districts)
            .with_context(|| state.clone())?;
        let populations = context["populations"].as_array().context("populations")?;
        let mut district_units = vec![0u64; districts];
        let mut district_populations = vec![0i64; districts];
        for (unit, &district) in assignment.iter().enumerate() {
            let district = district as usize;
            district_units[district] += 1;
            district_populations[district] += populations[unit].as_i64().unwrap_or(0);
        }
        for leaf in tree["leaves"].as_array().context("leaves")? {
            let district = leaf["district"].as_u64().context("leaf district")? as usize;
            if district >= districts
                || leaf["unit_count"] != district_units[district]
                || leaf["population"] != district_populations[district]
            {
                bail!("leaf accounting mismatch for {state} district {district}");
            }
        }
        let nodes = tree["nodes"].as_array().context("nodes")?;
        if !nodes.iter().all(|node| {
            node.pointer("/objective/max_population_deviation_scaled")
                == node.pointer("/population_proof/lower_bound")
                && node
                    .pointer("/population_proof/kind")
                    .and_then(Value::as_str)
                    == Some("ratio-arithmetic-floor")
        }) {
            bail!("population proof mismatch for {state}");
        }
        let units = tree["unit_count"].as_u64().context("unit count")?;
        let population = tree["population_total"].as_i64().context("population")?;
        let node_count = nodes.len() as u64;
        if !state_names.insert(state.clone()) {
            bail!("duplicate State package for {state}");
        }
        total_units += units;
        total_population += population;
        total_leaves += districts as u64;
        total_nodes += node_count;
        rows.push(json!({
            "state":state,"districts":districts,"unit_count":units,
            "population_total":population,"recursive_nodes":node_count,"leaves":districts,
            "package_manifest_sha256":sha256(&manifest_path)?,"tree_sha256":sha256(&tree_path)?,
            "builder_source_custody":if builder_custody { "verified" } else { "declared-hash-not-currently-matching" },
            "assignment_coverage":"verified","contiguity":"independently-verified",
            "population_proof":"arithmetic-floor-proved-all-nodes",
            "boundary_proof":"not-run","canonical_proof":"blocked-by-boundary"
        }));
    }
    rows.sort_by_key(|row| row["state"].as_str().unwrap_or("").to_owned());
    if state_names.len() != 50
        || total_units != rctx_report["unit_count"].as_u64().unwrap_or(u64::MAX)
        || total_population != rctx_report["population_total"].as_i64().unwrap_or(i64::MIN)
        || total_leaves != 435
        || total_nodes != 385
    {
        bail!("national aggregate coverage mismatch");
    }
    let claim = "All 50 State assignments cover every 2020 Census block exactly once, all 435 leaves are connected, and all 385 nontrivial recursive nodes attain their arithmetic population floors. Historical builder-source custody gaps are reported separately. Boundary and canonical optimality remain unproved.";
    let verification_path = out.join("national-tree-verification.json");
    write_json(
        &verification_path,
        &json!({
            "schema_version":"certified-national-operational-tree-verification-v1",
            "status":"verified-operational-complete-population-proved","state_count":50,
            "district_count":435,"recursive_node_count":385,"unit_count":total_units,
            "population_total":total_population,"omitted_units":0,"duplicate_units":0,
            "disconnected_leaves":0,"operational_builder_packages":44,
            "builder_source_custody_verified":44-builder_custody_gaps.len(),
            "builder_source_custody_gaps":builder_custody_gaps,"states":rows,
            "claim_boundary":claim
        }),
        true,
    )?;
    let proof_path = out.join("national-proof-coverage.json");
    write_json(
        &proof_path,
        &json!({
            "schema_version":"certified-national-proof-coverage-v1","status":"classified",
            "recursive_node_count":385,
            "population":{"proved_nodes":385,"unproved_nodes":0,"coverage_rate":1.0,"proof_kind":"ratio-arithmetic-floor"},
            "boundary":{"proved_nodes":0,"unproved_nodes":385,"coverage_rate":0.0,"status":"not-run"},
            "canonical":{"proved_nodes":0,"unproved_nodes":385,"coverage_rate":0.0,"status":"blocked-by-boundary"},
            "single_district_states":{"count":6,"objective_proofs":"not-applicable"},
            "claim_boundary":claim
        }),
        true,
    )?;
    let (source, source_hash) =
        write_source_snapshot(out, "bisect-ops-national-tree-verifier-source.rs")?;
    let manifest_path = out.join("national-tree-verification-manifest.json");
    write_json(
        &manifest_path,
        &json!({
            "schema_version":"certified-national-operational-tree-verification-package-v1",
            "package_id":"nationwide-2020-operational-tree-verification",
            "status":"verified-operational-complete-population-proved",
            "files":[
                {"path":"national-tree-verification.json","sha256":sha256(&verification_path)?},
                {"path":"national-proof-coverage.json","sha256":sha256(&proof_path)?}
            ],
            "verifier_path":source.file_name().and_then(|value| value.to_str()).context("verifier filename")?,
            "verifier_sha256":source_hash,"claim_boundary":claim
        }),
        true,
    )?;
    println!(
        "National operational tree verification: PASS (50 States, 435 leaves, 8,126,956 blocks)"
    );
    Ok(())
}

fn git_text(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output().context("run git")?;
    if !output.status.success() {
        bail!("git {} failed", args.join(" "));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn copy_artifact(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination).with_context(|| {
        format!(
            "copy release artifact {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn release_relative(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .context("release path outside bundle")?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn release_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(&path, files)?;
            } else {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort_by_key(|path| path.to_string_lossy().to_string());
    Ok(files)
}

fn assignment_for_state(state: &str, districts: u64) -> Result<(Value, Vec<u64>, PathBuf)> {
    let lower = state.to_lowercase();
    let context_path = PathBuf::from(format!("data/2020/certified/{lower}_blocks_2020.rctx"));
    let context = read_json(&context_path)?;
    let unit_count = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?
        .len();
    let assignment = if districts == 1 {
        vec![0; unit_count]
    } else {
        read_json(
            &Path::new("data/2020/certified/operational-trees")
                .join(&lower)
                .join("operational-tree.json"),
        )?["assignment"]
            .as_array()
            .context("operational assignment")?
            .iter()
            .map(|value| value.as_u64().context("assignment label"))
            .collect::<Result<_>>()?
    };
    if assignment.len() != unit_count || assignment.iter().any(|district| *district >= districts) {
        bail!("release assignment mismatch for {state}");
    }
    Ok((context, assignment, context_path))
}

fn write_assignment_csv(context: &Value, assignment: &[u64], destination: &Path) -> Result<()> {
    let unit_ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?;
    let mut writer = BufWriter::new(File::create(destination)?);
    writeln!(writer, "geoid,district_zero_based,district_one_based")?;
    for (unit, district) in unit_ids.iter().zip(assignment) {
        writeln!(
            writer,
            "{},{},{}",
            unit.as_str().context("unit id")?,
            district,
            district + 1
        )?;
    }
    writer.flush()?;
    Ok(())
}

fn render_centroid_map(
    centroids: &[(String, (f64, f64))],
    unit_ids: &[Value],
    assignment: &[u64],
    destination: &Path,
) -> Result<Value> {
    if centroids.len() != unit_ids.len() || assignment.len() != unit_ids.len() {
        bail!("centroid map universe length mismatch");
    }
    for ((geoid, _), expected) in centroids.iter().zip(unit_ids) {
        if Some(geoid.as_str()) != expected.as_str() {
            bail!("centroid map GEOID order mismatch");
        }
    }
    let (min_x, max_x, min_y, max_y) = centroids.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), (_, (x, y))| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    );
    let width = 1200u32;
    let height = 900u32;
    let padding = 24.0;
    let scale = ((width as f64 - 2.0 * padding) / (max_x - min_x).max(1.0))
        .min((height as f64 - 2.0 * padding) / (max_y - min_y).max(1.0));
    let drawn_width = (max_x - min_x) * scale;
    let drawn_height = (max_y - min_y) * scale;
    let offset_x = (width as f64 - drawn_width) / 2.0;
    let offset_y = (height as f64 - drawn_height) / 2.0;
    let scheme = CategoricalScheme::default();
    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("create release map pixmap")?;
    pixmap.fill(tiny_skia::Color::WHITE);
    let pixels = pixmap.pixels_mut();
    for ((_, (x, y)), district) in centroids.iter().zip(assignment) {
        let px = ((*x - min_x) * scale + offset_x).round() as i64;
        let py = ((max_y - *y) * scale + offset_y).round() as i64;
        let (r, g, b) = scheme.color(*district as usize);
        let color =
            tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255).context("opaque map color")?;
        for dy in 0..=1i64 {
            for dx in 0..=1i64 {
                let draw_x = px + dx;
                let draw_y = py + dy;
                if draw_x >= 0 && draw_y >= 0 && draw_x < width as i64 && draw_y < height as i64 {
                    pixels[draw_y as usize * width as usize + draw_x as usize] = color;
                }
            }
        }
    }
    pixmap.save_png(destination)?;
    Ok(json!({
        "kind":"projected-block-centroid-diagnostic",
        "projection":"EPSG:5070",
        "width":width,"height":height,"block_count":centroids.len(),
        "limitation":"Display diagnostic only; points are block centroids and do not represent exact district polygon boundaries."
    }))
}

fn count_screening(tree: &Value) -> Result<(u64, u64, u64, u64)> {
    let mut completed = 0u64;
    let mut timeouts = 0u64;
    let mut retained_nodes = 0u64;
    let nodes = tree["nodes"].as_array().context("nodes")?;
    for node in nodes {
        let Some(screening) = node.get("seed_screening").and_then(Value::as_array) else {
            continue;
        };
        retained_nodes += 1;
        for screen in screening {
            match screen["status"].as_str() {
                Some("completed") => completed += 1,
                Some("timeout") => timeouts += 1,
                _ => {}
            }
        }
    }
    Ok((
        completed,
        timeouts,
        retained_nodes,
        nodes.len() as u64 - retained_nodes,
    ))
}

fn build_national_release(out: &Path, created_at: &str) -> Result<()> {
    if out.exists() {
        bail!("release bundle already exists: {}", out.display());
    }
    if !git_text(&["status", "--porcelain"])?.is_empty() {
        bail!("release candidate must be generated from a clean working tree");
    }
    let git_commit = git_text(&["rev-parse", "HEAD"])?;
    let required_dirs = [
        "config",
        "runs/assignments",
        "analysis",
        "reports/maps",
        "reports/verification",
        "review",
        "limitations",
    ];
    for dir in required_dirs {
        fs::create_dir_all(out.join(dir))?;
    }
    let config = json!({
        "schema_version":"nationwide-operational-profile-v1","year":2020,
        "geography":"2020 Census tabulation blocks","structure":"standard recursive bisection",
        "seat_schedule":"floor(k/2)/ceil(k/2)","weights":"exact shared-boundary length",
        "discovery":"deterministic bounded METIS seed screening with population refinement",
        "population_target":"ratio arithmetic floor at every recursive node",
        "seed_frontier_max":128,
        "claim_boundary":"Operational profile; not the single-seed NRS v0.1 conformance profile."
    });
    let config_path = out.join("config/operational-profile.json");
    write_json(&config_path, &config, true)?;
    for (source, destination) in [
        (
            "docs/experiments/nationwide-2020/national-tree-verification.json",
            "reports/verification/national-tree-verification.json",
        ),
        (
            "docs/experiments/nationwide-2020/national-tree-verification-manifest.json",
            "reports/verification/national-tree-verification-manifest.json",
        ),
        (
            "docs/experiments/nationwide-2020/bisect-ops-national-tree-verifier-source.rs",
            "reports/verification/verifier-source.rs",
        ),
        (
            "docs/experiments/nationwide-2020/national-proof-coverage.json",
            "analysis/national-proof-coverage.json",
        ),
        (
            "docs/experiments/nationwide-2020/BUILDER_CUSTODY_DISPOSITION.md",
            "review/BUILDER_CUSTODY_DISPOSITION.md",
        ),
    ] {
        copy_artifact(Path::new(source), &out.join(destination))?;
    }
    let inventory = read_json(Path::new("docs/experiments/nationwide-2020/inventory.json"))?;
    let national = read_json(Path::new(
        "docs/experiments/nationwide-2020/national-tree-verification.json",
    ))?;
    let national_rows: BTreeMap<String, Value> = national["states"]
        .as_array()
        .context("national states")?
        .iter()
        .map(|row| {
            Ok((
                row["state"].as_str().context("national state")?.to_owned(),
                row.clone(),
            ))
        })
        .collect::<Result<_>>()?;
    let mut package_rows = Vec::new();
    let mut runtime_rows = Vec::new();
    let mut proof_size_rows = Vec::new();
    let mut map_rows = Vec::new();
    for state_row in inventory["states"].as_array().context("inventory states")? {
        let state = state_row["state"].as_str().context("state")?;
        let fips = state_row["fips"].as_str().context("fips")?;
        let districts = state_row["districts"].as_u64().context("districts")?;
        let (context, assignment, context_path) = assignment_for_state(state, districts)?;
        let assignment_path = out
            .join("runs/assignments")
            .join(format!("{}.csv", state.to_lowercase()));
        write_assignment_csv(&context, &assignment, &assignment_path)?;
        let shape = PathBuf::from(format!(
            "data/2020/tiger/blocks/tl_2020_{fips}_tabblock20/tl_2020_{fips}_tabblock20.shp"
        ));
        let centroids = read_tiger_block_centroids_projected(&shape)?;
        let map_path = out
            .join("reports/maps")
            .join(format!("{}.png", state.to_lowercase()));
        let map_metadata = render_centroid_map(
            &centroids,
            context
                .pointer("/units/unit_ids")
                .and_then(Value::as_array)
                .context("unit ids")?,
            &assignment,
            &map_path,
        )?;
        map_rows.push(
            json!({"state":state,"path":release_relative(out,&map_path)?,"metadata":map_metadata}),
        );
        let national_row = national_rows.get(state).context("national state row")?;
        if districts == 1 {
            package_rows.push(json!({
                "state":state,"districts":districts,"context_path":context_path,
                "context_sha256":sha256(&context_path)?,"assignment_path":release_relative(out,&assignment_path)?,
                "assignment_sha256":sha256(&assignment_path)?,"tree":"not-applicable-single-district"
            }));
            runtime_rows.push(json!({"state":state,"recursive_nodes":0,"completed_screens":0,"timeout_screens":0,"wall_clock_seconds":null,"status":"not-retained"}));
            proof_size_rows.push(json!({
                "state":state,"recursive_nodes":0,"population_proof_records":0,
                "tree_file_bytes":0,"package_manifest_bytes":0,
                "boundary_certificate_bytes":0,"canonical_certificate_bytes":0,
                "status":"not-applicable-single-district"
            }));
        } else {
            let package =
                Path::new("data/2020/certified/operational-trees").join(state.to_lowercase());
            let tree_path = package.join("operational-tree.json");
            let manifest_path = package.join("manifest.json");
            let tree = read_json(&tree_path)?;
            let nodes = tree["nodes"].as_array().context("tree nodes")?;
            let population_proof_records = nodes
                .iter()
                .filter(|node| node.get("population_proof").is_some_and(Value::is_object))
                .count();
            let (completed, timeouts, retained_nodes, missing_nodes) = count_screening(&tree)?;
            package_rows.push(json!({
                "state":state,"districts":districts,"context_path":context_path,
                "context_sha256":sha256(&context_path)?,"tree_path":tree_path,
                "tree_sha256":sha256(&tree_path)?,"package_manifest_sha256":sha256(&manifest_path)?,
                "builder_source_custody":national_row["builder_source_custody"],
                "assignment_path":release_relative(out,&assignment_path)?,
                "assignment_sha256":sha256(&assignment_path)?
            }));
            runtime_rows.push(json!({
                "state":state,"recursive_nodes":tree["nodes"].as_array().map_or(0,Vec::len),
                "completed_screens":completed,"timeout_screens":timeouts,"screen_timeout_seconds":180,
                "screen_history_nodes_retained":retained_nodes,
                "screen_history_nodes_not_retained":missing_nodes,
                "wall_clock_seconds":null,
                "status":if missing_nodes == 0 {"screen-history-retained-wall-clock-not-retained"} else {"partial-screen-history-wall-clock-not-retained"}
            }));
            proof_size_rows.push(json!({
                "state":state,"recursive_nodes":nodes.len(),
                "population_proof_records":population_proof_records,
                "tree_file_bytes":fs::metadata(&tree_path)?.len(),
                "package_manifest_bytes":fs::metadata(&manifest_path)?.len(),
                "boundary_certificate_bytes":0,"canonical_certificate_bytes":0,
                "status":"population-proofs-embedded-boundary-and-canonical-certificates-absent"
            }));
        }
        println!("{state}: release assignment and centroid map complete");
    }
    write_json(
        &out.join("runs/state-package-index.json"),
        &json!({"schema_version":"nationwide-2020-release-state-index-v1","states":package_rows}),
        true,
    )?;
    write_json(
        &out.join("analysis/runtime-evidence.json"),
        &json!({
            "schema_version":"nationwide-2020-runtime-evidence-v1","states":runtime_rows,
            "limitation":"Per-screen 180-second timeout events are retained. Successful-screen and State wall-clock durations were not retained and cannot be reconstructed."
        }),
        true,
    )?;
    let total_tree_bytes: u64 = proof_size_rows
        .iter()
        .filter_map(|row| row["tree_file_bytes"].as_u64())
        .sum();
    let total_manifest_bytes: u64 = proof_size_rows
        .iter()
        .filter_map(|row| row["package_manifest_bytes"].as_u64())
        .sum();
    let total_population_records: u64 = proof_size_rows
        .iter()
        .filter_map(|row| row["population_proof_records"].as_u64())
        .sum();
    write_json(
        &out.join("analysis/proof-size-evidence.json"),
        &json!({
            "schema_version":"nationwide-2020-proof-size-evidence-v1",
            "measurement_boundary":"Population proofs are embedded records, so retained bytes are reported at containing tree-file granularity. Boundary and canonical certificate bytes are zero because those proof stages were not run.",
            "totals":{"population_proof_records":total_population_records,
                "tree_file_bytes":total_tree_bytes,"package_manifest_bytes":total_manifest_bytes,
                "boundary_certificate_bytes":0,"canonical_certificate_bytes":0},
            "states":proof_size_rows
        }),
        true,
    )?;
    write_json(
        &out.join("reports/maps/manifest.json"),
        &json!({"schema_version":"nationwide-2020-centroid-map-package-v1","maps":map_rows,
            "claim_boundary":"Cartographic diagnostics based on projected block centroids; not exact district boundary polygons."}),
        true,
    )?;
    fs::write(
        out.join("README.md"),
        "# Nationwide 2020 Operational Evidence Package\n\nInternal release candidate covering all 50 States, 435 connected leaves, and 8,126,956 Census blocks. See `limitations/LIMITATIONS.md` before using any result.\n\nReplay verification from the repository root with:\n\n```text\ncargo run -p bisect-ops -- verify-national-trees\ncargo run -p bisect-ops -- verify-national-release release_staging/nationwide-2020-operational-v1\n```\n",
    )?;
    fs::write(
        out.join("limitations/LIMITATIONS.md"),
        "# Limitations And Non-Claims\n\n- Boundary and canonical optimality are unproved at all 385 nontrivial nodes.\n- Forty legacy packages do not retain matching historical builder bytes; see the custody disposition.\n- Successful-screen and State wall-clock runtimes were not retained.\n- Maps are block-centroid diagnostics, not exact dissolved district boundaries.\n- This bounded multi-seed operational run is not the single-seed NRS v0.1 conformance benchmark.\n- The package does not establish VRA compliance, partisan fairness, legal validity, court admissibility, official adoption, or public-release approval.\n",
    )?;
    fs::write(
        out.join("review/L1_REVIEW.md"),
        "# Internal L1 Review\n\nStatus: release-candidate pending human L2 review.\n\nThe required BISECT-EVIDENCE-PACKAGE-v1 layout and fields are present. Assignment, context, tree, verifier, analysis, report, map, limitation, and review artifacts are hash-bound. DATUM/SCALE/COMMONS/VAULT public review remains open; this package has not been externally published.\n",
    )?;
    let artifact_paths = release_files(out)?;
    let artifacts: Vec<Value> = artifact_paths
        .iter()
        .map(|path| {
            Ok(json!({
                "path":release_relative(out,path)?,"sha256":sha256(path)?
            }))
        })
        .collect::<Result<_>>()?;
    let current_exe = std::env::current_exe()?;
    let manifest = json!({
        "contract_id":"BISECT-EVIDENCE-PACKAGE-v1",
        "label":"nationwide-2020-operational-v1","year":2020,
        "scope":{"states":inventory["states"].as_array().context("states")?.iter().map(|row|row["state"].clone()).collect::<Vec<_>>(),"chamber":"U.S. House","release_subset":"all-50-states-2020-operational"},
        "created_at":created_at,"bisect_version":env!("CARGO_PKG_VERSION"),"git_commit":git_commit,
        "working_tree_status":"clean","build_features":[],
        "bisect_ops_binary_sha256":sha256(&current_exe)?,
        "metis_engine":{"refinement":"metis","engine_identity":"not-retained-in-historical-tree-artifacts"},
        "command_lines":[
            format!("cargo run -p bisect-ops -- build-national-release --created-at {created_at}"),
            "cargo run -p bisect-ops -- verify-national-trees",
            "cargo run -p bisect-ops -- verify-national-release release_staging/nationwide-2020-operational-v1"
        ],
        "config_path":"config/operational-profile.json","config_sha256":sha256(&config_path)?,
        "source_data":{"family":"2020 Census TIGER/Line tabulation blocks and PL 94-171","year":2020,"custody_status":"local-hash-bound-not-redistributed","national_rctx_verification_sha256":sha256(Path::new("docs/experiments/nationwide-2020/rctx-verification.json"))?},
        "artifacts":artifacts,"verification_status":"pass","claim_status":"release-candidate",
        "limitations":["limitations/LIMITATIONS.md","review/BUILDER_CUSTODY_DISPOSITION.md"],
        "non_claims":["exact boundary optimality","canonical tie optimality","NRS v0.1 conformance","VRA compliance","partisan fairness","legal or official adoption","external publication approval"],
        "supersedes":null
    });
    write_json(&out.join("MANIFEST.json"), &manifest, true)?;
    let mut hashes = String::new();
    for path in release_files(out)? {
        if path.file_name().and_then(|value| value.to_str()) == Some("HASHES.sha256") {
            continue;
        }
        hashes.push_str(&format!(
            "{}  {}\n",
            sha256(&path)?,
            release_relative(out, &path)?
        ));
    }
    fs::write(out.join("HASHES.sha256"), hashes)?;
    verify_national_release(out)?;
    println!("National release candidate: VERIFIED ({})", out.display());
    Ok(())
}

fn verify_national_release(bundle: &Path) -> Result<()> {
    for required in [
        "README.md",
        "MANIFEST.json",
        "HASHES.sha256",
        "config",
        "runs",
        "analysis",
        "reports",
        "review",
        "limitations",
    ] {
        if !bundle.join(required).exists() {
            bail!("release bundle missing {required}");
        }
    }
    let manifest = read_json(&bundle.join("MANIFEST.json"))?;
    let required_fields = [
        "contract_id",
        "label",
        "year",
        "scope",
        "created_at",
        "bisect_version",
        "git_commit",
        "working_tree_status",
        "build_features",
        "metis_engine",
        "command_lines",
        "config_path",
        "config_sha256",
        "source_data",
        "artifacts",
        "verification_status",
        "claim_status",
        "limitations",
        "non_claims",
        "supersedes",
    ];
    for field in required_fields {
        if manifest.get(field).is_none() {
            bail!("release manifest missing {field}");
        }
    }
    if manifest["contract_id"] != "BISECT-EVIDENCE-PACKAGE-v1"
        || manifest["verification_status"] != "pass"
        || manifest["claim_status"] != "release-candidate"
        || manifest["working_tree_status"] != "clean"
    {
        bail!("release manifest vocabulary or posture mismatch");
    }
    if sha256(&bundle.join(manifest["config_path"].as_str().context("config path")?))?
        != manifest["config_sha256"]
    {
        bail!("release config hash mismatch");
    }
    for artifact in manifest["artifacts"].as_array().context("artifacts")? {
        let path = bundle.join(artifact["path"].as_str().context("artifact path")?);
        if !path.is_file() || sha256(&path)? != artifact["sha256"] {
            bail!("release artifact hash mismatch: {}", path.display());
        }
    }
    let expected_hashes: BTreeMap<_, _> = fs::read_to_string(bundle.join("HASHES.sha256"))?
        .lines()
        .map(|line| {
            let (hash, path) = line.split_once("  ").context("HASHES line")?;
            Ok((path.to_owned(), hash.to_owned()))
        })
        .collect::<Result<_>>()?;
    let unhashed_files = release_files(bundle)?
        .into_iter()
        .filter(|path| path.file_name().and_then(|value| value.to_str()) != Some("HASHES.sha256"))
        .count();
    if expected_hashes.len() != unhashed_files {
        bail!("HASHES contains duplicate, missing, or extra entries");
    }
    for path in release_files(bundle)? {
        if path.file_name().and_then(|value| value.to_str()) == Some("HASHES.sha256") {
            continue;
        }
        let relative = release_relative(bundle, &path)?;
        if expected_hashes.get(&relative) != Some(&sha256(&path)?) {
            bail!("HASHES mismatch for {relative}");
        }
    }
    let index = read_json(&bundle.join("runs/state-package-index.json"))?;
    if index["states"].as_array().map_or(0, Vec::len) != 50 {
        bail!("release State index is incomplete");
    }
    let map_manifest = read_json(&bundle.join("reports/maps/manifest.json"))?;
    if map_manifest["maps"].as_array().map_or(0, Vec::len) != 50 {
        bail!("release map set is incomplete");
    }
    for map in map_manifest["maps"].as_array().unwrap() {
        let bytes = fs::read(bundle.join(map["path"].as_str().context("map path")?))?;
        if bytes.len() < 1_000 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            bail!("release map is not a nontrivial PNG");
        }
    }
    println!("National release package verification: PASS");
    Ok(())
}

fn component_count(adjacency: &[Vec<usize>]) -> usize {
    let mut seen = vec![false; adjacency.len()];
    let mut count = 0;
    for start in 0..adjacency.len() {
        if seen[start] {
            continue;
        }
        count += 1;
        seen[start] = true;
        let mut stack = vec![start];
        while let Some(unit) = stack.pop() {
            for &neighbor in &adjacency[unit] {
                if !seen[neighbor] {
                    seen[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
    }
    count
}

fn compare_rctx(left_path: &Path, right_path: &Path) -> Result<()> {
    let left = read_json(left_path)?;
    let right = read_json(right_path)?;
    for field in ["units", "populations", "graph"] {
        if left[field] != right[field] {
            bail!("RCTX parity mismatch in {field}");
        }
    }
    let adjacency = left
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    let directed_edges: usize = adjacency
        .iter()
        .map(|neighbors| neighbors.as_array().map_or(0, Vec::len))
        .sum();
    let directed_bridges = adjacency
        .iter()
        .flat_map(|neighbors| neighbors.as_array().into_iter().flatten())
        .filter(|edge| edge["kind"] == "bridge")
        .count();
    println!(
        "RCTX parity: PASS ({} units, {} edges, {} bridges)",
        adjacency.len(),
        directed_edges / 2,
        directed_bridges / 2
    );
    Ok(())
}

fn verify_ri_frontier(manifest_path: &Path, check_rctx: bool) -> Result<()> {
    let manifest = read_json(manifest_path)?;
    if manifest["schema_version"] != "certified-recursive-ri-frontier-package-v1" {
        bail!("unsupported RI certified frontier package");
    }
    let builder = PathBuf::from(manifest["builder_path"].as_str().context("builder_path")?);
    if sha256(&custody_source(&builder))? != manifest["builder_sha256"] {
        bail!("RI frontier builder hash mismatch");
    }
    let parent = manifest_path.parent().context("manifest parent")?;
    let report_path = parent.join(
        manifest["files"][0]["path"]
            .as_str()
            .context("report path")?,
    );
    if sha256(&report_path)? != manifest["files"][0]["sha256"] {
        bail!("RI frontier report hash mismatch");
    }
    let report = read_json(&report_path)?;
    if report["schema_version"] != "certified-recursive-ri-block-rctx-frontier-v1"
        || report["status"] != "blocked"
        || report["graph"]["unit_count"] != 25_649
        || report["graph"]["land_component_count"] != 2
        || report["graph"]["bridge_edge_count"] != 64
        || report["graph"]["final_component_count"] != 1
    {
        bail!("RI block graph frontier drift");
    }
    let components = report["graph"]["land_components"]
        .as_array()
        .context("land components")?;
    let component_units: u64 = components
        .iter()
        .map(|component| component["unit_count"].as_u64().unwrap_or(0))
        .sum();
    let component_population: i64 = components
        .iter()
        .map(|component| component["population"].as_i64().unwrap_or(0))
        .sum();
    if component_units != 25_649 || component_population != 1_097_379 {
        bail!("RI component totals mismatch");
    }
    if check_rctx {
        let rctx_path = PathBuf::from(report["rctx"]["path"].as_str().context("RCTX path")?);
        if sha256(&rctx_path)? != report["rctx"]["sha256"] {
            bail!("RI local RCTX custody mismatch");
        }
        let rctx = read_json(&rctx_path)?;
        let projection = json!({
            "units":rctx["units"],
            "graph":rctx["graph"],
            "populations":rctx["populations"],
            "source_hashes":rctx["source_hashes"]
        });
        if rctx["context_hash"] != canonical_hash(&projection)? {
            bail!("RI local RCTX context hash mismatch");
        }
    }
    println!("RI block RCTX frontier verification: PASS");
    Ok(())
}

fn verify_exact_frontier(manifest_path: &Path, check_sources: bool) -> Result<()> {
    let manifest = read_json(manifest_path)?;
    if manifest["schema_version"] != "exact-canonical-small-state-frontier-package-v1"
        || manifest["status"] != "blocked-real-data-frontier"
    {
        bail!("unsupported exact frontier package posture");
    }
    let analyzer = PathBuf::from(
        manifest["analyzer_path"]
            .as_str()
            .context("analyzer path")?,
    );
    if sha256(&custody_source(&analyzer))? != manifest["analyzer_sha256"] {
        bail!("frontier analyzer source hash mismatch");
    }
    let files = manifest["files"].as_array().context("package files")?;
    if files.len() != 1 {
        bail!("frontier package file inventory mismatch");
    }
    let report_path = manifest_path
        .parent()
        .context("manifest parent")?
        .join(files[0]["path"].as_str().context("report path")?);
    if sha256(&report_path)? != files[0]["sha256"] {
        bail!("frontier report hash mismatch");
    }
    let report = read_json(&report_path)?;
    let observed = &report["observed_instance"];
    let exact = &report["exact_reference"];
    if report["schema_version"] != "exact-canonical-small-state-frontier-v1"
        || report["status"] != "blocked"
        || exact["state_unit_count"] != observed["tiger_block_rows"]
        || exact["state_unit_count"] != 25_649
        || exact["unit_limit"] != 24
        || exact["units_above_limit"] != 25_625
        || exact["candidate_formula"] != "2^25648-1"
        || exact["candidate_decimal_digits"] != 7_721
        || exact["candidate_log10"] != "7720.817328789789694841975171893797"
        || exact["reference_limit_candidates"] != 8_388_607
        || exact["candidate_ratio_to_reference_log10"] != "7713.893638941290067081418590000000"
        || exact["years_at_one_billion_candidates_per_second_log10"]
            != "7704.318224822704467174690638532167"
        || observed["tiger_block_rows"] != observed["pl_block_rows"]
        || observed["positive_population_blocks"].as_u64().unwrap_or(0)
            + observed["zero_population_blocks"].as_u64().unwrap_or(0)
            != observed["pl_block_rows"].as_u64().unwrap_or(u64::MAX)
    {
        bail!("exact frontier arithmetic or posture drift");
    }
    if check_sources {
        for record in report["source_files"].as_array().context("source files")? {
            let path = Path::new(record["path"].as_str().context("source path")?);
            if fs::metadata(path)?.len() != record["bytes"].as_u64().context("source bytes")?
                || sha256(path)? != record["sha256"]
            {
                bail!("frontier source custody mismatch: {}", path.display());
            }
        }
        let shape =
            Path::new("data/2020/tiger/blocks/tl_2020_44_tabblock20/tl_2020_44_tabblock20.shp");
        let geo = Path::new("data/2020/redistricting/ri2020.pl/rigeo2020.pl");
        let population = Path::new("data/2020/redistricting/ri2020.pl/ri000012020.pl");
        let blocks = read_tiger_blocks_projected(shape)?;
        let populations = read_pl94_block_populations(geo, population)?;
        let block_ids: Vec<_> = blocks.iter().map(|block| &block.geoid).collect();
        let population_ids: Vec<_> = populations.iter().map(|record| &record.geoid).collect();
        let positive = populations
            .iter()
            .filter(|record| record.population > 0)
            .count();
        let zero = populations
            .iter()
            .filter(|record| record.population == 0)
            .count();
        let total: i64 = populations.iter().map(|record| record.population).sum();
        if blocks.len() != 25_649
            || block_ids != population_ids
            || positive != 21_382
            || zero != 4_267
            || total != 1_097_379
        {
            bail!("exact frontier current-source analysis drift");
        }
        let reference = Path::new(
            exact["reference_source"]
                .as_str()
                .context("reference source")?,
        );
        if sha256(reference)? != exact["reference_source_sha256"] {
            bail!("exact frontier Rust reference hash mismatch");
        }
    }
    println!("Small-State exact frontier package verification: PASS");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_state_rctx(
    year: u16,
    state_code: &str,
    state_fips: &str,
    state_name: &str,
    shapefile: &Path,
    tiger_archive: Option<&Path>,
    pl_geo: &Path,
    pl_population: &Path,
    rctx_path: &Path,
    report_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    let state_code = state_code.to_uppercase();
    let lower = state_code.to_lowercase();
    println!("{state_code}: reading and projecting TIGER blocks");
    let (blocks, tiger_shapefiles) = read_tiger_block_bundle(shapefile, year)?;
    println!("{state_code}: reading PL94 populations");
    let populations = read_pl94_block_populations_for_year(pl_geo, pl_population, year)?;
    let block_geoids: Vec<_> = blocks.iter().map(|block| block.geoid.clone()).collect();
    let population_geoids: Vec<_> = populations
        .iter()
        .map(|record| record.geoid.clone())
        .collect();
    if block_geoids != population_geoids {
        bail!("{state_code}: TIGER/PL block GEOID mismatch");
    }
    let population_values: Vec<_> = populations.iter().map(|record| record.population).collect();
    let geometry: Vec<_> = blocks
        .iter()
        .map(|block| block.geometry_wkb.clone())
        .collect();
    println!("{state_code}: building exact shared-boundary adjacency");
    let graph = build_adjacency_graph(&geometry, 1e-6)?;
    let land_component_count = component_count(&graph.adjacency);
    let mut land_weights: Vec<i64> = graph
        .edge_weights
        .values()
        .map(|metres| (metres * 1000.0).round_ties_even().max(1.0) as i64)
        .collect();
    land_weights.sort_unstable();
    let median_land_weight = *land_weights
        .get(land_weights.len() / 2)
        .context("state has no land-boundary edges")?;
    let centroids: Vec<_> = blocks.iter().map(|block| block.centroid).collect();
    let bridges = connect_island_components(&graph.adjacency, &centroids, &block_geoids);
    let mut adjacency: Vec<Vec<Value>> = vec![Vec::new(); blocks.len()];
    for (&(left, right), metres) in &graph.edge_weights {
        let weight = (metres * 1000.0).round_ties_even().max(1.0);
        adjacency[left].push(json!({"to":right,"kind":"boundary","weight":weight}));
        adjacency[right].push(json!({"to":left,"kind":"boundary","weight":weight}));
    }
    for &(left, right) in &bridges {
        let weight = median_land_weight as f64;
        adjacency[left].push(json!({"to":right,"kind":"bridge","weight":weight}));
        adjacency[right].push(json!({"to":left,"kind":"bridge","weight":weight}));
    }
    for neighbors in &mut adjacency {
        neighbors.sort_by_key(|edge| edge["to"].as_u64().unwrap_or(u64::MAX));
    }
    let final_adjacency: Vec<Vec<usize>> = adjacency
        .iter()
        .map(|neighbors| {
            neighbors
                .iter()
                .filter_map(|edge| edge["to"].as_u64().map(|value| value as usize))
                .collect()
        })
        .collect();
    let final_component_count = component_count(&final_adjacency);
    if final_component_count != 1 {
        bail!("{state_code}: block graph remains disconnected");
    }

    let mut units = json!({
        "unit_kind":"block",
        "state":state_code,
        "year":year,
        "canonical_order":"sorted-geoid",
        "unit_ids":block_geoids,
        "source_id":format!("{lower}-{year}-tiger-pl-block-county-bridged-adjacency")
    });
    units["unit_universe_hash"] = json!(canonical_hash(&units)?);
    let adjacent_source = Path::new("crates/bisect-data/src/adjacency.rs");
    let bridge_source = Path::new("crates/bisect-data/src/bridge.rs");
    let tiger_source = Path::new("crates/bisect-data/src/tiger.rs");
    let pl94_source = Path::new("crates/bisect-data/src/pl94.rs");
    let projection_source = Path::new("crates/bisect-data/src/projection.rs");
    let mut tiger_component_files = Vec::with_capacity(tiger_shapefiles.len() * 3);
    for tiger_shapefile in &tiger_shapefiles {
        for extension in ["shp", "dbf", "shx"] {
            let component = tiger_shapefile.with_extension(extension);
            if !component.is_file() {
                bail!(
                    "{state_code}: TIGER shapefile component is missing: {}",
                    component.display()
                );
            }
            tiger_component_files.push(component);
        }
    }
    tiger_component_files.sort_by_key(|path| portable_path(path));
    let tiger_archives = tiger_archive
        .map(|path| files_with_extension(path, "zip"))
        .transpose()?
        .unwrap_or_default();
    if tiger_archive.is_some() && tiger_archives.is_empty() {
        bail!("{state_code}: TIGER archive input contains no ZIP files");
    }
    let mut source_hashes = json!({
        "pl_geo":format!("sha256:{}",sha256(pl_geo)?),
        "pl_population":format!("sha256:{}",sha256(pl_population)?),
        "bridge_rule_source":format!("sha256:{}",sha256(bridge_source)?),
        "bridge_weight_rule_source":format!("sha256:{}",sha256(adjacent_source)?),
        "tiger_reader_source":format!("sha256:{}",sha256(tiger_source)?),
        "pl94_reader_source":format!("sha256:{}",sha256(pl94_source)?),
        "projection_source":format!("sha256:{}",sha256(projection_source)?),
    });
    if shapefile.is_file() {
        source_hashes["tiger_block_shp"] = json!(format!("sha256:{}", sha256(shapefile)?));
        source_hashes["tiger_block_dbf"] = json!(format!(
            "sha256:{}",
            sha256(&shapefile.with_extension("dbf"))?
        ));
        source_hashes["tiger_block_shx"] = json!(format!(
            "sha256:{}",
            sha256(&shapefile.with_extension("shx"))?
        ));
    } else {
        let hashes = source_hashes
            .as_object_mut()
            .context("source hashes must be an object")?;
        for component in &tiger_component_files {
            hashes.insert(
                format!("tiger_block_file:{}", portable_path(component)),
                json!(format!("sha256:{}", sha256(component)?)),
            );
        }
    }
    if tiger_archives.len() == 1 && tiger_archive.is_some_and(Path::is_file) {
        source_hashes["tiger_archive"] = json!(format!("sha256:{}", sha256(&tiger_archives[0])?));
    } else if !tiger_archives.is_empty() {
        let hashes = source_hashes
            .as_object_mut()
            .context("source hashes must be an object")?;
        for archive in &tiger_archives {
            hashes.insert(
                format!("tiger_archive_file:{}", portable_path(archive)),
                json!(format!("sha256:{}", sha256(archive)?)),
            );
        }
    }
    let projection = json!({
        "units":units,
        "graph":{"edge_semantics":"undirected","adjacency":adjacency},
        "populations":population_values,
        "source_hashes":source_hashes
    });
    let mut rctx = projection.clone();
    rctx["rctx_version"] = json!("0.1");
    rctx["context_hash"] = json!(canonical_hash(&projection)?);
    write_json(rctx_path, &rctx, false)?;

    let claim = "Hash-bound connected block context built by Rust; not a district certificate.";
    let report = json!({
        "schema_version":"certified-state-block-rctx-v1",
        "status":"ready",
        "state":state_name,
        "state_code":state_code,
        "state_fips":state_fips,
        "year":year,
        "rctx_path":portable_path(rctx_path),
        "rctx_bytes":fs::metadata(rctx_path)?.len(),
        "rctx_sha256":sha256(rctx_path)?,
        "context_hash":rctx["context_hash"],
        "unit_universe_hash":rctx["units"]["unit_universe_hash"],
        "unit_count":blocks.len(),
        "population_total":population_values.iter().sum::<i64>(),
        "land_edge_count":graph.n_edges,
        "land_component_count":land_component_count,
        "bridge_edge_count":bridges.len(),
        "final_component_count":final_component_count,
        "geometry_toolchain":{"implementation":"bisect-data","language":"rust","crs":"EPSG:5070"},
        "tiger_input_path":portable_path(shapefile),
        "tiger_source_file_count":tiger_component_files.len(),
        "tiger_archive_path":tiger_archive.filter(|path| path.is_file()).map(portable_path),
        "tiger_archive_sha256":tiger_archive.filter(|path| path.is_file()).map(sha256).transpose()?,
        "tiger_archive_files":hashed_source_map(&tiger_archives)?,
        "claim_boundary":claim
    });
    write_json(report_path, &report, true)?;
    let parent = manifest_path.parent().context("manifest parent")?;
    let (source, source_hash) =
        write_content_addressed_source_snapshot(parent, "bisect-ops-rctx-builder-source")?;
    let source_name = source
        .file_name()
        .context("source filename")?
        .to_string_lossy();
    let report_name = report_path
        .file_name()
        .context("report filename")?
        .to_string_lossy();
    let executable = std::env::current_exe().context("resolve RCTX builder executable")?;
    let executable_hash = sha256(&executable)?;
    let manifest = json!({
        "schema_version":"certified-state-block-rctx-package-v1",
        "package_id":format!("{lower}-{year}-block-rctx"),
        "year":year,
        "tiger_input_path":portable_path(shapefile),
        "tiger_source_file_count":tiger_component_files.len(),
        "tiger_archive_path":tiger_archive.filter(|path| path.is_file()).map(portable_path),
        "tiger_archive_sha256":tiger_archive.filter(|path| path.is_file()).map(sha256).transpose()?,
        "tiger_archive_files":hashed_source_map(&tiger_archives)?,
        "status":"ready",
        "files":[{"path":report_name,"sha256":sha256(report_path)?}],
        "builder_path":source_name,
        "builder_sha256":source_hash,
        "builder_executable_path":portable_path(&executable),
        "builder_executable_sha256":executable_hash,
        "claim_boundary":claim
    });
    write_json(manifest_path, &manifest, true)?;
    println!(
        "{state_code}: {} blocks, {} land edges, {} bridges",
        blocks.len(),
        graph.n_edges,
        bridges.len()
    );
    Ok(())
}

fn rctx_batch(
    year: u16,
    inventory_path: Option<&Path>,
    workers: usize,
    limit: Option<usize>,
) -> Result<()> {
    if workers == 0 {
        bail!("workers must be positive");
    }
    let suffix = match year {
        2000 => "00",
        2010 => "10",
        2020 => "20",
        _ => bail!("RCTX batch currently supports census years 2000, 2010, and 2020"),
    };
    let root = std::env::current_dir()?;
    let inventory_path = inventory_path
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(format!("docs/experiments/nationwide-{year}/inventory.json")));
    let inventory = read_json(&inventory_path)?;
    if inventory["state_count"] != 50 || inventory["district_count"] != 435 {
        bail!("national {year} inventory must bind 50 States and 435 districts");
    }
    let rows = inventory["states"].as_array().context("states")?;
    let by_state: BTreeMap<_, _> = rows
        .iter()
        .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
        .collect();
    let mut pending: Vec<Value> = inventory["batch_order"]
        .as_array()
        .context("batch_order")?
        .iter()
        .filter_map(|state| by_state.get(state.as_str()?).cloned())
        .filter(|row| {
            !root
                .join(format!(
                    "data/{year}/certified/{}_blocks_{year}.rctx",
                    row["state"].as_str().unwrap().to_lowercase()
                ))
                .is_file()
        })
        .collect();
    if let Some(limit) = limit {
        pending.truncate(limit);
    }
    let report_root = root.join(format!("docs/experiments/nationwide-{year}"));
    let ledger_path = report_root.join("rctx-build-ledger.json");
    if pending.is_empty() {
        let ledger = read_json(&ledger_path)?;
        println!(
            "National RCTX batch: {} built, {} failed, {} remaining",
            ledger["built_count"], ledger["failed_count"], ledger["remaining_count"]
        );
        return Ok(());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()?;
    let results:Vec<Value>=pool.install(||pending.par_iter().map(|row|{
        let state=row["state"].as_str().unwrap(); let lower=state.to_lowercase(); let state_name=row["name"].as_str().unwrap_or(state).to_lowercase().replace(' ',"_");
        let fips=row["fips"].as_str().unwrap_or("");
        let tiger_year=if year == 2000 { 2010 } else { year };
        let (shapefile, tiger_archive, pl_geo, pl_population) = if year == 2000 {
            let geography=root.join(format!("data/2000/redistricting/{lower}geo.upl"));
            (root.join(format!("data/2000/tiger/blocks/{lower}")), Some(root.join(format!("data/2000/tiger/archives/{lower}"))), geography.clone(), geography)
        } else {
            let pl_dir=root.join(format!("data/{year}/redistricting/{lower}{year}.pl"));
            let archive=root.join(format!("data/{year}/tiger/archives/tl_{tiger_year}_{fips}_tabblock{suffix}.zip"));
            (root.join(format!("data/{year}/tiger/blocks/tl_{tiger_year}_{fips}_tabblock{suffix}/tl_{tiger_year}_{fips}_tabblock{suffix}.shp")), archive.is_file().then_some(archive), pl_dir.join(format!("{lower}geo{year}.pl")), pl_dir.join(format!("{lower}00001{year}.pl")))
        };
        let rctx=root.join(format!("data/{year}/certified/{lower}_blocks_{year}.rctx"));
        let report=report_root.join(format!("rctx/{lower}.json"));
        let manifest=report_root.join(format!("rctx/{lower}-manifest.json"));
        let tiger_archive=tiger_archive.filter(|path| path.exists());
        let result=build_state_rctx(year,state,fips,&state_name,&shapefile,tiger_archive.as_deref(),&pl_geo,&pl_population,&rctx,&report,&manifest);
        match result { Ok(())=>json!({"state":state,"year":year,"block_count":row["block_count"],"status":"built","exit_code":0,"command":["bisect-ops","build-state-rctx",format!("--year={year}")],"output":"Rust-native state RCTX build completed."}),Err(error)=>json!({"state":state,"year":year,"block_count":row["block_count"],"status":"failed","exit_code":1,"command":["bisect-ops","build-state-rctx",format!("--year={year}")],"output":format!("{error:#}")}) }
    }).collect());
    let mut merged: BTreeMap<String, Value> = if ledger_path.is_file() {
        read_json(&ledger_path)?["results"]
            .as_array()
            .context("results")?
            .iter()
            .filter_map(|row| Some((row["state"].as_str()?.into(), row.clone())))
            .collect()
    } else {
        BTreeMap::new()
    };
    for row in results {
        println!(
            "{}: {}",
            row["state"].as_str().unwrap(),
            row["status"].as_str().unwrap()
        );
        merged.insert(row["state"].as_str().unwrap().into(), row);
    }
    let results: Vec<_> = merged.values().cloned().collect();
    let built = results.iter().filter(|v| v["status"] == "built").count();
    let failed = results.iter().filter(|v| v["status"] == "failed").count();
    let remaining = inventory["batch_order"]
        .as_array()
        .unwrap()
        .len()
        .saturating_sub(results.len());
    let ledger = json!({"schema_version":"certified-national-rctx-build-ledger-v1","year":year,"inventory_path":inventory_path.to_string_lossy().replace('\\',"/"),"results":results,"built_count":built,"failed_count":failed,"remaining_count":remaining,"claim_boundary":"Resumable engineering ledger; aggregate verification occurs after all State contexts exist."});
    write_json(&ledger_path, &ledger, true)?;
    println!("National RCTX batch: {built} built, {failed} failed, {remaining} remaining");
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::Build {
            bisect,
            context,
            out_dir,
            districts,
            root_seed,
            child_seed_0,
            child_seed_1,
            max_seed,
        } => {
            if max_seed == 0 {
                bail!("max-seed must be positive");
            }
            build(
                &bisect,
                &context,
                &out_dir,
                districts,
                root_seed,
                [child_seed_0, child_seed_1],
                max_seed,
            )
        }
        Action::Verify { package } => {
            let manifest = read_json(&package.join("manifest.json"))?;
            let builder = PathBuf::from(manifest["builder_path"].as_str().context("builder path")?);
            let builder_source = if builder.components().count() == 1 {
                package.join(&builder)
            } else {
                custody_source(&builder)
            };
            if sha256(&builder_source)?
                != manifest["builder_sha256"]
                    .as_str()
                    .context("builder hash")?
            {
                bail!("operational tree builder hash mismatch");
            }
            if let Some(base) = manifest["base_builder_path"].as_str() {
                if sha256(&custody_source(Path::new(base)))?
                    != manifest["base_builder_sha256"]
                        .as_str()
                        .context("base builder hash")?
                {
                    bail!("operational tree base-builder hash mismatch");
                }
            }
            let tree_path =
                package.join(manifest["files"][0]["path"].as_str().context("tree path")?);
            if sha256(&tree_path)?
                != manifest["files"][0]["sha256"]
                    .as_str()
                    .context("tree hash")?
            {
                bail!("operational tree hash mismatch");
            }
            let tree = read_json(&tree_path)?;
            let districts = tree["districts"].as_u64().context("districts")?;
            let assignment = tree["assignment"].as_array().context("assignment")?;
            let unit_count = tree["unit_count"].as_u64().context("unit_count")?;
            let labels: BTreeSet<_> = assignment.iter().filter_map(Value::as_u64).collect();
            let expected: BTreeSet<_> = (0..districts).collect();
            let leaf_units: u64 = tree["leaves"]
                .as_array()
                .context("leaves")?
                .iter()
                .map(|leaf| leaf["unit_count"].as_u64().unwrap_or(0))
                .sum();
            let leaf_population: i64 = tree["leaves"]
                .as_array()
                .unwrap()
                .iter()
                .map(|leaf| leaf["population"].as_i64().unwrap_or(0))
                .sum();
            let floors_match = tree["nodes"]
                .as_array()
                .context("nodes")?
                .iter()
                .all(|node| {
                    node.pointer("/objective/max_population_deviation_scaled")
                        == node.pointer("/population_proof/lower_bound")
                });
            if districts < 2
                || assignment.len() as u64 != unit_count
                || labels != expected
                || leaf_units != unit_count
                || leaf_population != tree["population_total"].as_i64().unwrap_or(i64::MIN)
                || !floors_match
            {
                bail!("operational tree coverage drift");
            }
            println!("Operational recursive tree package verification: PASS");
            Ok(())
        }
        Action::Batch {
            bisect,
            limit,
            retry_failed,
            max_seed,
        } => batch(&bisect, limit, retry_failed, max_seed),
        Action::AuditPython { staged, base } => audit_python(staged, base.as_deref()),
        Action::AnalyzeTree {
            state,
            package,
            rctx_report,
            report,
            manifest,
        } => analyze_tree(&state, &package, &rctx_report, &report, &manifest),
        Action::VerifyTreeReport { manifest } => verify_tree_report(&manifest),
        Action::VerifyNationalRctx {
            year,
            out_dir,
            context_root,
            require_complete,
        } => {
            let out_dir = out_dir
                .unwrap_or_else(|| PathBuf::from(format!("docs/experiments/nationwide-{year}")));
            let context_root =
                context_root.unwrap_or_else(|| PathBuf::from(format!("data/{year}/certified")));
            verify_national_rctx(year, &out_dir, &context_root, require_complete)
        }
        Action::VerifyNationalTrees {
            out_dir,
            package_root,
            context_root,
            one_district,
        } => verify_national_trees(&out_dir, &package_root, &context_root, &one_district),
        Action::BuildNationalRelease {
            out_dir,
            created_at,
        } => build_national_release(&out_dir, &created_at),
        Action::VerifyNationalRelease { bundle } => verify_national_release(&bundle),
        Action::NrsSeed {
            context,
            districts,
            standard_profile,
            legal_profile,
            out_dir,
            generated_at,
        } => build_nrs_seed_package(
            &context,
            districts,
            &standard_profile,
            &legal_profile,
            &out_dir,
            &generated_at,
        ),
        Action::VerifyNrsSeed { package, context } => verify_nrs_seed_package(&package, &context),
        Action::BuildNrsState {
            bisect,
            context,
            districts,
            seed_package,
            out_dir,
            generated_at,
        } => build_nrs_state(
            &bisect,
            &context,
            districts,
            &seed_package,
            &out_dir,
            &generated_at,
        ),
        Action::VerifyNrsState { package, context } => verify_nrs_state(&package, &context),
        Action::NrsBatch {
            year,
            bisect,
            inventory,
            standard_profile,
            legal_profile,
            out_dir,
            generated_at,
            limit,
            states,
            retry_failed,
        } => {
            let profiles =
                resolve_nrs_profiles(year, standard_profile.as_deref(), legal_profile.as_deref());
            nrs_batch(
                year,
                &bisect,
                &inventory,
                &profiles.0,
                &profiles.1,
                &out_dir,
                &generated_at,
                limit,
                &states,
                retry_failed,
            )
        }
        Action::VerifyNrsBatch {
            year,
            inventory,
            standard_profile,
            legal_profile,
            out_dir,
            require_complete,
        } => {
            let profiles =
                resolve_nrs_profiles(year, standard_profile.as_deref(), legal_profile.as_deref());
            verify_nrs_batch(
                year,
                &inventory,
                &profiles.0,
                &profiles.1,
                &out_dir,
                require_complete,
            )
        }
        Action::SummarizeNrsBatch {
            year,
            inventory,
            standard_profile,
            legal_profile,
            out_dir,
            report_dir,
        } => {
            let profiles =
                resolve_nrs_profiles(year, standard_profile.as_deref(), legal_profile.as_deref());
            summarize_nrs_batch(
                year,
                &inventory,
                &profiles.0,
                &profiles.1,
                &out_dir,
                &report_dir,
            )
        }
        Action::RctxBatch {
            year,
            inventory,
            workers,
            limit,
        } => rctx_batch(year, inventory.as_deref(), workers, limit),
        Action::BuildStateRctx {
            year,
            state_code,
            state_fips,
            state_name,
            shapefile,
            tiger_archive,
            pl_geo,
            pl_population,
            rctx,
            report,
            manifest,
        } => build_state_rctx(
            year,
            &state_code,
            &state_fips,
            &state_name,
            &shapefile,
            tiger_archive.as_deref(),
            &pl_geo,
            &pl_population,
            &rctx,
            &report,
            &manifest,
        ),
        Action::CompareRctx { left, right } => compare_rctx(&left, &right),
        Action::VerifyRiFrontier {
            manifest,
            check_rctx,
        } => verify_ri_frontier(&manifest, check_rctx),
        Action::VerifyExactFrontier {
            manifest,
            check_sources,
        } => verify_exact_frontier(&manifest, check_sources),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_context() -> Value {
        json!({
            "rctx_version":"0.1","context_hash":"old",
            "units":{"unit_ids":["a","b","c"],"source_id":"fixture","state":"XY"},
            "graph":{"edge_semantics":"undirected","adjacency":[
                [{"to":1,"weight":4}],
                [{"to":0,"weight":4},{"to":2,"weight":5}],
                [{"to":1,"weight":5}]
            ]},
            "populations":[10,20,30],"source_hashes":{}
        })
    }

    #[test]
    fn arithmetic_floor_matches_ratio_bound() {
        assert_eq!(ratio_floor(101, 3, 2), 1);
        assert_eq!(ratio_floor(100, 2, 1), 0);
    }

    #[test]
    fn nrs_generation_tolerance_uses_smaller_child_target_and_ceiling() {
        assert_eq!(nrs_generation_tolerance_scaled_bound(100, 1), 1);
        assert_eq!(nrs_generation_tolerance_scaled_bound(1_000, 1), 5);
        assert_eq!(nrs_generation_tolerance_scaled_bound(1_000, 2), 10);
    }

    #[test]
    fn nrs_batch_retries_windows_sharing_violations_only() {
        let locked = anyhow::Error::new(std::io::Error::from_raw_os_error(32));
        let ordinary =
            anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
        assert!(is_transient_file_lock(&locked));
        assert!(!is_transient_file_lock(&ordinary));
    }

    #[test]
    fn subset_remaps_edges_and_units() {
        let subset = subset_context(&tiny_context(), &[1, 2], "child".into()).unwrap();
        assert_eq!(subset["units"]["unit_ids"], json!(["b", "c"]));
        assert_eq!(subset["populations"], json!([20, 30]));
        assert_eq!(subset["graph"]["adjacency"][0][0]["to"], 1);
        assert!(subset["context_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
    }

    #[test]
    fn connectivity_detects_split_components() {
        let context = tiny_context();
        assert!(connected(&context, &[0, 0, 1], 0).unwrap());
        assert!(!connected(&context, &[0, 1, 0], 0).unwrap());
    }

    #[test]
    fn national_connectivity_rejects_disconnected_district() {
        let context = tiny_context();
        verify_connected_assignment(&context, &[0, 0, 1], 2).unwrap();
        assert!(verify_connected_assignment(&context, &[0, 1, 0], 2).is_err());
    }

    #[test]
    fn recursive_schedule_checks_odd_split_tree() {
        let tree = json!({
            "districts":3,
            "nodes":[{"path":"","seats":3},{"path":"1","seats":2}],
            "leaves":[
                {"path":"0","district":0},
                {"path":"10","district":1},
                {"path":"11","district":2}
            ]
        });
        verify_recursive_schedule(&tree).unwrap();
        let mut broken = tree;
        broken["leaves"][2]["path"] = json!("10");
        assert!(verify_recursive_schedule(&broken).is_err());
    }

    #[test]
    fn centroid_release_map_is_nontrivial_png() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("map.png");
        let centroids = vec![
            ("000000000000001".into(), (0.0, 0.0)),
            ("000000000000002".into(), (1.0, 0.0)),
            ("000000000000003".into(), (0.0, 1.0)),
            ("000000000000004".into(), (1.0, 1.0)),
        ];
        let ids = vec![
            json!("000000000000001"),
            json!("000000000000002"),
            json!("000000000000003"),
            json!("000000000000004"),
        ];
        let metadata = render_centroid_map(&centroids, &ids, &[0, 0, 1, 1], &destination).unwrap();
        let bytes = fs::read(destination).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(bytes.len() > 1_000);
        assert_eq!(metadata["block_count"], 4);
    }

    #[test]
    fn screening_count_tolerates_unretained_legacy_history() {
        let tree = json!({"nodes":[
            {"path":"","seed":1},
            {"path":"0","seed_screening":[
                {"status":"completed"},{"status":"timeout"},{"status":"completed"}
            ]}
        ]});
        assert_eq!(count_screening(&tree).unwrap(), (2, 1, 1, 1));
    }

    #[test]
    fn nrs_seed_vector_001_matches_specification() {
        let manifest = json!({
            "adjacency_sha256":"00","algorithm_profile_sha256":"11",
            "canonicalization_version":"canonical-json-v1","census_release":"test",
            "district_count":2,"geographic_vintage":"test","legal_profile_sha256":"22",
            "population_sha256":"33","reference_engine_sha256":"44",
            "unit_index_sha256":"55"
        });
        let (digest, seed, engine_seed) = nrs_seed(&manifest).unwrap();
        assert_eq!(
            digest,
            "e50326ede53a03cd59ffe98bb95ff04e784ad607bdd242ebda4f927b0decf690"
        );
        assert_eq!(seed, 14_772_715_961_905_972_197);
        assert_eq!(engine_seed, (seed % 2_147_483_647) as u32);
    }

    #[test]
    fn nrs_seed_canonicalizes_object_key_order_and_whitespace() {
        let left: Value = serde_json::from_str("{\"z\":1,\"a\":2}").unwrap();
        let right: Value = serde_json::from_str(" { \"a\" : 2, \"z\" : 1 } ").unwrap();
        assert_eq!(
            serde_json::to_vec(&left).unwrap(),
            serde_json::to_vec(&right).unwrap()
        );
        assert_eq!(nrs_seed(&left).unwrap(), nrs_seed(&right).unwrap());
    }

    #[test]
    fn tiger_bundle_inventory_is_recursive_sorted_and_extension_filtered() {
        let temp = tempfile::tempdir().unwrap();
        let county = temp.path().join("county");
        fs::create_dir(&county).unwrap();
        fs::write(temp.path().join("b.SHP"), b"").unwrap();
        fs::write(county.join("a.shp"), b"").unwrap();
        fs::write(county.join("a.dbf"), b"").unwrap();
        let files = files_with_extension(temp.path(), "shp").unwrap();
        assert_eq!(files, vec![temp.path().join("b.SHP"), county.join("a.shp")]);
    }

    #[test]
    fn source_custody_paths_reject_escape_and_absolute_paths() {
        let root = Path::new("workspace");
        assert_eq!(
            governed_source_path(root, "data/2000/source.zip").unwrap(),
            root.join("data/2000/source.zip")
        );
        assert!(governed_source_path(root, "../outside.zip").is_err());
        assert!(governed_source_path(root, "C:/outside.zip").is_err());
    }

    #[test]
    fn committed_ri_frontier_verifies_without_local_data() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("docs/experiments/certified-recursive/manifest.json");
        verify_ri_frontier(&manifest, false).unwrap();
    }
}

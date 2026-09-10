use anyhow::{bail, Context};
use clap::Subcommand;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct CeremonyArgs {
    #[command(subcommand)]
    pub command: CeremonySubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CeremonySubcommand {
    /// Publish the frozen rule and create an open genesis transcript.
    Start(CeremonyStartArgs),
    /// Publish the next complete bisection depth after the review interval.
    Announce(CeremonyAnnounceArgs),
    /// Permanently halt an open ceremony and retain its accepted prefix.
    Halt(CeremonyHaltArgs),
    /// Verify a transcript, its hash chain, schedule, timing, and tree binding.
    Verify(CeremonyVerifyArgs),
}

#[derive(Debug, clap::Args)]
pub struct CeremonyStartArgs {
    /// Complete certified-bisection-tree JSON used to bind the ceremony.
    #[arg(long)]
    pub tree: PathBuf,
    /// Frozen rule-profile file; its SHA-256 is committed at genesis.
    #[arg(long)]
    pub rule_profile: PathBuf,
    /// Public genesis timestamp, expressed as Unix seconds.
    #[arg(long)]
    pub genesis_unix_seconds: i64,
    /// Minimum delay before root publication and between later rounds.
    #[arg(long)]
    pub review_seconds: u64,
    /// Minimum number of distinct receipts required at every publication.
    #[arg(long, default_value_t = 2)]
    pub minimum_witnesses: usize,
    /// External receipt identifier or URL. Repeat once per witness.
    #[arg(long = "witness", required = true)]
    pub witnesses: Vec<String>,
    /// New immutable transcript JSON path.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct CeremonyAnnounceArgs {
    #[arg(long)]
    pub tree: PathBuf,
    /// Previously published open transcript.
    #[arg(long)]
    pub transcript: PathBuf,
    /// Public timestamp for this round, expressed as Unix seconds.
    #[arg(long)]
    pub published_unix_seconds: i64,
    /// External receipt identifier or URL. Repeat once per witness.
    #[arg(long = "witness", required = true)]
    pub witnesses: Vec<String>,
    /// New immutable transcript JSON path; the input is never overwritten.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct CeremonyHaltArgs {
    #[arg(long)]
    pub tree: PathBuf,
    #[arg(long)]
    pub transcript: PathBuf,
    /// Public reason the ceremony cannot continue.
    #[arg(long)]
    pub reason: String,
    /// New immutable halted-transcript JSON path.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct CeremonyVerifyArgs {
    /// Complete tree for final binding. Omit while verifying an open prefix.
    #[arg(long)]
    pub tree: Option<PathBuf>,
    #[arg(long)]
    pub transcript: PathBuf,
}

pub fn run_ceremony(command: &CeremonySubcommand) -> anyhow::Result<()> {
    match command {
        CeremonySubcommand::Start(args) => run_start(args),
        CeremonySubcommand::Announce(args) => run_announce(args),
        CeremonySubcommand::Halt(args) => run_halt(args),
        CeremonySubcommand::Verify(args) => run_verify(args),
    }
}

fn run_start(args: &CeremonyStartArgs) -> anyhow::Result<()> {
    let tree = read_tree(&args.tree)?;
    let root_instance_hash = tree
        .nodes
        .first()
        .context("certified tree has no root node")?
        .instance
        .hash()?;
    let config = bisect_ilp::CeremonyConfig {
        rule_profile_hash: format!("sha256:{}", bisect_report::sha256_file(&args.rule_profile)?),
        root_instance_hash,
        district_count: tree.k,
        genesis_unix_seconds: args.genesis_unix_seconds,
        minimum_review_seconds: args.review_seconds,
        minimum_witness_receipts: args.minimum_witnesses,
        genesis_witness_receipts: args.witnesses.clone(),
    };
    let transcript = bisect_ilp::start_ceremony(&tree, config)?;
    write_new_json(&args.out, &transcript)?;
    println!(
        "[PASS] ceremony started: id={} transcript={} next_round=0",
        transcript.ceremony_id, transcript.transcript_id
    );
    Ok(())
}

fn run_announce(args: &CeremonyAnnounceArgs) -> anyhow::Result<()> {
    let tree = read_tree(&args.tree)?;
    let mut transcript = read_transcript(&args.transcript)?;
    bisect_ilp::announce_next_round(
        &mut transcript,
        &tree,
        args.published_unix_seconds,
        args.witnesses.clone(),
    )?;
    write_new_json(&args.out, &transcript)?;
    let released = transcript
        .rounds
        .last()
        .context("announcement produced no round")?;
    println!(
        "[PASS] ceremony round {} announced: cuts={} status={:?} transcript={}",
        released.round_index,
        released.cuts.len(),
        transcript.status,
        transcript.transcript_id
    );
    Ok(())
}

fn run_halt(args: &CeremonyHaltArgs) -> anyhow::Result<()> {
    let tree = read_tree(&args.tree)?;
    let mut transcript = read_transcript(&args.transcript)?;
    bisect_ilp::halt_ceremony(&mut transcript, &tree, args.reason.clone())?;
    write_new_json(&args.out, &transcript)?;
    println!(
        "[PASS] ceremony halted: rounds={} transcript={}",
        transcript.rounds.len(),
        transcript.transcript_id
    );
    Ok(())
}

fn run_verify(args: &CeremonyVerifyArgs) -> anyhow::Result<()> {
    let transcript = read_transcript(&args.transcript)?;
    if let Some(tree_path) = &args.tree {
        let tree = read_tree(tree_path)?;
        bisect_ilp::verify_ceremony(&transcript, &tree)?;
    } else {
        bisect_ilp::verify_ceremony_prefix(&transcript)?;
        if transcript.status == bisect_ilp::CeremonyStatus::Completed {
            bail!("completed ceremony verification requires --tree for final binding");
        }
    }
    println!(
        "[PASS] ceremony verified: status={:?} rounds={} transcript={}",
        transcript.status,
        transcript.rounds.len(),
        transcript.transcript_id
    );
    Ok(())
}

fn read_tree(path: &Path) -> anyhow::Result<bisect_ilp::CertifiedBisectionTree> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read certified tree {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse certified tree {}", path.display()))
}

fn read_transcript(path: &Path) -> anyhow::Result<bisect_ilp::CeremonyTranscript> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("read ceremony transcript {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse ceremony transcript {}", path.display()))
}

/// Persist an immutable public artifact without exposing a partial JSON file.
fn write_new_json<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if path.exists() {
        bail!(
            "refusing to overwrite immutable ceremony artifact {}",
            path.display()
        );
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create ceremony output directory {}", parent.display()))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("create temporary ceremony artifact in {}", parent.display()))?;
    serde_json::to_writer_pretty(temporary.as_file_mut(), value)?;
    temporary.as_file_mut().write_all(b"\n")?;
    temporary.as_file_mut().sync_all()?;
    temporary
        .persist_noclobber(path)
        .map_err(|error| error.error)
        .with_context(|| format!("publish immutable ceremony artifact {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture_tree(path: &Path) {
        let unit_ids = (0..8).map(|unit| format!("u{unit:02}")).collect::<Vec<_>>();
        let (k_left, k_right) = bisect_ilp::canonical_seat_split(4).unwrap();
        let root = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1; 8],
            edges: (0..7)
                .map(|left| bisect_ilp::ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
            k_parent: 4,
            k_left,
            k_right,
            orientation_rule: bisect_ilp::canonical_orientation_rule(k_left, k_right),
        };
        let tree = bisect_ilp::solve_certified_bisection_tree_bounded(root).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&tree).unwrap()).unwrap();
    }

    #[test]
    fn immutable_writer_refuses_overwrite() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("transcript.json");
        write_new_json(&path, &serde_json::json!({"version": 1})).unwrap();
        let error = write_new_json(&path, &serde_json::json!({"version": 2})).unwrap_err();
        assert!(error.to_string().contains("refusing to overwrite"));
        let stored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(stored["version"], 1);
    }

    #[test]
    fn command_lifecycle_starts_announces_and_verifies() {
        let directory = tempfile::tempdir().unwrap();
        let tree_path = directory.path().join("tree.json");
        let profile_path = directory.path().join("rule.json");
        let genesis_path = directory.path().join("00-genesis.json");
        let round_zero_path = directory.path().join("01-root.json");
        let completed_path = directory.path().join("02-complete.json");
        write_fixture_tree(&tree_path);
        std::fs::write(&profile_path, b"{\"rule\":\"certified-bisection\"}").unwrap();

        run_start(&CeremonyStartArgs {
            tree: tree_path.clone(),
            rule_profile: profile_path,
            genesis_unix_seconds: 1_000,
            review_seconds: 100,
            minimum_witnesses: 1,
            witnesses: vec!["log:genesis".into()],
            out: genesis_path.clone(),
        })
        .unwrap();
        run_announce(&CeremonyAnnounceArgs {
            tree: tree_path.clone(),
            transcript: genesis_path,
            published_unix_seconds: 1_100,
            witnesses: vec!["log:root".into()],
            out: round_zero_path.clone(),
        })
        .unwrap();
        run_verify(&CeremonyVerifyArgs {
            tree: None,
            transcript: round_zero_path.clone(),
        })
        .unwrap();
        run_announce(&CeremonyAnnounceArgs {
            tree: tree_path.clone(),
            transcript: round_zero_path,
            published_unix_seconds: 1_200,
            witnesses: vec!["log:depth-1".into()],
            out: completed_path.clone(),
        })
        .unwrap();
        run_verify(&CeremonyVerifyArgs {
            tree: Some(tree_path),
            transcript: completed_path.clone(),
        })
        .unwrap();

        let transcript = read_transcript(&completed_path).unwrap();
        assert_eq!(transcript.status, bisect_ilp::CeremonyStatus::Completed);
        assert_eq!(transcript.rounds.len(), 2);
    }
}

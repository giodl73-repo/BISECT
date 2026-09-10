//! Public, staged release protocol for a certified bisection tree.
//!
//! A ceremony publishes all certified cuts at one tree depth as a round.  A
//! configured review interval must pass before the next depth may be
//! published.  The verifier checks ordering, certificate bindings, the hash
//! chain, and elapsed time.  It cannot prove that a claimed timestamp or
//! witness receipt was externally published; that requires an external
//! transparency log or trusted timestamp service.

use crate::{
    certified_split::canonical_hash, certified_tree::derive_child_instance,
    verify_certified_bisection_tree_bounded, verify_certified_split_bounded,
    CertifiedBisectionTree, CertifiedSplitArtifacts, CertifiedSplitError, CertifiedSplitResult,
    CertifiedTreeError,
};
use bisect_core::BisectionTree;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const CERTIFIED_BISECTION_CEREMONY_SCHEMA_VERSION: &str = "certified-bisection-ceremony-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyConfig {
    pub rule_profile_hash: String,
    pub root_instance_hash: String,
    pub district_count: usize,
    pub genesis_unix_seconds: i64,
    pub minimum_review_seconds: u64,
    pub minimum_witness_receipts: usize,
    /// Receipts proving that the rule and clock origin were published before
    /// the first cut announcement.
    pub genesis_witness_receipts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutAnnouncement {
    pub node_path: String,
    pub instance_hash: String,
    pub certificate_id: String,
    /// Self-contained evidence for this cut; future descendants stay absent.
    pub artifacts: CertifiedSplitArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyRound {
    pub round_index: usize,
    pub published_unix_seconds: i64,
    pub previous_round_id: Option<String>,
    pub cuts: Vec<CutAnnouncement>,
    /// Opaque externally verifiable receipt identifiers or URLs.
    pub witness_receipts: Vec<String>,
    pub round_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CeremonyStatus {
    Open,
    Completed,
    Halted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyTranscript {
    pub schema_version: String,
    pub ceremony_id: String,
    pub config: CeremonyConfig,
    pub status: CeremonyStatus,
    pub halt_reason: Option<String>,
    pub tree_id: Option<String>,
    pub rounds: Vec<CeremonyRound>,
    pub transcript_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CeremonyError {
    #[error(transparent)]
    Split(#[from] CertifiedSplitError),
    #[error(transparent)]
    Tree(#[from] CertifiedTreeError),
    #[error("unsupported ceremony schema: {0}")]
    Schema(String),
    #[error("ceremony configuration is invalid")]
    InvalidConfig,
    #[error("ceremony id mismatch")]
    CeremonyIdMismatch,
    #[error("transcript id mismatch")]
    TranscriptIdMismatch,
    #[error("ceremony does not bind to the supplied certified tree")]
    TreeBindingMismatch,
    #[error("ceremony rounds are not the canonical depth prefix")]
    RoundScheduleMismatch,
    #[error("ceremony round hash chain is invalid")]
    RoundChainMismatch,
    #[error("ceremony review interval has not elapsed")]
    ReviewIntervalViolation,
    #[error("ceremony round has insufficient distinct witness receipts")]
    InsufficientWitnessReceipts,
    #[error("ceremony status and released rounds disagree")]
    StatusMismatch,
    #[error("halted ceremonies require a reason; other statuses prohibit one")]
    HaltReasonMismatch,
}

impl CeremonyConfig {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        canonical_hash(self)
    }
}

impl CeremonyRound {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            round_index: usize,
            published_unix_seconds: i64,
            previous_round_id: &'a Option<String>,
            cuts: &'a [CutAnnouncement],
            witness_receipts: &'a [String],
        }
        canonical_hash(&Projection {
            round_index: self.round_index,
            published_unix_seconds: self.published_unix_seconds,
            previous_round_id: &self.previous_round_id,
            cuts: &self.cuts,
            witness_receipts: &self.witness_receipts,
        })
    }
}

impl CeremonyTranscript {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            ceremony_id: &'a str,
            config: &'a CeremonyConfig,
            status: CeremonyStatus,
            halt_reason: &'a Option<String>,
            tree_id: &'a Option<String>,
            rounds: &'a [CeremonyRound],
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            ceremony_id: &self.ceremony_id,
            config: &self.config,
            status: self.status,
            halt_reason: &self.halt_reason,
            tree_id: &self.tree_id,
            rounds: &self.rounds,
        })
    }
}

/// Start a publicly committed ceremony before any cut has been announced.
pub fn start_ceremony(
    tree: &CertifiedBisectionTree,
    config: CeremonyConfig,
) -> Result<CeremonyTranscript, CeremonyError> {
    verify_certified_bisection_tree_bounded(tree)?;
    let ceremony_id = config.compute_id()?;
    let mut transcript = CeremonyTranscript {
        schema_version: CERTIFIED_BISECTION_CEREMONY_SCHEMA_VERSION.to_string(),
        ceremony_id,
        config,
        status: CeremonyStatus::Open,
        halt_reason: None,
        tree_id: None,
        rounds: Vec::new(),
        transcript_id: String::new(),
    };
    transcript.transcript_id = transcript.compute_id()?;
    verify_ceremony(&transcript, tree)?;
    Ok(transcript)
}

/// Publish the next complete tree depth after its review interval.
///
/// The update is transactional: `transcript` changes only if the proposed
/// round and resulting transcript pass full verification.
pub fn announce_next_round(
    transcript: &mut CeremonyTranscript,
    tree: &CertifiedBisectionTree,
    published_unix_seconds: i64,
    witness_receipts: Vec<String>,
) -> Result<(), CeremonyError> {
    verify_ceremony(transcript, tree)?;
    if transcript.status != CeremonyStatus::Open {
        return Err(CeremonyError::StatusMismatch);
    }
    let groups = expected_round_cuts(tree)?;
    let round_index = transcript.rounds.len();
    let cuts = groups
        .get(round_index)
        .cloned()
        .ok_or(CeremonyError::StatusMismatch)?;
    let previous_round_id = transcript.rounds.last().map(|round| round.round_id.clone());
    let mut round = CeremonyRound {
        round_index,
        published_unix_seconds,
        previous_round_id,
        cuts,
        witness_receipts,
        round_id: String::new(),
    };
    round.round_id = round.compute_id()?;

    let mut candidate = transcript.clone();
    candidate.rounds.push(round);
    if candidate.rounds.len() == groups.len() {
        candidate.status = CeremonyStatus::Completed;
        candidate.tree_id = Some(tree.tree_id.clone());
    }
    candidate.transcript_id = candidate.compute_id()?;
    verify_ceremony(&candidate, tree)?;
    *transcript = candidate;
    Ok(())
}

/// Permanently halt an open ceremony while preserving its accepted prefix.
/// Resumption requires a new ceremony.
pub fn halt_ceremony(
    transcript: &mut CeremonyTranscript,
    tree: &CertifiedBisectionTree,
    reason: String,
) -> Result<(), CeremonyError> {
    verify_ceremony(transcript, tree)?;
    if transcript.status != CeremonyStatus::Open || reason.trim().is_empty() {
        return Err(CeremonyError::HaltReasonMismatch);
    }
    let mut candidate = transcript.clone();
    candidate.status = CeremonyStatus::Halted;
    candidate.halt_reason = Some(reason);
    candidate.transcript_id = candidate.compute_id()?;
    verify_ceremony(&candidate, tree)?;
    *transcript = candidate;
    Ok(())
}

/// Build a completed transcript from an already-certified tree.
///
/// `published_unix_seconds` and `witness_receipts` contain one entry per tree
/// depth.  Production callers should obtain both from external publication
/// infrastructure rather than manufacturing them locally.
pub fn build_completed_ceremony(
    tree: &CertifiedBisectionTree,
    config: CeremonyConfig,
    published_unix_seconds: &[i64],
    witness_receipts: &[Vec<String>],
) -> Result<CeremonyTranscript, CeremonyError> {
    let groups = expected_round_cuts(tree)?;
    if groups.len() != published_unix_seconds.len() || groups.len() != witness_receipts.len() {
        return Err(CeremonyError::RoundScheduleMismatch);
    }
    let mut transcript = start_ceremony(tree, config)?;
    for round_index in 0..groups.len() {
        announce_next_round(
            &mut transcript,
            tree,
            published_unix_seconds[round_index],
            witness_receipts[round_index].clone(),
        )?;
    }
    Ok(transcript)
}

pub fn verify_ceremony(
    transcript: &CeremonyTranscript,
    tree: &CertifiedBisectionTree,
) -> Result<(), CeremonyError> {
    verify_ceremony_prefix(transcript)?;
    verify_certified_bisection_tree_bounded(tree)?;
    let root_hash = tree
        .nodes
        .first()
        .ok_or(CeremonyError::TreeBindingMismatch)?
        .instance
        .hash()?;
    if transcript.config.root_instance_hash != root_hash
        || transcript.config.district_count != tree.k
    {
        return Err(CeremonyError::TreeBindingMismatch);
    }
    let expected = expected_round_cuts(tree)?;
    if transcript.rounds.len() > expected.len()
        || transcript
            .rounds
            .iter()
            .zip(expected.iter())
            .any(|(round, cuts)| round.cuts != *cuts)
    {
        return Err(CeremonyError::RoundScheduleMismatch);
    }
    if transcript.status == CeremonyStatus::Completed
        && transcript.tree_id.as_deref() != Some(tree.tree_id.as_str())
    {
        return Err(CeremonyError::TreeBindingMismatch);
    }
    Ok(())
}

/// Verify all public evidence disclosed so far without seeing future cuts.
pub fn verify_ceremony_prefix(transcript: &CeremonyTranscript) -> Result<(), CeremonyError> {
    if transcript.schema_version != CERTIFIED_BISECTION_CEREMONY_SCHEMA_VERSION {
        return Err(CeremonyError::Schema(transcript.schema_version.clone()));
    }
    if transcript.config.rule_profile_hash.trim().is_empty()
        || transcript.config.root_instance_hash.trim().is_empty()
        || transcript.config.district_count < 2
        || transcript.config.minimum_review_seconds == 0
        || transcript.config.minimum_witness_receipts == 0
    {
        return Err(CeremonyError::InvalidConfig);
    }
    validate_receipts(
        &transcript.config.genesis_witness_receipts,
        transcript.config.minimum_witness_receipts,
    )?;
    if transcript.ceremony_id != transcript.config.compute_id()? {
        return Err(CeremonyError::CeremonyIdMismatch);
    }
    if transcript.transcript_id != transcript.compute_id()? {
        return Err(CeremonyError::TranscriptIdMismatch);
    }

    let schedule = BisectionTree::from_k(transcript.config.district_count);
    let expected_paths = schedule_round_paths(&schedule);
    if transcript.rounds.len() > expected_paths.len() {
        return Err(CeremonyError::RoundScheduleMismatch);
    }
    let schedule_by_path = schedule
        .nodes
        .iter()
        .map(|node| (node.path.as_str(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut released: std::collections::BTreeMap<String, &CertifiedSplitArtifacts> =
        std::collections::BTreeMap::new();
    for (index, round) in transcript.rounds.iter().enumerate() {
        let submitted_paths = round
            .cuts
            .iter()
            .map(|cut| cut.node_path.as_str())
            .collect::<Vec<_>>();
        if round.round_index != index || submitted_paths != expected_paths[index] {
            return Err(CeremonyError::RoundScheduleMismatch);
        }
        let expected_previous = index
            .checked_sub(1)
            .map(|previous| transcript.rounds[previous].round_id.clone());
        if round.previous_round_id != expected_previous || round.round_id != round.compute_id()? {
            return Err(CeremonyError::RoundChainMismatch);
        }
        validate_receipts(
            &round.witness_receipts,
            transcript.config.minimum_witness_receipts,
        )?;
        let review_seconds = i64::try_from(transcript.config.minimum_review_seconds)
            .map_err(|_| CeremonyError::ReviewIntervalViolation)?;
        let earliest = if index == 0 {
            transcript
                .config
                .genesis_unix_seconds
                .checked_add(review_seconds)
                .ok_or(CeremonyError::ReviewIntervalViolation)?
        } else {
            transcript.rounds[index - 1]
                .published_unix_seconds
                .checked_add(review_seconds)
                .ok_or(CeremonyError::ReviewIntervalViolation)?
        };
        if round.published_unix_seconds < earliest {
            return Err(CeremonyError::ReviewIntervalViolation);
        }
        for cut in &round.cuts {
            let artifacts = &cut.artifacts;
            let path = artifacts.instance.node_path.as_str();
            let schedule_node = schedule_by_path
                .get(path)
                .ok_or(CeremonyError::RoundScheduleMismatch)?;
            if cut.node_path != path
                || cut.instance_hash != artifacts.instance.hash()?
                || cut.certificate_id != artifacts.certificate.certificate_id
                || artifacts.instance.k_parent != schedule_node.k
                || artifacts.instance.k_left != schedule_node.k_left
                || artifacts.instance.k_right != schedule_node.k_right
            {
                return Err(CeremonyError::RoundScheduleMismatch);
            }
            verify_certified_split_bounded(
                &artifacts.instance,
                &artifacts.certificate,
                &artifacts.proof,
            )?;
            if path.is_empty() {
                if artifacts.instance.hash()? != transcript.config.root_instance_hash {
                    return Err(CeremonyError::TreeBindingMismatch);
                }
            } else {
                let (parent_path, label_text) = path.split_at(path.len() - 1);
                let label = if label_text == "0" { 0 } else { 1 };
                let parent = released
                    .get(parent_path)
                    .ok_or(CeremonyError::RoundScheduleMismatch)?;
                let CertifiedSplitResult::Optimal { assignment, .. } = &parent.certificate.result
                else {
                    return Err(CeremonyError::RoundScheduleMismatch);
                };
                let expected_child = derive_child_instance(
                    &parent.instance,
                    assignment,
                    label,
                    artifacts.instance.k_parent,
                    path.to_string(),
                    parent.certificate.certificate_id.clone(),
                )?;
                if artifacts.instance != expected_child {
                    return Err(CeremonyError::TreeBindingMismatch);
                }
            }
            released.insert(path.to_string(), artifacts);
        }
    }

    match transcript.status {
        CeremonyStatus::Completed => {
            if transcript.rounds.len() != expected_paths.len() || transcript.tree_id.is_none() {
                return Err(CeremonyError::StatusMismatch);
            }
        }
        CeremonyStatus::Open => {
            if transcript.rounds.len() >= expected_paths.len() || transcript.tree_id.is_some() {
                return Err(CeremonyError::StatusMismatch);
            }
        }
        CeremonyStatus::Halted => {
            if transcript.tree_id.is_some() {
                return Err(CeremonyError::StatusMismatch);
            }
        }
    }
    let has_reason = transcript
        .halt_reason
        .as_ref()
        .is_some_and(|reason| !reason.trim().is_empty());
    if has_reason != (transcript.status == CeremonyStatus::Halted) {
        return Err(CeremonyError::HaltReasonMismatch);
    }
    Ok(())
}

fn validate_receipts(receipts: &[String], minimum: usize) -> Result<(), CeremonyError> {
    let distinct = receipts
        .iter()
        .map(|receipt| receipt.trim())
        .filter(|receipt| !receipt.is_empty())
        .collect::<BTreeSet<_>>();
    if distinct.len() < minimum {
        return Err(CeremonyError::InsufficientWitnessReceipts);
    }
    Ok(())
}

fn expected_round_cuts(
    tree: &CertifiedBisectionTree,
) -> Result<Vec<Vec<CutAnnouncement>>, CertifiedSplitError> {
    let max_depth = tree
        .nodes
        .iter()
        .map(|node| node.instance.node_path.len())
        .max()
        .unwrap_or(0);
    let mut groups = Vec::with_capacity(max_depth + 1);
    for depth in 0..=max_depth {
        let mut cuts = tree
            .nodes
            .iter()
            .filter(|node| node.instance.node_path.len() == depth)
            .map(|node| {
                Ok(CutAnnouncement {
                    node_path: node.instance.node_path.clone(),
                    instance_hash: node.instance.hash()?,
                    certificate_id: node.certificate.certificate_id.clone(),
                    artifacts: node.clone(),
                })
            })
            .collect::<Result<Vec<_>, CertifiedSplitError>>()?;
        cuts.sort_by(|left, right| left.node_path.cmp(&right.node_path));
        if !cuts.is_empty() {
            groups.push(cuts);
        }
    }
    Ok(groups)
}

fn schedule_round_paths(schedule: &BisectionTree) -> Vec<Vec<&str>> {
    let max_depth = schedule
        .nodes
        .iter()
        .map(|node| node.path.len())
        .max()
        .unwrap_or(0);
    (0..=max_depth)
        .filter_map(|depth| {
            let paths = schedule
                .nodes
                .iter()
                .filter(|node| node.path.len() == depth)
                .map(|node| node.path.as_str())
                .collect::<Vec<_>>();
            (!paths.is_empty()).then_some(paths)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        canonical_orientation_rule, canonical_seat_split, certified_split_unit_universe_hash,
        solve_certified_bisection_tree_bounded, CertifiedSplitInstance, ExactEdge,
        CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION, CERTIFIED_SPLIT_MODEL_ID,
    };

    fn tree() -> CertifiedBisectionTree {
        let unit_ids = (0..8).map(|unit| format!("u{unit:02}")).collect::<Vec<_>>();
        let (k_left, k_right) = canonical_seat_split(4).unwrap();
        solve_certified_bisection_tree_bounded(CertifiedSplitInstance {
            schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1; 8],
            edges: (0..7)
                .map(|left| ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
            k_parent: 4,
            k_left,
            k_right,
            orientation_rule: canonical_orientation_rule(k_left, k_right),
        })
        .unwrap()
    }

    fn config(tree: &CertifiedBisectionTree) -> CeremonyConfig {
        CeremonyConfig {
            rule_profile_hash: "sha256:rule".to_string(),
            root_instance_hash: tree.nodes[0].instance.hash().unwrap(),
            district_count: tree.k,
            genesis_unix_seconds: 1_000,
            minimum_review_seconds: 100,
            minimum_witness_receipts: 1,
            genesis_witness_receipts: vec!["log:genesis".into()],
        }
    }

    #[test]
    fn completed_round_ceremony_verifies() {
        let tree = tree();
        let transcript = build_completed_ceremony(
            &tree,
            config(&tree),
            &[1_100, 1_200],
            &[vec!["log:root".into()], vec!["log:depth-1".into()]],
        )
        .unwrap();
        assert_eq!(transcript.rounds[0].cuts.len(), 1);
        assert_eq!(transcript.rounds[1].cuts.len(), 2);
        assert_eq!(verify_ceremony(&transcript, &tree), Ok(()));
    }

    #[test]
    fn ceremony_advances_one_public_round_at_a_time() {
        let tree = tree();
        let mut transcript = start_ceremony(&tree, config(&tree)).unwrap();
        assert_eq!(transcript.status, CeremonyStatus::Open);
        assert!(transcript.rounds.is_empty());

        announce_next_round(&mut transcript, &tree, 1_100, vec!["log:root".into()]).unwrap();
        assert_eq!(transcript.status, CeremonyStatus::Open);
        assert_eq!(transcript.rounds.len(), 1);
        assert!(transcript.tree_id.is_none());

        announce_next_round(&mut transcript, &tree, 1_200, vec!["log:depth-1".into()]).unwrap();
        assert_eq!(transcript.status, CeremonyStatus::Completed);
        assert_eq!(transcript.tree_id.as_deref(), Some(tree.tree_id.as_str()));
    }

    #[test]
    fn failed_announcement_leaves_transcript_unchanged() {
        let tree = tree();
        let mut transcript = start_ceremony(&tree, config(&tree)).unwrap();
        let before = transcript.clone();
        assert_eq!(
            announce_next_round(&mut transcript, &tree, 1_099, vec!["log:early".into()]),
            Err(CeremonyError::ReviewIntervalViolation)
        );
        assert_eq!(transcript, before);
    }

    #[test]
    fn ceremony_rejects_early_descendant_round() {
        let tree = tree();
        let err = build_completed_ceremony(
            &tree,
            config(&tree),
            &[1_100, 1_199],
            &[vec!["log:root".into()], vec!["log:depth-1".into()]],
        )
        .unwrap_err();
        assert_eq!(err, CeremonyError::ReviewIntervalViolation);
    }

    #[test]
    fn ceremony_rejects_cut_tamper_even_with_rehashed_envelope() {
        let tree = tree();
        let mut transcript = build_completed_ceremony(
            &tree,
            config(&tree),
            &[1_100, 1_200],
            &[vec!["log:root".into()], vec!["log:depth-1".into()]],
        )
        .unwrap();
        transcript.rounds[1].cuts[0].certificate_id = "sha256:substitute".into();
        transcript.rounds[1].round_id = transcript.rounds[1].compute_id().unwrap();
        transcript.transcript_id = transcript.compute_id().unwrap();
        assert_eq!(
            verify_ceremony(&transcript, &tree),
            Err(CeremonyError::RoundScheduleMismatch)
        );
    }

    #[test]
    fn ceremony_rejects_duplicate_witness_receipts() {
        let tree = tree();
        let mut config = config(&tree);
        config.minimum_witness_receipts = 2;
        config.genesis_witness_receipts = vec!["log:a".into(), "log:b".into()];
        let err = build_completed_ceremony(
            &tree,
            config,
            &[1_100, 1_200],
            &[
                vec!["log:same".into(), "log:same".into()],
                vec!["log:a".into(), "log:b".into()],
            ],
        )
        .unwrap_err();
        assert_eq!(err, CeremonyError::InsufficientWitnessReceipts);
    }

    #[test]
    fn ceremony_rejects_whitespace_variants_of_one_witness_receipt() {
        let tree = tree();
        let mut config = config(&tree);
        config.minimum_witness_receipts = 2;
        config.genesis_witness_receipts = vec!["log:a".into(), " log:a ".into()];
        assert_eq!(
            start_ceremony(&tree, config),
            Err(CeremonyError::InsufficientWitnessReceipts)
        );
    }

    #[test]
    fn open_and_halted_transcripts_are_strict_prefixes_without_tree_id() {
        let tree = tree();
        let completed = build_completed_ceremony(
            &tree,
            config(&tree),
            &[1_100, 1_200],
            &[vec!["log:root".into()], vec!["log:depth-1".into()]],
        )
        .unwrap();

        let mut open = completed.clone();
        open.status = CeremonyStatus::Open;
        open.tree_id = None;
        open.rounds.pop();
        open.transcript_id = open.compute_id().unwrap();
        assert_eq!(verify_ceremony(&open, &tree), Ok(()));

        let mut halted = open;
        halted.status = CeremonyStatus::Halted;
        halted.halt_reason = Some("independent verifier rejected the round".into());
        halted.transcript_id = halted.compute_id().unwrap();
        assert_eq!(verify_ceremony(&halted, &tree), Ok(()));
    }

    #[test]
    fn halted_ceremony_preserves_prefix_and_cannot_advance() {
        let tree = tree();
        let mut transcript = start_ceremony(&tree, config(&tree)).unwrap();
        announce_next_round(&mut transcript, &tree, 1_100, vec!["log:root".into()]).unwrap();
        let accepted_round = transcript.rounds[0].clone();
        halt_ceremony(
            &mut transcript,
            &tree,
            "certificate challenge sustained".into(),
        )
        .unwrap();
        assert_eq!(transcript.status, CeremonyStatus::Halted);
        assert_eq!(transcript.rounds, vec![accepted_round]);
        assert_eq!(
            announce_next_round(&mut transcript, &tree, 1_200, vec!["log:forbidden".into()]),
            Err(CeremonyError::StatusMismatch)
        );
    }
}

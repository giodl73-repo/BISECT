//! In-memory adapter around the unchanged upstream VeriPB checking kernel.
use std::{cell::RefCell, io::Cursor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use veripb_checker::{checker::ForwardsChecker, config::CheckerConfig, context::Context, error::ForwardsCheckerError};
use veripb_formula::prelude::{VarNameRecorder, VarIdx};
use veripb_parser::{error::ParserError, opb_parser::parse_opb_reader, pbp_parser::{parser::{parse_proof, ParserArgs}, legacy_v2::parser::Version2Parser, proof_processor::ProofProcessor}};
use veripb_rules::{ConclusionResult, OutputType, VeriPBRule};

const MAX_MODEL: usize = 8 * 1024 * 1024;
const MAX_SOURCE: usize = 3 * 1024 * 1024;
const MAX_PROOF: usize = 64 * 1024 * 1024;
const MAX_REQUEST: usize = 192 * 1024 * 1024;
const COMMIT: &str = "a50f0483d65aa019918d03ccc590b71dc23c666c";
pub mod native_split;
pub mod native_tree;
pub mod tree_export;
pub mod model_prepare;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub opb: String,
    pub proof: String,
    pub opb_sha256: String,
    pub proof_sha256: String,
    pub require_unsat: bool,
}

#[derive(Serialize)]
pub struct Report {
    schema: &'static str,
    status: &'static str,
    conclusion: &'static str,
    opb_sha256: String,
    proof_sha256: String,
    checker: &'static str,
    checker_upstream_commit: &'static str,
    vendor_provenance_sha256: String,
    proof_version: String,
    checked_rules: usize,
    scope: &'static str,
}

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn error_message(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    for _ in 0..4 {
        let Some(error) = source else { break };
        message.push_str(": "); message.push_str(&error.to_string());
        source = error.source();
    }
    message.truncate(message.floor_char_boundary(4096));
    message
}

struct GuardedChecker {
    checker: ForwardsChecker,
    conclusion: Option<&'static str>,
    rules: usize,
}

impl VarNameRecorder for GuardedChecker {
    fn add_by_name(&mut self, name: &str) -> VarIdx { self.checker.add_by_name(name) }
}
impl ProofProcessor for GuardedChecker {
    type Error = ForwardsCheckerError;
    fn process_version(&mut self, major: i64, minor: i64) { self.checker.process_version(major, minor); }
    fn process_var_name(&mut self, line: usize, name: &str, auxiliary: bool) -> Result<usize, Self::Error> { self.checker.process_var_name(line, name, auxiliary) }
    fn last_returned_constraint_id(&mut self) -> Option<isize> { self.checker.last_returned_constraint_id() }
    fn should_trace(&self, _: usize) -> bool { false }
    fn process_rule(&mut self, line: usize, rule: VeriPBRule) -> Result<(), Self::Error> {
        // Browser proofs cannot request filesystem I/O or unsupported native
        // timers. These refusals also apply to the native adapter used in tests.
        let refused = matches!(&rule, VeriPBRule::OutputRule(output) if !matches!(output.output_type, OutputType::None))
            || matches!(&rule, VeriPBRule::StartTimerRule(_) | VeriPBRule::EndTimerRule(_));
        if refused { return Err(ParserError::IOError(std::io::Error::new(std::io::ErrorKind::Unsupported, "External output and timer rules are unavailable in this in-memory checker")).into()); }
        let conclusion = match &rule {
            VeriPBRule::ConclusionRule(value) => Some(match value.result {
                ConclusionResult::Unsatisfiable => "UNSAT",
                ConclusionResult::Satisfiable => "SAT",
                ConclusionResult::Bounds => "BOUNDS",
                ConclusionResult::None => "NONE",
                ConclusionResult::EnumerationComplete => "ENUMERATION_COMPLETE",
                ConclusionResult::EnumerationPartial => "ENUMERATION_PARTIAL",
            }),
            _ => None,
        };
        // Record the conclusion only after the real kernel has accepted it.
        self.checker.process_rule(line, rule)?;
        if conclusion.is_some() { self.conclusion = conclusion; }
        self.rules += 1;
        Ok(())
    }
}

pub fn check(request: Request) -> Result<Report, String> {
    if request.opb.is_empty() || request.opb.len() > MAX_MODEL || request.proof.is_empty() || request.proof.len() > MAX_PROOF || request.opb.contains('\0') || request.proof.contains('\0') { return Err("Nonempty NUL-free model/proof required; limits are 8 MiB and 64 MiB".into()); }
    let opb_sha256 = hash(request.opb.as_bytes());
    let proof_sha256 = hash(request.proof.as_bytes());
    if opb_sha256 != request.opb_sha256 || proof_sha256 != request.proof_sha256 { return Err("Model/proof checksum mismatch before checking".into()); }
    let (formula, variables, labels, preserved) = parse_opb_reader(Cursor::new(request.opb.as_bytes())).map_err(|e| error_message(&e))?;
    let config = CheckerConfig { force_checked_deletion: true, derivation_path: "proof.pbp".into(), ..Default::default() };
    let checker = ForwardsChecker::new(Context::new(config, variables, preserved), formula).map_err(|e| error_message(&e))?;
    let mut checker = GuardedChecker { checker, conclusion: None, rules: 0 };
    let mut reader = Cursor::new(request.proof.as_bytes());
    let parser_args = ParserArgs { show_progress: false, trace_enabled: false, store_assertion_annotations: false };
    if let Err(error) = parse_proof(Some(request.proof.len() as u64), &mut reader, &mut checker, labels, parser_args) {
        match error {
            ForwardsCheckerError::Parse(error) if error.is_version_error() => {
                Version2Parser::new(&mut checker).parse("proof.pbp", reader).map_err(|e| error_message(&e))?;
            }
            error => return Err(error_message(&error)),
        }
    }
    let context = &checker.checker.context;
    if !context.has_end_proof || !context.has_conclusion || context.assertion_used || !context.strong_solution_guarantees { return Err("Complete independently checked proof with strong guarantees required".into()); }
    let conclusion = checker.conclusion.ok_or("Missing checked conclusion")?;
    if request.require_unsat && conclusion != "UNSAT" { return Err("An UNSAT conclusion is required for this decision proof".into()); }
    Ok(Report { schema: "bisect-external-proof-check-v1", status: "proof-checked", conclusion,
        opb_sha256, proof_sha256, checker: "VeriPB 3.0.2", checker_upstream_commit: COMMIT,
        vendor_provenance_sha256: hash(include_bytes!("../../../third_party/veripb/provenance.json")),
        proof_version: format!("{}.{}", context.major_version.unwrap_or(0), context.minor_version.unwrap_or(0)), checked_rules: checker.rules,
        scope: "The supplied formula and proof bytes only. No native request identity, complete plan optimality, source truth or legal compliance is certified." })
}

pub fn execute(bytes: &[u8]) -> Vec<u8> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input { Proof(Request), Adopt(native_split::AdoptRequest), Tree(native_tree::AdoptTreeRequest), Children(native_tree::DeriveChildrenRequest), Export(tree_export::ExportRequest), Prepare(model_prepare::PrepareRequest) }
    let result: Result<serde_json::Value, String> = if bytes.len() > MAX_REQUEST { Err("Request exceeds 192 MiB".into()) }
        else { serde_json::from_slice::<Input>(bytes).map_err(|e| e.to_string()).and_then(|input| match input {
            Input::Proof(request) => check(request).and_then(|report| serde_json::to_value(report).map_err(|e| e.to_string())),
            Input::Adopt(request) => native_split::adopt(request).and_then(|certificate| serde_json::to_value(certificate).map_err(|e| e.to_string())),
            Input::Tree(request) => native_tree::adopt_tree(request).and_then(|certificate| serde_json::to_value(certificate).map_err(|e| e.to_string())),
            Input::Children(request) => native_tree::derive_proved_children(request),
            Input::Export(request) => tree_export::export(request),
            Input::Prepare(request) => model_prepare::prepare(request),
        }) };
    serde_json::to_vec(&match result { Ok(report) => serde_json::json!({"ok": true, "result": report}), Err(error) => serde_json::json!({"ok": false, "error": error}) }).expect("serializable report")
}

thread_local! { static RESPONSE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) }; }
#[unsafe(no_mangle)]
pub extern "C" fn proof_alloc(length: usize) -> *mut u8 {
    if length > MAX_REQUEST { return std::ptr::null_mut(); }
    Box::into_raw(vec![0u8; length].into_boxed_slice()) as *mut u8
}
/// # Safety
/// Pointer/length must identify one live input allocated by this module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn proof_free(pointer: *mut u8, length: usize) {
    unsafe { drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(pointer, length))); }
}
/// # Safety
/// Pointer/length identify live input bytes; returned output is borrowed until next call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn proof_execute(pointer: *const u8, length: usize) -> *const u8 {
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) };
    RESPONSE.with(|response| { *response.borrow_mut() = execute(bytes); response.borrow().as_ptr() })
}
#[unsafe(no_mangle)]
pub extern "C" fn proof_response_length() -> usize { RESPONSE.with(|response| response.borrow().len()) }

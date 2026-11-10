//! Parser for the VeriPB proof format.
//!
//! The parser is designed to be reusable through implementing the [`ProofProcessor`](proof_processor::ProofProcessor) trait. Then the VeriPB parser can be used through calling the function [`parse_proof`](parser::parse_proof()).

pub mod legacy_v2;
pub mod lexer;
pub mod parser;
pub mod proof_processor;
pub mod reader;
pub mod utils;

#![allow(clippy::restriction)]

//! Crate for parsing strings and/or files to VeriPB data structures.
//!
//! For example use of the parsing functions, have a look at `examples`. These can be called using
//! ```bash
//! cargo run -r --example <example_name>
//! ```
//!
//! # Supported Parsers
//! Parsers for the following functionality are implemented in this library:
//! - CNF parser (see [`cnf_parser`])
//! - OPB parser (see [`opb_parser`])
//! - Pseudo-Boolean (VeriPB) proof (see [`pbp_parser`])
//! - WCNF parser (see [`wcnf_parser`])
//!
//! The OPB parser and the proof parser (see [`pbp_parser`]) share the representation of a
//! constraint that is being parsed (see [`terms`]).

pub mod cnf_parser;
pub mod cnf_token;
pub mod error;
pub mod io;
pub mod opb_parser;
pub mod opb_token;
pub mod pbp_parser;
pub mod terms;
pub mod wcnf_parser;
pub mod wcnf_token;

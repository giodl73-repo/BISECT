//! Writer for formatting elaborated proof.
//!
//! This module provides the [`Elaborator`], which provides helpful functions for writing to the elaborated proof.

use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    path::PathBuf,
};

use crate::keywords;

const INDENT: &str = "\t";

/// Writes the elaborated proof and tracks output constraint IDs.
///
/// An important goal of the [`Elaborator`] is to write the proof linearly to the output `file` to make it possible to stream the elaborated proof to another tool. Hence, the cursor for writing should never be moved backwards. This requires buffering elaborated proof steps that might not be needed in the end.
#[derive(Debug)]
pub struct Elaborator {
    file: BufWriter<File>,
    /// When set, writes go to `buffer` instead of `file`.
    use_buffer: bool,
    /// Buffer writing to file if elaborated proof might be discarded.
    buffer: Vec<u8>,
    /// `current_id` snapshot taken when buffering began, restored on forget.
    buffered_id: usize,
    /// ID of the most recently written output constraint.
    pub current_id: usize,
    /// Current indentation depth
    current_indent_level: u8,
}

impl Elaborator {
    /// Construct a new output proof writing to the specified file.
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let mut writer = BufWriter::new(File::create(path)?);
        writer.write_all(keywords::HEADER.as_bytes())?;
        Ok(Elaborator {
            file: writer,
            use_buffer: false,
            buffer: Vec::new(),
            buffered_id: 0,
            current_id: 0,
            current_indent_level: 0,
        })
    }

    /// Write a string to the proof.
    #[inline]
    pub(crate) fn write(&mut self, buf: &str) {
        if self.use_buffer {
            self.buffer.extend(buf.as_bytes());
        } else {
            self.file.write_all(buf.as_bytes()).unwrap();
        }
    }

    /// Write a string to the proof followed by a newline.
    #[inline]
    pub(crate) fn writeln(&mut self, buf: &str) {
        self.write(buf);
        self.end_line();
    }

    /// Write the newline symbol.
    #[inline]
    pub(crate) fn end_line(&mut self) {
        self.write("\n");
    }

    /// Increment the indent level.
    pub(crate) fn inc_indent(&mut self) {
        self.current_indent_level += 1;
    }

    /// Decrement the indent level.
    pub(crate) fn dec_indent(&mut self) {
        debug_assert!(
            self.current_indent_level > 0,
            "indent underflow: dec_indent called at level 0"
        );
        self.current_indent_level -= 1;
    }

    /// Write ident characters corresponding to the current indent level.
    pub(crate) fn write_indent(&mut self) {
        for _ in 0..self.current_indent_level {
            self.write(INDENT);
        }
    }

    /// Write a `Display` value (typically an integer id) to the proof
    /// (avoids allocating a string).
    #[inline]
    pub(crate) fn write_display<T: std::fmt::Display>(&mut self, value: T) {
        if self.use_buffer {
            write!(self.buffer, "{value}").unwrap();
        } else {
            write!(self.file, "{value}").unwrap();
        }
    }

    /// Increment the current ID and write the next ID to to the proof.
    #[inline]
    pub fn write_inc_id(&mut self) {
        self.current_id += 1;
        self.write_display(self.current_id);
    }

    /// Increment the current ID that the `Elaborator` keeps track of and return the new incremented ID.
    #[inline]
    pub fn inc_id(&mut self) -> usize {
        self.current_id += 1;
        self.current_id
    }

    /// Decrement the current ID.
    #[inline]
    pub fn dec_id(&mut self) {
        self.current_id -= 1;
    }

    /// Write the proof to a buffer instead of the file until `write_buffered_proof` is called.
    ///
    /// The advantage of buffering is that no incorrect proof is ever written to the proof file, which might be a problem if the elaborated proof is streamed into the formally verified checker.
    #[inline]
    pub fn enable_buffered_proof(&mut self) {
        self.buffered_id = self.current_id;
        self.use_buffer = true;
    }

    /// Write the buffered proof to the file and reset the buffer.
    ///
    /// The advantage of buffering is that no incorrect proof is ever written to the proof file, which might be a problem if the elaborated proof is streamed into the formally verified checker.
    #[inline]
    pub fn write_buffered_proof(&mut self) {
        self.file.write_all(&self.buffer).unwrap();
        self.disable_buffered_proof();
    }

    /// Forget the buffered proof and reset the elaborator state to before the buffering started. This is especially important for the elaborated constraint ID.
    ///
    /// The advantage of buffering is that no incorrect proof is ever written to the proof file, which might be a problem if the elaborated proof is streamed into the formally verified checker.
    #[inline]
    pub fn forget_buffered_proof(&mut self) {
        self.current_id = self.buffered_id;
        self.disable_buffered_proof();
    }

    /// Disable buffer.
    ///
    /// The advantage of buffering is that no incorrect proof is ever written to the proof file, which might be a problem if the elaborated proof is streamed into the formally verified checker.
    #[inline]
    pub(crate) fn disable_buffered_proof(&mut self) {
        self.use_buffer = false;
        self.buffer.clear();
    }
}

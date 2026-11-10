//! Plain-text file helpers for the browser library distribution.
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

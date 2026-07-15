//! Streaming reader for 2020 Census PL 94-171 block populations.

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use thiserror::Error;

const BLOCK_SUMLEV: &[u8] = b"750";
const BLOCK_GEOID_PREFIX: &[u8] = b"7500000US";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pl94BlockPopulation {
    /// 15-character block GEOID: state(2) + county(3) + tract(6) + block(4).
    pub geoid: String,
    pub population: i64,
}

#[derive(Debug, Error)]
pub enum Pl94Error {
    #[error("PL 94-171 I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("malformed {file} line {line}: {message}")]
    Malformed {
        file: &'static str,
        line: usize,
        message: String,
    },
    #[error("duplicate block LOGRECNO {0}")]
    DuplicateLogrecno(String),
    #[error("population file has no record for block LOGRECNO {0}")]
    MissingPopulation(String),
}

/// Join block geography records to total population records by LOGRECNO.
///
/// Both inputs are read as bytes because PL files are Latin-1, while the
/// selected identifiers and numeric fields are ASCII. Results are sorted by
/// GEOID for deterministic TIGER joins.
pub fn read_pl94_block_populations<G: AsRef<Path>, P: AsRef<Path>>(
    geo_path: G,
    population_path: P,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    let geo = BufReader::new(File::open(geo_path)?);
    let population = BufReader::new(File::open(population_path)?);
    parse_pl94_block_populations(geo, population)
}

fn parse_pl94_block_populations<G: BufRead, P: BufRead>(
    geo: G,
    population: P,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    let mut blocks: HashMap<Vec<u8>, (String, Option<i64>)> = HashMap::new();
    for (line_index, line) in geo.split(b'\n').enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        let fields: Vec<_> = line.split(|byte| *byte == b'|').collect();
        if fields.len() <= 8 {
            if ascii_trim(&line).is_empty() {
                continue;
            }
            return Err(malformed("geography", line_number, "fewer than 9 fields"));
        }
        if ascii_trim(fields[2]) != BLOCK_SUMLEV {
            continue;
        }
        let logrecno = ascii_trim(fields[7]).to_vec();
        let encoded_geoid = ascii_trim(fields[8]);
        let Some(geoid) = encoded_geoid.strip_prefix(BLOCK_GEOID_PREFIX) else {
            return Err(malformed(
                "geography",
                line_number,
                "block GEOID does not start with 7500000US",
            ));
        };
        if geoid.len() != 15 || !geoid.iter().all(u8::is_ascii_digit) {
            return Err(malformed(
                "geography",
                line_number,
                "block GEOID suffix must contain 15 digits",
            ));
        }
        let geoid = String::from_utf8(geoid.to_vec()).expect("validated ASCII digits");
        if blocks.insert(logrecno.clone(), (geoid, None)).is_some() {
            return Err(Pl94Error::DuplicateLogrecno(ascii_string(&logrecno)));
        }
    }

    for (line_index, line) in population.split(b'\n').enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        let fields: Vec<_> = line.split(|byte| *byte == b'|').collect();
        if fields.len() <= 5 {
            if ascii_trim(&line).is_empty() {
                continue;
            }
            return Err(malformed("population", line_number, "fewer than 6 fields"));
        }
        let logrecno = ascii_trim(fields[4]);
        let Some((_, value)) = blocks.get_mut(logrecno) else {
            continue;
        };
        if value.is_some() {
            return Err(Pl94Error::DuplicateLogrecno(ascii_string(logrecno)));
        }
        let text = std::str::from_utf8(ascii_trim(fields[5]))
            .map_err(|_| malformed("population", line_number, "population is not ASCII"))?;
        *value =
            Some(text.parse::<i64>().map_err(|_| {
                malformed("population", line_number, "population is not an integer")
            })?);
    }

    let mut result = Vec::with_capacity(blocks.len());
    for (logrecno, (geoid, population)) in blocks {
        let population =
            population.ok_or_else(|| Pl94Error::MissingPopulation(ascii_string(&logrecno)))?;
        result.push(Pl94BlockPopulation { geoid, population });
    }
    result.sort_by(|left, right| left.geoid.cmp(&right.geoid));
    Ok(result)
}

fn ascii_trim(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

fn ascii_string(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

fn malformed(file: &'static str, line: usize, message: impl Into<String>) -> Pl94Error {
    Pl94Error::Malformed {
        file,
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn geo_line(sumlev: &str, logrecno: &str, geoid: &str) -> String {
        format!("PLST|AA|{sumlev}|||||{logrecno}|{geoid}|unused\n")
    }

    fn population_line(logrecno: &str, population: &str) -> String {
        format!("PLST|AA|000|01|{logrecno}|{population}|unused\n")
    }

    #[test]
    fn joins_blocks_and_sorts_by_geoid() {
        let geo = format!(
            "{}{}{}",
            geo_line("750", "0000002", "7500000US440010002001001"),
            geo_line("040", "0000003", "0400000US44"),
            geo_line("750", "0000001", "7500000US440010001001001")
        );
        let population = format!(
            "{}{}{}",
            population_line("0000003", "1097379"),
            population_line("0000002", "7"),
            population_line("0000001", "11")
        );

        let blocks = parse_pl94_block_populations(
            Cursor::new(geo.into_bytes()),
            Cursor::new(population.into_bytes()),
        )
        .unwrap();
        assert_eq!(
            blocks,
            vec![
                Pl94BlockPopulation {
                    geoid: "440010001001001".into(),
                    population: 11,
                },
                Pl94BlockPopulation {
                    geoid: "440010002001001".into(),
                    population: 7,
                },
            ]
        );
    }

    #[test]
    fn accepts_latin1_in_unselected_fields() {
        let mut geo = geo_line("750", "1", "7500000US440010001001001").into_bytes();
        geo.extend_from_slice(b"PLST|AA|040|||||2|0400000US44|\xe9\n");
        let blocks = parse_pl94_block_populations(
            Cursor::new(geo),
            Cursor::new(population_line("1", "5").into_bytes()),
        )
        .unwrap();
        assert_eq!(blocks[0].population, 5);
    }

    #[test]
    fn rejects_missing_block_population() {
        let error = parse_pl94_block_populations(
            Cursor::new(geo_line("750", "42", "7500000US440010001001001")),
            Cursor::new(Vec::<u8>::new()),
        )
        .unwrap_err();
        assert!(matches!(error, Pl94Error::MissingPopulation(value) if value == "42"));
    }

    #[test]
    fn rejects_malformed_block_geoid() {
        let error = parse_pl94_block_populations(
            Cursor::new(geo_line("750", "42", "440010001001001")),
            Cursor::new(population_line("42", "5")),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Pl94Error::Malformed {
                file: "geography",
                ..
            }
        ));
    }
}

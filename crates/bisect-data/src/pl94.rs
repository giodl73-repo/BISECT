//! Streaming readers for Census PL 94-171 block populations.

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
    #[error("duplicate block GEOID {0}")]
    DuplicateGeoid(String),
    #[error("population file has no record for block LOGRECNO {0}")]
    MissingPopulation(String),
    #[error("unsupported PL 94-171 census year {0}")]
    UnsupportedYear(u16),
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

/// Join block geography records to total population records for a supported
/// decennial PL 94-171 format.
///
/// The 2020 release uses pipe-delimited geography and population files. The
/// 2010 release uses fixed-width geography and comma-delimited population.
/// The 2000 release carries POP100 directly in its fixed-width geography file.
pub fn read_pl94_block_populations_for_year<G: AsRef<Path>, P: AsRef<Path>>(
    geo_path: G,
    population_path: P,
    year: u16,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    let geo = BufReader::new(File::open(geo_path)?);
    match year {
        2000 => parse_pl94_2000_block_populations(geo),
        2010 => {
            parse_pl94_2010_block_populations(geo, BufReader::new(File::open(population_path)?))
        }
        2020 => parse_pl94_block_populations(geo, BufReader::new(File::open(population_path)?)),
        _ => Err(Pl94Error::UnsupportedYear(year)),
    }
}

fn parse_pl94_2000_block_populations<G: BufRead>(
    geo: G,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    let mut blocks = HashMap::new();
    for (line_index, line) in geo.split(b'\n').enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        if ascii_trim(&line).is_empty() {
            continue;
        }
        if line.len() < 301 {
            return Err(malformed(
                "geography",
                line_number,
                "2000 fixed-width record is shorter than 301 bytes",
            ));
        }
        if ascii_trim(&line[8..11]) != BLOCK_SUMLEV {
            continue;
        }
        let mut geoid = Vec::with_capacity(15);
        geoid.extend_from_slice(ascii_trim(&line[29..31]));
        geoid.extend_from_slice(ascii_trim(&line[31..34]));
        geoid.extend_from_slice(ascii_trim(&line[55..61]));
        geoid.extend_from_slice(ascii_trim(&line[62..66]));
        if geoid.len() != 15 || !geoid.iter().all(u8::is_ascii_digit) {
            return Err(malformed(
                "geography",
                line_number,
                "2000 block fields must form a 15-digit GEOID",
            ));
        }
        let geoid = String::from_utf8(geoid).expect("validated ASCII digits");
        let population = std::str::from_utf8(ascii_trim(&line[292..301]))
            .map_err(|_| malformed("geography", line_number, "POP100 is not ASCII"))?
            .parse::<i64>()
            .map_err(|_| malformed("geography", line_number, "POP100 is not an integer"))?;
        if blocks.insert(geoid.clone(), population).is_some() {
            return Err(Pl94Error::DuplicateGeoid(geoid));
        }
    }
    let mut result = blocks
        .into_iter()
        .map(|(geoid, population)| Pl94BlockPopulation { geoid, population })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.geoid.cmp(&right.geoid));
    Ok(result)
}

fn parse_pl94_2010_block_populations<G: BufRead, P: BufRead>(
    geo: G,
    population: P,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    let mut blocks: HashMap<Vec<u8>, (String, Option<i64>)> = HashMap::new();
    for (line_index, line) in geo.split(b'\n').enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        if ascii_trim(&line).is_empty() {
            continue;
        }
        if line.len() < 65 {
            return Err(malformed(
                "geography",
                line_number,
                "2010 fixed-width record is shorter than 65 bytes",
            ));
        }
        if ascii_trim(&line[8..11]) != BLOCK_SUMLEV {
            continue;
        }
        let logrecno = ascii_trim(&line[18..25]).to_vec();
        let mut geoid = Vec::with_capacity(15);
        geoid.extend_from_slice(ascii_trim(&line[27..29]));
        geoid.extend_from_slice(ascii_trim(&line[29..32]));
        geoid.extend_from_slice(ascii_trim(&line[54..60]));
        geoid.extend_from_slice(ascii_trim(&line[61..65]));
        if geoid.len() != 15 || !geoid.iter().all(u8::is_ascii_digit) {
            return Err(malformed(
                "geography",
                line_number,
                "2010 block fields must form a 15-digit GEOID",
            ));
        }
        let geoid = String::from_utf8(geoid).expect("validated ASCII digits");
        if blocks.insert(logrecno.clone(), (geoid, None)).is_some() {
            return Err(Pl94Error::DuplicateLogrecno(ascii_string(&logrecno)));
        }
    }

    join_delimited_population(blocks, population, b',')
}

fn join_delimited_population<P: BufRead>(
    mut blocks: HashMap<Vec<u8>, (String, Option<i64>)>,
    population: P,
    delimiter: u8,
) -> Result<Vec<Pl94BlockPopulation>, Pl94Error> {
    for (line_index, line) in population.split(b'\n').enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        let fields: Vec<_> = line.split(|byte| *byte == delimiter).collect();
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

    #[test]
    fn parses_2000_fixed_width_block_population() {
        let mut line = vec![b' '; 400];
        line[0..6].copy_from_slice(b"uPL   ");
        line[6..8].copy_from_slice(b"CA");
        line[8..11].copy_from_slice(b"750");
        line[29..31].copy_from_slice(b"06");
        line[31..34].copy_from_slice(b"001");
        line[55..61].copy_from_slice(b"427100");
        line[61] = b'1';
        line[62..66].copy_from_slice(b"1100");
        line[292..301].copy_from_slice(b"123      ");
        line.push(b'\n');
        assert_eq!(
            parse_pl94_2000_block_populations(Cursor::new(line)).unwrap(),
            vec![Pl94BlockPopulation {
                geoid: "060014271001100".to_owned(),
                population: 123,
            }]
        );
    }

    fn geo_line(sumlev: &str, logrecno: &str, geoid: &str) -> String {
        format!("PLST|AA|{sumlev}|||||{logrecno}|{geoid}|unused\n")
    }

    fn population_line(logrecno: &str, population: &str) -> String {
        format!("PLST|AA|000|01|{logrecno}|{population}|unused\n")
    }

    fn geo_line_2010(
        sumlev: &str,
        logrecno: &str,
        state: &str,
        county: &str,
        tract: &str,
        block: &str,
    ) -> Vec<u8> {
        let mut line = vec![b' '; 65];
        line[0..4].copy_from_slice(b"PLST");
        line[8..11].copy_from_slice(sumlev.as_bytes());
        line[18..25].copy_from_slice(logrecno.as_bytes());
        line[27..29].copy_from_slice(state.as_bytes());
        line[29..32].copy_from_slice(county.as_bytes());
        line[54..60].copy_from_slice(tract.as_bytes());
        line[60] = block.as_bytes()[0];
        line[61..65].copy_from_slice(block.as_bytes());
        line.push(b'\n');
        line
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
    fn joins_2010_fixed_width_blocks_and_sorts_by_geoid() {
        let mut geo = Vec::new();
        geo.extend(geo_line_2010(
            "750", "0000002", "44", "001", "000200", "1002",
        ));
        geo.extend(geo_line_2010(
            "040", "0000003", "44", "000", "000000", "0000",
        ));
        geo.extend(geo_line_2010(
            "750", "0000001", "44", "001", "000100", "1001",
        ));
        let population = b"PLST,RI,000,01,0000003,1052567,unused\n\
                           PLST,RI,000,01,0000002,7,unused\n\
                           PLST,RI,000,01,0000001,11,unused\n";

        let blocks =
            parse_pl94_2010_block_populations(Cursor::new(geo), Cursor::new(population.as_slice()))
                .unwrap();

        assert_eq!(
            blocks,
            vec![
                Pl94BlockPopulation {
                    geoid: "440010001001001".to_owned(),
                    population: 11,
                },
                Pl94BlockPopulation {
                    geoid: "440010002001002".to_owned(),
                    population: 7,
                },
            ]
        );
    }

    #[test]
    fn reads_local_2010_rhode_island_when_available() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let geo = root.join("data/2010/redistricting/ri2010.pl/rigeo2010.pl");
        let population = root.join("data/2010/redistricting/ri2010.pl/ri000012010.pl");
        if !geo.is_file() || !population.is_file() {
            return;
        }
        let blocks = read_pl94_block_populations_for_year(geo, population, 2010).unwrap();
        assert_eq!(blocks.len(), 25_181);
        assert_eq!(
            blocks.iter().map(|block| block.population).sum::<i64>(),
            1_052_567
        );
        assert!(blocks.windows(2).all(|pair| pair[0].geoid < pair[1].geoid));
    }

    #[test]
    fn reads_local_2000_california_when_available() {
        let geography =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/2000/redistricting/cageo.upl");
        if !geography.is_file() {
            return;
        }
        let blocks = read_pl94_block_populations_for_year(&geography, &geography, 2000).unwrap();
        assert_eq!(blocks.len(), 533_163);
        assert_eq!(
            blocks.iter().map(|block| block.population).sum::<i64>(),
            33_871_648
        );
        assert!(blocks.windows(2).all(|pair| pair[0].geoid < pair[1].geoid));
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

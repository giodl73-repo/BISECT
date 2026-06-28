use crate::*;

pub(crate) fn required(row: usize, field: &str, value: String) -> Result<String, RcountIoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RcountIoError::MissingStatementCsvField {
            row,
            field: field.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

pub(crate) fn read_csv_rows(path: &Path) -> Result<Vec<Vec<String>>, RcountIoError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;
    reader
        .records()
        .map(|record| {
            Ok(record?
                .iter()
                .map(|field| field.trim().to_string())
                .collect())
        })
        .collect()
}

pub(crate) fn section_data_row<'a>(
    rows: &'a [Vec<String>],
    section: &str,
) -> Result<&'a Vec<String>, RcountIoError> {
    let index = rows
        .iter()
        .position(|row| row.first().map_or(false, |field| field == section))
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: section.to_string(),
        })?;
    rows.get(index + 2)
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: section.to_string(),
        })
}

pub(crate) fn ri_field<'a>(
    row: &'a [String],
    index: usize,
    field: &str,
) -> Result<&'a str, RcountIoError> {
    row.get(index)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaField {
            field: field.to_string(),
        })
}

pub(crate) fn ri_i64(row: &[String], index: usize, field: &str) -> Result<i64, RcountIoError> {
    let value = ri_field(row, index, field)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: field.to_string(),
            value: value.to_string(),
        })
}

pub(crate) fn ri_u32(row: &[String], index: usize, field: &str) -> Result<u32, RcountIoError> {
    let value = ri_field(row, index, field)?;
    value
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: field.to_string(),
            value: value.to_string(),
        })
}

pub(crate) fn validate_ri_sample_sources(
    report_rows: &[Vec<String>],
    rounds_row: &[String],
    ballot_retrieval_csv: &Path,
) -> Result<(), RcountIoError> {
    let declared_sample_size = ri_u32(rounds_row, 3, "sample size")?;
    let sampled_ballots = ri_sampled_ballot_keys(report_rows)?;
    let retrieval_ballots = ri_retrieval_ballot_keys(ballot_retrieval_csv)?;
    if sampled_ballots.len() != retrieval_ballots.len() {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot row count".to_string(),
            value: format!(
                "report={}, retrieval={}",
                sampled_ballots.len(),
                retrieval_ballots.len()
            ),
        });
    }
    if sampled_ballots.len() > declared_sample_size as usize {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot row count".to_string(),
            value: format!(
                "report={} exceeds declared sample size {}",
                sampled_ballots.len(),
                declared_sample_size
            ),
        });
    }
    if sampled_ballots != retrieval_ballots {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot keys".to_string(),
            value: "audit report sampled ballots differ from retrieval CSV".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn ri_sampled_ballot_keys(
    rows: &[Vec<String>],
) -> Result<BTreeSet<String>, RcountIoError> {
    let section_index = rows
        .iter()
        .position(|row| {
            row.first()
                .map_or(false, |field| field == "######## SAMPLED BALLOTS ########")
        })
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: "######## SAMPLED BALLOTS ########".to_string(),
        })?;
    let mut keys = BTreeSet::new();
    for row in rows.iter().skip(section_index + 2) {
        if row.first().map_or(true, |field| field.is_empty()) {
            continue;
        }
        if row
            .first()
            .is_some_and(|field| field.starts_with("######## "))
        {
            break;
        }
        let container = ri_field(row, 1, "sampled ballot container")?;
        let tabulator = ri_field(row, 2, "sampled ballot tabulator")?;
        let batch_name = ri_field(row, 3, "sampled ballot batch name")?;
        let ballot_number = ri_field(row, 4, "sampled ballot position")?;
        let _ticket = ri_field(row, 5, "sampled ballot ticket")?
            .strip_prefix("Round 1:")
            .map(str::trim)
            .ok_or_else(|| RcountIoError::InvalidRhodeIslandRlaField {
                field: "sampled ballot ticket".to_string(),
                value: row[5].clone(),
            })?;
        let key = ri_ballot_key(container, tabulator, batch_name, ballot_number);
        if !keys.insert(key.clone()) {
            return Err(RcountIoError::InvalidRhodeIslandRlaField {
                field: "sampled ballot keys".to_string(),
                value: format!("duplicate sampled ballot key {key}"),
            });
        }
    }
    Ok(keys)
}

pub(crate) fn ri_retrieval_ballot_keys(path: &Path) -> Result<BTreeSet<String>, RcountIoError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut keys = BTreeSet::new();
    for (index, row) in reader.deserialize::<RhodeIslandRetrievalRow>().enumerate() {
        let row_number = index + 2;
        let row = row?;
        let container = required_ri_string(row_number, "Container", row.container)?;
        let tabulator = required_ri_string(row_number, "Tabulator", row.tabulator)?;
        let batch_name = required_ri_string(row_number, "Batch Name", row.batch_name)?;
        let ballot_number = required_ri_string(row_number, "Ballot Number", row.ballot_number)?;
        let _ticket = required_ri_string(row_number, "Ticket Numbers", row.ticket_numbers)?;
        let key = ri_ballot_key(&container, &tabulator, &batch_name, &ballot_number);
        if !keys.insert(key.clone()) {
            return Err(RcountIoError::InvalidRhodeIslandRlaField {
                field: "retrieval ballot keys".to_string(),
                value: format!("duplicate retrieval ballot key {key}"),
            });
        }
    }
    Ok(keys)
}

pub(crate) fn ri_ballot_key(
    container: &str,
    tabulator: &str,
    batch_name: &str,
    ballot_number: &str,
) -> String {
    format!(
        "{}|{}|{}|{}",
        container.trim().trim_start_matches('0'),
        tabulator.trim().trim_start_matches('0'),
        batch_name.trim(),
        ballot_number.trim()
    )
}

pub(crate) fn required_ri_string(
    row: usize,
    field: &str,
    value: String,
) -> Result<String, RcountIoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RcountIoError::MissingRhodeIslandRlaField {
            field: format!("{field} row {row}"),
        });
    }
    Ok(trimmed.to_string())
}

pub(crate) fn parse_ri_i64_string(
    row: usize,
    field: &str,
    value: String,
) -> Result<i64, RcountIoError> {
    let value = required_ri_string(row, field, value)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: format!("{field} row {row}"),
            value,
        })
}

pub(crate) fn parse_ri_vote_totals(value: &str) -> Result<Vec<(String, i64)>, RcountIoError> {
    value
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (label, votes) =
                part.split_once(':')
                    .ok_or_else(|| RcountIoError::InvalidRhodeIslandRlaField {
                        field: "vote totals".to_string(),
                        value: value.to_string(),
                    })?;
            let votes = votes.trim().parse::<i64>().map_err(|_| {
                RcountIoError::InvalidRhodeIslandRlaField {
                    field: "vote totals".to_string(),
                    value: value.to_string(),
                }
            })?;
            Ok((label.trim().to_string(), votes))
        })
        .collect()
}

pub(crate) fn parse_percent_ppm(value: &str) -> Result<u32, RcountIoError> {
    let trimmed = value.trim().trim_end_matches('%');
    let percent =
        trimmed
            .parse::<f64>()
            .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
                field: "risk limit".to_string(),
                value: value.to_string(),
            })?;
    if !(0.0..100.0).contains(&percent) {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "risk limit".to_string(),
            value: value.to_string(),
        });
    }
    Ok((percent * 10_000.0).round() as u32)
}

pub(crate) fn normalize_seed(value: &str) -> Result<String, RcountIoError> {
    let seed = if value.contains('e') || value.contains('E') {
        let parsed =
            value
                .parse::<f64>()
                .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
                    field: "random seed".to_string(),
                    value: value.to_string(),
                })?;
        format!("{parsed:.0}")
    } else {
        value.trim().to_string()
    };
    if seed.chars().all(|ch| ch.is_ascii_digit()) {
        Ok(seed)
    } else {
        Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "random seed".to_string(),
            value: value.to_string(),
        })
    }
}

pub(crate) fn slug_id(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

pub(crate) fn parse_i64(row: usize, field: &str, value: String) -> Result<i64, RcountIoError> {
    let value = required(row, field, value)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidStatementCsvField {
            row,
            field: field.to_string(),
            value,
        })
}

pub(crate) fn parse_u32(row: usize, field: &str, value: String) -> Result<u32, RcountIoError> {
    let value = required(row, field, value)?;
    value
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidStatementCsvField {
            row,
            field: field.to_string(),
            value,
        })
}

pub(crate) fn parse_selection_kind(
    row: usize,
    value: String,
) -> Result<SelectionKind, RcountIoError> {
    match required(row, "selection_kind", value)?.as_str() {
        "candidate" => Ok(SelectionKind::Candidate),
        "write-in-bucket" => Ok(SelectionKind::WriteInBucket),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "selection_kind".to_string(),
            value: other.to_string(),
        }),
    }
}

pub(crate) fn parse_reporting_unit_kind(
    row: usize,
    value: String,
) -> Result<ReportingUnitKind, RcountIoError> {
    match required(row, "reporting_unit_kind", value)?.as_str() {
        "precinct" => Ok(ReportingUnitKind::Precinct),
        "split-precinct" => Ok(ReportingUnitKind::SplitPrecinct),
        "vote-center" => Ok(ReportingUnitKind::VoteCenter),
        "central-count-batch" => Ok(ReportingUnitKind::CentralCountBatch),
        "mail-batch" => Ok(ReportingUnitKind::MailBatch),
        "provisional-batch" => Ok(ReportingUnitKind::ProvisionalBatch),
        "jurisdiction-total" => Ok(ReportingUnitKind::JurisdictionTotal),
        "district-total" => Ok(ReportingUnitKind::DistrictTotal),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "reporting_unit_kind".to_string(),
            value: other.to_string(),
        }),
    }
}

pub(crate) fn parse_count_status(row: usize, value: String) -> Result<CountStatus, RcountIoError> {
    match required(row, "status", value)?.as_str() {
        "unofficial" => Ok(CountStatus::Unofficial),
        "canvassed" => Ok(CountStatus::Canvassed),
        "recounted" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        "withdrawn" => Ok(CountStatus::Withdrawn),
        "superseded" => Ok(CountStatus::Superseded),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "status".to_string(),
            value: other.to_string(),
        }),
    }
}

pub(crate) fn require_same(
    row: usize,
    id: &str,
    field: &str,
    prior: &str,
    value: &str,
) -> Result<(), RcountIoError> {
    if prior != value {
        return Err(RcountIoError::ConflictingStatementCsvField {
            row,
            id: id.to_string(),
            field: field.to_string(),
            prior: prior.to_string(),
            value: value.to_string(),
        });
    }
    Ok(())
}

pub(crate) fn array_field<'a>(
    value: &'a Value,
    field: &str,
) -> Result<Vec<&'a Value>, RcountIoError> {
    match value.get(field) {
        Some(Value::Array(values)) => Ok(values.iter().collect()),
        Some(other) => Ok(vec![other]),
        None => Err(RcountIoError::MissingNistCdfField {
            field: field.to_string(),
        }),
    }
}

pub(crate) fn optional_array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    match value.get(field) {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    }
}

pub(crate) fn nist_id(value: &Value, field: &str) -> Result<String, RcountIoError> {
    value
        .get("@id")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("Id"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: format!("{field}.@id"),
        })
}

pub(crate) fn nist_text(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    value
        .get("Text")
        .and_then(Value::as_array)
        .and_then(|texts| texts.first())
        .and_then(|text| text.get("Value"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(crate) fn nist_gp_unit_ref(value: &Value) -> Result<String, RcountIoError> {
    value
        .get("GpUnitId")
        .or_else(|| value.get("GpUnit"))
        .or_else(|| value.get("ReportingUnit"))
        .and_then(|field| {
            field
                .as_str()
                .map(ToString::to_string)
                .or_else(|| {
                    field
                        .get("@id")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .or_else(|| {
                    field
                        .get("$ref")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
        })
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: "GpUnitId".to_string(),
        })
}

pub(crate) fn nist_count(value: &Value, field: &str) -> Result<i64, RcountIoError> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: field.to_string(),
        })
}

pub(crate) fn optional_nist_count(value: &Value, field: &str) -> Result<i64, RcountIoError> {
    match value.get(field) {
        Some(count) => count
            .as_i64()
            .ok_or_else(|| RcountIoError::InvalidNistCdfField {
                field: field.to_string(),
                value: count.to_string(),
            }),
        None => Ok(0),
    }
}

pub(crate) fn parse_nist_status(value: &str) -> Result<CountStatus, RcountIoError> {
    match value {
        "unofficial" | "pre-election" | "election-night" => Ok(CountStatus::Unofficial),
        "canvassed" | "canvass" | "official" => Ok(CountStatus::Canvassed),
        "recounted" | "recount" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        other => Err(RcountIoError::InvalidNistCdfField {
            field: "ResultsStatus".to_string(),
            value: other.to_string(),
        }),
    }
}

pub(crate) fn parse_nist_reporting_unit_kind(
    value: &str,
) -> Result<ReportingUnitKind, RcountIoError> {
    match value {
        "precinct" | "Precinct" => Ok(ReportingUnitKind::Precinct),
        "split-precinct" | "split_precinct" | "SplitPrecinct" => {
            Ok(ReportingUnitKind::SplitPrecinct)
        }
        "vote-center" | "VoteCenter" => Ok(ReportingUnitKind::VoteCenter),
        "district" | "District" => Ok(ReportingUnitKind::DistrictTotal),
        "county" | "state" | "jurisdiction" | "County" | "State" => {
            Ok(ReportingUnitKind::JurisdictionTotal)
        }
        other => Err(RcountIoError::InvalidNistCdfField {
            field: "GpUnit.Type".to_string(),
            value: other.to_string(),
        }),
    }
}

pub(crate) fn ensure_nist_unit(
    units: &mut BTreeMap<String, ReportingUnit>,
    reporting_unit_id: &str,
) {
    units
        .entry(reporting_unit_id.to_string())
        .or_insert(ReportingUnit {
            reporting_unit_id: reporting_unit_id.to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "nist-cdf".to_string(),
            source_ids: vec!["source:nist-cdf-json".to_string()],
            valid_from: None,
            valid_to: None,
        });
}

pub(crate) fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), RcountIoError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

pub(crate) fn write_synthetic_source_export(
    dir: &Path,
    package: &RcountPackage,
) -> Result<SourceEntry, RcountIoError> {
    let path = PathBuf::from("sources").join("synthetic-summary-export.json");
    let full_path = dir.join(&path);
    let value = serde_json::json!({
        "source_format": "synthetic-summary-export-v1",
        "contest_count": package.contests.len(),
        "reporting_unit_count": package.reporting_units.len(),
        "batch_count": package.batches.len(),
        "lineage_count": package.lineage.len(),
        "rhist_ref_count": package.rhist_refs.len(),
        "rctx_ref_count": package.rctx_refs.len(),
        "inclusion_proof_count": package.inclusion_proofs.len(),
        "cvr_count": package.cvr.len(),
        "rla_audit_count": package.rla_audits.len(),
        "manual_audit_count": package.manual_audits.len(),
        "batch_comparison_audit_count": package.batch_comparison_audits.len(),
        "summary_count": package.summaries.len(),
        "status_event_count": package.status_events.len(),
    });
    let bytes = serde_json::to_vec_pretty(&value)?;
    fs::write(&full_path, &bytes)?;
    Ok(SourceEntry {
        source_id: "source:synthetic-summary-export".to_string(),
        path: path.to_string_lossy().replace('\\', "/"),
        sha256: source_bytes_hash(&bytes),
    })
}

pub(crate) fn source_bytes_hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(SOURCE_HASH_PREFIX);
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

pub(crate) fn package_relative_source_path(path: &str) -> Result<PathBuf, RcountIoError> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(RcountIoError::InvalidSourcePath {
            path: path.to_string(),
        });
    }
    let mut components = candidate.components();
    match components.next() {
        Some(std::path::Component::Normal(first)) if first == "sources" => {}
        _ => {
            return Err(RcountIoError::InvalidSourcePath {
                path: path.to_string(),
            });
        }
    }
    Ok(candidate.to_path_buf())
}

pub(crate) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RcountIoError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn write_ndjson<T: Serialize>(path: &Path, records: &[T]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

pub(crate) fn read_ndjson<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Vec<T>, RcountIoError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line)?);
    }
    Ok(records)
}

pub(crate) fn read_optional_ndjson<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Vec<T>, RcountIoError> {
    if path.exists() {
        read_ndjson(path)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn write_lines(path: &Path, lines: &[&str]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for line in lines {
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

//! External assignment import creates a package; it never executes an engine.
use base64::{engine::general_purpose::STANDARD, Engine};
use bisect_report::label_import as native;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
const CAP: usize = 16 * 1024 * 1024;
#[derive(Clone, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    name: String,
    base64: String,
}
#[derive(Clone, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    label: String,
    year: String,
    created: String,
    format: Option<String>,
    source: Source,
    dbf: Option<Source>,
    shx: Option<Source>,
    registry_json: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlanInput {
    input: Input,
    state_name: String,
    context_json: String,
    expected_index_sha256: String,
    district_basis: String,
}
pub(crate) fn prepare_plan(input:PlanInput)->Result<Value,String> {
    let imported=run(input.input)?;
    prepare_imported_plan(&imported,&input.state_name,&input.context_json,&input.expected_index_sha256,&input.district_basis)
}
pub(crate) fn prepare_imported_plan(imported:&Value,state_name:&str,context_json:&str,expected_index_sha256:&str,district_basis:&str)->Result<Value,String> {
    let index=imported["index_json"].as_str().ok_or("Missing native import index.")?;
    if native::sha256_hex(index.as_bytes())!=expected_index_sha256 {return Err("Native import index changed; run import again before pairing a context.".into());}
    if state_name.is_empty() || !state_name.bytes().all(|b|b.is_ascii_lowercase()||b==b'_') {return Err("Choose an imported state.".into());}
    let assignment_path=format!("runs/{}/{}/{}/assignments.json",imported["label"].as_str().unwrap(),imported["year"].as_str().unwrap(),state_name);
    let text=imported["files"][&assignment_path].as_str().ok_or("Selected state is absent from the native import.")?;
    let assignments:BTreeMap<String,u64>=serde_json::from_str(text).map_err(|e|e.to_string())?;
    let context=crate::plan_audit_text::context(Some(&context_json))?.unwrap();
    let mut bindings=json!({"state_name":state_name,"district_basis":district_basis,
        "source_name":imported["source"]["name"],"source_sha256":imported["source"]["sha256"],
        "index_sha256":expected_index_sha256,"assignments_sha256":native::sha256_hex(text.as_bytes()),
        "context_sha256":native::sha256_hex(context_json.as_bytes())});
    for key in ["dbf","shx"] {if !imported[key].is_null() {bindings[format!("{key}_sha256")]=imported[key]["sha256"].clone();}}
    let document=native::align_context(&assignments,&context,&state_name,imported["label"].as_str().unwrap(),imported["year"].as_str().unwrap(),imported["created"].as_str().unwrap(),&district_basis,bindings.clone())?;
    let document_json=rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?;
    crate::plan_audit_text::plan(&document_json)?;
    let files=BTreeMap::from([("imported.rplan",document_json.clone()),("context.rctx",context_json.to_string()),("imported-assignments.json",text.into()),("import-index.json",index.into()),("import-record.json",serde_json::to_string_pretty(&bindings).map_err(|e|e.to_string())?)]);
    let hashes:BTreeMap<_,_>=files.iter().map(|(p,t)|(*p,native::sha256_hex(t.as_bytes()))).collect();
    Ok(json!({"schema_version":"bisect-label-import-plan-v1","document_json":document_json,"context_json":context_json,
        "plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,
        "state_name":state_name,"units":assignments.len(),"districts":document.plan.k,"unit_kind":context.units.unit_kind,"district_basis":district_basis,
        "bindings":bindings,"files":files,"file_sha256":hashes,"import_registry_json":imported["registry_json"],"execution_performed":false,
        "scope":"External assignments reparsed with native label import and aligned to the exact supplied RCTX unit order. All context bytes retained. No generation, certification, registry installation or source authentication. Explicitly run analysis or audit separately. At most 100,000 units, 500 districts, 3 MiB RPLAN and 8 MiB context; original nationwide capacity is unverified."}))
}
fn decode(source: &Source) -> Result<Vec<u8>, String> {
    if source.name.is_empty()
        || source.name.len() > 200
        || source.name.chars().any(char::is_control)
        || source.name.contains(['/', '\\', ':'])
        || source.base64.len() > CAP * 4 / 3 + 4
    {
        return Err(
            "Select a local source name up to 200 bytes and source bytes up to 16 MiB.".into(),
        );
    }
    STANDARD
        .decode(&source.base64)
        .map_err(|_| "Invalid assignment source base64.".into())
}
fn le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().unwrap())
}
fn header(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 100
        || u32::from_be_bytes(bytes[..4].try_into().unwrap()) != 9994
        || le(&bytes[28..32]) != 1000
        || u64::from(u32::from_be_bytes(bytes[24..28].try_into().unwrap())) * 2
            != bytes.len() as u64
    {
        return Err("Invalid shapefile header or declared byte length.".into());
    }
    Ok(())
}
fn shapefile_preflight(shp: &[u8], dbf: &[u8], shx: Option<&[u8]>) -> Result<(), String> {
    header(shp)?;
    if let Some(index) = shx {
        header(index)?;
        if (index.len() - 100) % 8 != 0 {
            return Err("Invalid SHX index length.".into());
        }
    }
    let mut offset = 100;
    let mut records = Vec::new();
    while offset < shp.len() {
        if shp.len() - offset < 8 {
            return Err("Truncated shape record header.".into());
        }
        let length = u64::from(u32::from_be_bytes(
            shp[offset + 4..offset + 8].try_into().unwrap(),
        )) * 2;
        records.push((offset as u64, length));
        offset += 8;
        if records.len() > 500_000 || length < 4 || length > (shp.len() - offset) as u64 {
            return Err("Shape record count or length exceeds selected bytes.".into());
        }
        let body = &shp[offset..offset + length as usize];
        let kind = le(&body[..4]);
        let required = match kind {
            0 => 4,
            1 => 20,
            11 => 28,
            21 => 28,
            8 | 18 | 28 => {
                if body.len() < 40 {
                    return Err("Truncated multipoint record.".into());
                }
                let points = u64::from(le(&body[36..40]));
                40 + 16 * points + if kind == 18 { 16 + 8 * points } else { 0 }
            }
            3 | 5 | 13 | 15 | 23 | 25 | 31 => {
                if body.len() < 44 {
                    return Err("Truncated multipart record.".into());
                }
                let parts = u64::from(le(&body[36..40]));
                let points = u64::from(le(&body[40..44]));
                let required = 44
                    + 4 * parts
                    + 16 * points
                    + if kind == 31 { 4 * parts } else { 0 }
                    + if [13, 15, 31].contains(&kind) {
                        16 + 8 * points
                    } else {
                        0
                    };
                if required > body.len() as u64 || parts == 0 && points > 0 {
                    return Err("Shape part/point counts exceed selected record bytes.".into());
                }
                let mut previous = 0;
                for i in 0..parts as usize {
                    let start = le(&body[44 + 4 * i..48 + 4 * i]);
                    if u64::from(start) >= points || i > 0 && start < previous {
                        return Err("Invalid shape part index.".into());
                    }
                    previous = start;
                }
                required
            }
            _ => return Err("Unsupported native shape record type.".into()),
        };
        if required > body.len() as u64 {
            return Err("Shape point count exceeds selected record bytes.".into());
        }
        offset += length as usize;
    }
    if let Some(index) = shx {
        if (index.len() - 100) / 8 != records.len() {
            return Err("SHX count does not match shape records.".into());
        }
        for (i, (offset, length)) in records.iter().enumerate() {
            let p = 100 + 8 * i;
            if u64::from(u32::from_be_bytes(index[p..p + 4].try_into().unwrap())) * 2 != *offset
                || u64::from(u32::from_be_bytes(index[p + 4..p + 8].try_into().unwrap())) * 2
                    != *length
            {
                return Err("SHX entry does not match shape record.".into());
            }
        }
    }
    if dbf.len() < 33 {
        return Err("Truncated assignment DBF header.".into());
    }
    let rows = u64::from(le(&dbf[4..8]));
    let start = usize::from(u16::from_le_bytes(dbf[8..10].try_into().unwrap()));
    let width = usize::from(u16::from_le_bytes(dbf[10..12].try_into().unwrap()));
    if rows != records.len() as u64
        || rows > 500_000
        || start < 33
        || start > dbf.len()
        || width == 0
        || start as u64 + rows * width as u64 > dbf.len() as u64
    {
        return Err(
            "DBF row count/width exceeds selected bytes or differs from shape records.".into(),
        );
    }
    let mut position = 32;
    let mut fields = std::collections::BTreeSet::new();
    let mut sum = 1;
    while position < start && dbf[position] != 13 {
        if start - position < 32 {
            return Err("Truncated DBF field descriptor.".into());
        }
        let name = &dbf[position..position + 11];
        let end = name.iter().position(|b| *b == 0).unwrap_or(name.len());
        if !fields.insert(
            name[..end]
                .iter()
                .map(u8::to_ascii_lowercase)
                .collect::<Vec<_>>(),
        ) {
            return Err("Duplicate case-insensitive DBF field name.".into());
        }
        sum += usize::from(dbf[position + 16]);
        position += 32;
    }
    if position >= start || sum > width {
        return Err("DBF fields exceed declared header/record width.".into());
    }
    Ok(())
}
pub(crate) fn run(input: Input) -> Result<Value, String> {
    bisect_report::label_rename::validate_name(&input.label)?;
    if input.label.len() > 100
        || ["__proto__", "constructor", "prototype"].contains(&input.label.as_str())
        || input.year.len() != 4
        || !input.year.bytes().all(|b| b.is_ascii_digit())
        || input.created.trim().is_empty()
        || input.created.len() > 200
        || input.created.chars().any(char::is_control)
        || input
            .registry_json
            .as_ref()
            .is_some_and(|s| s.len() > 1024 * 1024)
    {
        return Err("Choose a safe label up to 100 bytes, four-digit year, timestamp up to 200 bytes and registry up to 1 MiB.".into());
    }
    let primary = decode(&input.source)?;
    let dbf = input.dbf.as_ref().map(decode).transpose()?;
    let shx = input.shx.as_ref().map(decode).transpose()?;
    if primary.len() + dbf.as_ref().map_or(0, Vec::len) + shx.as_ref().map_or(0, Vec::len) > CAP {
        return Err("Assignment sources exceed 16 MiB combined.".into());
    }
    let format=input.format.as_deref().or_else(||native::detect_format(std::path::Path::new(&input.source.name))).ok_or_else(||format!("[CONFIG] cannot auto-detect format for '{}'; use --format csv|geojson|shapefile|rplan",input.source.name))?;
    let assignments = match format {
        "shapefile" => {
            let dbf = dbf.as_deref().ok_or("Select the shapefile DBF sidecar.")?;
            shapefile_preflight(&primary, dbf, shx.as_deref())?;
            native::parse_shapefile_bytes(&primary, dbf, shx.as_deref(), &input.source.name)?
        }
        "csv" | "geojson" | "rplan" => {
            if dbf.is_some() || shx.is_some() {
                return Err("DBF/SHX parts apply only to shapefile import.".into());
            }
            let text = std::str::from_utf8(&primary)
                .map_err(|_| "Assignment text must be valid UTF-8.")?;
            match format {
                "csv" => native::parse_csv_assignments(text)?,
                "rplan" => native::parse_rplan_assignments(text)?,
                _ => native::parse_geojson_assignments(text)?,
            }
        }
        other => {
            return Err(format!(
                "[CONFIG] unknown format '{other}'; expected csv, geojson, shapefile, or rplan"
            ))
        }
    };
    if assignments.len() > 500_000
        || assignments.keys().any(|g| g.len() > 200 || g.chars().any(char::is_control))
    {
        return Err("Import supports 500,000 units and GEOIDs up to 200 bytes without control characters.".into());
    }
    let (states, unknown) = native::group(assignments)?;
    let source_sha = native::sha256_hex(&primary);
    let index = native::build_index(
        &input.label,
        &input.year,
        &input.source.name,
        format,
        &source_sha,
        &states,
        &input.created,
    );
    let mut registry =
        bisect_report::label_registry::parse(input.registry_json.as_deref().unwrap_or(""))?;
    if registry.len() > 500
        || registry.iter().any(|(n, e)| {
            n.len() > 200 || e.built.len() > 100 || e.analyzed.len() > 100 || e.reported.len() > 100
        })
    {
        return Err("Registry exceeds browser label/stage bounds.".into());
    }
    bisect_report::label_registry::mark_built(
        registry.entry(input.label.clone()).or_default(),
        &input.year,
    );
    if registry.len() > 500 || registry[&input.label].built.len() > 100 {
        return Err("Imported registry exceeds browser label/stage bounds.".into());
    }
    let registry_json = serde_json::to_string_pretty(&registry).map_err(|e| e.to_string())?;
    let index_json = serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?;
    let root = format!("runs/{}/{}/", input.label, input.year);
    let index_path = root.clone() + "index.json";
    let mut files = BTreeMap::from([
        (index_path.clone(), index_json.clone()),
        (".bisect".into(), registry_json.clone()),
    ]);
    for (state, assignments) in &states {
        files.insert(
            format!("{root}{state}/assignments.json"),
            serde_json::to_string_pretty(assignments).map_err(|e| e.to_string())?,
        );
    }
    if files.values().map(String::len).sum::<usize>() > CAP {
        return Err("Imported output files exceed 16 MiB.".into());
    }
    let hashes: BTreeMap<_, _> = files
        .iter()
        .map(|(p, t)| (p.clone(), native::sha256_hex(t.as_bytes())))
        .collect();
    Ok(
        json!({"schema_version":"bisect-label-import-v1","status":"imported-external-assignments","label":input.label,"year":input.year,"format":format,"created":input.created,"index_json":index_json,"index_path":index_path,"states":index["states"],"summary":index["summary"],"unknown_geoids":unknown,"files":files,"file_sha256":hashes,"registry_json":registry_json,"source":{"name":input.source.name,"sha256":source_sha,"bytes":primary.len()},"dbf":input.dbf.as_ref().zip(dbf.as_ref()).map(|(s,b)|json!({"name":s.name,"sha256":native::sha256_hex(b),"bytes":b.len()})),"shx":input.shx.as_ref().zip(shx.as_ref()).map(|(s,b)|json!({"name":s.name,"sha256":native::sha256_hex(b),"bytes":b.len()})),"execution_performed":false,"scope":"Shared native label-import parsers, state-FIPS grouping, sorted assignment serialization, build index and built-stage update. Imported assignments are external and unverified: no engine, population, contiguity, source authentication or certification occurs. CSV, GeoJSON and shapefile labels retain exact native unsigned 64-bit integers (including zero); RPLAN zero-based labels become one-based. Duplicate GEOIDs keep the last parsed value; unknown state prefixes are omitted and disclosed. Shape geometry is read then ignored; the native index hashes only SHP, with separate selected DBF/SHX hashes recorded here. Existing analyzed/reported stages are preserved by native mark_built and may be stale after replacing assignments. Registry snapshot is returned for explicit review/export, never installed. Native CLI registry object ordering is not a byte-parity claim. Input/output are bounded to 16 MiB each; original nationwide capacity is unverified."}),
    )
}

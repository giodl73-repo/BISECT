//! Native Fetch LODES aggregation kernels with no filesystem or network dependencies.
#[derive(Debug,serde::Serialize)]
pub struct ImportReport {pub csv_text:String,pub written_rows:usize,pub short_rows:usize,pub invalid_geoid_rows:usize}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn wac_preserves_native_floating_sum_then_integer_cast(){
        let body="w_geocode,C000,CNS07,CNS09,CNS10,CNS11,CNS01,CNS02,CNS05,CNS08\n440010001001001,1.5,0,0,0,0,0,0,0,0\n440010001001002,1.5,0,0,0,0,0,0,0,0\n";
        let r=wac(std::io::Cursor::new(body)).unwrap();assert_eq!(r.written_rows,1);assert!(r.csv_text.ends_with("44001000100,3,0,0,0,0,0,0,0,0\n"));
    }
    #[test] fn reordered_short_and_non_boundary_utf8_rows_are_skipped(){
        let body="S000,h_geocode,w_geocode\n1,440010001001001\n2,1234567890é,440010001001001\n";
        let r=od(std::io::Cursor::new(body)).unwrap();assert_eq!(r.short_rows,1);assert_eq!(r.invalid_geoid_rows,1);assert_eq!(r.written_rows,0);
    }
    #[test] fn od_maximum_total_is_exact_and_overflow_rejects(){
        let body="h_geocode,w_geocode,S000\n440010001001001,440010002001001,18446744073709551615\n";
        assert!(od(std::io::Cursor::new(body)).unwrap().csv_text.ends_with("18446744073709551615\n"));
        let extra=format!("{body}440010001001002,440010002001002,1\n");assert!(od(std::io::Cursor::new(extra)).unwrap_err().contains("exceeds u64"));
    }
}
pub fn wac<R: std::io::BufRead>(reader: R) -> Result<ImportReport, String> {
    use std::collections::HashMap;
    let mut short_rows=0;
    let mut invalid_geoid_rows=0;
    // Aggregate block rows to tracts
    // tract_geoid (11 chars) → [c000, cns07, cns09, cns10, cns11, cns01, cns02, cns05, cns08]
    let mut tracts: HashMap<String, [f64; 9]> = HashMap::new();

    let mut lines = reader.lines();
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("LODES WAC: empty or unreadable file".to_string()),
    };

    // Find column indices from header
    let cols: Vec<&str> = header.split(',').collect();
    let idx = |name: &str| -> Result<usize, String> {
        cols.iter()
            .position(|c| c.trim_matches('"') == name)
            .ok_or_else(|| format!("LODES WAC: column '{name}' not found in header"))
    };
    let i_geo = idx("w_geocode")?;
    let i_c000 = idx("C000")?;
    let i_c07 = idx("CNS07")?;
    let i_c09 = idx("CNS09")?;
    let i_c10 = idx("CNS10")?;
    let i_c11 = idx("CNS11")?;
    let i_c01 = idx("CNS01")?;
    let i_c02 = idx("CNS02")?;
    let i_c05 = idx("CNS05")?;
    let i_c08 = idx("CNS08")?;

    for line in lines {
        let line = line.map_err(|e| format!("LODES read: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= *[i_geo,i_c000,i_c07,i_c09,i_c10,i_c11,i_c01,i_c02,i_c05,i_c08].iter().max().unwrap() {
            short_rows+=1;
            continue;
        }

        let block_geoid = fields[i_geo].trim_matches('"');
        let Some(tract_geoid)=block_geoid.get(..11) else {invalid_geoid_rows+=1;continue;};

        let parse = |i: usize| -> f64 { fields[i].trim_matches('"').parse::<f64>().unwrap_or(0.0) };

        let entry = tracts.entry(tract_geoid.to_string()).or_insert([0.0; 9]);
        entry[0] += parse(i_c000);
        entry[1] += parse(i_c07);
        entry[2] += parse(i_c09);
        entry[3] += parse(i_c10);
        entry[4] += parse(i_c11);
        entry[5] += parse(i_c01);
        entry[6] += parse(i_c02);
        entry[7] += parse(i_c05);
        entry[8] += parse(i_c08);
    }

    // Write output CSV
    let mut out=String::new();
    use std::fmt::Write;
    writeln!(
        out,
        "geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08"
    )
    .map_err(|e| e.to_string())?;
    let mut sorted: Vec<_> = tracts.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let written_rows=sorted.len();
    for (geoid, v) in sorted {
        writeln!(
            out,
            "{geoid},{},{},{},{},{},{},{},{},{}",
            v[0] as u64,
            v[1] as u64,
            v[2] as u64,
            v[3] as u64,
            v[4] as u64,
            v[5] as u64,
            v[6] as u64,
            v[7] as u64,
            v[8] as u64
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(ImportReport {csv_text:out,written_rows,short_rows,invalid_geoid_rows})
}

pub fn od<R: std::io::BufRead>(reader: R) -> Result<ImportReport, String> {
    use std::collections::HashMap;
    let mut short_rows=0;
    let mut invalid_geoid_rows=0;
    // Aggregate block-pair rows to tract-pair level
    // (home_tract, work_tract) -> total S000 jobs
    let mut od_pairs: HashMap<(String, String), u64> = HashMap::new();

    let mut lines = reader.lines();
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("LODES OD: empty or unreadable file".to_string()),
    };

    // Find column indices from header
    let cols: Vec<&str> = header.split(',').collect();
    let idx = |name: &str| -> Result<usize, String> {
        cols.iter()
            .position(|c| c.trim_matches('"') == name)
            .ok_or_else(|| format!("LODES OD: column '{name}' not found in header"))
    };
    let i_home = idx("h_geocode")?;
    let i_work = idx("w_geocode")?;
    let i_s000 = idx("S000")?;

    for line in lines {
        let line = line.map_err(|e| format!("LODES OD read: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= i_home.max(i_work).max(i_s000) {
            short_rows+=1;
            continue;
        }

        let home_block = fields[i_home].trim_matches('"');
        let work_block = fields[i_work].trim_matches('"');
        let (Some(home),Some(work))=(home_block.get(..11),work_block.get(..11)) else {invalid_geoid_rows+=1;continue;};
        let home_tract = home.to_string();
        let work_tract = work.to_string();
        let jobs: u64 = fields[i_s000].trim_matches('"').parse::<u64>().unwrap_or(0);

        let entry=od_pairs.entry((home_tract, work_tract)).or_insert(0);
        *entry=entry.checked_add(jobs).ok_or_else(|| "LODES OD tract-pair job total exceeds u64".to_string())?;
    }

    // Write output CSV: sorted by home_geoid then work_geoid
    let mut out=String::new();
    use std::fmt::Write;
    writeln!(out, "home_geoid,work_geoid,s000").map_err(|e| e.to_string())?;
    let mut sorted: Vec<_> = od_pairs.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let written_rows=sorted.len();
    for ((home, work), jobs) in sorted {
        writeln!(out, "{home},{work},{jobs}").map_err(|e| e.to_string())?;
    }
    Ok(ImportReport {csv_text:out,written_rows,short_rows,invalid_geoid_rows})
}


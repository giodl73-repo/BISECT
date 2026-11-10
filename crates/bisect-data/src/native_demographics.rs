//! Historical CLI demographic CSV semantics, without filesystem access.
//! These compatibility readers intentionally preserve native zero defaults,
//! duplicate replacement and simple comma splitting. Browser strict import
//! remains a separate API; callers must disclose compatibility defaults.
use std::collections::HashMap;

/// Inspect the historical reader without silently losing its compatibility
/// decisions. Non-finite total-population fractions are represented explicitly.
pub fn inspect_native_demographics(content:&str,basis:&str)->Result<serde_json::Value,String>{
    use serde_json::json;
    if content.lines().skip(1).filter(|s|!s.trim().is_empty()).take(100_001).count()>100_000{return Err("Native demographic inspection exceeds 100,000 rows.".into());}
    let total=basis=="total-population";
    if !total && !["voting-age-population","citizen-voting-age-population"].contains(&basis){return Err("Unknown demographic basis.".into());}
    let mut records=vec![];
    if total {
        for (geoid,fraction) in parse_native_demographics(content)? {records.push(json!({"geoid":geoid,"fraction":if fraction.is_finite(){Some(fraction)}else{None},"non_finite_fraction":!fraction.is_finite()}));}
    } else {
        for (geoid,c) in parse_native_vap_demographics(content)? {records.push(json!({"geoid":geoid,"total":c.total_vap,"minority":c.minority_vap,"fraction":if c.total_vap==0.0{0.0}else{c.minority_vap/c.total_vap},"non_finite_fraction":false}));}
    }
    records.sort_by(|a,b|a["geoid"].as_str().cmp(&b["geoid"].as_str()));
    let columns:Vec<_>=content.lines().next().unwrap_or("").split(',').map(str::trim).collect();
    let pick=|names:&[&str]|names.iter().find_map(|name|columns.iter().position(|c|c==name)).unwrap();
    let id=pick(if total{&["GEOID"]}else{&["GEOID","geoid"]});
    let a=pick(if total{&["total_pop"]}else{&["total_vap","vap","cvap"]});
    let b=pick(if total{&["white_non_hispanic"]}else{&["minority_vap"]});
    let (mut rows,mut defaults,mut duplicates,mut zero_totals,mut clamped)=(0,0,0,0,0);
    let mut seen=std::collections::HashSet::new();
    for line in content.lines().skip(1).filter(|s|!s.trim().is_empty()){
        rows+=1;if rows>100_000{return Err("Native demographic inspection exceeds 100,000 rows.".into());}
        let fields:Vec<_>=line.split(',').collect();let geoid=format!("{:0>11}",fields.get(id).unwrap_or(&"").trim());if !seen.insert(geoid){duplicates+=1;}
        let mut number=|i|fields.get(i).and_then(|s:&&str|s.trim().parse::<f64>().ok()).unwrap_or_else(||{defaults+=1;0.0});
        let x=number(a);let y=number(b);if x<=0.0{zero_totals+=1;}
        if total && x>0.0 {let f=(x-y)/x;if f<0.0||f>1.0{clamped+=1;}}
    }
    Ok(json!({"schema_version":"bisect-native-demographic-inspection-v1","basis":basis,"records":records,"diagnostics":{"source_rows":rows,"numeric_zero_defaults":defaults,"duplicate_replacements":duplicates,"nonpositive_totals":zero_totals,"clamped_fractions":clamped},"policy":"Historical native comma splitting, column precedence, numeric zero defaults and last-row duplicate replacement. This inspection does not validate state scope, complete graph coverage, source authenticity or legal compliance. Native missing-tract alignment is a separate operation."}))
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VapDemographicRecord {
    pub total_vap: f64,
    pub minority_vap: f64,
}

pub fn parse_native_demographics(content: &str) -> Result<HashMap<String, f64>, String> {
    let mut lines = content.lines();
    let header = lines.next().ok_or("empty demographics file")?;
    let cols: Vec<&str> = header.split(',').collect();

    let col = |name: &str| -> Result<usize, String> {
        cols.iter()
            .position(|&c| c.trim() == name)
            .ok_or_else(|| format!("column '{name}' not found in header: {}", header))
    };

    let i_geoid = col("GEOID")?;
    let i_total = col("total_pop")?;
    let i_white = col("white_non_hispanic")?;

    let mut result = HashMap::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();

        let geoid = format!("{:0>11}", fields.get(i_geoid).unwrap_or(&"").trim());
        let total: f64 = fields
            .get(i_total)
            .and_then(|f| f.trim().parse().ok())
            .unwrap_or(0.0);
        let white: f64 = fields
            .get(i_white)
            .and_then(|f| f.trim().parse().ok())
            .unwrap_or(0.0);

        if total <= 0.0 {
            result.insert(geoid, 0.0);
            continue;
        }

        let minority_frac = ((total - white) / total).clamp(0.0, 1.0);
        result.insert(geoid, minority_frac);
    }

    Ok(result)
}

pub fn parse_native_vap_demographics(
    content: &str,
) -> Result<HashMap<String, VapDemographicRecord>, String> {
    let mut lines = content.lines();
    let header = lines.next().ok_or("empty VAP demographics file")?;
    let cols: Vec<&str> = header.split(',').map(str::trim).collect();

    let col = |names: &[&str]| -> Result<usize, String> {
        names
            .iter()
            .find_map(|name| cols.iter().position(|&c| c == *name))
            .ok_or_else(|| {
                format!(
                    "VAP demographics missing required column; expected one of {:?} in header: {}",
                    names, header
                )
            })
    };

    let i_geoid = col(&["GEOID", "geoid"])?;
    let i_total_vap = col(&["total_vap", "vap", "cvap"])?;
    let i_minority_vap = col(&["minority_vap"])?;

    let mut result = HashMap::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid = format!("{:0>11}", fields.get(i_geoid).unwrap_or(&"").trim());
        let total_vap: f64 = fields
            .get(i_total_vap)
            .and_then(|field| field.trim().parse().ok())
            .unwrap_or(0.0);
        let minority_vap: f64 = fields
            .get(i_minority_vap)
            .and_then(|field| field.trim().parse().ok())
            .unwrap_or(0.0);
        if !total_vap.is_finite()
            || !minority_vap.is_finite()
            || total_vap < 0.0
            || minority_vap < 0.0
            || minority_vap > total_vap
        {
            return Err(format!(
                "invalid VAP demographics for GEOID {geoid}: total_vap={total_vap}, minority_vap={minority_vap}"
            ));
        }
        result.insert(
            geoid,
            VapDemographicRecord {
                total_vap,
                minority_vap,
            },
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_reports_native_defaults_replacements_and_nonfinite_values() {
        let r=inspect_native_demographics("GEOID,total_pop,white_non_hispanic\n01001020100,100,bad\n01001020200,bad,10\n01001020300,100,200\n01001020100,100,60\n01001020400,NaN,0\n","total-population").unwrap();
        assert_eq!(r["diagnostics"],serde_json::json!({"source_rows":5,"numeric_zero_defaults":2,"duplicate_replacements":1,"nonpositive_totals":1,"clamped_fractions":1}));
        assert_eq!(r["records"][0]["fraction"],0.4);
        assert_eq!(r["records"][3]["fraction"],serde_json::Value::Null);
        assert_eq!(r["records"][3]["non_finite_fraction"],true);
    }

    #[test]
    fn total_population_preserves_native_padding_defaults_clamping_and_duplicates() {
        let result=parse_native_demographics("GEOID,total_pop,white_non_hispanic\r\n1001020100,100,20\r\n01001020200,bad,10\r\n01001020300,100,bad\r\n01001020400,100,200\r\n01001020500,100,-50\r\n1001020100,100,60\r\n").unwrap();
        assert_eq!(result.len(),5);
        assert_eq!(result["01001020100"],0.4);
        assert_eq!(result["01001020200"],0.0);
        assert_eq!(result["01001020300"],1.0);
        assert_eq!(result["01001020400"],0.0);
        assert_eq!(result["01001020500"],1.0);
    }

    #[test]
    fn total_population_preserves_simple_split_and_header_behavior() {
        assert!(parse_native_demographics("\u{feff}GEOID,total_pop,white_non_hispanic\n").is_err());
        assert!(parse_native_demographics("").is_err());
        assert!(parse_native_demographics("GEOID,total_pop,white_non_hispanic\n").unwrap().is_empty());
        let result=parse_native_demographics("GEOID,total_pop,white_non_hispanic\n01001020100,100,\"20\"\n").unwrap();
        assert_eq!(result["01001020100"],1.0);
    }

    #[test]
    fn vap_preserves_alias_precedence_zero_defaults_and_duplicates() {
        let result=parse_native_vap_demographics("geoid,cvap,vap,total_vap,minority_vap\n1001020100,80,90,100,20\n01001020200,bad,bad,bad,bad\n1001020100,10,30,50,25\n").unwrap();
        assert_eq!(result["01001020100"],VapDemographicRecord{total_vap:50.,minority_vap:25.});
        assert_eq!(result["01001020200"],VapDemographicRecord{total_vap:0.,minority_vap:0.});
    }

    #[test]
    fn vap_retains_invalid_count_refusals() {
        for row in ["100,101","-1,0","100,-1","NaN,0","100,inf"] {
            assert!(parse_native_vap_demographics(&format!("GEOID,total_vap,minority_vap\n01001020100,{row}\n")).is_err());
        }
    }
}


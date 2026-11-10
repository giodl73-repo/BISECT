//! Exact native TSV policy, separated from filesystem authority.
use std::collections::{BTreeMap,HashMap};

pub fn parse_native(text:&str,source_name:&str)->Result<BTreeMap<String,f64>,String>{
    let mut result=BTreeMap::new();let mut data_started=false;
    for(line_num,line)in text.lines().enumerate(){
        let line=line.trim();if line.is_empty()||line.starts_with('#'){continue;}
        let parts:Vec<&str>=line.split('\t').collect();
        if parts.len()<2{return Err(format!("{source_name}:{}: expected 2 tab-separated columns, got {}",line_num+1,parts.len()));}
        if !data_started{if parts[1].trim().parse::<f64>().is_err(){data_started=true;continue;}data_started=true;}
        let geoid=format!("{:0>11}",parts[0].trim());
        let share:f64=parts[1].trim().parse().map_err(|e|format!("{source_name}:{}: cannot parse dem_share '{}': {e}",line_num+1,parts[1]))?;
        if !(0.0..=1.0).contains(&share){return Err(format!("{source_name}:{}: dem_share {} for {} not in [0.0, 1.0]",line_num+1,share,geoid));}
        result.insert(geoid,share);
    }
    Ok(result)
}
pub fn align_native(shares:&BTreeMap<String,f64>,index_to_geoid:&HashMap<usize,String>,n:usize)->Vec<f64>{
    (0..n).map(|i|index_to_geoid.get(&i).and_then(|id|shares.get(id)).copied().unwrap_or(0.5)).collect()
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn native_sparse_padding_header_extra_columns_and_last_row_win(){
  let text="# comment\nanything\tshare name\textra\n1001000000\t0.8\textra\n1001000000\t0.2\n999\t0.6\n";
  let parsed=parse_native(text,"selected.tsv").unwrap();assert_eq!(parsed["01001000000"],0.2);assert_eq!(parsed["00000000999"],0.6);
  assert_eq!(align_native(&parsed,&HashMap::from([(0,"01001000000".into()),(1,"01001000001".into())]),2),vec![0.2,0.5]);
 }
 #[test]fn empty_and_negative_zero_keep_native_semantics(){
  assert!(parse_native("","x").unwrap().is_empty());let p=parse_native("01001000000\t-0.0\n","x").unwrap();assert_eq!(p["01001000000"].to_bits(),(-0.0f64).to_bits());
  // The native reader does not remove an embedded BOM from a headerless ID.
  assert!(!parse_native("\u{feff}01001000000\t0.8","x").unwrap().contains_key("01001000000"));
 }
 #[test]fn malformed_consumed_rows_fail_with_native_line_context(){
  for t in ["id\tshare\n01001000000\tbad","one-column","id\tshare\n01001000000\tNaN","id\tshare\n01001000000\t1.1"]{assert!(parse_native(t,"selected.tsv").unwrap_err().contains("selected.tsv:"));}
 }
}

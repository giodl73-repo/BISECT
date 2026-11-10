//! Native post-weighter community-of-interest edge adjustment.
use std::collections::{BTreeMap,HashMap};
use serde::de::{Visitor,MapAccess,Error};
/// Sparse GEOID weights. Missing units use 1; unknown units have no effect.
/// Ambiguous duplicate keys and invalid factors are rejected before adjustment.
pub fn parse(text:&str)->Result<BTreeMap<String,f64>,String>{
 struct Weights;
 impl<'de> Visitor<'de> for Weights{type Value=BTreeMap<String,f64>;
  fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a JSON object of finite nonnegative COI weights")}
  fn visit_map<M:MapAccess<'de>>(self,mut map:M)->Result<Self::Value,M::Error>{let mut values=BTreeMap::new();while let Some((key,value))=map.next_entry::<String,f64>()?{if !value.is_finite()||value<0.0{return Err(M::Error::custom("COI weights must be finite and nonnegative."));}if values.insert(key,value).is_some(){return Err(M::Error::custom("Duplicate COI unit ID."));}}Ok(values)}
 }
 let mut reader=serde_json::Deserializer::from_str(text);
 let values=serde::de::Deserializer::deserialize_map(&mut reader,Weights).map_err(|e|format!("cannot parse COI weights JSON: {e}"))?;reader.end().map_err(|e|e.to_string())?;Ok(values)
}
pub fn apply(mut edges:HashMap<(usize,usize),f64>,weights:&BTreeMap<String,f64>,ids:&HashMap<usize,String>)->Result<HashMap<(usize,usize),f64>,String>{
 // Compute every value before committing; overflow never leaves partial output.
 let mut adjusted=Vec::with_capacity(edges.len());
 for (&(u,v),&base) in &edges{let get=|i|ids.get(&i).and_then(|id|weights.get(id)).copied().unwrap_or(1.0);let value=base*(get(u)*get(v)).sqrt();if !value.is_finite()||value<0.0{return Err("COI edge adjustment is nonfinite or negative.".into());}adjusted.push(((u,v),value));}
 for (edge,value) in adjusted{edges.insert(edge,value);}Ok(edges)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn sparse_geometric_mean_and_zero_match_native_formula(){let edges=HashMap::from([((0,1),10.0),((1,2),4.0)]);let ids=HashMap::from([(0,"u".into()),(1,"v".into()),(2,"w".into())]);let weights=parse(r#"{"u":4,"v":9,"unused":100}"#).unwrap();assert_eq!(apply(edges.clone(),&weights,&ids).unwrap(),HashMap::from([((0,1),60.0),((1,2),12.0)]));assert_eq!(apply(HashMap::new(),&weights,&ids).unwrap().len(),0);let zero=parse(r#"{"u":0}"#).unwrap();assert_eq!(apply(edges,&zero,&ids).unwrap()[&(0,1)],0.0);}
 #[test]fn malformed_ambiguous_and_overflowing_weights_reject(){for text in [r#"{"u":-1}"#,r#"{"u":1,"\u0075":2}"#,r#"{"u":1e400}"#,r#"[]"#,r#"{"u":1} null"#]{assert!(parse(text).is_err());}assert!(apply(HashMap::from([((0,1),10.0)]),&parse(r#"{"u":1e308,"v":1e308}"#).unwrap(),&HashMap::from([(0,"u".into()),(1,"v".into())])).is_err());}
}

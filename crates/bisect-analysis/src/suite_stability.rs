//! Native suite stability reporting on in-memory assignments.
//! The historical CLI calls this Jaccard, but measures matching labels / maximum map size.
use std::collections::HashMap;
use serde::Serialize;

pub const STABILITY_THRESHOLD:f64=0.95;
#[derive(Debug,Serialize)]
pub struct SuiteStabilityResult {
 pub num_plans:usize,
 pub num_pairs:usize,
 pub mean_similarity:f64,
 pub min_similarity:f64,
 pub max_similarity:f64,
 pub stable:bool,
 pub unstable_pairs:Vec<(String,String,f64)>,
}
pub fn compute_jaccard_similarity(a:&HashMap<String,usize>,b:&HashMap<String,usize>)->f64 {
 if a.is_empty()||b.is_empty(){return 0.0;}
 let matching=a.iter().filter(|(id,d)|b.get(*id)==Some(*d)).count();
 matching as f64/a.len().max(b.len()) as f64
}
pub fn analyze_suite_stability(plans:&[(String,HashMap<String,usize>)])->Result<SuiteStabilityResult,String>{
 if plans.len()<2{return Err("At least 2 plan labels are required for stability analysis.".into());}
 let mut similarities=Vec::new();let mut unstable_pairs=Vec::new();
 for i in 0..plans.len(){for j in i+1..plans.len(){
  let similarity=compute_jaccard_similarity(&plans[i].1,&plans[j].1);similarities.push(similarity);
  if similarity<STABILITY_THRESHOLD{unstable_pairs.push((plans[i].0.clone(),plans[j].0.clone(),similarity));}
 }}
 let num_pairs=similarities.len();
 Ok(SuiteStabilityResult{num_plans:plans.len(),num_pairs,
 mean_similarity:similarities.iter().sum::<f64>()/num_pairs as f64,
 min_similarity:similarities.iter().copied().fold(f64::INFINITY,f64::min),
 max_similarity:similarities.iter().copied().fold(f64::NEG_INFINITY,f64::max),
 stable:unstable_pairs.is_empty(),unstable_pairs})
}
#[cfg(test)]
mod tests {
 use super::*;
 fn assignments(labels:&[usize])->HashMap<String,usize>{labels.iter().enumerate().map(|(i,d)|(format!("unit-{i}"),*d)).collect()}
 #[test]fn identical_and_renumbered(){let a=assignments(&[1,1,2,2]);let b=assignments(&[2,2,1,1]);assert_eq!(compute_jaccard_similarity(&a,&a),1.0);assert_eq!(compute_jaccard_similarity(&a,&b),0.0);}
 #[test]fn native_partial_unit_behavior(){assert_eq!(compute_jaccard_similarity(&assignments(&[1,1]),&assignments(&[1])),0.5);assert_eq!(compute_jaccard_similarity(&HashMap::new(),&assignments(&[1])),0.0);}
 #[test]fn pair_order_and_summary(){let a=assignments(&[1,1,2,2]);let b=assignments(&[1,2,1,2]);let out=analyze_suite_stability(&[("a".into(),a.clone()),("b".into(),a),("c".into(),b)]).unwrap();assert_eq!(out.num_pairs,3);assert_eq!(out.mean_similarity,2.0/3.0);assert_eq!(out.min_similarity,0.5);assert_eq!(out.max_similarity,1.0);assert!(!out.stable);assert_eq!(out.unstable_pairs[0],("a".into(),"c".into(),0.5));}
 #[test]fn threshold_and_minimum(){let a=assignments(&vec![1;20]);let mut b=a.clone();b.insert("unit-0".into(),2);assert_eq!(compute_jaccard_similarity(&a,&b),STABILITY_THRESHOLD);assert!(analyze_suite_stability(&[("a".into(),a),("b".into(),b)]).unwrap().stable);assert!(analyze_suite_stability(&[]).is_err());}
}

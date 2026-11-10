//! Native Verify's literal GEOID/district agreement rule.
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug,Clone,PartialEq,Serialize)]
pub struct AssignmentAgreement {
    pub original_units:usize,
    pub reference_units:usize,
    pub matching_units:usize,
    pub denominator:usize,
    pub original_only_units:usize,
    pub reference_only_units:usize,
    pub changed_district_units:usize,
    pub similarity:f64,
}

/// Preserve native Verify's denominator: max(map sizes), not set-union size.
/// District labels are compared literally; they are never permuted to improve agreement.
pub fn assignment_agreement(original:&HashMap<String,usize>,reference:&HashMap<String,usize>)->AssignmentAgreement {
    let matching_units=original.iter().filter(|(unit,district)|reference.get(*unit)==Some(*district)).count();
    let common=original.keys().filter(|unit|reference.contains_key(*unit)).count();
    let denominator=original.len().max(reference.len());
    AssignmentAgreement {original_units:original.len(),reference_units:reference.len(),matching_units,denominator,original_only_units:original.len()-common,reference_only_units:reference.len()-common,changed_district_units:common-matching_units,similarity:if denominator==0{0.0}else{matching_units as f64/denominator as f64}}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn map(items:&[(&str,usize)])->HashMap<String,usize>{items.iter().map(|(u,d)|(u.to_string(),*d)).collect()}
    #[test]
    fn native_denominator_is_not_set_union(){let r=assignment_agreement(&map(&[("a",1),("b",2)]),&map(&[("a",1),("c",2)]));assert_eq!(r.denominator,2);assert_eq!(r.similarity,0.5);assert_eq!(r.original_only_units,1);assert_eq!(r.reference_only_units,1);}
    #[test]
    fn district_relabeling_changes_agreement(){let r=assignment_agreement(&map(&[("a",1),("b",2)]),&map(&[("a",2),("b",1)]));assert_eq!(r.similarity,0.0);assert_eq!(r.changed_district_units,2);}
    #[test]
    fn empty_maps_have_zero_similarity(){assert_eq!(assignment_agreement(&map(&[]),&map(&[])).similarity,0.0);assert_eq!(assignment_agreement(&map(&[("a",0)]),&map(&[])).original_only_units,1);}
    #[test]
    fn asymmetric_coverage_and_zero_district_are_retained(){let r=assignment_agreement(&map(&[("a",0)]),&map(&[("a",0),("b",1)]));assert_eq!(r.similarity,0.5);assert_eq!(r.matching_units,1);assert_eq!(r.reference_only_units,1);}
}

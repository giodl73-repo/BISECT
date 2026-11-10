//! Native State weight order, shared by filesystem and selected-byte hosts.
use std::collections::HashMap;
use bisect_core::edge_weights::*;
use crate::state_config::WeightSpec;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Route {Unweighted,SingleDistrict,Compose}
/// Preserve the native early branch, including currently unused declarations.
pub fn route(spec:&WeightSpec,districts:usize)->Route{
 if !spec.geographic&&!spec.minority_weighting&&spec.partisan_shares.is_none()&&spec.alpha_county<1e-10&&spec.alpha_mcd<1e-10&&spec.alpha_place<1e-10&&spec.alpha_vtd<1e-10{Route::Unweighted}else if districts==1{Route::SingleDistrict}else{Route::Compose}
}
#[derive(Default)]
pub struct LoadedSignals {
 pub minority_fractions:Option<Vec<f64>>,
 pub partisan_shares:Option<Vec<f64>>,
 pub economic:Option<Vec<bisect_core::economic_character::EconChar>>,
 pub housing:Option<Vec<bisect_core::housing_character::HousingChar>>,
}
/// Loading policy belongs to the caller: required minority/partisan errors are
/// not converted into defaults; missing/error character files omit that stage.
pub fn compose(spec:&WeightSpec,adjacency:&[Vec<usize>],physical:&EdgeMap,ids:&HashMap<usize,String>,loaded:LoadedSignals)->Result<EdgeMap,String>{
 let edges:Vec<_>=adjacency.iter().enumerate().flat_map(|(i,ns)|ns.iter().filter(move |&&j|j>i).map(move |&j|(i,j))).collect();
 let mut composer=ComposedWeighter::new();
 if spec.geographic{composer=composer.push(GeographicWeighter::from_map(physical.clone()));}
 if spec.minority_weighting{composer=composer.push(MinorityOverrideWeighter::new(edges.clone(),loaded.minority_fractions.ok_or("Native minority stage requires loaded fractions")?,0.40));}
 if spec.partisan_shares.as_ref().is_some_and(|p|!p.as_os_str().is_empty()){composer=composer.push(PartisanOverrideWeighter::new(edges,loaded.partisan_shares.ok_or("Native partisan stage requires loaded shares")?,spec.dem_threshold,spec.rep_threshold));}
 if spec.alpha_county>1e-10{composer=composer.push(SubdivisionWeighter::county_only(ids,adjacency.len(),spec.alpha_county));}
 if spec.economic_character{if let Some(values)=loaded.economic{composer=composer.push(EconomicCharacterWeighter::new(values,spec.econ_alpha));}}
 if spec.housing_character{if let Some(values)=loaded.housing{composer=composer.push(HousingCharacterWeighter::new(values,spec.econ_alpha));}}
 // Native zone discovery currently warns and omits its stage. It does not
 // infer explicit zone membership or consume the unused subdivision alphas.
 Ok(if composer.is_empty(){physical.clone()}else{composer.apply()})
}

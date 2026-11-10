//! Native State electoral metadata; generation still targets equal district populations.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
 #[serde(default="single_member")]pub seats_per_district:usize,
 #[serde(default,skip_serializing_if="Option::is_none")]pub total_seats:Option<usize>,
}
fn single_member()->usize{1}
impl Input {
 pub(crate) fn resolve(&self,districts:usize)->Result<(usize,usize,&'static str),String>{
  if districts==0||districts>500||self.seats_per_district>u32::MAX as usize||self.total_seats.is_some_and(|v|v>u32::MAX as usize){return Err("Electoral metadata supports 1–500 districts, unsigned 32-bit seats-per-district and total-seat values.".into());}
  // CLI State computes default total from the requested count, before max(1).
  let total64=self.total_seats.map(|v|v as u64).unwrap_or(self.seats_per_district as u64*districts as u64);
  if total64>u32::MAX as u64 {return Err("Derived total seats exceed the unsigned 32-bit WASM manifest representation.".into());}
  let total=total64 as usize;
  Ok((self.seats_per_district.max(1),total,if self.seats_per_district<=1{"single_member"}else{"multi_member_uniform"}))
 }
 pub(crate) fn hash(&self)->Result<String,String>{rplan_core::canonical_sha256(&serde_json::to_value(self).map_err(|e|e.to_string())?).map_err(|e|e.to_string())}
 pub(crate) fn evidence(&self,districts:usize,population:i64)->Result<Value,String>{
  let (seats,total,system)=self.resolve(districts)?;
  Ok(json!({"schema_version":"bisect-electoral-settings-v1","requested_seats_per_district":self.seats_per_district,"requested_total_seats":self.total_seats,"seats_per_district":seats,"total_seats":total,"electoral_system":system,"districts":districts,"uniform_total_consistent":total as u64==seats as u64*districts as u64,"ideal_population_per_seat":population as f64/total.max(1) as f64,"ideal_population_per_district":population as f64/districts as f64,"settings_sha256":self.hash()?,"scope":"Native State manifest/settings semantics. Default total seats equals requested seats-per-district times district count; effective seats-per-district and the per-seat population denominator are each clamped to at least one. Equal district population targets and assignments do not change. Inconsistent totals are disclosed as recorded metadata; variable district magnitudes and seat-weighted generation/audits are not implemented by this native path. Electoral metadata is unauthenticated and does not establish legal eligibility or an electoral-system outcome."}))
 }
}

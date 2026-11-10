//! Scoped, bounded observation of native intermediate-round events without filesystem access.
use std::{cell::RefCell,collections::{BTreeMap,HashMap},path::Path};
#[derive(Clone,Debug)]
pub struct CapturedRound {pub name:String,pub assignments:BTreeMap<usize,usize>,pub complete_universe:bool}
pub type RoundObserver=fn(&str,usize,usize,usize)->Result<(),String>;
struct State {units:usize,entries:usize,frames:Vec<CapturedRound>,failure:Option<String>,observer:Option<RoundObserver>}
thread_local! {static CAPTURE:RefCell<Option<State>>=const {RefCell::new(None)};}
pub struct RoundCapture {finished:bool}
impl RoundCapture {
 pub fn start(units:usize)->Result<Self,String>{Self::start_observed(units,None)}
 /// Observe actual capture callsites. Indices restart for each capture (including retries).
 pub fn start_observed(units:usize,observer:Option<RoundObserver>)->Result<Self,String>{
  if super::round_monitor::active(){return Err("Nested native round observation is not supported".into());}
  if units==0||units>100_000{return Err("Round capture supports 1–100,000 units".into());}
  CAPTURE.with(|slot|{let mut slot=slot.borrow_mut();if slot.is_some(){return Err("Nested round capture is not supported".into());}*slot=Some(State{units,entries:0,frames:vec![],failure:None,observer});Ok(Self{finished:false})})
 }
 pub fn finish(mut self)->Result<Vec<CapturedRound>,String>{self.finished=true;CAPTURE.with(|slot|{let state=slot.borrow_mut().take().ok_or("Round capture is inactive")?;if let Some(error)=state.failure{return Err(error);}Ok(state.frames)})}
}
impl Drop for RoundCapture {fn drop(&mut self){if !self.finished{CAPTURE.with(|slot|{slot.borrow_mut().take();});}}}
pub(crate) fn round_output_enabled(directory:Option<&Path>)->bool{directory.is_some()||CAPTURE.with(|slot|slot.borrow().is_some())||super::round_monitor::active()}
pub(crate) fn observe_round(name:&str,assignments:&HashMap<usize,usize>)->Result<(),String>{
 super::round_monitor::observe(name,assignments)?;
 let notification=CAPTURE.with(|slot|{
  let mut slot=slot.borrow_mut();let Some(state)=slot.as_mut() else{return Ok(None);};
  let error=if state.frames.len()>=128||state.entries.saturating_add(assignments.len())>2_000_000 {Some("Round capture exceeds 128 events or two million assignment entries")}
   else if name.is_empty()||name.len()>80||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_') {Some("Round capture event name is invalid")}
   else if assignments.iter().any(|(unit,region)|*unit>=state.units||*region==0||*region>u32::MAX as usize){Some("Round capture has an invalid unit index or region")}
   else{None};
  if let Some(error)=error{state.failure=Some(error.into());return Err(error.into());}
  if let Some(error)=&state.failure{return Err(error.clone());}
  state.entries+=assignments.len();state.frames.push(CapturedRound{name:name.into(),assignments:assignments.iter().map(|(k,v)|(*k,*v)).collect(),complete_universe:assignments.len()==state.units});Ok(state.observer.map(|observe|(observe,state.frames.len(),state.units)))
 })?;
 // Release the capture borrow before calling host/user code. A notification
 // failure latches exactly like a capture writer failure, even if ignored upstream.
 if let Some((observe,index,units))=notification {
  if let Err(error)=observe(name,index,assignments.len(),units){CAPTURE.with(|slot|{if let Some(state)=slot.borrow_mut().as_mut(){state.failure=Some(error.clone());}});return Err(error);}
 }
 Ok(())
}
#[cfg(test)] mod tests{
 use super::*;
 #[test] fn scoped_events_and_cleanup(){let capture=RoundCapture::start(2).unwrap();assert!(RoundCapture::start(2).is_err());observe_round("depth_01",&HashMap::from([(1,2),(0,1)])).unwrap();observe_round("depth_01",&HashMap::from([(0,3)])).unwrap();let frames=capture.finish().unwrap();assert_eq!(frames.len(),2);assert_eq!(frames[0].assignments.keys().copied().collect::<Vec<_>>(),vec![0,1]);assert!(frames[0].complete_universe);assert!(!frames[1].complete_universe);assert!(!round_output_enabled(None));drop(RoundCapture::start(2).unwrap());assert!(!round_output_enabled(None));}
 #[test] fn ignored_writer_failures_are_not_silently_truncated(){let capture=RoundCapture::start(1).unwrap();for _ in 0..128{observe_round("depth_00",&HashMap::from([(0,1)])).unwrap();}assert!(observe_round("depth_00",&HashMap::from([(0,1)])).is_err());assert!(capture.finish().is_err());assert!(!round_output_enabled(None));}
 #[test] fn invalid_unit_and_region_latch_failure(){for assignment in [HashMap::from([(2,1)]),HashMap::from([(0,0)])]{let capture=RoundCapture::start(2).unwrap();assert!(observe_round("depth_00",&assignment).is_err());assert!(capture.finish().is_err());}}
}

#[cfg(test)] mod observer_tests {
 use super::*;
 thread_local!{static SEEN:RefCell<Vec<(String,usize,usize,usize)>>=const {RefCell::new(vec![])};}
 fn notify(name:&str,index:usize,assigned:usize,units:usize)->Result<(),String>{
  assert!(RoundCapture::start(units).is_err()); // Reentry reports an error instead of a borrow panic.
  SEEN.with(|events|events.borrow_mut().push((name.into(),index,assigned,units)));Ok(())
 }
 fn refuse(_: &str,_:usize,_:usize,_:usize)->Result<(),String>{Err("host refused observation".into())}
 #[test] fn observations_are_actual_ordered_attempt_local_events(){
  SEEN.with(|v|v.borrow_mut().clear());
  for _ in 0..2 {let capture=RoundCapture::start_observed(2,Some(notify)).unwrap();observe_round("depth_01",&HashMap::from([(0,1)])).unwrap();observe_round("depth_02",&HashMap::from([(0,1),(1,2)])).unwrap();assert_eq!(capture.finish().unwrap().len(),2);}
  SEEN.with(|v|assert_eq!(*v.borrow(),vec![("depth_01".into(),1,1,2),("depth_02".into(),2,2,2),("depth_01".into(),1,1,2),("depth_02".into(),2,2,2)]));
 }
 #[test] fn host_failure_latches_and_drop_removes_observer(){
  let capture=RoundCapture::start_observed(1,Some(refuse)).unwrap();assert!(observe_round("depth_01",&HashMap::from([(0,1)])).is_err());assert!(capture.finish().unwrap_err().contains("host refused"));
  drop(RoundCapture::start_observed(1,Some(refuse)).unwrap());let capture=RoundCapture::start(1).unwrap();observe_round("depth_01",&HashMap::from([(0,1)])).unwrap();assert_eq!(capture.finish().unwrap().len(),1);
 }
}

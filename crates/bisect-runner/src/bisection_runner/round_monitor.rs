//! Constant-space observation of existing native round callsites, without retaining a trace.
use std::{cell::RefCell,collections::HashMap};

#[derive(Clone,Debug)]
pub struct RoundMonitorEvent {
 pub phase:&'static str,pub name:Option<String>,pub index:u64,pub notified:u64,
 pub units:usize,pub assigned_units:usize,pub invalid_entries:u64,pub invalid_name:bool,pub complete_universe:bool,
}
pub type MonitorObserver=fn(&RoundMonitorEvent)->Result<(),String>;
struct State {last:RoundMonitorEvent,observer:MonitorObserver,failure:Option<String>}
thread_local!{static MONITOR:RefCell<Option<State>>=const {RefCell::new(None)};}
pub struct RoundMonitor {finished:bool}
pub(crate) fn active()->bool{MONITOR.with(|s|s.borrow().is_some())}
impl RoundMonitor {
 pub fn start(units:usize,observer:MonitorObserver)->Result<Self,String>{
  if super::round_capture::round_output_enabled(None){return Err("Nested native round observation is not supported".into());}
  let event=RoundMonitorEvent{phase:"entered",name:None,index:0,notified:0,units,assigned_units:0,invalid_entries:0,invalid_name:false,complete_universe:false};
  MONITOR.with(|s|*s.borrow_mut()=Some(State{last:event.clone(),observer,failure:None}));
  let guard=Self{finished:false};
  // Preserve ordinary graph-validation errors for invalid dimensions. No native
  // method executes in that case; Drop retires the unused observer.
  if (1..=1_000_000).contains(&units){observer(&event)?;}
  Ok(guard)
 }
 pub fn finish(mut self)->Result<(),String>{
  self.finished=true;let mut state=MONITOR.with(|s|s.borrow_mut().take()).ok_or("Native round monitor is inactive")?;
  if let Some(error)=state.failure{return Err(error);}
  state.last.phase="closed";(state.observer)(&state.last)
 }
}
impl Drop for RoundMonitor {fn drop(&mut self){if !self.finished{MONITOR.with(|s|s.borrow_mut().take());}}}
pub(crate) fn observe(name:&str,assignments:&HashMap<usize,usize>)->Result<(),String>{
 let notification=MONITOR.with(|slot|{
  let mut slot=slot.borrow_mut();let Some(state)=slot.as_mut()else{return Ok(None)};
  if let Some(error)=&state.failure{return Err(error.clone());}
  let index=state.last.index.checked_add(1).ok_or("Native observation counter exhausted")?;
  let units=state.last.units;
  let assigned=assignments.iter().filter(|(u,r)|**u<units&&**r>0&&**r<=u32::MAX as usize).count();
  let valid_name=!name.is_empty()&&name.len()<=80&&name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_');
  let notify=index<=16||index.is_power_of_two();let notified=state.last.notified+u64::from(notify);
  state.last=RoundMonitorEvent{phase:"snapshot",name:valid_name.then(||name.into()),index,notified,units,assigned_units:assigned,invalid_entries:(assignments.len()-assigned)as u64,invalid_name:!valid_name,complete_universe:assigned==units&&assignments.len()==units};
  // At most 75 snapshot notifications over the whole u64 counter range.
  // Always count every call and retain only its small final summary.
  Ok(notify.then(||(state.observer,state.last.clone())))
 })?;
 if let Some((observer,event))=notification {if let Err(error)=observer(&event){MONITOR.with(|s|{if let Some(state)=s.borrow_mut().as_mut(){state.failure=Some(error.clone());}});return Err(error);}}
 Ok(())
}

#[cfg(test)]mod tests{
 use super::*;
 use super::super::round_capture::{RoundCapture,observe_round};
 thread_local!{static EVENTS:RefCell<Vec<RoundMonitorEvent>>=const{RefCell::new(vec![])};}
 fn retain(event:&RoundMonitorEvent)->Result<(),String>{EVENTS.with(|v|v.borrow_mut().push(event.clone()));Ok(())}
 fn fail(_: &RoundMonitorEvent)->Result<(),String>{Err("monitor host failure".into())}
 fn fail_snapshot(event:&RoundMonitorEvent)->Result<(),String>{if event.phase=="snapshot"{fail(event)}else{Ok(())}}
 #[test]fn constant_space_sampling_counts_every_call_beyond_capture_limits(){
  EVENTS.with(|v|v.borrow_mut().clear());let monitor=RoundMonitor::start(1,retain).unwrap();assert!(RoundCapture::start(1).is_err());
  for _ in 0..300{observe_round("depth_01",&HashMap::from([(0,1)])).unwrap();}
  MONITOR.with(|v|assert_eq!(v.borrow().as_ref().unwrap().last.index,300));monitor.finish().unwrap();
  EVENTS.with(|v|{let e=v.borrow();assert_eq!(e.len(),22);assert_eq!(e.first().unwrap().phase,"entered");let last=e.last().unwrap();assert_eq!(last.phase,"closed");assert_eq!(last.index,300);assert_eq!(last.notified,20);assert_eq!(last.assigned_units,1);assert!(last.complete_universe);});assert!(!active());
 }
 #[test]fn observer_failures_and_drop_retire_monitor(){
  assert!(RoundMonitor::start(1,fail).is_err());assert!(!active());let monitor=RoundMonitor::start(1,fail_snapshot).unwrap();assert!(observe_round("depth_01",&HashMap::from([(0,1)])).is_err());assert!(monitor.finish().unwrap_err().contains("host failure"));assert!(!active());drop(RoundMonitor::start(1,retain).unwrap());assert!(!active());
 }
 #[test]fn invalid_snapshot_entries_are_disclosed_without_rejecting_normal_execution(){
  EVENTS.with(|v|v.borrow_mut().clear());let monitor=RoundMonitor::start(1,retain).unwrap();observe_round("<invalid>",&HashMap::from([(0,0),(9,1)])).unwrap();monitor.finish().unwrap();EVENTS.with(|v|{let e=v.borrow();assert_eq!(e[1].invalid_entries,2);assert!(e[1].invalid_name);assert_eq!(e[1].name,None);assert_eq!(e[1].assigned_units,0);assert!(!e[1].complete_universe);});
 }
}

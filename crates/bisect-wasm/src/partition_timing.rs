//! Opt-in observations around dispatch only, before metrics/population checks.
use std::cell::RefCell;
use serde_json::{json, Value};

struct State { events: Vec<Value>, next: usize, omitted: usize, dispatching: bool }
thread_local! { static ACTIVE: RefCell<Option<State>> = const { RefCell::new(None) }; }

#[cfg(target_arch="wasm32")]
#[link(wasm_import_module="env")]
extern "C" { fn bisect_monotonic_millis() -> f64; }
fn now() -> f64 {
    #[cfg(target_arch="wasm32")]
    { unsafe { bisect_monotonic_millis() } }
    #[cfg(not(target_arch="wasm32"))]
    { thread_local! { static ORIGIN: std::time::Instant = std::time::Instant::now(); }
      ORIGIN.with(|origin| origin.elapsed().as_secs_f64()*1000.0) }
}
fn clock() -> &'static str { if cfg!(target_arch="wasm32") { "host-monotonic-milliseconds" } else { "std-instant" } }
fn elapsed(start:f64,end:f64)->Option<f64>{
    (start.is_finite() && end.is_finite() && start>=0.0 && end>=start).then_some(end-start).filter(|v|v.is_finite())
}

pub(crate) struct Capture { active: bool }
impl Capture {
    pub(crate) fn start()->Result<Self,String>{
        ACTIVE.with(|cell| { let mut state=cell.borrow_mut();
            if state.is_some(){return Err("Partition timing capture is already active.".into());}
            *state=Some(State{events:Vec::new(),next:0,omitted:0,dispatching:false});Ok(Self{active:true})
        })
    }
    pub(crate) fn finish(mut self)->Value{
        let state=ACTIVE.with(|cell|cell.borrow_mut().take()).expect("Owned timing capture");self.active=false;
        json!({"schema_version":"bisect-partition-timing-report-v1","clock":clock(),"events":state.events,"omitted_events":state.omitted,
            "scope":"Observed successful dispatcher returns per population-retry attempt. Weight/source preparation, final population validation, metrics and export are excluded. Multiscale includes its sampler dispatch. Values are runtime observations, not deterministic evidence, source authenticity or performance guarantees."})
    }
}
impl Drop for Capture {fn drop(&mut self){if self.active{ACTIVE.with(|cell|{cell.borrow_mut().take();});}}}

pub(crate) struct Attempt { measurement: Option<(f64,Value)> }
impl Attempt {
    pub(crate) fn start(state:&str,year:&str,units:usize,districts:usize,seed:Option<u64>)->Self{
        let meta=ACTIVE.with(|cell| { let mut slot=cell.borrow_mut();let s=slot.as_mut()?;
            if s.dispatching{return None;}s.dispatching=true;let attempt=s.next;s.next+=1;
            Some(json!({"schema_version":"bisect-native-partition-timing-v1","state":state,"year":year,"units":units,"districts":districts,"attempt":attempt,"seed":seed.map(|v|v.to_string()),"clock":clock()}))
        });
        Self{measurement:meta.map(|meta|(now(),meta))}
    }
    pub(crate) fn finish(mut self)->Result<(),String>{
        if let Some((start,mut event))=self.measurement.take(){
            let duration=elapsed(start,now());event["elapsed_ms"]=json!(duration);event["status"]=json!(if duration.is_some(){"observed"}else{"clock-unavailable"});
            ACTIVE.with(|cell| {if let Some(s)=cell.borrow_mut().as_mut(){s.dispatching=false;if s.events.len()<64{s.events.push(event.clone());}else{s.omitted+=1;}}});
            crate::progress::emit(&event)?;
        }Ok(())
    }
}
impl Drop for Attempt {fn drop(&mut self){if self.measurement.is_some(){ACTIVE.with(|cell|{if let Some(s)=cell.borrow_mut().as_mut(){s.dispatching=false;}});}}}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn invalid_clocks_are_explicitly_unavailable(){
        assert_eq!(elapsed(2.0,5.0),Some(3.0));assert_eq!(elapsed(0.0,0.0),Some(0.0));
        for (a,b) in [(2.0,1.0),(-1.0,2.0),(f64::NAN,1.0),(1.0,f64::INFINITY)]{assert_eq!(elapsed(a,b),None);}
    }
    #[test] fn nested_dispatch_does_not_double_count_and_seed_stays_exact(){
        let capture=Capture::start().unwrap();let outer=Attempt::start("RI","2020",4,2,Some(u64::MAX));
        Attempt::start("RI","2020",4,2,Some(2)).finish().unwrap();outer.finish().unwrap();
        let report=capture.finish();assert_eq!(report["events"].as_array().unwrap().len(),1);assert_eq!(report["events"][0]["seed"],u64::MAX.to_string());assert_eq!(report["events"][0]["attempt"],0);
    }
    #[test] fn failed_dispatch_and_dropped_capture_release_state(){
        {let _capture=Capture::start().unwrap();let _attempt=Attempt::start("RI","2020",4,2,None);}
        let capture=Capture::start().unwrap();assert!(capture.finish()["events"].as_array().unwrap().is_empty());
        let capture=Capture::start().unwrap();Attempt::start("RI","2020",4,2,None).finish().unwrap();assert_eq!(capture.finish()["events"][0]["attempt"],0);
    }
    #[test] fn bounded_retention_reports_omissions(){
        let capture=Capture::start().unwrap();for _ in 0..70{Attempt::start("RI","2020",4,2,None).finish().unwrap();}
        let report=capture.finish();assert_eq!(report["events"].as_array().unwrap().len(),64);assert_eq!(report["omitted_events"],6);
    }
}

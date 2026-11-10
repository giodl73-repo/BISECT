//! Bounded, instance-local national map construction over one state input at a time.
//! Retains native dissolved district geometry, never all raw state source strings.
use serde::Deserialize;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;

const MAX_DISSOLVED_POSITIONS: usize = 4_000_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Begin { pub dpi: u32, pub map_type: String, #[serde(default)]pub allow_empty: bool, #[serde(default)]pub analysis_policy:AnalysisPolicy,#[serde(default)]pub output_profile:OutputProfile }
#[derive(Deserialize,Default,Clone,Copy,PartialEq)]
#[serde(rename_all="kebab-case")]
pub(crate) enum AnalysisPolicy {#[default]Validated,NativeDefaults}

#[derive(Deserialize,Default,Clone,Copy,PartialEq)]
#[serde(rename_all="kebab-case")]
pub(crate) enum OutputProfile {#[default]Standard,NativeOriginal}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Append {
    pub session: u32,
    pub state: String,
    pub input: crate::map_export::Input,
}

#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct RawAppend {pub session:u32,pub state:String,pub input:crate::native_assignment_map::Input}

#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct TigerAppend {pub session:u32,pub state:String,pub input:crate::native_tiger_assignment_map::Input}
pub(crate) fn append_tiger_step(input:TigerAppend)->Result<Value,String>{
 let slug=validate_settings(input.session,&input.state,input.input.dpi,&input.input.map_type)?;
 if input.input.state!=input.state{return Err("Native TIGER state differs from stream state".into());}
 let native=STORAGE.with(|s|s.borrow().active.as_ref().is_some_and(|c|c.session==input.session&&c.analysis_policy==AnalysisPolicy::NativeDefaults));
 if !native{return Err("Native TIGER loading requires explicit native national analysis policy".into());}
 let result=crate::native_tiger_assignment_map::prepare(&input.input).and_then(|(geometry,k,analysis,metadata)|commit(input.session,&input.state,&input.input.map_type,slug,geometry,k,analysis,metadata));
 Ok(match result{Ok(progress)=>json!({"schema_version":"bisect-national-map-append-step-v1","status":"appended","progress":progress,"error":Value::Null}),Err(error)=>json!({"schema_version":"bisect-national-map-append-step-v1","status":"state-error","progress":Value::Null,"error":error})})
}

struct State {
    districts: Vec<bisect_map::renderer::MapDistrict>,
    provenance: Value,
}
struct Context {
    session: u32,
    dpi: u32,
    map_type: String,
    allow_empty: bool,
    analysis_policy:AnalysisPolicy,
    output_profile:OutputProfile,
    year: Option<Value>,
    states: BTreeMap<String, State>,
    units: u64,
    districts: usize,
    source_positions: u64,
    dissolved_positions: usize,
}
#[derive(Default)]
struct Storage { next_session: u32, active: Option<Context> }
thread_local! { static STORAGE: RefCell<Storage> = RefCell::new(Storage::default()); }

fn state_slug(code: &str) -> Result<String, String> {
    let manifest: bisect_core::fetch_plan::Manifest = serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
    if code.len()!=2 || code == "DC" || !code.bytes().all(|b|b.is_ascii_uppercase()) { return Err("National map requires an uppercase state code.".into()); }
    manifest.states.get(code).map(|state|state.name.to_lowercase().replace(' ', "_"))
        .ok_or_else(||"National map requires one of the 50 state codes.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn reset() { STORAGE.with(|s|*s.borrow_mut()=Storage::default()); }
    #[test]
    fn lifecycle_rejects_stale_sessions_without_dropping_active_work() {
        reset();
        assert!(begin(Begin{dpi:73,map_type:"districts".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).is_err());
        assert!(begin(Begin{dpi:72,map_type:"rounds".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).is_err());
        let first=begin(Begin{dpi:72,map_type:"districts".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).unwrap()["session"].as_u64().unwrap() as u32;
        assert!(begin(Begin{dpi:150,map_type:"districts".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).is_err());
        assert!(finish(first).is_err());
        assert!(abort(first+1).is_err());
        assert!(finish(first+1).is_err());
        assert_eq!(abort(first).unwrap()["status"],"aborted");
        assert!(abort(first).is_err());
        let second=begin(Begin{dpi:150,map_type:"districts".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).unwrap()["session"].as_u64().unwrap() as u32;
        assert!(second>first);
        assert!(abort(first).is_err());
        abort(second).unwrap();
    }
}

pub(crate) fn begin(input: Begin) -> Result<Value, String> {
    if input.output_profile==OutputProfile::NativeOriginal&&input.analysis_policy!=AnalysisPolicy::NativeDefaults{return Err("Original TIGER output profile requires explicit native analysis policy".into());}
    if ![72,100,150,200,300].contains(&input.dpi) || !["districts","political","demographic","compactness"].contains(&input.map_type.as_str()) {
        return Err("Choose a native national map DPI and map type.".into());
    }
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.active.is_some() { return Err("A national map stream is already active. Finish or abort it first.".into()); }
        let session = storage.next_session.checked_add(1).ok_or("National map session IDs exhausted.")?;
        storage.next_session = session;
        storage.active = Some(Context { session, dpi:input.dpi, map_type:input.map_type, allow_empty:input.allow_empty, analysis_policy:input.analysis_policy, output_profile:input.output_profile, year:None, states:BTreeMap::new(), units:0, districts:0, source_positions:0, dissolved_positions:0 });
        Ok(json!({"schema_version":"bisect-national-map-stream-v1","status":"started","session":session}))
    })
}

fn validate_settings(session:u32,state:&str,dpi:u32,map_type:&str)->Result<String,String>{
 let slug=state_slug(state)?;
 STORAGE.with(|storage|{let storage=storage.borrow();let context=storage.active.as_ref().filter(|c|c.session==session).ok_or("No matching active national map session.")?;
 if context.states.contains_key(&slug)||context.states.len()>=50{return Err("National map state was already appended or exceeds 50 states.".into());}
 if context.dpi!=dpi||context.map_type!=map_type{return Err("State map DPI/type differs from the national stream.".into());}Ok::<(),String>(())})?;Ok(slug)
}
fn validate_standard_source_profile(session:u32)->Result<(),String>{
 if STORAGE.with(|s|s.borrow().active.as_ref().is_some_and(|c|c.session==session&&c.output_profile==OutputProfile::NativeOriginal)){return Err("Original TIGER output profile accepts direct TIGER sources only".into());}Ok(())
}
fn validate_append(input:&Append)->Result<String,String>{
 if input.input.analysis_policy!=crate::map_export::StateAnalysisPolicy::Validated{return Err("State-only analysis policy is not permitted in a national stream".into());}let slug=validate_settings(input.session,&input.state,input.input.dpi,&input.input.map_type)?;validate_standard_source_profile(input.session)?;Ok(slug)
}
fn validate_raw(input:&RawAppend)->Result<String,String>{
 let slug=validate_settings(input.session,&input.state,input.input.dpi,&input.input.map_type)?;validate_standard_source_profile(input.session)?;
 if input.state!=input.input.state{return Err("Raw source state differs from queued national state".into());}
 if !STORAGE.with(|s|s.borrow().active.as_ref().is_some_and(|c|c.analysis_policy==AnalysisPolicy::NativeDefaults)){return Err("Raw national assignments require explicit native-defaults analysis policy".into());}Ok(slug)
}
pub(crate) fn append_raw(input:RawAppend)->Result<Value,String>{
 let slug=validate_raw(&input)?;let (geometries,k,analysis,provenance)=crate::native_assignment_map::prepare_national(&input.input)?;
 commit(input.session,&input.state,&input.input.map_type,slug,geometries,k,analysis,provenance)
}
pub(crate) fn append_raw_step(input:RawAppend)->Result<Value,String>{
 validate_raw(&input)?;match append_raw(input){Ok(progress)=>Ok(json!({"schema_version":"bisect-national-map-append-step-v1","status":"appended","progress":progress,"error":Value::Null})),Err(error)=>Ok(json!({"schema_version":"bisect-national-map-append-step-v1","status":"state-error","progress":Value::Null,"error":error}))}
}
// A state failure is recoverable only after lifecycle/settings preflight passes.
pub(crate) fn append_step(input:Append)->Result<Value,String>{
 validate_append(&input)?;
 match append(input){
  Ok(progress)=>Ok(json!({"schema_version":"bisect-national-map-append-step-v1","status":"appended","progress":progress,"error":Value::Null})),
  Err(error)=>Ok(json!({"schema_version":"bisect-national-map-append-step-v1","status":"state-error","progress":Value::Null,"error":error})),
 }
}
pub(crate) fn append(input: Append) -> Result<Value, String> {
    let slug=validate_append(&input)?;
    let native_analysis=STORAGE.with(|s|s.borrow().active.as_ref().is_some_and(|c|c.session==input.session&&c.analysis_policy==AnalysisPolicy::NativeDefaults));
    let (geometries, k, analysis, provenance) = if native_analysis{crate::national_map_analysis::prepare(&input.input)?}else{crate::map_export::prepare(&input.input)?};
    commit(input.session,&input.state,&input.input.map_type,slug,geometries,k,analysis,provenance)
}
fn commit(session:u32,state:&str,map_type:&str,slug:String,geometries:bisect_map::renderer::DistrictGeometries,k:usize,analysis:Value,mut provenance:Value)->Result<Value,String>{
    if provenance["state"].as_str()!=Some(state) || provenance["year"].is_null() { return Err("State plan must declare the matching state and year.".into()); }
    let positions = geometries.values().map(|mp|mp.0.iter().map(|poly|
        poly.exterior().0.len()+poly.interiors().iter().map(|ring|ring.0.len()).sum::<usize>()
    ).sum::<usize>()).sum::<usize>();
    let districts = bisect_map::renderer::state_map_items(geometries,k,map_type,&analysis).map_err(|e|e.to_string())?;
    let units = provenance["units"].as_u64().ok_or("Missing native unit count.")?;
    let source_positions = provenance["source_positions"].as_u64().ok_or("Missing native source position count.")?;
    let year = provenance["year"].clone();
    if let Some(fields)=provenance.as_object_mut() { for key in ["schema_version","status","scope"] { fields.remove(key); } }
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        let context = storage.active.as_mut().filter(|c|c.session==session).ok_or("No matching active national map session.")?;
        if context.year.as_ref().is_some_and(|y|y!=&year) { return Err("National state plans must share a year.".into()); }
        let max_positions=if context.output_profile==OutputProfile::NativeOriginal{8_000_000}else{MAX_DISSOLVED_POSITIONS};
        if context.units+units>100_000 || context.districts+districts.len()>500 || context.dissolved_positions+positions>max_positions { return Err(if context.output_profile==OutputProfile::NativeOriginal{"Original TIGER national map exceeds 100,000 units, 500 districts or eight million dissolved positions."}else{"National map stream exceeds 100,000 units, 500 districts or four million dissolved positions."}.into()); }
        // Commit only after every check passes. Failed appends preserve prior states.
        context.units+=units;context.districts+=districts.len();context.source_positions+=source_positions;context.dissolved_positions+=positions;context.year=Some(year);
        context.states.insert(slug,State{districts,provenance:provenance.clone()});
        Ok(json!({"schema_version":"bisect-national-map-stream-v1","status":"state-prepared","session":session,"state":state,"states":context.states.len(),"units":context.units,"districts":context.districts,"source_positions":context.source_positions,"dissolved_positions":context.dissolved_positions,"source":provenance}))
    })
}

pub(crate) fn abort(session: u32) -> Result<Value,String> {
    STORAGE.with(|storage| {
        let mut storage=storage.borrow_mut();
        if !storage.active.as_ref().is_some_and(|c|c.session==session) { return Err("No matching active national map session.".into()); }
        storage.active=None;
        Ok(json!({"schema_version":"bisect-national-map-stream-v1","status":"aborted","session":session}))
    })
}

pub(crate) fn finish(session: u32) -> Result<Value,String> {
    let context=STORAGE.with(|storage| {
        let mut storage=storage.borrow_mut();
        let context=storage.active.as_ref().filter(|c|c.session==session).ok_or("No matching active national map session.")?;
        if context.states.is_empty()&&!context.allow_empty { return Err("Append at least one state before finishing a national map.".into()); }
        Ok::<Context,String>(storage.active.take().unwrap())
    })?;
    let original_output=context.output_profile==OutputProfile::NativeOriginal;
    let mut districts=Vec::with_capacity(context.districts);let mut provenance=vec![];
    for (_,state) in context.states { districts.extend(state.districts);provenance.push(state.provenance); }
    let raw_states=provenance.iter().filter(|p|p["map_source"]=="native-assignments").count();
    let (w,h)=bisect_map::canvas_size_from_dpi(context.dpi,16.0,1.6);
    let projection=bisect_map::InsetProjection::us_national(w,h);
    let svg=bisect_map::renderer::build_national_svg(&districts,&projection,w,h);
    let mut metadata=json!({"schema_version":"bisect-native-district-map-v1","status":"rendered","map_scope":"national","map_type":context.map_type,"dpi":context.dpi,"year":context.year,"units":context.units,"districts":context.districts,"provided_states":provenance,"execution_performed":false,"source_delivery":"state-stream","source_positions":context.source_positions,"dissolved_positions":context.dissolved_positions,"scope":"Supplied states prepared one at a time using the same native geometry checks and national renderer. Only dissolved district geometries are retained between appends; raw state source buffers are not stored in the session. States use native state-name ordering and original source hashes. Analysis geometry and assignments are unchanged. Bounds apply per source and to total units, districts, dissolved positions and SVG/PNG output. This display does not certify source authenticity, legal compliance or complete national coverage; actual all-state original-source capacity requires separate verification."});
    if original_output{metadata["output_profile"]=json!("native-original");metadata["svg_byte_limit"]=json!(128*1024*1024);metadata["dissolved_position_limit"]=json!(8_000_000);}
    let mut result=if original_output{crate::map_export::outputs_native_original(metadata,svg,w,h)?}else{crate::map_export::outputs(metadata,svg,w,h,4800)?};
    if raw_states>0{result["map_source"]=json!("native-assignment-stream");result["plan_hash"]=Value::Null;result["raw_assignment_states"]=json!(raw_states);result["scope"]=json!("National supplied-source stream with original native assignments. Each provided state records its actual source type and original hashes; raw states do not create or bind an RPLAN. Native national defaults/row semantics apply. State geometry and key mappings are explicit source claims, not complete national coverage, source authentication, accepted plan lineage or certification. Source and total geometry/work/output bounds remain.");}Ok(result)
}

#[cfg(test)]
mod partial_tests {
 use super::*;
 #[test]fn explicit_empty_stream_finishes_and_releases_its_session(){
  STORAGE.with(|s|*s.borrow_mut()=Storage::default());
  let session=begin(Begin{dpi:72,map_type:"districts".into(),allow_empty:true,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).unwrap()["session"].as_u64().unwrap() as u32;
  let input=||serde_json::from_value::<Append>(json!({"session":session,"state":"RI","input":{"document_json":"{}","unit_geometry_json":"{}","dpi":72}})).unwrap();
  let mut stale=input();stale.session+=1;assert!(append_step(stale).is_err());
  let failure=append_step(input()).unwrap();assert_eq!(failure["status"],"state-error");assert!(failure["progress"].is_null());
  let result=finish(session).unwrap();assert_eq!(result["districts"],0);assert_eq!(result["units"],0);assert_eq!(result["provided_states"],json!([]));assert!(result["year"].is_null());
  assert!(abort(session).is_err());let next=begin(Begin{dpi:72,map_type:"districts".into(),allow_empty:false,analysis_policy:AnalysisPolicy::Validated,output_profile:OutputProfile::Standard}).unwrap()["session"].as_u64().unwrap() as u32;abort(next).unwrap();
 }
}

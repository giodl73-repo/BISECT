//! Explicit graph/district placeholders, bound entirely inside Rust. Other JSON
//! value tokens remain raw until the existing typed request validator sees them.
use serde::{Deserialize,Serialize,de::{Visitor,MapAccess,Error}};
use serde_json::{Value,json,value::RawValue};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fmt};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input{pub template_json:String,pub graph_json:String,pub districts:usize}
#[derive(Serialize)]
struct RawMap(BTreeMap<String,Box<RawValue>>);
impl<'de> Deserialize<'de> for RawMap{
 fn deserialize<D:serde::Deserializer<'de>>(deserializer:D)->Result<Self,D::Error>{
  struct Fields;
  impl<'de> Visitor<'de> for Fields{
   type Value=RawMap;
   fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("a request object with unique fields")}
   fn visit_map<M:MapAccess<'de>>(self,mut map:M)->Result<RawMap,M::Error>{
    let mut fields=BTreeMap::new();while let Some((key,value))=map.next_entry::<String,Box<RawValue>>()?{if fields.insert(key,value).is_some(){return Err(M::Error::custom("Duplicate prepared template field."));}}Ok(RawMap(fields))
   }
  }
  deserializer.deserialize_map(Fields)
 }
}
fn map(text:&str)->Result<RawMap,String>{serde_json::from_str(text).map_err(|e|format!("Invalid prepared template: {e}"))}
fn raw<T:Serialize>(value:&T)->Result<Box<RawValue>,String>{serde_json::value::to_raw_value(value).map_err(|e|e.to_string())}
fn bind_request(mut request:RawMap,graph:&crate::engine::PreparedGraph,districts:usize)->Result<RawMap,String>{
 if request.0.get("graph").map(|v|v.get())!=Some("null"){return Err("Declare graph:null in the engine template; an existing graph is never replaced.".into());}
 let mut options=map(request.0.get("options").ok_or("Template requires engine options.")?.get())?;
 let selected=options.0.get("districts").ok_or("Declare options.districts:null or an explicit matching count.")?;
 if selected.get()=="null"{options.0.insert("districts".into(),raw(&districts)?);}else{
  let value:usize=serde_json::from_str(selected.get()).map_err(|_|"Template district count must be null or an unsigned integer.")?;
  if value!=districts{return Err("Explicit template district count differs from the selected native allocation.".into());}
 }
 request.0.insert("options".into(),raw(&options)?);request.0.insert("graph".into(),raw(graph)?);Ok(request)
}
pub fn bind(input:Input)->Result<Value,String>{
 if input.template_json.len()>25*1024*1024||input.graph_json.len()>25*1024*1024||input.districts<1||input.districts>500{return Err("Select template/graph JSON up to25MiB each and 1-500 districts.".into());}
 let graph:crate::engine::PreparedGraph=serde_json::from_str(&input.graph_json).map_err(|e|format!("Invalid selected graph: {e}"))?;
 let mut template=map(&input.template_json)?;
 if let Some(operation)=template.0.get("operation"){
  let operation:String=serde_json::from_str(operation.get()).map_err(|_|"Template operation must be a string.")?;
  match operation.as_str(){
   "run-engine-build"=>{let request=map(template.0.get("request").ok_or("Build template requires request.")?.get())?;template.0.insert("request".into(),raw(&bind_request(request,&graph,input.districts)?)?);},
   "run-multiscale"|"run-multiscale-build"=>{let mut inner=map(template.0.get("input").ok_or("Multiscale template requires input.")?.get())?;let request=map(inner.0.get("request").ok_or("Multiscale template requires input.request.")?.get())?;inner.0.insert("request".into(),raw(&bind_request(request,&graph,input.districts)?)?);template.0.insert("input".into(),raw(&inner)?);},
   _=>return Err("Only ordinary, engine-build and multiscale engine templates are supported.".into())
  }
 }else{template=bind_request(template,&graph,input.districts)?;}
 let bound=serde_json::to_string(&template).map_err(|e|e.to_string())?;
 let review=crate::prepared_engine::run(crate::prepared_engine::Input{context_source:None,inspect_only:true,request_json:bound.clone(),label:"Template binding".into(),chamber:"congressional".into(),created_at:String::new()})?;
 Ok(json!({"schema_version":"bisect-prepared-engine-binding-v1","state":graph.state,"year":graph.year,"unit_kind":graph.effective_unit_kind(),"units":graph.geoids.len(),"districts":input.districts,"template_sha256":format!("{:x}",Sha256::digest(input.template_json.as_bytes())),"graph_sha256":format!("{:x}",Sha256::digest(input.graph_json.as_bytes())),"bound_request_json":bound,"bound_request_sha256":review["request_sha256"],"review":review,"execution_performed":false,"csv_loading_performed":false,"scope":"Explicit graph:null and district:null placeholder binding followed by typed native graph/base-option review. Other raw value tokens are retained, including numeric u64 values. No partition, source acquisition, CSV loading or plan certification. Existing graphs and conflicting explicit district counts reject."}))
}

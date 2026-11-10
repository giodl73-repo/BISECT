//! Native location Doctor over explicit source bytes and selected-root observations.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::Engine;
use std::collections::BTreeMap;
use crate::location_policy::Candidate;
use bisect_analysis::location_registry::{PolicyRegistry,chamber_balance_tolerance};

#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct PolicySources{pub primary:Option<Candidate>,pub alias:Option<Candidate>,pub home:Option<Candidate>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct ManifestSources{pub primary:Option<Candidate>,pub home:Option<Candidate>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Entry{pub path:String,pub kind:String}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Snapshot{pub entries:Vec<Entry>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input{pub settings:crate::doctor::Settings,pub output_base:String,pub policy:PolicySources,pub manifest:ManifestSources,pub outputs:Option<Snapshot>,pub workspace:Option<Snapshot>}

fn bytes(candidate:&Candidate,max:usize)->Result<(String,Option<Vec<u8>>),String>{
 let (name,encoded)=match candidate{Candidate::File{name,base64}=>(name,Some(base64)),Candidate::Unavailable{name}=>(name,None)};
 if name.is_empty()||name.len()>200||name.chars().any(char::is_control){return Err("Invalid native Doctor source display name.".into());}
 let Some(encoded)=encoded else{return Ok((name.clone(),None));};
 if encoded.len()>max.div_ceil(3)*4{return Err("Native Doctor source exceeds its byte bound.".into());}
 let data=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_|"Invalid native Doctor source base64.")?;
 if data.len()>max{return Err("Native Doctor source exceeds its byte bound.".into());}Ok((name.clone(),Some(data)))
}
fn policy(sources:&PolicySources,home:bool)->Result<(Value,Value),String>{
 let mut attempts=vec![];
 for (role,candidate) in [("primary",sources.primary.as_ref()),("alias",sources.alias.as_ref()),("home",if home{sources.home.as_ref()}else{None})]{
  if role=="home"&&!home{continue;}
  let Some(candidate)=candidate else{attempts.push(json!({"role":role,"status":"not-configured"}));continue;};
  let (name,data)=bytes(candidate,512*1024)?;
  let Some(data)=data else{attempts.push(json!({"role":role,"name":name,"status":"unavailable"}));continue;};
  let hash=format!("sha256:{:x}",Sha256::digest(&data));
  let text=match std::str::from_utf8(&data){Ok(v)=>v,Err(_)=>{attempts.push(json!({"role":role,"name":name,"status":"read-refused","sha256":hash}));continue;}};
  match serde_json::from_str::<Value>(text){
   Ok(value)=>{attempts.push(json!({"role":role,"name":name,"status":"selected","sha256":hash}));return Ok((value,json!({"selected_role":role,"selected_sha256":hash,"attempts":attempts})));},
   Err(e)=>attempts.push(json!({"role":role,"name":name,"status":"parse-refused","sha256":hash,"error":e.to_string()})),
  }
 }
 let text=bisect_analysis::location_policy::EMBEDDED_POLICY;
 Ok((serde_json::from_str(text).map_err(|e|e.to_string())?,json!({"selected_role":"embedded","selected_sha256":format!("sha256:{:x}",Sha256::digest(text.as_bytes())),"attempts":attempts})))
}
fn manifest(sources:&ManifestSources,code:&str,year:&str)->Result<(Option<u64>,Value),String>{
 for (role,candidate) in [("primary",sources.primary.as_ref()),("home",sources.home.as_ref())]{
  let Some(candidate)=candidate else{continue;};let (name,data)=bytes(candidate,2*1024*1024)?;
  let Some(data)=data else{return Ok((None,json!({"selected_role":role,"name":name,"status":"read-refused"})));};
  let hash=format!("sha256:{:x}",Sha256::digest(&data));
  let parsed=std::str::from_utf8(&data).map_err(|e|e.to_string()).and_then(|s|serde_json::from_str::<bisect_core::fetch_plan::ManifestWithDistricts<u64>>(s).map_err(|e|e.to_string()));
  return Ok(match parsed{
   Ok(m)=>(m.states.get(code).and_then(|s|s.districts.get(year)).copied().filter(|n|*n>0),json!({"selected_role":role,"name":name,"status":"selected","sha256":hash})),
   Err(e)=>(None,json!({"selected_role":role,"name":name,"status":"parse-refused","sha256":hash,"error":e})),
  });
 }
 let text=include_str!("../../../data/manifest.json");let m:bisect_core::fetch_plan::ManifestWithDistricts<u64>=serde_json::from_str(text).map_err(|e|e.to_string())?;
 Ok((m.states.get(code).and_then(|s|s.districts.get(year)).copied().filter(|n|*n>0),json!({"selected_role":"embedded","status":"selected","sha256":format!("sha256:{:x}",Sha256::digest(text.as_bytes()))})))
}
// A selected root grants no outside path authority. Match the current native Windows
// reference for portable ASCII components; other path domains remain unsupported.
fn key(path:&str)->Result<String,String>{
 if !path.is_ascii()||path.contains(['\\',':','\0','*','?','"','<','>','|'])||path.starts_with('/')||path.len()>1024||path.chars().any(char::is_control){return Err("Path is outside portable Windows-ASCII snapshot support.".into());}
 let mut parts=vec![];for part in path.split('/'){
  if part=="."{continue;}if part.is_empty()||part==".."||part.ends_with(['.',' ']){return Err("Path is outside selected-root support.".into());}parts.push(part.to_ascii_lowercase());
 }Ok(parts.join("/"))
}
fn inventory(snapshot:Option<Snapshot>)->Result<Option<BTreeMap<String,String>>,String>{
 let Some(snapshot)=snapshot else{return Ok(None);};if snapshot.entries.len()>20_000{return Err("Native Doctor snapshot exceeds 20,000 entries.".into());}
 let mut result=BTreeMap::new();let mut total=0;
 for e in snapshot.entries{total+=e.path.len();if total>8*1024*1024||!["file","directory"].contains(&e.kind.as_str()){return Err("Invalid native Doctor snapshot entry.".into());}let k=key(&e.path)?;if k.is_empty()||result.insert(k,e.kind).is_some(){return Err("Duplicate/aliased native Doctor snapshot path.".into());}}
 for path in result.keys(){let mut parent=path.rsplit_once('/').map(|(p,_)|p);while let Some(p)=parent{if result.get(p).map(String::as_str)!=Some("directory"){return Err("Snapshot paths require directory ancestors.".into());}parent=p.rsplit_once('/').map(|(p,_)|p);}}
 Ok(Some(result))
}
fn exists(inventory:&Option<BTreeMap<String,String>>,path:&str,role:&str)->Result<bool,String>{
 let path=key(path)?;let inventory=inventory.as_ref().ok_or_else(||format!("Select and review the {role} root before this path observation."))?;
 Ok(path.is_empty()||inventory.contains_key(&path))
}
fn joined(base:&str,path:&str)->String{if base.is_empty(){path.into()}else{format!("{}/{path}",base.trim_end_matches(['/', '\\']))}}
fn template(text:&str,code:&str,year:&str)->String{text.replace("{year}",year).replace("{state_lower}",&code.to_lowercase()).replace("{state_upper}",code)}
fn finish(status:&str,exit:Option<u8>,text:String,sources:Value,observations:Value)->Value{
 json!({"schema_version":"bisect-native-location-doctor-v1","status":status,"reference_cli_exit_code":exit,"console_text":text,"sources":sources,"observations":observations,"path_semantics":"windows-ascii-selected-root","execution_performed":false,"scope":"Native location Doctor observations over explicit bytes and selected-root snapshots. No environment/home/path probing, referenced file-body reads, downloads or partition execution. Imported snapshots are claims, not live access. Presence is not adjacency/geometry validation; native completion is not legal certification. Native stderr OS warnings are represented by source attempts, not forged byte-identical errors."})
}
pub(crate) fn inspect(input:Input)->Result<Value,String>{
 let settings=input.settings;for v in [&settings.state,&settings.year,&settings.chamber,&settings.resolution]{if v.trim().is_empty()||v.len()>200||v.chars().any(char::is_control){return Err("Supply plain nonempty Doctor fields up to 200 UTF-8 bytes.".into());}}
 if input.output_base.len()>4096||input.output_base.chars().any(char::is_control){return Err("Invalid Doctor output display prefix.".into());}
 let outputs=inventory(input.outputs)?;let workspace=inventory(input.workspace)?;
 let code=settings.state.to_uppercase();let (database,registry_source)=policy(&input.policy,true)?;let registry=PolicyRegistry::new(&database);
 let mut sources=json!({"registry":registry_source,"tolerance":null,"manifest":null});let mut observed=json!({});
 let mut text=format!("=== bisect doctor: {code} ===\n\n");
 if !registry.has_location(&code){text+=&format!("[FAIL] Location '{code}' not found in location_policy.json.\n       To add it: edit bisect/data/location_policy.json\n       Or set BISECT_LOCATION_POLICY to a custom policy file.\n");return Ok(finish("native-failure",Some(1),text,sources,observed));}
 text+=&format!("[PASS] Location '{code}' is registered.\n");
 if let Err(e)=registry.validate_year(&code,&settings.year){text+=&format!("[FAIL] {e}\n");return Ok(finish("native-failure",Some(1),text,sources,observed));}
 text+=&format!("[PASS] Year '{}' is valid for {code}.  Available: {}\n",settings.year,registry.available_years(&code).join(", "));
 let chamber=registry.chamber_districts(&code,&settings.chamber,&settings.year);let congressional=registry.congressional_districts(&code,&settings.year);
 let fallback=if congressional.is_none(){let (n,selection)=manifest(&input.manifest,&code,&settings.year)?;sources["manifest"]=selection;n}else{None};
 let count=chamber.or(congressional).or(fallback);observed["districts"]=json!(count.map(|n|n.to_string()));
 if let Some(n)=count{text+=&format!("[PASS] {code} {} districts: {n}\n",settings.chamber);}else{text+=&format!("[WARN] No district count found for '{code}' chamber '{}' year '{}'. Use --districts to specify.\n",settings.chamber,settings.year);}
 let (tolerance,tolerance_source)=policy(&input.policy,false)?;sources["tolerance"]=tolerance_source;
 let tol=chamber_balance_tolerance(&tolerance,&code,&settings.chamber)*100.;observed["tolerance_percent"]=json!(tol);
 text+=&format!("[INFO] Balance tolerance: {tol:.1}% (from policy; override with --balance-tolerance)\n");
 if let Some(w)=registry.granularity_warning(&code,&settings.year,&settings.chamber,&settings.resolution){text+=&format!("[WARN] {w}\n");}else{text+=&format!("[PASS] Resolution '{}' is appropriate for {code} {}.\n",settings.resolution,settings.chamber);}
 let convention=database.get(&code).and_then(|e|e.get("adjacency_convention"));
 let hint=template(convention.and_then(|v|v.get("build_command")).and_then(Value::as_str).unwrap_or(""),&code,&settings.year);
 let kind=convention.and_then(|v|v.get("convention")).and_then(Value::as_str).unwrap_or("us_census");
 let custom=convention.and_then(|v|v.get("path_template")).and_then(Value::as_str).map(|s|template(s,&code,&settings.year));
 let mut chosen=None;let mut adjacency_error=None;
 if kind=="us_census"{
  let bg=["block_group","block-group"].contains(&settings.resolution.as_str());let mut names=vec![format!("{}_{}adjacency_{}.pkl",code.to_lowercase(),if bg{"bg_"}else{""},settings.year)];if bg{names.push(format!("{}_adjacency_{}.pkl",code.to_lowercase(),settings.year));}
  for name in names{for version in ["V3","V4"]{let relative=format!("{version}/data/{}/adjacency/{name}",settings.year);match exists(&outputs,&relative,"outputs"){
   Ok(true)=>{chosen=Some((joined(&input.output_base,&relative),true));break;},Ok(false)=>{},Err(e)=>{observed["pending_path"]=json!(relative);return Ok(finish("needs-observations",None,text+&format!("[UNVERIFIED] {e}\n"),sources,observed));}
  }}if chosen.is_some(){break;}}
  if chosen.is_none()&&custom.is_none(){adjacency_error=Some(format!("adjacency pkl not found for '{code}' year '{}'. Run: {hint}",settings.year));}
 }else if custom.is_none(){adjacency_error=Some(format!("no path_template in adjacency_convention for '{code}'"));}
 if chosen.is_none(){if let Some(path)=custom{
  if path.is_empty(){chosen=Some((path,false));}else{let portable=path.replace('\\',"/");match exists(&workspace,&portable,"workspace"){
   Ok(found)=>chosen=Some((portable,found)),Err(e)=>{observed["pending_path"]=json!(path);return Ok(finish("needs-observations",None,text+&format!("[UNVERIFIED] {e}\n"),sources,observed));}
  }}
 }}
 if let Some((path,found))=chosen{observed["adjacency"]=json!({"path":path,"exists":found});text+=&format!("[{}] Adjacency file {}: {path}\n",if found{"PASS"}else{"MISS"},if found{"found"}else{"missing"});if !found&&!hint.is_empty(){text+=&format!("       Build with: {hint}\n");}}
 else if let Some(e)=adjacency_error{observed["adjacency"]=json!({"error":e});text+=&format!("[WARN] Could not resolve adjacency path: {e}\n");}
 let tiger=format!("V3/data/{}/tiger",settings.year);let tiger_key=match key(&tiger){Ok(k)=>k,Err(e)=>return Ok(finish("needs-observations",None,text+&format!("[UNVERIFIED] {e}\n"),sources,observed))};
 let Some(output_entries)=outputs else{return Ok(finish("needs-observations",None,text+"[UNVERIFIED] Select and review the outputs root for TIGER observations.\n",sources,observed));};
 let has_tiger=output_entries.get(&tiger_key).is_some_and(|k|k=="directory")&&output_entries.keys().any(|p|p.starts_with(&(tiger_key.clone()+"/")));observed["tiger_nonempty"]=json!(has_tiger);let display=joined(&input.output_base,&tiger);
 if has_tiger{text+=&format!("[PASS] TIGER geometry found (maps available): {display}\n");}else{text+=&format!("[INFO] TIGER geometry not found (maps will fail): {display}\n       Run: bisect fetch --type tiger --states {code} --year {}\n",settings.year);}
 if let Some(location)=database.get(&code){
  if settings.chamber!="congressional"{if let Some(nesting)=location.get("nesting_requirement").and_then(Value::as_str).filter(|s|*s!="null"){text+=&format!("[INFO] Nesting: {nesting} ({} ratio). Use 'bisect suite' for multi-chamber.\n",location.get("nesting_ratio").and_then(Value::as_str).unwrap_or("?"));}}
  if let Some(standard)=location.get("compactness_standard").and_then(Value::as_str){text+=&format!("[INFO] Compactness standard: {standard}\n");}
 }
 text+=&format!("\n[OK] Doctor check complete for {code} {} {}.\n",settings.chamber,settings.year);Ok(finish("complete",Some(0),text,sources,observed))
}

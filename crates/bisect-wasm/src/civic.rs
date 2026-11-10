use base64::{engine::general_purpose::STANDARD,Engine};
use bisect_analysis::civic::{assess_rows,detect_conflicts,parse_and_canonicalize_csv,DetectedEncoding,ValidateMode};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub label:String,pub source_name:String,pub csv_base64:String}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestInput{pub source_name:String,pub manifest_json:String}
pub(crate) fn inventory(inputs:Vec<ManifestInput>)->Result<Value,String>{
 use bisect_analysis::civic::{parse_manifest,show_manifest,list_header,list_row};
 if inputs.is_empty()||inputs.len()>20{return Err("Select 1–20 civic manifests.".into());}
 let mut names=BTreeSet::new();let mut total=0;let mut rows=Vec::new();let mut lines=vec![list_header()];
 for input in inputs{
  total+=input.manifest_json.len();if input.source_name.is_empty()||input.source_name.len()>200||!names.insert(input.source_name.clone())||input.manifest_json.is_empty()||input.manifest_json.len()>1024*1024||total>8*1024*1024{return Err("Civic manifests need unique source names up to 200 UTF-8 bytes, 1 MiB per source and 8 MiB total.".into());}
  let hash=format!("sha256:{:x}",Sha256::digest(input.manifest_json.as_bytes()));
  let row=match parse_manifest(input.manifest_json.as_bytes()){
   Err(e)=>json!({"source_name":input.source_name,"source_sha256":hash,"status":"invalid-manifest","native_list_included":false,"error":e.to_string(),"native_show_json":null}),
   Ok(m)=>{lines.push(list_row(&m));json!({"source_name":input.source_name,"source_sha256":hash,"status":"parsed","native_list_included":true,"error":null,"native_show_json":show_manifest(&m).map_err(|e|e.to_string())?,"label":m.label,"submitter":m.submitter,"state":m.state,"year":m.year,"validate_mode":m.validate_mode,"schema":m.schema_version,"rows":m.normalized.row_count.to_string(),"errors":m.validation.errors.to_string(),"warnings":m.validation.warnings.to_string(),"snapshot_count":m.url_snapshots.len().to_string(),"submitted_at":m.submitted_at,"ingested_at":m.ingested_at,"version":m.version,"build_commit":m.build_commit})}
  };rows.push(row);
 }
 let parsed=rows.iter().filter(|r|r["native_list_included"]==true).count();
 Ok(json!({"schema_version":"bisect-civic-inventory-v1","status":if parsed==rows.len(){"parsed"}else{"some-manifests-invalid"},"entries":rows,"native_list_text":lines.join("\n"),"scope":"Shared native civic List/Show schema reader and formatting for supplied manifests. Native List skips malformed manifests; their errors are visible here. Native parsing accepts unknown fields and arbitrary schema/version/time/hash labels; a parsed record is not a verified ingestion. Native Show drops unknown fields during typed serialization; exact original source remains in the project. All u64 recorded counters remain exact decimal strings or native JSON text. Original/normalized CSV, validation logs and URL snapshots are not read, rehashed or authenticated; paths and URLs remain unopened. Selected file order determines list order. Folder selection cannot discover empty directories and reads only manifest.json files. Saved sources and results remain inert until Run."}))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IngestSettings {pub label:String,pub submitter:String,pub timestamp:String,#[serde(default)]pub snapshots:Option<Vec<SnapshotObservation>>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotObservation{pub url:String,pub fetched_at:String,pub http_status:u16,pub content_type:String,pub body_base64:Option<String>,pub headers_text:Option<String>,pub snapshot_format:String,pub error:Option<String>}
pub(crate) fn ingest(input:Input,state:&str,year:&str,mode:&str,settings:IngestSettings)->Result<Value,String>{
 use bisect_analysis::civic::{build_ingest_manifest,IngestMetadata,IngestFacts};
 let label=&settings.label;
 if label.is_empty()||label.len()>100||!label.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||["__proto__","constructor","prototype"].contains(&label.as_str()){return Err("Civic package label needs 1–100 ASCII letters, digits, underscores or hyphens, excluding reserved names.".into());}
 if settings.submitter.len()>200||settings.timestamp.is_empty()||settings.timestamp.len()>100{return Err("Submitter is limited to 200 UTF-8 bytes; supply a timestamp label of 1–100 bytes.".into());}
 let source_name=input.source_name.clone();let original_base64=input.csv_base64.clone();
 let assessment=inspect(vec![input],state,year,mode)?;let report=&assessment["inputs"][0];
 if let Some(error)=report["error"].as_str(){return Err(error.into());}
 let original=STANDARD.decode(&original_base64).map_err(|_|"Invalid civic source base64.")?;
 let normalized=report["normalized_csv"].as_str().ok_or("No normalized civic CSV produced.")?.as_bytes();
 let mut warnings:Vec<String>=serde_json::from_value(report["warnings"].clone()).map_err(|_|"Invalid native civic warnings.")?;
 let mut snapshots=Vec::new();let mut snapshot_files=std::collections::BTreeMap::new();
 if let Some(mut observations)=settings.snapshots{
  observations.sort_by(|a,b|a.url.cmp(&b.url));
  if observations.len()>20{return Err("Civic snapshots support at most 20 unique source URLs.".into());}
  let expected:BTreeSet<String>=serde_json::from_value(assessment["source_urls"].clone()).map_err(|_|"Missing source URL inventory")?;
  let mut seen=BTreeSet::new();let mut total=0usize;
  for o in observations{
   if !expected.contains(&o.url)||!seen.insert(o.url.clone())||o.fetched_at.is_empty()||o.fetched_at.len()>100||o.content_type.len()>1000||!matches!(o.snapshot_format.as_str(),"headers-body"|"browser-fetch-observed-headers-body"){return Err("Snapshots must uniquely cover selected CSV source URLs with bounded recorded metadata.".into());}
   let record=if let Some(error)=o.error{
    if error.len()>1000||o.http_status!=0||o.body_base64.is_some()||o.headers_text.is_some()||!o.content_type.is_empty(){return Err("Failed snapshot observations must have status zero and no response bytes.".into());}
    let record=bisect_analysis::civic_snapshot::failed(&o.url,&o.fetched_at,&o.snapshot_format);if let Some(w)=bisect_analysis::civic_snapshot::warning(&record,Some(&error)){warnings.push(w);}record
   }else{
    if !(100..=599).contains(&o.http_status){return Err("Snapshot responses require HTTP status 100–599.".into());}
    let encoded=o.body_base64.ok_or("Snapshot body bytes are required, including an empty encoding")?;let headers=o.headers_text.ok_or("Snapshot headers are required")?;
    if encoded.len()>6990508||headers.len()>65536{return Err("Snapshot sources exceed the 5 MiB body plus one-byte truncation witness or 64 KiB header limit.".into());}
    let body=STANDARD.decode(&encoded).map_err(|_|"Invalid snapshot Base64")?;
    if body.len()>bisect_analysis::civic_snapshot::MAX_BODY_BYTES+1||STANDARD.encode(&body)!=encoded{return Err("Snapshot body must be canonical Base64 with at most a one-byte truncation witness.".into());}
    total+=body.len()+headers.len();if total>16*1024*1024{return Err("Snapshot observations exceed 16 MiB total.".into());}
    let packet=bisect_analysis::civic_snapshot::build(&o.url,&o.fetched_at,o.http_status,&o.content_type,&body,headers,&o.snapshot_format);
    if let Some(w)=bisect_analysis::civic_snapshot::warning(&packet.record,None){warnings.push(w);}
    snapshot_files.insert(format!("snapshots/{}",packet.body_filename),STANDARD.encode(&packet.body));snapshot_files.insert(format!("snapshots/{}",packet.headers_filename),STANDARD.encode(packet.headers.as_bytes()));packet.record
   };snapshots.push(record);
  }
  if seen!=expected{return Err("Record one success or failure observation for every unique CSV source URL.".into());}
  snapshots.sort_by(|a,b|a.url.cmp(&b.url));
 }
 let log=warnings.join("\n");
 let manifest=build_ingest_manifest(IngestMetadata{label:settings.label.clone(),submitter:settings.submitter,submitted_at:settings.timestamp.clone(),ingested_at:settings.timestamp,year:year.into(),state:state.into(),validate_mode:mode.into(),original_path:source_name,version:env!("CARGO_PKG_VERSION").into(),build_commit:env!("BISECT_BUILD_COMMIT").into(),rustc_version:env!("BISECT_RUSTC_VERSION").into()},IngestFacts{original:&original,normalized,row_count:report["row_count"].as_u64().unwrap_or(0) as usize,encoding:if report["encoding"]=="utf-8-bom"{DetectedEncoding::Utf8WithBom}else{DetectedEncoding::Utf8},errors:report["errors"].as_u64().unwrap_or(0) as usize,warnings,url_snapshots:snapshots});
 let text=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
 let prefix=format!("v1/civic_inputs/{}",settings.label);
 let mut files=std::collections::BTreeMap::from([(format!("{prefix}/original.csv"),original_base64),(format!("{prefix}/normalized.csv"),STANDARD.encode(normalized)),(format!("{prefix}/validation_log.txt"),STANDARD.encode(log.as_bytes())),(format!("{prefix}/manifest.json"),STANDARD.encode(text.as_bytes()))]);
 for (path,bytes) in snapshot_files{files.insert(format!("{prefix}/{path}"),bytes);}
 Ok(json!({"schema_version":"bisect-civic-ingest-v1","status":if manifest.validation.errors==0{"ingested"}else{"ingested-with-warnings"},"manifest":manifest,"manifest_json":text,"files_base64":files,"assessment":assessment,"scope":"Shared native civic ingest manifest, exact original/normalized CSV and validation log, with optional supplied URL snapshot observations packaged by the native snapshot kernel. Browser observed headers have an explicit browser-fetch format; missing HTTP protocol/header visibility is not fabricated. Native sha8 sidecar naming and overwrite semantics are retained. Raw observations and timestamp labels are unverified host claims. Supplied timestamp labels populate native submitted_at and ingested_at; they are not authenticated clock readings. Source name is informational and never opened. Build metadata describes this compilation, not authenticated provenance. Structural validation retains native limitations; state/year membership, confidence/date fields and candidate race are not certified. Warning modes retain invalid rows. ZIP uses the native v1/civic_inputs layout. No URL fetch, DNS resolution or legal finding occurs. Saved projects execute nothing; imported exports require Run."}))
}

pub(crate) fn inspect(inputs:Vec<Input>,state:&str,year:&str,mode:&str)->Result<Value,String>{
 if inputs.is_empty()||inputs.len()>20{return Err("Select 1–20 labeled civic CSV inputs.".into());}
 if state.is_empty()||state.len()>100||year.is_empty()||year.len()>20{return Err("Supply state and year labels within 100 and 20 UTF-8 bytes.".into());}
 let mode=ValidateMode::from_str(mode)?;let mut labels=BTreeSet::new();let mut bytes=Vec::new();let mut total=0;
 for input in &inputs{
  if input.label.is_empty()||input.label.len()>100||input.source_name.is_empty()||input.source_name.len()>200||!labels.insert(input.label.clone()){return Err("Civic input labels must be unique and nonempty (100 UTF-8 bytes); source names up to 200 bytes.".into());}
  if input.csv_base64.len()>4*1024*1024{return Err("Combined civic source bytes are limited to 3 MiB.".into());}
  let raw=STANDARD.decode(&input.csv_base64).map_err(|_|"Invalid civic source base64.")?;
  total+=raw.len();if raw.is_empty()||total>3*1024*1024{return Err("Select nonempty civic CSV inputs totaling at most 3 MiB.".into());}bytes.push(raw);
 }
 let mut reports=Vec::new();let mut parsed=Vec::new();let mut count=0;let mut all_valid=true;
 for(input,raw)in inputs.iter().zip(bytes){
  let hash=format!("sha256:{:x}",Sha256::digest(&raw));
  let mut report=json!({"label":input.label,"source_name":input.source_name,"source_sha256":hash,"native_exit_code":1,"error":null,"normalized_csv":null,"normalized_sha256":null,"warnings":[],"errors":0,"row_count":null,"encoding":null,"line_endings":null});
  match parse_and_canonicalize_csv(&raw){
   Err(e)=>{report["error"]=json!(e.to_string());all_valid=false;},
   Ok((rows,normalized,encoding,line_endings))=>{
    count+=rows.len();if count>50_000{return Err("Civic assessment supports at most 50,000 rows across all inputs.".into());}
    report["row_count"]=json!(rows.len());report["encoding"]=json!(match encoding{DetectedEncoding::Utf8=>"utf-8",DetectedEncoding::Utf8WithBom=>"utf-8-bom"});report["line_endings"]=json!(line_endings);
    match assess_rows(&rows,state,year,mode){
     Err(e)=>{report["error"]=json!(e.to_string());all_valid=false;},
     Ok((errors,warnings))=>{
      report["native_exit_code"]=json!(0);report["errors"]=json!(errors);report["warnings"]=json!(warnings);report["normalized_sha256"]=json!(format!("sha256:{:x}",Sha256::digest(&normalized)));
      report["normalized_csv"]=json!(String::from_utf8(normalized).map_err(|_|"Native normalized CSV was not UTF-8.")?);parsed.push((input.label.clone(),rows));
     }
    }
   }
  }reports.push(report);
 }
 let source_urls:BTreeSet<_>=parsed.iter().flat_map(|(_,rows)|rows.iter().filter(|r|!r.source_url.is_empty()).map(|r|r.source_url.clone())).collect();
 let source_url_validation:Vec<_>=source_urls.iter().map(|url|{let error=bisect_analysis::civic::validate_source_url(url).err().map(|e|e.to_string());json!({"url":url,"native_valid":error.is_none(),"error":error})}).collect();
 let conflicts=if all_valid{Some(detect_conflicts(&parsed))}else{None};
 let conflict_json=conflicts.as_ref().map(|c|serde_json::to_string_pretty(c).expect("serializable civic report"));
 let status=if !all_valid{"invalid-input"}else if reports.iter().any(|r|r["errors"]!=0){"accepted-with-warnings"}else{"assessed"};
 Ok(json!({"schema_version":"bisect-civic-assessment-v1","status":status,"source_urls":source_urls,"source_url_validation":source_url_validation,"state":state,"year":year,"validate_mode":mode.as_str(),"inputs":reports,"conflicts_available":all_valid,"conflicts":conflicts,"conflicts_json":conflict_json,"scope":"Shared native civic CSV normalization, structural GEOID and parsed URL validation, and native COI overlap/label-mismatch detection. State/year are supplied labels; no tract-universe membership check occurs. Native ingestion currently does not validate confidence or submitted_at values. Warning modes retain invalid rows. Normalization preserves source column order for extras and native trimming/quoting behavior. Conflict detection uses every row, including repeated rows within one submission; its candidate-race disagreement list is a native placeholder. URLs are not fetched, authenticated or DNS-resolved. Canonicalization does not sanitize spreadsheet formulas. No ingestion manifest, URL snapshot, race-of-candidate assessment or legal finding is produced. Saved projects execute nothing; imported results remain unverified until Run."}))
}

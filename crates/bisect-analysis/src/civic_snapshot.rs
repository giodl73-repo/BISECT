//! Pure native URL snapshot packaging; transport and clocks remain host observations.
use crate::civic::UrlSnapshotRecord;
use sha2::{Digest,Sha256};
pub const MAX_BODY_BYTES:usize=5*1024*1024;
pub struct SnapshotFiles{pub record:UrlSnapshotRecord,pub body:Vec<u8>,pub headers:String,pub body_filename:String,pub headers_filename:String}
pub fn build(url:&str,fetched_at:&str,status:u16,content_type:&str,body:&[u8],headers:String,format:&str)->SnapshotFiles{
 let truncated=body.len()>MAX_BODY_BYTES;let body=body[..body.len().min(MAX_BODY_BYTES)].to_vec();let sha=format!("{:x}",Sha256::digest(&body));let prefix=&sha[..8];let body_filename=format!("{prefix}.body");let headers_filename=format!("{prefix}.headers.txt");
 let record=UrlSnapshotRecord{url:url.into(),fetched_at:fetched_at.into(),http_status:status,content_type:content_type.into(),content_length_bytes:body.len() as u64,truncated,body_sha256:sha,snapshot_path:body_filename.clone(),snapshot_format:format.into()};
 SnapshotFiles{record,body,headers,body_filename,headers_filename}
}
pub fn failed(url:&str,fetched_at:&str,format:&str)->UrlSnapshotRecord{UrlSnapshotRecord{url:url.into(),fetched_at:fetched_at.into(),http_status:0,content_type:String::new(),content_length_bytes:0,truncated:false,body_sha256:String::new(),snapshot_path:String::new(),snapshot_format:format.into()}}
pub fn warning(record:&UrlSnapshotRecord,error:Option<&str>)->Option<String>{
 if let Some(error)=error{Some(format!("[WARN] URL snapshot failed for {}: {error}",record.url))}
 else if record.http_status>=400||record.http_status==0{Some(format!("[WARN] URL snapshot returned status {} for {}",record.http_status,record.url))}else{None}
}

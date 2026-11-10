use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::Engine;
use std::io::{Read,BufReader};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub source_name:String,#[serde(default)]pub csv_text:Option<String>,#[serde(default)]pub source_base64:Option<String>,pub kind:String,pub state:String,pub year:String,pub data_year:String,pub source_label:String}
// Hash/count decompressed bytes as the native aggregator reads them. Do not
// materialize a potentially large expanded CSV in JS or a second Rust buffer.
struct SourceReader<R>{inner:R,hash:Sha256,bytes:usize}
impl<R:Read> Read for SourceReader<R>{
    fn read(&mut self,buf:&mut [u8])->std::io::Result<usize>{
        if buf.is_empty(){return Ok(0);}
        let remaining=(128*1024*1024usize).saturating_sub(self.bytes);
        let length=buf.len().min(remaining.saturating_add(1));
        let n=self.inner.read(&mut buf[..length])?;
        if n>remaining{return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"LODES decompressed source exceeds 128 MiB"));}
        self.hash.update(&buf[..n]);self.bytes+=n;Ok(n)
    }
}
pub(crate) fn prepare(input:Input)->Result<Value,String>{
    if input.source_name.is_empty()||input.source_name.len()>200{return Err("Select a source name up to 200 bytes.".into());}
    let bytes=match (input.csv_text,input.source_base64){
        (Some(text),None)=>text.into_bytes(),
        (None,Some(encoded))=>{if encoded.len()>((16*1024*1024+2)/3)*4{return Err("LODES source exceeds 16 MiB".into());}base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|e|format!("LODES source base64: {e}"))?},
        _=>return Err("Select exactly one CSV text or base64 source.".into())
    };
    if bytes.is_empty()||bytes.len()>16*1024*1024{return Err("Select nonempty LODES CSV or gzip up to 16 MiB.".into());}
    crate::tract_input::import_scope(&input.state,&input.year,&input.source_label)?;
    if input.data_year.len()!=4||!input.data_year.bytes().all(|b|b.is_ascii_digit()){return Err("LODES observation year must contain four digits.".into());}
    let gzip=bytes.starts_with(&[0x1f,0x8b]);
    let raw:Box<dyn Read>=if gzip{Box::new(flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes)))}else{Box::new(std::io::Cursor::new(&bytes))};
    let mut source=SourceReader{inner:raw,hash:Sha256::new(),bytes:0};
    let reader=BufReader::new(&mut source);
    let report=match input.kind.as_str(){"wac"=>bisect_core::lodes_import::wac(reader)?,"od"=>bisect_core::lodes_import::od(reader)?,_=>return Err("LODES kind must be wac or od".into())};
    let character=if input.kind=="wac"{Some(crate::character_input::import_csv(report.csv_text.as_bytes(),&input.state,&input.year,&input.data_year,&input.source_label,"economic"))}else{None};
    let (character_status,character_input,character_error)=match character{Some(Ok(v))=>("ready",Some(v),None),Some(Err(e))=>("unavailable",None,Some(e)),None=>("not-applicable",None,None)};
    Ok(json!({"schema_version":"bisect-lodes-preparation-v1","status":"prepared","kind":input.kind,"state":input.state,"year":input.year,"data_year":input.data_year,"source_name":input.source_name,
        "source_sha256":format!("{:x}",Sha256::digest(&bytes)),"source_byte_length":bytes.len(),"source_encoding":if gzip{"gzip"}else{"csv"},"decoded_byte_length":source.bytes,"decoded_sha256":format!("{:x}",source.hash.finalize()),"csv_text":report.csv_text,"csv_sha256":format!("{:x}",Sha256::digest(report.csv_text.as_bytes())),
        "written_rows":report.written_rows,"skipped_short_rows":report.short_rows,"skipped_geoid_rows":report.invalid_geoid_rows,"character_status":character_status,"character_input":character_input,"character_error":character_error,"execution_performed":false,
        "scope":"Shared native Fetch aggregation, retaining native column lookup, numeric defaults, WAC floating sums and casts, OD integer sums and sorted tract output. Gzip is detected by its bytes and streamed through the same single-member Rust decoder as native Fetch, with checksum validation and a 128 MiB decoded limit; bytes after that member are not aggregated. Source and decoded hashes describe distinct byte streams. Malformed short rows and non-sliceable GEOID prefixes are skipped and disclosed; OD overflow fails. WAC character input requires the strict state importer; OD preserves cross-state pairs. Selected years and source label remain claims. No network acquisition, installation, cache activation, graph coverage check or engine execution occurs."}))
}

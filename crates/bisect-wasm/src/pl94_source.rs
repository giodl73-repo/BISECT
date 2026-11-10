//! Selected-byte boundary around the native PL 94-171 block population reader.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Cursor,Read,BufReader};

pub(crate) struct DecodedSource<'a>{inner:Box<dyn Read+'a>,hash:Sha256,length:usize}
impl Read for DecodedSource<'_>{
 fn read(&mut self,buffer:&mut [u8])->std::io::Result<usize>{
  if buffer.is_empty(){return Ok(0);}let remaining=(512*1024*1024usize).saturating_sub(self.length);let limit=buffer.len().min(remaining.saturating_add(1));let n=self.inner.read(&mut buffer[..limit])?;
  if n>remaining{return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"PL94 decoded source exceeds 512 MiB"));}self.hash.update(&buffer[..n]);self.length+=n;Ok(n)
 }
}
pub(crate) fn reader(bytes:&[u8])->BufReader<DecodedSource<'_>>{
 let cursor=Cursor::new(bytes);let inner:Box<dyn Read>=if bytes.starts_with(&[0x1f,0x8b]){Box::new(flate2::read::GzDecoder::new(cursor))}else{Box::new(cursor)};
 BufReader::new(DecodedSource{inner,hash:Sha256::new(),length:0})
}
pub(crate) fn decoded_identity(bytes:&[u8],reader:&BufReader<DecodedSource<'_>>)->Value{json!({"encoding":if bytes.starts_with(&[0x1f,0x8b]){"gzip"}else{"plain"},"decoded_byte_length":reader.get_ref().length,"decoded_sha256":format!("{:x}",reader.get_ref().hash.clone().finalize())})}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source { pub(crate) name: String, pub(crate) base64: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input { pub(crate) year: u16, pub(crate) geography: Source, pub(crate) population: Option<Source> }
pub(crate) fn bytes(source: &Source) -> Result<Vec<u8>, String> {
    const LIMIT: usize = 64 * 1024 * 1024;
    if source.name.is_empty() || source.name.len() > 200 || source.base64.len() > ((LIMIT+2)/3)*4 {
        return Err("Select named PL94 sources up to 64 MiB each".into());
    }
    let bytes=STANDARD.decode(&source.base64).map_err(|_|"Invalid PL94 source base64")?;
    if bytes.is_empty() || bytes.len()>LIMIT || STANDARD.encode(&bytes)!=source.base64 {return Err("Invalid PL94 source size or encoding".into());}
    Ok(bytes)
}
pub(crate) fn prepare(input: Input) -> Result<Value, String> {
    if ![2000,2010,2020].contains(&input.year) {return Err("Select Census year 2000, 2010 or 2020".into());}
    let geography=bytes(&input.geography)?;
    // Retain unused selected population bytes in provenance without interpreting them.
    let population=input.population.as_ref().map(bytes).transpose()?;
    let mut geo_reader=reader(&geography);let mut pop_reader=population.as_deref().map(reader);
    let blocks=bisect_data::pl94::read_pl94_block_populations_from_readers(
        &mut geo_reader,
        || pop_reader.as_mut().ok_or_else(||std::io::Error::new(std::io::ErrorKind::NotFound,"Select population segment 1 for 2010/2020")),
        input.year,
    ).map_err(|e|e.to_string())?;
    let mut csv=String::from("geoid,population\n");
    let mut total=0i128; let mut negative=0usize;
    let mut states=std::collections::BTreeSet::new();
    for block in &blocks {csv.push_str(&format!("{},{}\n",block.geoid,block.population));total+=i128::from(block.population);negative+=usize::from(block.population<0);states.insert(&block.geoid[..2]);}
    let identity=|source:&Source,data:&[u8]|json!({"name":source.name,"byte_length":data.len(),"sha256":format!("{:x}",Sha256::digest(data))});
    Ok(json!({"schema_version":"bisect-pl94-preparation-v1","status":"prepared","year":input.year,"block_count":blocks.len(),"total_population":total.to_string(),"negative_population_rows":negative,"state_fips":states,"geography":identity(&input.geography,&geography),"geography_decoded":decoded_identity(&geography,&geo_reader),"population":input.population.as_ref().zip(population.as_ref()).map(|(s,b)|identity(s,b)),"population_decoded":if input.year==2000{None}else{population.as_ref().zip(pop_reader.as_ref()).map(|(b,r)|decoded_identity(b,r))},"population_used":input.year!=2000,"csv_text":csv,"csv_sha256":format!("{:x}",Sha256::digest(csv.as_bytes())),"execution_performed":false,"scope":"Shared native PL94 block population reader and exact sorted integer CSV. Native negative counts are retained and disclosed. 2000 reads POP100 from geography only; 2010/2020 join population segment 1 by LOGRECNO. Plain or single-member gzip is detected from bytes, decoded in Rust with checksum validation and a 512 MiB per-stream bound; trailing gzip members are not read. Selected and decoded hashes bind distinct byte streams. No ZIP extraction, TIGER geometry, graph assembly, tract aggregation, data activation or engine run occurred. Selected source bounds do not establish real-state coverage."}))
}

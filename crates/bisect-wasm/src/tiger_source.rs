//! Selected SHP/DBF/SHX bytes through the shared native TIGER readers.
use base64::{engine::general_purpose::STANDARD,Engine};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::io::{Cursor,Read};
use std::borrow::Cow;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {pub(crate) name:String,pub(crate) base64:String}
#[derive(Deserialize)]
#[serde(rename_all="kebab-case")]
pub(crate) enum Kind {Tracts,BlockGroups,Blocks,BlockCentroids,Counties}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub(crate) kind:Kind,pub(crate) year:u16,pub(crate) shp:Source,pub(crate) dbf:Source,pub(crate) shx:Option<Source>}
pub(crate) fn bytes(source:&Source)->Result<Vec<u8>,String>{
 bytes_with_limit(source,64*1024*1024)
}
pub(crate) fn bytes_with_limit(source:&Source,limit:usize)->Result<Vec<u8>,String>{
 if source.name.is_empty()||source.name.len()>200||source.base64.len()>((limit+2)/3)*4{return Err(format!("Select named TIGER parts up to {} MiB each",limit/1024/1024));}
 let data=STANDARD.decode(&source.base64).map_err(|_|"Invalid TIGER part base64")?;
 if data.is_empty()||data.len()>limit||STANDARD.encode(&data)!=source.base64{return Err("Invalid TIGER part size or encoding".into());}Ok(data)
}
pub(crate) fn decoded_part(raw:&[u8])->Result<Cow<'_,[u8]>,String>{
 if !raw.starts_with(&[0x1f,0x8b]){return Ok(Cow::Borrowed(raw));}
 let mut decoded=Vec::new();flate2::read::GzDecoder::new(raw).take(128*1024*1024+1).read_to_end(&mut decoded).map_err(|e|format!("Invalid TIGER gzip: {e}"))?;
 if decoded.len()>128*1024*1024{return Err("Decoded TIGER part exceeds 128 MiB".into());}Ok(Cow::Owned(decoded))
}
pub(crate) fn decoded_identity(raw:&[u8],decoded:&[u8])->Value{json!({"encoding":if raw.starts_with(&[0x1f,0x8b]){"gzip"}else{"plain"},"decoded_byte_length":decoded.len(),"decoded_sha256":format!("{:x}",Sha256::digest(decoded))})}
fn u32le(data:&[u8])->usize{u32::from_le_bytes(data.try_into().expect("four bytes")) as usize}
fn header(data:&[u8])->Result<(),String>{
 if data.len()<100||u32::from_be_bytes(data[..4].try_into().unwrap())!=9994||u64::from(u32::from_be_bytes(data[24..28].try_into().unwrap()))*2!=data.len() as u64{return Err("TIGER SHP/SHX header length must match selected bytes".into());}Ok(())
}
pub(crate) fn preflight(shp:&[u8],shx:Option<&[u8]>)->Result<(),String>{
 header(shp)?;if let Some(index)=shx{header(index)?;if (index.len()-100)%8!=0{return Err("Malformed TIGER SHX index length".into());}}
 let mut offset=100;let mut entries=Vec::new();
 while offset<shp.len(){
  if shp.len()-offset<8{return Err("Truncated TIGER shape record header".into());}
  let declared=u64::from(u32::from_be_bytes(shp[offset+4..offset+8].try_into().unwrap()))*2;
  entries.push((offset as u64,declared));offset+=8;if declared<4||declared>(shp.len()-offset) as u64{return Err("TIGER shape record exceeds selected bytes".into());}let size=declared as usize;
  let shape=&shp[offset..offset+size];let kind=u32le(&shape[..4]);
  if kind==5||kind==15{
   if size<44{return Err("Truncated TIGER polygon header".into());}
   let parts=u32le(&shape[36..40]);let points=u32le(&shape[40..44]);
   let required=44u64+4*parts as u64+16*points as u64+if kind==15{16+8*points as u64}else{0};
   if required>size as u64{return Err("TIGER polygon counts exceed selected record bytes".into());}
   let mut previous=0;for part in 0..parts{let start=u32le(&shape[44+part*4..48+part*4]);if start>=points||part>0&&start<previous{return Err("TIGER polygon part index exceeds selected points".into());}previous=start;}
  }else if kind!=0{return Err("Native TIGER reader requires Polygon or PolygonZ records".into());}
  offset+=size;
 }
 if let Some(index)=shx{
  if (index.len()-100)/8!=entries.len(){return Err("TIGER SHX index count does not match SHP records".into());}
  for (i,(offset,size)) in entries.iter().enumerate(){let start=100+i*8;let selected_offset=u64::from(u32::from_be_bytes(index[start..start+4].try_into().unwrap()))*2;let selected_size=u64::from(u32::from_be_bytes(index[start+4..start+8].try_into().unwrap()))*2;if selected_offset!=*offset||selected_size!=*size{return Err("TIGER SHX entry does not match sequential SHP record".into());}}
 }Ok(())
}
pub(crate) fn prepare(input:Input)->Result<Value,String>{
 if ![2000,2010,2020].contains(&input.year){return Err("Select TIGER Census year 2000, 2010 or 2020".into());}
 let limit=if matches!(input.kind,Kind::Counties){96*1024*1024}else{64*1024*1024};
 let read=|s:&Source|bytes_with_limit(s,limit);
 let shp_raw=read(&input.shp)?;let dbf_raw=read(&input.dbf)?;let shx_raw=input.shx.as_ref().map(read).transpose()?;
 let shp=decoded_part(&shp_raw)?;let dbf=decoded_part(&dbf_raw)?;let shx=shx_raw.as_deref().map(decoded_part).transpose()?;
 preflight(&shp,shx.as_deref())?;
 let shape=Cursor::new(shp.as_ref());let attributes=Cursor::new(dbf.as_ref());let index=shx.as_deref().map(Cursor::new);
 let (kind,crs,records)=match input.kind {
  Kind::Tracts|Kind::BlockGroups=>{let groups=matches!(input.kind,Kind::BlockGroups);let units=bisect_data::tiger::read_tiger_units_from_streams(shape,attributes,index,groups).map_err(|e|e.to_string())?;
   (if groups{"block-groups"}else{"tracts"},"file-native",units.into_iter().map(|r|json!({"geoid":r.geoid,"geometry_wkb_base64":STANDARD.encode(r.geometry_wkb),"aland":r.aland.to_string(),"awater":r.awater.to_string(),"population":r.population.to_string()})).collect::<Vec<_>>())},
  Kind::Counties=>{let units=bisect_data::tiger::read_tiger_census_units_from_streams(shape,attributes,index,5).map_err(|e|e.to_string())?;
   ("counties","file-native",units.into_iter().map(|r|json!({"geoid":r.geoid,"geometry_wkb_base64":STANDARD.encode(r.geometry_wkb),"aland":r.aland.to_string(),"awater":r.awater.to_string(),"population":r.population.to_string()})).collect::<Vec<_>>())},
  Kind::Blocks=>{let units=bisect_data::tiger::read_tiger_blocks_projected_from_streams(shape,attributes,index,input.year).map_err(|e|e.to_string())?;
   ("blocks","EPSG:5070",units.into_iter().map(|r|json!({"geoid":r.geoid,"geometry_wkb_base64":STANDARD.encode(r.geometry_wkb),"centroid":r.centroid,"aland":r.aland.to_string(),"awater":r.awater.to_string()})).collect())},
  Kind::BlockCentroids=>{let units=bisect_data::tiger::read_tiger_block_centroids_projected_from_streams(shape,attributes,index,input.year).map_err(|e|e.to_string())?;
   ("block-centroids","EPSG:5070",units.into_iter().map(|(geoid,centroid)|json!({"geoid":geoid,"centroid":centroid})).collect())},
 };
 // Identity binds this artifact's actual floating-point geometry, not a cross-ABI claim.
 let records_json=serde_json::to_string(&records).map_err(|e|e.to_string())?;
 if records_json.len()>128*1024*1024{return Err("Decoded TIGER records exceed 128 MiB".into());}
 let identity=|source:&Source,data:&[u8]|json!({"name":source.name,"byte_length":data.len(),"sha256":format!("{:x}",Sha256::digest(data))});
 Ok(json!({"schema_version":"bisect-tiger-preparation-v1","status":"prepared","kind":kind,"year":input.year,"geometry_crs":crs,"record_count":records.len(),"records":records,"records_json":records_json,"records_sha256":format!("{:x}",Sha256::digest(records_json.as_bytes())),"shp":identity(&input.shp,&shp_raw),"dbf":identity(&input.dbf,&dbf_raw),"shx":input.shx.as_ref().zip(shx_raw.as_ref()).map(|(s,b)|identity(s,b)),"shp_decoded":decoded_identity(&shp_raw,&shp),"dbf_decoded":decoded_identity(&dbf_raw,&dbf),"shx_decoded":shx_raw.as_ref().zip(shx.as_ref()).map(|(raw,decoded)|decoded_identity(raw,decoded)),"execution_performed":false,"scope":"Shared native TIGER reader over selected SHP/DBF/optional SHX parts. Tracts/block groups retain file-native WKB and population placeholder zero. Blocks use native EPSG:5070 projection; centroid-only mode retains no polygon WKB. Year selects native block field names; tract/group aliases follow native precedence regardless of selected year. Exact area integers cross as text. File names and year are source claims; no PRJ was checked. Plain or single-member gzip parts retain selected and decoded byte identities; gzip CRC is checked and each decoded seekable part is bounded at 128 MiB. Trailing gzip members are not read. Browser preflight rejects mismatched declared lengths and polygon counts before decoding. No ZIP extraction, PL94 join, graph/adjacency assembly, data activation or engine run occurred. Source and output bounds do not prove real-state coverage."}))
}

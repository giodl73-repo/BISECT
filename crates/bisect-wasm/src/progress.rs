//! Borrowed progress bytes are copied by the host before this function returns.
#[cfg(target_arch="wasm32")]
#[link(wasm_import_module="env")]
extern "C" {fn bisect_progress(pointer:*const u8,length:usize)->i32;}
pub(crate) fn emit(value:&serde_json::Value)->Result<(),String>{
 #[cfg(target_arch="wasm32")]
 {let bytes=serde_json::to_vec(value).map_err(|e|e.to_string())?;
  if bytes.len()>64*1024*1024{return Err("Progress checkpoint exceeds 64 MiB.".into());}
  if unsafe{bisect_progress(bytes.as_ptr(),bytes.len())}!=0{return Err("Host could not retain the completed build checkpoint.".into());}
 }
 #[cfg(not(target_arch="wasm32"))]
 let _=value;
 Ok(())
}

//! One bounded-memory streaming SHA-256 session per worker/module instance.
use sha2::{Digest,Sha256};
thread_local!{static HASH:std::cell::RefCell<Option<Sha256>>=const{std::cell::RefCell::new(None)};}
#[no_mangle]pub extern "C" fn bisect_hash_start(){HASH.with(|h|*h.borrow_mut()=Some(Sha256::new()));}
/// # Safety
/// Input is a live module buffer of `length` bytes; updates are at most 1 MiB.
#[no_mangle]pub unsafe extern "C" fn bisect_hash_update(pointer:*const u8,length:usize)->i32{
 if length>1024*1024{return 1;}
 HASH.with(|h|match h.borrow_mut().as_mut(){Some(hash)=>{hash.update(std::slice::from_raw_parts(pointer,length));0},None=>1})
}
/// # Safety
/// Output points to a live module buffer of at least 32 bytes.
#[no_mangle]pub unsafe extern "C" fn bisect_hash_finish(output:*mut u8)->i32{
 HASH.with(|h|match h.borrow_mut().take(){Some(hash)=>{std::ptr::copy_nonoverlapping(hash.finalize().as_ptr(),output,32);0},None=>1})
}

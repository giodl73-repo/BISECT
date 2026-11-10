//! Explicit optional backend selection; omission preserves the existing engine.
use crate::engine::Options;
pub(crate) fn c_available()->bool { cfg!(all(feature="c-metis-sidecar",target_arch="wasm32")) }
pub(crate) fn validate(options:&Options)->Result<bool,String>{
    match (options.metis_engine.as_deref(),options.metis_runtime.as_deref()) {
        (None|Some("metis-core"),None)=>Ok(false),
        (Some("c-ffi"),Some("msvcrt-v1")) if c_available()=>Ok(true),
        (Some("c-ffi"),Some("msvcrt-v1"))=>Err("This engine build has no C METIS sidecar bridge.".into()),
        (Some("c-ffi"),_)=>Err("C METIS requires the explicit msvcrt-v1 runtime profile.".into()),
        _=>Err("Select metis-core without a runtime, or c-ffi with msvcrt-v1; no backend is substituted.".into()),
    }
}
pub(crate) struct Guard {
    #[cfg(feature="c-metis-sidecar")]
    _inner:bisect_apportion::metis_sidecar::BackendGuard,
}
pub(crate) fn enter(options:&Options)->Result<Guard,String>{
    let enabled=validate(options)?;
    #[cfg(feature="c-metis-sidecar")]
    {Ok(Guard{_inner:bisect_apportion::metis_sidecar::BackendGuard::enter(enabled)})}
    #[cfg(not(feature="c-metis-sidecar"))]
    {let _=enabled;Ok(Guard{})}
}
#[no_mangle]
pub extern "C" fn bisect_c_metis_bridge_version()->u32 {u32::from(c_available())}

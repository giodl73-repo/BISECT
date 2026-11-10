//! Public JavaScript modules shared by the native server and static catalog.
//! Cargo discovers the complete flat web/lab set, including worker modules.
include!(concat!(env!("OUT_DIR"), "/input_assets.rs"));

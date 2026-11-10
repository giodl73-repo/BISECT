//! Owned-test child: capture the actual native launch protocol, never start a TUI.
fn main(){
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let registry=std::env::var("BISECT_REGISTRY_JSON").expect("Native hub must inject registry");
    println!("{}",serde_json::json!({"arguments":args,"registry_json":registry}));
}

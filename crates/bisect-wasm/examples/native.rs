//! Reference execution for native Rust/WASM equivalence checks.
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let request = serde_json::from_str(&input).unwrap();
    match bisect_wasm::engine::execute(request) {
        Ok(result) => println!("{}", result),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

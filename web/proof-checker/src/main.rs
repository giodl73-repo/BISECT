use std::io::{Read, Write};
fn main() {
    let mut bytes = Vec::new();
    std::io::stdin().take(192 * 1024 * 1024 + 1).read_to_end(&mut bytes).expect("stdin");
    std::io::stdout().write_all(&bisect_proof_checker::execute(&bytes)).expect("stdout");
}

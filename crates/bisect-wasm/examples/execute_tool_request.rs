fn main() -> Result<(),Box<dyn std::error::Error>> {
    let path=std::env::args().nth(1).ok_or("practitioner request JSON path required")?;
    let request=serde_json::from_slice(&std::fs::read(path)?)?;
    println!("{}",serde_json::to_string(&bisect_wasm::toolkit::execute(request)?)?);
    Ok(())
}

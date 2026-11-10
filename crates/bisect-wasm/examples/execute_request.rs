//! Native reference for the same typed request executed by the WASM boundary.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("engine request JSON path required")?;
    let request = std::fs::read(path)?;
    // Use the public native ABI rather than reaching into a private engine
    // module. The owned input and borrowed response stay live for this call.
    let response = unsafe {
        let pointer = bisect_wasm::bisect_execute(request.as_ptr(), request.len());
        std::slice::from_raw_parts(pointer, bisect_wasm::bisect_response_length())
    };
    let value: serde_json::Value = serde_json::from_slice(response)?;
    if value["ok"] != true {
        return Err(value["error"].as_str().unwrap_or("Native engine failed").to_owned().into());
    }
    println!("{}", serde_json::to_string(&value["result"])?);
    Ok(())
}

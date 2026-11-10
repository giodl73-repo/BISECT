//! Native reference for instance-local multi-request toolkit protocols.
//! One bounded JSON request per line; no filesystem paths are followed by Rust.
use std::io::{BufRead, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path=std::env::args().nth(1).ok_or("toolkit request sequence path required")?;
    let mut reader=std::io::BufReader::new(std::fs::File::open(path)?);
    let mut output=std::io::BufWriter::new(std::io::stdout().lock());
    loop {
        let mut line=Vec::new();
        // Read in chunks instead of allocating an unbounded hostile line.
        loop {
            let (consumed,finished)={
                let bytes=reader.fill_buf()?;if bytes.is_empty(){(0,true)}else{
                    let n=bytes.iter().position(|b|*b==b'\n').map_or(bytes.len(),|i|i+1);
                    if line.len()+n>70*1024*1024{return Err("Toolkit sequence request exceeds 70 MiB.".into());}
                    line.extend_from_slice(&bytes[..n]);(n,bytes[n-1]==b'\n')
                }
            };
            reader.consume(consumed);if finished{break;}
        }
        if line.is_empty(){break;}
        let request=serde_json::from_slice(&line)?;
        let result=match bisect_wasm::toolkit::execute(request) {
            Ok(result)=>serde_json::json!({"ok":true,"result":result}),
            Err(error)=>serde_json::json!({"ok":false,"error":error}),
        };
        serde_json::to_writer(&mut output,&result)?;writeln!(output)?;output.flush()?;
    }
    Ok(())
}

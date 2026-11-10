fn main()->Result<(),Box<dyn std::error::Error>> {
    use std::io::{Read,Write};
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let bytes=std::fs::read(&args[0])?;
    let timestamp=args[2].parse::<u64>()?;
    let listener=std::net::TcpListener::bind("127.0.0.1:0")?;
    let url=format!("http://{}/source",listener.local_addr()?);
    let server=std::thread::spawn(move||->std::io::Result<()>{
        let (mut stream,_)=listener.accept()?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        let mut request=vec![];let mut byte=[0];
        while !request.ends_with(b"\r\n\r\n") {if request.len()>32768{return Err(std::io::Error::other("header too long"));}stream.read_exact(&mut byte)?;request.push(byte[0]);}
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len())?;
        stream.write_all(&bytes)?;Ok(())
    });
    let mut plan=fletch_core::fetch_plan_with_kind(&args[3],url,fletch_core::SourceKind::Http)?;
    plan.version=if args[4]=="none" {None} else {Some(args[4].clone())};
    let fetched=fletch_core::fetch_to_cache(&plan,fletch_core::FetchOptions::new(std::path::PathBuf::from(&args[1])).with_force(true).with_fetched_at_ms(timestamp))?;
    server.join().map_err(|_|"HTTP oracle failed")??;
    print!("{}",serde_json::to_string_pretty(&vec![fetched.entry])?);Ok(())
}

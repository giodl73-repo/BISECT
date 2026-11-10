//! Offline oracle for actual native download/preparation/completion branches.
fn main()->Result<(),String>{
    use std::io::{Read,Write};
    let args:Vec<_>=std::env::args().collect();if args.len()!=5{return Err("Expected kind, source file, output root and supplied/404".into());}
    let listener=std::net::TcpListener::bind("127.0.0.1:0").map_err(|e|e.to_string())?;let address=listener.local_addr().map_err(|e|e.to_string())?;
    let bytes=std::fs::read(&args[2]).map_err(|e|e.to_string())?;let missing=args[4]=="404";
    let server=std::thread::spawn(move||->Result<(),String>{let(mut stream,_)=listener.accept().map_err(|e|e.to_string())?;stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).map_err(|e|e.to_string())?;let mut request=vec![];let mut byte=[0];while !request.ends_with(b"\r\n\r\n"){if request.len()>16384{return Err("HTTP header too large".into());}stream.read_exact(&mut byte).map_err(|e|e.to_string())?;request.push(byte[0]);}let status=if missing{"404 Not Found"}else{"200 OK"};write!(stream,"HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len()).map_err(|e|e.to_string())?;stream.write_all(&bytes).map_err(|e|e.to_string())?;Ok(())});
    let root=std::path::Path::new(&args[3]);std::fs::create_dir_all(root).map_err(|e|e.to_string())?;let target=root.join("native.csv");let marker=root.join("native.done");
    let item=bisect_cli::fetch::FetchItem {state_code:"RI".into(),year:"2020".into(),kind:args[1].clone(),url:Some(format!("http://{address}/source")),local_path:target.clone(),done_marker:marker.clone(),available_locally:false};
    let mut manifest:bisect_cli::fetch::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;manifest.local_data_dir=root.join("data").to_string_lossy().into_owned();
    bisect_cli::fetch::download_items(&[item.clone()],true,false,&manifest,0)?;
    server.join().map_err(|_|"HTTP oracle thread failed".to_string())??;
    println!("{}",serde_json::json!({"csv_text":std::fs::read_to_string(target).map_err(|e|e.to_string())?,"marker_text":std::fs::read_to_string(marker).map_err(|e|e.to_string())?,"is_done":item.is_done()}));Ok(())
}

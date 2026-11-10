//! Execute the native filesystem import with the browser's recorded timestamp.
use serde_json::Value;
fn main()->Result<(),String>{
 let args:Vec<_>=std::env::args().collect();let request:Value=serde_json::from_str(&std::fs::read_to_string(&args[1]).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;let input=&request["input"];
 let text=|v:&Value|v.as_str().ok_or("Missing reference string").map(str::to_owned);
 let source_name=text(&input["source"]["name"])?;let source=std::path::Path::new(&source_name);
 bisect_cli::import_label::run_label_import_at(&text(&input["label"])? ,source,&text(&input["year"])? ,input["format"].as_str(),&text(&input["created"])? )
}
